//! Source-owned red-label runtime memory implementation.

use super::machine_player::*;
use super::machine_sound::*;
use super::machine_video::*;
use super::machine_world::*;
use super::*;
use crate::game::{
    PLAYER_EXPLOSION_PIECE_LIMIT, PlayerExplosionCloudSnapshot, PlayerExplosionPieceSnapshot,
    SOURCE_EXPLOSION_LIFETIME_FRAMES, SOURCE_PLAYER_EXPLOSION_COLORS,
    SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS, SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
    SOURCE_TERRAIN_BLOW_STATUS_BIT, TerrainBlowSnapshot, TerrainBlowStage,
    source_explosion_frame_index, source_player_explosion_color_index_for_pointer,
};
use crate::machine_state::{
    EXPANDED_OBJECT_DETAIL_LIMIT, ExpandedObjectDetailState, ExpandedObjectKindState,
    ExpandedObjectState, OBJECT_LIST_DETAIL_LIMIT, ObjectListDetailState, ObjectListNameState,
    ObjectListState,
};
use crate::renderer::SpriteId;

impl RedLabelRuntimeMemory {
    pub fn new_cold_boot() -> Result<Self, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object_table_range = table_range(&layout, "object")?;
        let process_table_range = table_range(&layout, "process")?;
        let super_process_table_range = table_range(&layout, "super_process")?;
        let shell_head = linked_list(&lists, "shell_object")?.head_address;
        let mut memory = Self {
            ram: cleared_main_cpu_ram(),
            palette_ram: [0; PALETTE_RAM_SIZE],
            hardware_map: 0,
            cmos: [0xF0; CMOS_RAM_SIZE],
            object_table_range,
            process_table_range,
            super_process_table_range,
            shell_head_range: usize::from(shell_head)..usize::from(shell_head) + 2,
            attract_instruction_laser_addresses: Vec::new(),
            live_object_addresses: Vec::new(),
            live_expanded_object_addresses: Vec::new(),
            live_defender_wordmark_coalesced: false,
        };
        let cmos_defaults = red_label_cmos_defaults()?;
        memory.apply_cmos_defaults(&cmos_defaults)?;
        memory.apply_todays_high_score_defaults(&cmos_defaults)?;
        Ok(memory)
    }

    /// Initializes the RAM list heads and free lists described by red-label
    /// `PINIT` and `OINIT`; the actual addresses come from the embedded
    /// `phr6.src` layout/list assets.
    pub fn new_initialized() -> Result<Self, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let mut memory = Self::new_cold_boot()?;
        memory.initialize_process_lists(&layout, &lists)?;
        memory.initialize_object_lists(&layout, &lists)?;
        Ok(memory)
    }

    pub fn ram(&self) -> &[u8] {
        self.ram.as_slice()
    }

    pub fn ram_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.ram.len() {
            return None;
        }
        Some(&self.ram[start..end])
    }

    pub fn visible_rgba_image(&self) -> Option<RenderedImage> {
        render_defender_visible_rgba(self.ram.as_slice(), &self.palette_ram)
    }

    pub fn visible_palette_indices(&self) -> Option<Vec<u8>> {
        render_defender_visible_palette_indices(self.ram.as_slice(), &self.palette_ram)
    }

    pub fn visible_pixel_nibbles(&self) -> Option<Vec<u8>> {
        render_defender_visible_pixel_nibbles(self.ram.as_slice())
    }

    pub fn visible_video_crc32(&self) -> Option<u32> {
        self.visible_pixel_nibbles()
            .map(|visible_nibbles| crc32(&visible_nibbles))
    }

    pub fn palette_ram(&self) -> &[u8; PALETTE_RAM_SIZE] {
        &self.palette_ram
    }

    pub fn hardware_map(&self) -> u8 {
        self.hardware_map
    }

    pub(super) fn write_hardware_map(&mut self, value: u8, writes: &mut Vec<u8>) {
        self.hardware_map = value;
        writes.push(value);
    }

    pub(super) fn begin_irq_hardware_map_sequence(&mut self) -> (u8, Vec<u8>) {
        let hardware_map_before = self.hardware_map;
        let mut writes = Vec::new();
        self.write_hardware_map(0, &mut writes);
        (hardware_map_before, writes)
    }

    pub(super) fn finish_irq_hardware_map_sequence(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        writes: &mut Vec<u8>,
    ) -> Result<u8, String> {
        self.write_hardware_map(0, writes);
        let restored_map = self.read_field_byte(layout, "base_page", "MAPCR")?;
        self.write_hardware_map(restored_map, writes);
        Ok(restored_map)
    }

    /// Source-shaped `IRQ`/`IRQB` color-mapping block: load `U` with
    /// `CRAM+16`, then push `PCRAM+10..+15`, `PCRAM+4..+9`, and
    /// `PCRAM+0..+3` through `PSHU`, leaving hardware color RAM equal to the
    /// current pseudo-color RAM bytes.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1978-L1987>.
    pub fn copy_red_label_color_mapping_to_palette_ram(
        &mut self,
    ) -> Result<RedLabelPaletteCopy, String> {
        let layout = red_label_ram_layout()?;
        self.copy_color_mapping_to_palette_ram(&layout)
    }

    pub(super) fn copy_color_mapping_to_palette_ram(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelPaletteCopy, String> {
        let range = field_range(layout, "base_page", "PCRAM")?;
        let bytes = self
            .ram_range(range.clone())
            .ok_or_else(|| String::from("red-label PCRAM range is outside RAM"))?;
        if bytes.len() != PALETTE_RAM_SIZE {
            return Err(format!(
                "red-label PCRAM has {} byte(s), expected {PALETTE_RAM_SIZE}",
                bytes.len()
            ));
        }
        let mut source = [0; PALETTE_RAM_SIZE];
        source.copy_from_slice(bytes);

        let mut cursor = PALETTE_RAM_SIZE;
        for (source_start, length) in [(10, 6), (4, 6), (0, 4)] {
            cursor -= length;
            self.palette_ram[cursor..cursor + length]
                .copy_from_slice(&source[source_start..source_start + length]);
        }

        Ok(RedLabelPaletteCopy {
            source_start: range.start,
            target_start: RED_LABEL_COLOR_RAM_START,
            bytes_copied: PALETTE_RAM_SIZE as u8,
            palette_ram: self.palette_ram,
        })
    }

    pub fn cmos_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.cmos.len() {
            return None;
        }
        Some(&self.cmos[start..end])
    }

    pub(super) fn apply_cmos_defaults(
        &mut self,
        defaults: &[RedLabelCmosDefault],
    ) -> Result<(), String> {
        for default in defaults {
            let range = default.cell_range().ok_or_else(|| {
                format!(
                    "red-label CMOS default `{}` has an invalid cell range",
                    default.symbol
                )
            })?;
            let start = usize::from(range.start);
            let end = usize::from(range.end);
            let cells = default.encoded_cells();
            if start > end || end > self.cmos.len() || end - start != cells.len() {
                return Err(format!(
                    "red-label CMOS default `{}` range 0x{:02X}..0x{:02X} does not match {} cell(s)",
                    default.symbol,
                    range.start,
                    range.end,
                    cells.len()
                ));
            }
            self.cmos[start..end].copy_from_slice(&cells);
        }
        Ok(())
    }

    pub(super) fn apply_todays_high_score_defaults(
        &mut self,
        defaults: &[RedLabelCmosDefault],
    ) -> Result<(), String> {
        let cells = red_label_high_score_default_cells(defaults)?;
        let start = usize::from(RED_LABEL_THSTAB_START);
        let end = start
            .checked_add(cells.len())
            .ok_or_else(|| String::from("red-label today's high-score default range overflows"))?;
        if end > self.ram.len() {
            return Err(format!(
                "red-label today's high-score default range 0x{start:04X}..0x{end:04X} overflows RAM"
            ));
        }
        self.ram[start..end].copy_from_slice(&cells);
        Ok(())
    }

    pub(super) fn read_cmos_byte_by_symbol(&self, symbol: &'static str) -> Result<u8, String> {
        let offset = cmos_symbol_offset(symbol)?;
        cmos_sram_read_byte(&self.cmos, usize::from(offset))
            .ok_or_else(|| format!("red-label CMOS byte `{symbol}` overflows CMOS RAM"))
    }

    pub(super) fn read_cmos_word_by_symbol(&self, symbol: &'static str) -> Result<u16, String> {
        let offset = cmos_symbol_offset(symbol)?;
        cmos_sram_read_word(&self.cmos, usize::from(offset))
            .ok_or_else(|| format!("red-label CMOS word `{symbol}` overflows CMOS RAM"))
    }

    pub(super) fn write_cmos_byte_by_symbol(
        &mut self,
        symbol: &'static str,
        value: u8,
    ) -> Result<RedLabelCmosByteWrite, String> {
        let offset = cmos_symbol_offset(symbol)?;
        cmos_sram_write_byte(&mut self.cmos, usize::from(offset), value)
            .ok_or_else(|| format!("red-label CMOS byte `{symbol}` overflows CMOS RAM"))?;
        Ok(RedLabelCmosByteWrite {
            symbol,
            offset,
            value,
        })
    }

    pub(super) fn write_cmos_word_by_symbol(
        &mut self,
        symbol: &'static str,
        value: u16,
    ) -> Result<RedLabelCmosWordWrite, String> {
        let offset = cmos_symbol_offset(symbol)?;
        cmos_sram_write_word(&mut self.cmos, usize::from(offset), value)
            .ok_or_else(|| format!("red-label CMOS word `{symbol}` overflows CMOS RAM"))?;
        Ok(RedLabelCmosWordWrite {
            symbol,
            offset,
            value,
        })
    }

    pub(super) fn replace_cmos(&mut self, cmos: CmosRam) {
        self.cmos = cmos;
    }

    pub(super) fn all_time_high_score_value(&self) -> Result<u32, String> {
        Ok(self
            .high_score_entry(RuntimeHighScoreTable::AllTime, 0)?
            .score)
    }

    pub(super) fn live_high_score_qualifying_rank(&self, score: u32) -> Result<Option<u8>, String> {
        self.high_score_qualifying_rank(RuntimeHighScoreTable::TodaysGreatest, score)
    }

    pub(super) fn high_score_qualifying_rank(
        &self,
        table: RuntimeHighScoreTable,
        score: u32,
    ) -> Result<Option<u8>, String> {
        if score > RED_LABEL_HIGH_SCORE_MAX_SCORE {
            return Ok(None);
        }
        for index in 0..RED_LABEL_HIGH_SCORE_ENTRIES {
            if score > self.high_score_entry(table, index)?.score {
                return Ok(Some(
                    u8::try_from(index + 1).expect("red-label high-score rank should fit in u8"),
                ));
            }
        }
        Ok(None)
    }

    pub(super) fn insert_high_score(
        &mut self,
        table: RuntimeHighScoreTable,
        score: u32,
        initials: [u8; RED_LABEL_INITIALS_ENTRY_CHARS],
    ) -> Result<Option<u8>, String> {
        if !red_label_high_score_initials_are_valid(&initials) {
            return Err(String::from(
                "red-label high-score initials must be uppercase ASCII or source blank bytes",
            ));
        }
        let Some(rank) = self.high_score_qualifying_rank(table, score)? else {
            return Ok(None);
        };
        let insert_index = usize::from(rank - 1);
        let mut entries = [RuntimeHighScoreEntry::EMPTY; RED_LABEL_HIGH_SCORE_ENTRIES];
        for (index, entry) in entries.iter_mut().enumerate() {
            *entry = self.high_score_entry(table, index)?;
        }
        for index in (insert_index + 1..RED_LABEL_HIGH_SCORE_ENTRIES).rev() {
            entries[index] = entries[index - 1];
        }
        entries[insert_index] = RuntimeHighScoreEntry { score, initials };
        for (index, entry) in entries.iter().copied().enumerate() {
            self.write_high_score_entry(table, index, entry)?;
        }
        Ok(Some(rank))
    }

    pub(super) fn high_score_entry(
        &self,
        table: RuntimeHighScoreTable,
        index: usize,
    ) -> Result<RuntimeHighScoreEntry, String> {
        let start = high_score_entry_offset(table, index)?;
        let source = self.high_score_table_cells(table);
        let bytes = [
            sram_cell_read_byte(source, start).ok_or_else(|| {
                format!("red-label {} high-score byte 0 overflows", table.label())
            })?,
            sram_cell_read_byte(source, start + 2).ok_or_else(|| {
                format!("red-label {} high-score byte 1 overflows", table.label())
            })?,
            sram_cell_read_byte(source, start + 4).ok_or_else(|| {
                format!("red-label {} high-score byte 2 overflows", table.label())
            })?,
        ];
        if bytes.iter().any(|byte| !is_bcd_byte(*byte)) {
            return Err(format!(
                "red-label {} high-score bytes are not valid BCD",
                table.label()
            ));
        }
        let score = bcd_digits_to_u32(&bytes);
        if score > RED_LABEL_HIGH_SCORE_MAX_SCORE {
            return Err(format!(
                "red-label {} high score {score} exceeds {RED_LABEL_HIGH_SCORE_MAX_SCORE}",
                table.label()
            ));
        }
        let initials = [
            sram_cell_read_byte(source, start + 6)
                .ok_or_else(|| format!("red-label {} initial 0 overflows", table.label()))?,
            sram_cell_read_byte(source, start + 8)
                .ok_or_else(|| format!("red-label {} initial 1 overflows", table.label()))?,
            sram_cell_read_byte(source, start + 10)
                .ok_or_else(|| format!("red-label {} initial 2 overflows", table.label()))?,
        ];
        if !red_label_high_score_initials_are_valid(&initials) {
            return Err(format!(
                "red-label {} high-score initials are not valid uppercase ASCII or source blank bytes",
                table.label()
            ));
        }
        Ok(RuntimeHighScoreEntry { score, initials })
    }

    pub(super) fn high_score_tables(&self) -> Result<HighScoreTablesState, String> {
        Ok(HighScoreTablesState {
            all_time: self.high_score_table_snapshot(RuntimeHighScoreTable::AllTime)?,
            todays_greatest: self
                .high_score_table_snapshot(RuntimeHighScoreTable::TodaysGreatest)?,
        })
    }

    fn high_score_table_snapshot(
        &self,
        table: RuntimeHighScoreTable,
    ) -> Result<[HighScoreTableEntryState; RED_LABEL_HIGH_SCORE_ENTRIES], String> {
        let mut entries = [HighScoreTableEntryState::EMPTY; RED_LABEL_HIGH_SCORE_ENTRIES];
        for (index, entry) in entries.iter_mut().enumerate() {
            let source = self.high_score_entry(table, index)?;
            *entry = HighScoreTableEntryState {
                rank: u8::try_from(index + 1).expect("red-label high-score rank should fit in u8"),
                score: source.score,
                initials: source.initials,
            };
        }
        Ok(entries)
    }

    pub(super) fn write_high_score_entry(
        &mut self,
        table: RuntimeHighScoreTable,
        index: usize,
        entry: RuntimeHighScoreEntry,
    ) -> Result<(), String> {
        let start = high_score_entry_offset(table, index)?;
        let score_bytes = high_score_bcd_bytes(entry.score)?;
        if !red_label_high_score_initials_are_valid(&entry.initials) {
            return Err(String::from(
                "red-label high-score initials must be uppercase ASCII or source blank bytes",
            ));
        }
        let target = self.high_score_table_cells_mut(table);
        for (offset, value) in [
            (0, score_bytes[0]),
            (2, score_bytes[1]),
            (4, score_bytes[2]),
            (6, entry.initials[0]),
            (8, entry.initials[1]),
            (10, entry.initials[2]),
        ] {
            sram_cell_write_byte(target, start + offset, value).ok_or_else(|| {
                format!(
                    "red-label {} high-score entry {index} write overflows",
                    table.label()
                )
            })?;
        }
        Ok(())
    }

    pub(super) fn high_score_table_cells(&self, table: RuntimeHighScoreTable) -> &[u8] {
        match table {
            RuntimeHighScoreTable::AllTime => self.cmos.as_slice(),
            RuntimeHighScoreTable::TodaysGreatest => self.ram.as_slice(),
        }
    }

    pub(super) fn high_score_table_cells_mut(&mut self, table: RuntimeHighScoreTable) -> &mut [u8] {
        match table {
            RuntimeHighScoreTable::AllTime => self.cmos.as_mut_slice(),
            RuntimeHighScoreTable::TodaysGreatest => self.ram.as_mut_slice(),
        }
    }

    pub fn object_table_crc32(&self) -> u32 {
        crc32(&self.ram[self.object_table_range.clone()])
    }

    pub(super) fn object_list_state(&self) -> Result<ObjectListState, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let mut state = ObjectListState {
            active_count: self.object_list_count(&layout, &lists, "active_object")?,
            inactive_count: self.object_list_count(&layout, &lists, "inactive_object")?,
            projectile_count: self.object_list_count(&layout, &lists, "shell_object")?,
            visible_count: self.visible_active_object_count(&layout, &lists)?,
            evidence_crc32: self.object_table_crc32(),
            detail_count: 0,
            details: [ObjectListDetailState::EMPTY; OBJECT_LIST_DETAIL_LIMIT],
        };

        self.collect_object_list_details(
            &layout,
            &lists,
            "active_object",
            ObjectListNameState::Active,
            &mut state,
        )?;
        self.collect_object_list_details(
            &layout,
            &lists,
            "inactive_object",
            ObjectListNameState::Inactive,
            &mut state,
        )?;
        self.collect_object_list_details(
            &layout,
            &lists,
            "shell_object",
            ObjectListNameState::Projectile,
            &mut state,
        )?;

        Ok(state)
    }

    fn object_list_count(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
        list: &str,
    ) -> Result<u16, String> {
        let object_table = table_descriptor(layout, "object")?;
        let mut object_address = self.read_word(linked_list(lists, list)?.head_address)?;
        let mut count = 0u16;

        for _ in 0..object_table.entries {
            if object_address == 0 {
                return Ok(count);
            }
            object_table_for_address(layout, object_address)?;
            count = count.saturating_add(1);
            object_address = self.read_object_word(layout, object_address, "OLINK")?;
        }

        Err(format!(
            "red-label {list} object list did not terminate within object table size"
        ))
    }

    fn visible_active_object_count(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
    ) -> Result<u16, String> {
        let object_table = table_descriptor(layout, "object")?;
        let mut object_address =
            self.read_word(linked_list(lists, "active_object")?.head_address)?;
        let mut count = 0u16;

        for _ in 0..object_table.entries {
            if object_address == 0 {
                return Ok(count);
            }
            object_table_for_address(layout, object_address)?;
            let screen_address = self.read_object_screen_address(layout, object_address)?;
            let picture_address = self.read_object_word(layout, object_address, "OPICT")?;
            if screen_address != 0 && red_label_object_picture(picture_address).is_ok() {
                count = count.saturating_add(1);
            }
            object_address = self.read_object_word(layout, object_address, "OLINK")?;
        }

        Err(String::from(
            "red-label active object list did not terminate within object table size",
        ))
    }

    fn collect_object_list_details(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
        list: &str,
        list_name: ObjectListNameState,
        state: &mut ObjectListState,
    ) -> Result<(), String> {
        let object_table = table_descriptor(layout, "object")?;
        let mut object_address = self.read_word(linked_list(lists, list)?.head_address)?;

        for _ in 0..object_table.entries {
            if object_address == 0 || usize::from(state.detail_count) >= OBJECT_LIST_DETAIL_LIMIT {
                return Ok(());
            }
            object_table_for_address(layout, object_address)?;
            state.details[usize::from(state.detail_count)] =
                self.object_list_detail(layout, object_table, list_name, object_address)?;
            state.detail_count += 1;
            object_address = self.read_object_word(layout, object_address, "OLINK")?;
        }

        Err(format!(
            "red-label {list} object list did not terminate within object table size"
        ))
    }

    fn object_list_detail(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_table: &RedLabelRamLayoutEntry,
        list: ObjectListNameState,
        object_address: u16,
    ) -> Result<ObjectListDetailState, String> {
        let screen = self.read_object_screen_address(layout, object_address)?;
        let [screen_x, screen_y] = screen.to_be_bytes();
        let picture_address = self.read_object_word(layout, object_address, "OPICT")?;
        let picture = red_label_object_picture(picture_address).ok();
        Ok(ObjectListDetailState {
            list,
            address: object_address,
            slot: entry_index_for_address(object_table, object_address)?,
            screen_x,
            screen_y,
            world_x: self.read_object_word(layout, object_address, "OX16")?,
            world_y: self.read_object_word(layout, object_address, "OY16")?,
            velocity_x: self.read_object_word(layout, object_address, "OXV")?,
            velocity_y: self.read_object_word(layout, object_address, "OYV")?,
            picture_address,
            picture_label: picture.map(|picture| picture.label.as_str()),
            picture_size: picture.map(|picture| (picture.width, picture.height)),
            primary_image_address: picture.map(|picture| picture.primary_image),
            alternate_image_address: picture.and_then(|picture| picture.alternate_image),
            mapped_sprite: picture
                .and_then(|picture| SpriteId::for_object_picture_label(&picture.label))
                .map(|sprite| sprite.0),
            object_type: self.read_object_byte(layout, object_address, "OTYP")?,
            scanner_color: self.read_object_word(layout, object_address, "OBJCOL")?,
        })
    }

    pub(super) fn expanded_object_state(&self) -> Result<ExpandedObjectState, String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        let last_slot = self.read_field_word(&layout, "base_page", "LSEXPL")?;
        let mut state = ExpandedObjectState {
            active_count: 0,
            last_slot_address: (last_slot != 0).then_some(last_slot),
            detail_count: 0,
            details: [ExpandedObjectDetailState::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT],
        };

        for entry_index in 0..table.entries {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            if size == 0 {
                continue;
            }

            state.active_count = state.active_count.saturating_add(1);
            if usize::from(state.detail_count) >= EXPANDED_OBJECT_DETAIL_LIMIT {
                continue;
            }

            state.details[usize::from(state.detail_count)] =
                self.expanded_object_detail(&layout, slot_address, size)?;
            state.detail_count += 1;
        }

        Ok(state)
    }

    fn expanded_object_detail(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        size: u16,
    ) -> Result<ExpandedObjectDetailState, String> {
        let descriptor_address = self.read_appearance_word(layout, slot_address, "OBDESC")?;
        let picture = red_label_object_picture(descriptor_address).ok();
        let picture_label = picture.map(|picture| picture.label.as_str());
        let score_popup = score_popup_metadata_for_picture_label(picture_label);
        let kind = expanded_object_kind_for_detail(size, picture_label);
        let explosion_frame = source_expanded_explosion_frame(kind, size);
        let erase_address = self.read_appearance_word(layout, slot_address, "ERASES")?;
        let [center_x, center_y] = self
            .read_appearance_word(layout, slot_address, "CENTER")?
            .to_be_bytes();
        let [top_left_x, top_left_y] = self
            .read_appearance_word(layout, slot_address, "TOPLFT")?
            .to_be_bytes();
        let object_address = self.read_appearance_word(layout, slot_address, "OBJPTR")?;

        Ok(ExpandedObjectDetailState {
            kind,
            slot_address,
            size,
            descriptor_address,
            picture_label,
            picture_size: picture.map(|picture| (picture.width, picture.height)),
            mapped_sprite: picture
                .and_then(|picture| SpriteId::for_object_picture_label(&picture.label))
                .map(|sprite| sprite.0),
            erase_address,
            center_x,
            center_y,
            top_left_x,
            top_left_y,
            object_address: (object_address != 0).then_some(object_address),
            score_popup_lifetime_ticks: score_popup.map(|metadata| metadata.lifetime_ticks),
            score_popup_value: score_popup.map(|metadata| metadata.value),
            explosion_frame,
            explosion_lifetime_frames: explosion_frame.map(|_| SOURCE_EXPLOSION_LIFETIME_FRAMES),
        })
    }

    pub(super) fn player_explosion_cloud(
        &self,
    ) -> Result<Option<PlayerExplosionCloudSnapshot>, String> {
        let layout = red_label_ram_layout()?;
        let color_table_address = red_label_player_death_table("PXCOL")?.address;
        let color_pointer = self.read_field_word(&layout, "player_explosion_state", "PCOLP")?;
        let Some(source_color_index) =
            source_player_explosion_color_index_for_pointer(color_pointer, color_table_address)
        else {
            return Ok(None);
        };
        let source_color = SOURCE_PLAYER_EXPLOSION_COLORS[usize::from(source_color_index)];
        if source_color == 0 {
            return Ok(None);
        }

        let source_color_counter =
            self.read_field_byte(&layout, "player_explosion_state", "PCOLC")?;
        let mut snapshot = PlayerExplosionCloudSnapshot {
            source_color,
            source_color_counter,
            source_color_index,
            frame: source_player_explosion_frame(source_color_index, source_color_counter),
            ..PlayerExplosionCloudSnapshot::EMPTY
        };
        let table = table_descriptor(&layout, "player_explosion_table")?;

        for entry_index in 0..table.entries.min(PLAYER_EXPLOSION_PIECE_LIMIT as u16) {
            let piece_address = table.base + entry_index * table.entry_size;
            let screen_address = self.read_player_explosion_word(&layout, piece_address, "PSCR")?;
            if screen_address == 0 {
                continue;
            }
            let x_position = self.read_player_explosion_word(&layout, piece_address, "PXPOST")?;
            let split = x_position.to_be_bytes()[1] & 0x80 != 0;
            if !self.player_explosion_screen_piece_visible(screen_address, split) {
                continue;
            }

            snapshot.push_piece(PlayerExplosionPieceSnapshot {
                position: crate::systems::ScreenPosition::from_packed(screen_address),
                split,
            });
        }

        Ok(Some(snapshot))
    }

    fn player_explosion_screen_piece_visible(&self, screen_address: u16, split: bool) -> bool {
        let primary_visible = matches!(self.read_word(screen_address), Ok(word) if word != 0);
        let secondary_visible = split
            && matches!(
                self.read_word(screen_address.wrapping_add(0x0100)),
                Ok(word) if word != 0
            );
        primary_visible || secondary_visible
    }

    pub(super) fn terrain_blow_snapshot(&self) -> Result<Option<TerrainBlowSnapshot>, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & SOURCE_TERRAIN_BLOW_STATUS_BIT == 0 {
            return Ok(None);
        }

        let terrain_table = field_range(&layout, "terrain_screen_table", "STBL")?;
        let scanner_table = field_range(&layout, "scanner_terrain_erase", "STETAB")?;
        let (stage, source_iteration, source_sleep_remaining) =
            self.terrain_blow_process_state(&layout).unwrap_or((
                TerrainBlowStage::Completed,
                SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
                None,
            ));

        Ok(Some(TerrainBlowSnapshot {
            stage,
            status_terrain_blown: true,
            source_iteration,
            source_iteration_limit: SOURCE_TERRAIN_BLOW_ITERATION_LIMIT,
            source_sleep_remaining,
            source_pseudo_color: self
                .read_byte(field_range(&layout, "base_page", "PCRAM")?.start)?,
            source_overload_counter: self.read_field_byte(&layout, "base_page", "OVCNT")?,
            terrain_erase_entries: table_word_entries(&terrain_table)?,
            scanner_terrain_erase_entries: table_word_entries(&scanner_table)?,
            terrain_words_remaining: self.indirect_nonzero_words_in_table(terrain_table)?,
            scanner_terrain_words_remaining: self.indirect_nonzero_words_in_table(scanner_table)?,
            explosions_per_pass: SOURCE_TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
        }))
    }

    fn terrain_blow_process_state(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(TerrainBlowStage, u8, Option<u8>), String> {
        let process_address = self.current_process_address(layout)?;
        let routine_address = self.read_process_word(layout, process_address, "PADDR")?;
        let iteration = self.read_process_byte(layout, process_address, "PD")?;
        let sleep_remaining = Some(self.read_process_byte(layout, process_address, "PTIME")?);

        if routine_address == red_label_routine_address("TERBLO")?
            || routine_address == red_label_routine_address("TBL3")?
        {
            return Ok((
                TerrainBlowStage::ExplosionPassSleeping,
                iteration,
                sleep_remaining,
            ));
        }
        if routine_address == red_label_routine_address("TBL4")? {
            return Ok((
                TerrainBlowStage::FlashClearedSleeping,
                iteration,
                sleep_remaining,
            ));
        }

        Ok((TerrainBlowStage::Completed, iteration, None))
    }

    fn indirect_nonzero_words_in_table(&self, table: std::ops::Range<u16>) -> Result<u16, String> {
        table_word_entries(&table)?;
        let mut remaining = 0u16;
        let mut cursor = table.start;
        while cursor != table.end {
            let screen_address = self.read_word(cursor)?;
            if screen_address != 0 && self.read_word(screen_address)? != 0 {
                remaining = remaining.saturating_add(1);
            }
            cursor = cursor.wrapping_add(2);
        }
        Ok(remaining)
    }

    pub fn process_table_crc32(&self) -> u32 {
        crc32(&self.ram[self.process_table_range.clone()])
    }

    pub fn super_process_table_crc32(&self) -> u32 {
        crc32(&self.ram[self.super_process_table_range.clone()])
    }

    pub fn shell_table_crc32(&self) -> u32 {
        crc32(&self.ram[self.shell_head_range.clone()])
    }

    pub(super) fn clear_trace_sinit_ram_to(&mut self, end_address: u16) -> Result<(), String> {
        if !(RED_LABEL_TRACE_SINIT_RAM_CLEAR_START..=RED_LABEL_TRACE_SINIT_RAM_CLEAR_END)
            .contains(&end_address)
        {
            return Err(format!(
                "red-label SINIT clear target 0x{end_address:04X} is outside 0x{:04X}..0x{:04X}",
                RED_LABEL_TRACE_SINIT_RAM_CLEAR_START, RED_LABEL_TRACE_SINIT_RAM_CLEAR_END
            ));
        }
        self.clear_range(RED_LABEL_TRACE_SINIT_RAM_CLEAR_START..end_address)
    }

    pub(super) fn clear_shell_head(&mut self) -> Result<(), String> {
        let range = self.shell_head_range.clone();
        let start = u16::try_from(range.start)
            .map_err(|_| String::from("red-label shell head start does not fit u16"))?;
        let end = u16::try_from(range.end)
            .map_err(|_| String::from("red-label shell head end does not fit u16"))?;
        self.clear_range(start..end)
    }

    pub(super) fn write_red_label_rand_state(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        state: RandState,
    ) -> Result<(), String> {
        self.write_field_byte(layout, "base_page", "SEED", state.seed)?;
        self.write_field_byte(layout, "base_page", "HSEED", state.hseed)?;
        self.write_field_byte(layout, "base_page", "LSEED", state.lseed)
    }

    /// Advance the source `defb6.src` power-up RAM-fill loop through `RAM6`.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1491-L1515>.
    pub(super) fn advance_power_up_ram_fill_to(
        &mut self,
        fill: &mut RedLabelPowerUpRamFill,
        target_address: u16,
    ) -> Result<(), String> {
        if target_address > MAIN_CPU_RAM_SIZE as u16 {
            return Err(format!(
                "red-label power-up RAM-fill target 0x{target_address:04X} is invalid"
            ));
        }

        while fill.next_address < target_address {
            let value = if fill.next_address.is_multiple_of(2) {
                let [a, b] =
                    red_label_crom0_ram_test_next_word(u16::from_be_bytes([fill.a, fill.b]))
                        .to_be_bytes();
                fill.a = a;
                fill.b = b;
                a
            } else {
                fill.b
            };
            self.write_byte(fill.next_address, value)?;
            fill.next_address = fill.next_address.wrapping_add(1);
        }

        Ok(())
    }

    /// Source-shaped `GETOB`: take an object cell from `OFREE`, prelink it
    /// against `OPTR`, clear `OBJX`/`OBJY`, and clear `OTYP`.
    pub fn get_object_cell(&mut self) -> Result<u16, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(&layout, "object")?;
        let free_head = linked_list(&lists, "free_object")?.head_address;
        let object_address = self.read_word(free_head)?;
        if object_address == 0 {
            return Err(String::from("red-label `OFREE` object list is empty"));
        }

        entry_index_for_address(object, object_address)?;
        let next_free = self.read_object_word(&layout, object_address, "OLINK")?;
        self.write_word(free_head, next_free)?;
        let active_head = linked_list(&lists, "active_object")?.head_address;
        let old_active = self.read_word(active_head)?;
        self.write_object_word(&layout, object_address, "OLINK", old_active)?;
        self.write_object_bytes(&layout, object_address, "OBJX", &[0])?;
        self.write_object_bytes(&layout, object_address, "OBJY", &[0])?;
        self.write_object_bytes(&layout, object_address, "OTYP", &[0])?;
        Ok(object_address)
    }

    /// Source-shaped `OBINIT`: call `GETOB`, then fill the caller-supplied
    /// object descriptor fields. The caller still decides when to store `OPTR`.
    pub fn init_object_cell(
        &mut self,
        process_address: u16,
        descriptor: RedLabelObjectDescriptor,
    ) -> Result<RedLabelCreatedObject, String> {
        let layout = red_label_ram_layout()?;
        process_table_for_address(&layout, process_address)?;
        let object_address = self.get_object_cell()?;
        self.write_object_word(&layout, object_address, "OBJID", process_address)?;
        self.write_object_word(&layout, object_address, "OPICT", descriptor.picture_address)?;
        self.write_object_word(
            &layout,
            object_address,
            "OCVECT",
            descriptor.collision_vector_address,
        )?;
        self.write_object_word(&layout, object_address, "OBJCOL", descriptor.scanner_color)?;
        Ok(RedLabelCreatedObject {
            object_address,
            process_address,
            descriptor,
        })
    }

    /// Helper for the source comment after `GETOB`: callers store the returned
    /// object pointer into `OPTR` after initialization.
    pub fn activate_object_cell(&mut self, object_address: u16) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        object_table_for_address(&layout, object_address)?;
        self.write_word(
            linked_list(&lists, "active_object")?.head_address,
            object_address,
        )
    }

    pub(super) fn return_unlinked_object_to_free_list(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<(), String> {
        let lists = red_label_linked_lists()?;
        object_table_for_address(layout, object_address)?;
        let free_head = linked_list(&lists, "free_object")?.head_address;
        let old_free = self.read_word(free_head)?;
        self.write_word(free_head, object_address)?;
        self.write_object_word(layout, object_address, "OLINK", old_free)
    }

    /// Source-shaped `APVCT` / `APST`: prepend the object to `OPTR`, replace
    /// its picture with `NULOB`, and allocate one appearance RAM slot when the
    /// object is inside the source `$2600` relative-X limit.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/samexap7.src#L19-L64>.
    pub fn start_appearance_for_object(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelAppearanceStart, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        object_table_for_address(&layout, object_address)?;

        let original_picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
        let null_picture_address = red_label_object_picture_address("NULOB")?;
        self.write_object_word(&layout, object_address, "OPICT", null_picture_address)?;
        self.write_word(
            linked_list(&lists, "active_object")?.head_address,
            object_address,
        )?;

        let relative_x = self
            .read_object_word(&layout, object_address, "OX16")?
            .wrapping_sub(self.read_field_word(&layout, "base_page", "BGL")?);
        if relative_x > RED_LABEL_APPEARANCE_ON_SCREEN_LIMIT {
            self.write_object_word(&layout, object_address, "OPICT", original_picture_address)?;
            return Ok(RedLabelAppearanceStart {
                object_address,
                original_picture_address,
                final_picture_address: original_picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                sound_loaded: None,
            });
        }

        let Some((slot_address, erased_previous_slot)) = self.allocate_appearance_slot(&layout)?
        else {
            self.write_object_word(&layout, object_address, "OPICT", original_picture_address)?;
            return Ok(RedLabelAppearanceStart {
                object_address,
                original_picture_address,
                final_picture_address: original_picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                sound_loaded: None,
            });
        };

        let sound_loaded = if self.read_field_byte(&layout, "base_page", "STATUS")? & 0x80 == 0 {
            self.load_sound_table_by_label("APSND")?
        } else {
            None
        };
        let object_type = self.read_object_byte(&layout, object_address, "OTYP")?;
        self.write_object_bytes(&layout, object_address, "OTYP", &[object_type | 0x02])?;
        self.write_appearance_word(
            &layout,
            slot_address,
            "RSIZE",
            RED_LABEL_APPEARANCE_INITIAL_SIZE,
        )?;
        self.write_appearance_word(&layout, slot_address, "OBDESC", original_picture_address)?;
        self.write_appearance_word(
            &layout,
            slot_address,
            "ERASES",
            slot_address.wrapping_add(0x40),
        )?;
        self.write_appearance_word(&layout, slot_address, "OBJPTR", object_address)?;

        Ok(RedLabelAppearanceStart {
            object_address,
            original_picture_address,
            final_picture_address: null_picture_address,
            relative_x,
            slot_address: Some(slot_address),
            erased_previous_slot,
            sound_loaded,
        })
    }

    /// Source-shaped RAM-visible `EXST`: allocate an expanded-object RAM slot
    /// for the object picture, initialize size/top-left/center, and remember
    /// the slot in `LSEXPL`; later `EXPU` calls draw it through the translated
    /// `EWRITE` path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/samexap7.src#L66-L117>.
    pub fn start_explosion_for_object(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelExplosionStart, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
        let relative_x = self
            .read_object_word(&layout, object_address, "OX16")?
            .wrapping_sub(self.read_field_word(&layout, "base_page", "BGL")?);
        if relative_x.to_be_bytes()[0] > 0x26 {
            return Ok(RedLabelExplosionStart {
                object_address,
                picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                top_left: None,
                center: None,
            });
        }
        self.write_field_word(&layout, "base_page", "XSTART", relative_x)?;

        let Some((slot_address, erased_previous_slot)) = self.allocate_appearance_slot(&layout)?
        else {
            return Ok(RedLabelExplosionStart {
                object_address,
                picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                top_left: None,
                center: None,
            });
        };

        self.write_field_word(&layout, "base_page", "LSEXPL", slot_address)?;
        self.write_appearance_word(&layout, slot_address, "RSIZE", 0x0100)?;
        self.write_appearance_word(&layout, slot_address, "OBDESC", picture_address)?;
        self.write_appearance_word(
            &layout,
            slot_address,
            "ERASES",
            slot_address.wrapping_add(0x40),
        )?;

        let scaled_x = relative_x.wrapping_shl(2).to_be_bytes()[0];
        let object_y = self
            .read_object_word(&layout, object_address, "OY16")?
            .to_be_bytes()[0];
        let top_left = u16::from_be_bytes([scaled_x, object_y]);
        self.write_appearance_word(&layout, slot_address, "TOPLFT", top_left)?;

        let center = self.explosion_start_center(&layout, picture_address, top_left)?;
        self.write_appearance_word(&layout, slot_address, "CENTER", center)?;

        Ok(RedLabelExplosionStart {
            object_address,
            picture_address,
            relative_x,
            slot_address: Some(slot_address),
            erased_previous_slot,
            top_left: Some(top_left),
            center: Some(center),
        })
    }

    /// Source-shaped RAM-visible `EXPU`: walk `RAMALS`, update positive-size
    /// explosions and negative-size appearances, restore object pictures when
    /// an appearance is done/offscreen, run `EERASE`, then run the translated
    /// `EWRITE` expanded-object video writer for live slots.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/samexap7.src#L122-L202>.
    pub fn update_expanded_objects(&mut self) -> Result<Vec<RedLabelExpandedUpdate>, String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        let mut updates = Vec::new();

        for entry_index in 0..table.entries {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;

            if status & 0x04 != 0 {
                if size & 0x8000 != 0 {
                    updates.push(self.kill_appearance_slot(&layout, slot_address)?);
                } else if size != 0 {
                    self.write_appearance_word(&layout, slot_address, "RSIZE", 0)?;
                    updates.push(RedLabelExpandedUpdate::ExplosionKilled {
                        slot_address,
                        erased_previous_image: false,
                    });
                }
                continue;
            }

            if size == 0 {
                continue;
            }

            if size & 0x8000 != 0 {
                updates.push(self.advance_appearance_slot(&layout, slot_address, size)?);
            } else {
                updates.push(self.advance_explosion_slot(&layout, slot_address, size)?);
            }
        }

        Ok(updates)
    }

    pub fn clear_live_expanded_object_addresses(&mut self) -> Result<(), String> {
        let recorded_addresses = std::mem::take(&mut self.live_expanded_object_addresses);
        for (address, width) in recorded_addresses {
            if width == 2 {
                self.write_word(address, 0)?;
            } else {
                self.write_byte(address, 0)?;
            }
        }
        Ok(())
    }

    /// Source-shaped `KILLOB`: search active objects, then inactive objects,
    /// unlink the matching cell, and push it back to `OFREE`.
    pub fn kill_object_cell(&mut self, object_address: u16) -> Result<u16, String> {
        self.kill_object_cell_from_lists(object_address, &["active_object", "inactive_object"])
    }

    /// Source-shaped visible `KILOFF`: unlink the object through `KILLOB`,
    /// select character map 2 like `OFSHIT`, then clear the current picture
    /// footprint using the object descriptor dimensions.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1150-L1159>.
    pub fn kill_object_cell_offscreen(&mut self, object_address: u16) -> Result<u16, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let screen_address = self.read_object_screen_address(&layout, object_address)?;
        let picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
        let previous_link_address = self.kill_object_cell(object_address)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.erase_object_picture_by_descriptor(screen_address, picture_address)?;
        Ok(previous_link_address)
    }

    /// Source-shaped `KILSHL`: search only the `SPTR` shell-object list and
    /// push the matching cell back to `OFREE`.
    pub fn kill_shell_cell(&mut self, object_address: u16) -> Result<u16, String> {
        self.kill_object_cell_from_lists(object_address, &["shell_object"])
    }

    /// Source-shaped `GETSHL`: allocate a shell object from `OFREE`, reject
    /// saturated/offscreen firing objects, initialize the shell cell, and link
    /// it through `SPTR`.
    pub fn get_shell_cell(
        &mut self,
        firing_object_address: u16,
        owner_address: u16,
        descriptor: RedLabelShellDescriptor,
    ) -> Result<Option<RedLabelCreatedShell>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        object_table_for_address(&layout, firing_object_address)?;

        let shell_count = self.read_field_byte(&layout, "base_page", "BMBCNT")?;
        if shell_count >= RED_LABEL_SHELL_LIMIT {
            return Ok(None);
        }

        let source_x = self.read_object_word(&layout, firing_object_address, "OX16")?;
        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let screen_delta = source_x.wrapping_sub(background_left);
        if screen_delta >= RED_LABEL_SHELL_SCREEN_RANGE {
            return Ok(None);
        }

        let screen_x = ((screen_delta << 2) >> 8) as u8;
        let source_y = self.read_object_byte(&layout, firing_object_address, "OY16")?;
        if source_y <= RED_LABEL_Y_MIN {
            return Ok(None);
        }

        let free_head = linked_list(&lists, "free_object")?.head_address;
        let shell_address = self.read_word(free_head)?;
        if shell_address == 0 {
            return Ok(None);
        }
        object_table_for_address(&layout, shell_address)?;

        self.write_object_bytes(&layout, shell_address, "OBJX", &[screen_x])?;
        self.write_object_bytes(&layout, shell_address, "OBJY", &[source_y])?;
        self.write_object_word(
            &layout,
            shell_address,
            "OX16",
            u16::from_be_bytes([screen_x, source_y]),
        )?;
        self.write_object_word(
            &layout,
            shell_address,
            "OY16",
            u16::from_be_bytes([source_y, screen_x]),
        )?;
        self.write_object_word(&layout, shell_address, "OBJID", owner_address)?;
        self.write_object_word(&layout, shell_address, "OXV", 0)?;
        self.write_object_word(&layout, shell_address, "OYV", 0)?;
        self.write_object_word(
            &layout,
            shell_address,
            "OBJCOL",
            descriptor.output_routine_address,
        )?;
        self.write_object_word(&layout, shell_address, "OPICT", descriptor.picture_address)?;
        self.write_object_word(
            &layout,
            shell_address,
            "OCVECT",
            descriptor.kill_routine_address,
        )?;
        self.write_object_bytes(
            &layout,
            shell_address,
            "ODATA",
            &[RED_LABEL_SHELL_LIMIT, RED_LABEL_SHELL_LIMIT],
        )?;

        let next_free = self.read_object_word(&layout, shell_address, "OLINK")?;
        self.write_word(free_head, next_free)?;
        let shell_head = linked_list(&lists, "shell_object")?.head_address;
        let old_shell_head = self.read_word(shell_head)?;
        self.write_object_word(&layout, shell_address, "OLINK", old_shell_head)?;
        self.write_field_byte(&layout, "base_page", "BMBCNT", shell_count.wrapping_add(1))?;
        self.write_word(shell_head, shell_address)?;

        Ok(Some(RedLabelCreatedShell {
            shell_address,
            firing_object_address,
            owner_address,
            descriptor,
        }))
    }

    /// Source-shaped `SHSCAN`: walk `SPTR`, decrement live shell timers, and
    /// reclaim shells whose alive byte is clear or whose timer just reached
    /// zero.
    pub fn scan_shell_list(&mut self) -> Result<Vec<u16>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(&layout, "object")?;
        let shell_head = linked_list(&lists, "shell_object")?.head_address;
        let free_head = linked_list(&lists, "free_object")?.head_address;
        let mut previous_link_address = shell_head;
        let mut reclaimed = Vec::new();
        let mut scanned = 0;

        loop {
            let current = self.read_word(previous_link_address)?;
            if current == 0 {
                return Ok(reclaimed);
            }
            if scanned >= object.entries {
                return Err(String::from(
                    "red-label shell list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, current)?;

            let data_range = object_field_range_for_address(&layout, current, "ODATA")?;
            let alive = self.read_byte(data_range.start + 1)?;
            let should_reclaim = if alive == 0 {
                true
            } else {
                let timer = self.read_byte(data_range.start)?.wrapping_sub(1);
                self.write_byte(data_range.start, timer)?;
                timer == 0
            };

            if should_reclaim {
                let next = self.read_object_word(&layout, current, "OLINK")?;
                self.write_word(previous_link_address, next)?;
                let old_free = self.read_word(free_head)?;
                self.write_word(free_head, current)?;
                self.write_object_word(&layout, current, "OLINK", old_free)?;
                let shell_count = self.read_field_byte(&layout, "base_page", "BMBCNT")?;
                self.write_field_byte(&layout, "base_page", "BMBCNT", shell_count.wrapping_sub(1))?;
                reclaimed.push(current);
            } else {
                previous_link_address =
                    object_field_range_for_address(&layout, current, "OLINK")?.start;
            }
        }
    }

    /// Source-shaped `OSCAN`: walk active objects and move offscreen entries
    /// from `OPTR` to the inactive-object list at `IPTR`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3311-L3328>.
    pub fn scan_active_objects_for_offscreen(&mut self) -> Result<Vec<u16>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(&layout, "object")?;
        let active_head = linked_list(&lists, "active_object")?.head_address;
        let inactive_head = linked_list(&lists, "inactive_object")?.head_address;
        let left_boundary = self
            .read_field_word(&layout, "base_page", "BGL")?
            .wrapping_sub(RED_LABEL_OBJECT_INACTIVE_LEFT_BUFFER);
        self.write_field_word(&layout, "base_page", "XTEMP", left_boundary)?;

        let mut previous_link_address = active_head;
        let mut moved = Vec::new();
        let mut scanned = 0;
        loop {
            let current = self.read_word(previous_link_address)?;
            if current == 0 {
                return Ok(moved);
            }
            if scanned >= object.entries {
                return Err(String::from(
                    "red-label active object list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, current)?;

            let x_position = self.read_object_word(&layout, current, "OX16")?;
            if x_position.wrapping_sub(left_boundary) < RED_LABEL_OBJECT_ACTIVE_RANGE {
                previous_link_address =
                    object_field_range_for_address(&layout, current, "OLINK")?.start;
                continue;
            }

            let next = self.read_object_word(&layout, current, "OLINK")?;
            self.write_word(previous_link_address, next)?;
            let old_inactive = self.read_word(inactive_head)?;
            self.write_object_word(&layout, current, "OLINK", old_inactive)?;
            self.write_word(inactive_head, current)?;
            moved.push(current);
        }
    }

    /// Source-shaped `ISCAN`: advance inactive object positions by eight
    /// velocity ticks, wrap Y through the cabinet bounds, and move visible
    /// entries from `IPTR` back to `OPTR`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3333-L3373>.
    pub fn scan_inactive_objects_for_on_screen(&mut self) -> Result<Vec<u16>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(&layout, "object")?;
        let active_head = linked_list(&lists, "active_object")?.head_address;
        let inactive_head = linked_list(&lists, "inactive_object")?.head_address;
        let left_boundary = self
            .read_field_word(&layout, "base_page", "BGL")?
            .wrapping_sub(RED_LABEL_OBJECT_INACTIVE_LEFT_BUFFER);
        self.write_field_word(&layout, "base_page", "XTEMP", left_boundary)?;

        let mut previous_link_address = inactive_head;
        let mut moved = Vec::new();
        let mut scanned = 0;
        loop {
            let current = self.read_word(previous_link_address)?;
            if current == 0 {
                return Ok(moved);
            }
            if scanned >= object.entries {
                return Err(String::from(
                    "red-label inactive object list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, current)?;

            let new_y = self.inactive_object_next_y(&layout, current)?;
            self.write_object_word(&layout, current, "OY16", new_y)?;

            let new_x = self
                .read_object_word(&layout, current, "OXV")?
                .wrapping_shl(3)
                .wrapping_add(self.read_object_word(&layout, current, "OX16")?);
            self.write_object_word(&layout, current, "OX16", new_x)?;

            if new_x.wrapping_sub(left_boundary) >= RED_LABEL_OBJECT_ACTIVE_RANGE {
                previous_link_address =
                    object_field_range_for_address(&layout, current, "OLINK")?.start;
                continue;
            }

            let next = self.read_object_word(&layout, current, "OLINK")?;
            self.write_word(previous_link_address, next)?;
            let old_active = self.read_word(active_head)?;
            self.write_object_word(&layout, current, "OLINK", old_active)?;
            self.write_word(active_head, current)?;
            moved.push(current);
        }
    }

    /// Source-shaped `SCPROC` first stage: run `ISCAN`, then sleep to `SCP1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3298-L3301>.
    pub fn start_scanner_process_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let activated_objects = self.scan_inactive_objects_for_on_screen()?;
        let wakeup_address = red_label_routine_address("SCP1")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(RedLabelScannerProcessStep::InactiveScannedSleeping {
            process_address,
            activated_objects,
            wakeup_address,
        })
    }

    /// Source-shaped `SCP1`: run `OSCAN` and `SHSCAN`, then sleep to `SCP2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3302-L3304>.
    pub fn continue_scanner_process_object_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let deactivated_objects = self.scan_active_objects_for_offscreen()?;
        let reclaimed_shells = self.scan_shell_list()?;
        let wakeup_address = red_label_routine_address("SCP2")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(RedLabelScannerProcessStep::ActiveAndShellScannedSleeping {
            process_address,
            deactivated_objects,
            reclaimed_shells,
            wakeup_address,
        })
    }

    /// Source-shaped `SCP2`: select the scanner bank, run `SCNRV`, then sleep
    /// back to `SCPROC`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3305-L3307>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L1182-L1276>.
    pub fn continue_scanner_process_display_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let raster = self.draw_scanner_raster()?;
        let scanner_vector_address = raster.scanner_vector_address;
        let wakeup_address = red_label_routine_address("SCPROC")?;
        self.sleep_current_process(4, wakeup_address)?;
        Ok(RedLabelScannerProcessStep::ScannerDisplaySleeping {
            process_address,
            scanner_vector_address,
            raster,
            wakeup_address,
        })
    }

    /// Source-shaped bank-1 `SCNRV`: clear previous scanner blips, draw the
    /// miniature terrain, draw active/inactive object blips, store `SETEND`,
    /// and draw the current player blip.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L1182-L1276>.
    pub fn draw_scanner_raster(&mut self) -> Result<RedLabelScannerRaster, String> {
        let layout = red_label_ram_layout()?;
        let selected_map = 1;
        self.write_field_byte(&layout, "base_page", "MAPCR", selected_map)?;

        let object_erase_table = field_range(&layout, "scanner_object_erase", "SETAB")?;
        let setend = self.read_field_word(&layout, "base_page", "SETEND")?;
        let old_object_blip_words_cleared =
            self.clear_old_scanner_object_blips(object_erase_table.clone(), setend)?;
        let previous_player_blip_address = self.clear_previous_scanner_player_blip(setend)?;

        let scan_left = self
            .read_field_word(&layout, "base_page", "BGL")?
            .wrapping_sub(0x8000u16.wrapping_sub(150 * 32));
        self.write_field_word(&layout, "base_page", "XTEMP", scan_left)?;
        let terrain_blips = if self.read_field_byte(&layout, "base_page", "STATUS")? & 0x02 == 0 {
            Some(self.draw_scanner_terrain_blips(&layout, scan_left)?)
        } else {
            None
        };
        self.draw_scanner_bezel()?;

        let mut erase_cursor = object_erase_table.start;
        let active_object_blips =
            self.draw_scanner_object_blips(&layout, "OPTR", scan_left, &mut erase_cursor)?;
        let inactive_object_blips =
            self.draw_scanner_object_blips(&layout, "IPTR", scan_left, &mut erase_cursor)?;
        self.write_field_word(&layout, "base_page", "SETEND", erase_cursor)?;
        let player_blip =
            self.draw_scanner_player_blip(&layout, erase_cursor, previous_player_blip_address)?;

        Ok(RedLabelScannerRaster {
            scanner_vector_address: red_label_routine_address("SCNRV")?,
            selected_map,
            old_object_blip_words_cleared,
            terrain_blips,
            active_object_blips,
            inactive_object_blips,
            setend: erase_cursor,
            player_blip,
        })
    }

    pub(super) fn clear_old_scanner_object_blips(
        &mut self,
        object_erase_table: std::ops::Range<u16>,
        setend: u16,
    ) -> Result<u16, String> {
        if setend != 0 && (setend < object_erase_table.start || setend > object_erase_table.end) {
            return Err(format!(
                "red-label SETEND 0x{setend:04X} is outside SETAB 0x{:04X}..0x{:04X}",
                object_erase_table.start, object_erase_table.end
            ));
        }

        let mut cleared = 0u16;
        let mut cursor = object_erase_table.start;
        while cursor < setend {
            let chunk_end = cursor
                .checked_add(8)
                .ok_or_else(|| String::from("red-label SETAB clear cursor overflowed"))?;
            if chunk_end > object_erase_table.end {
                return Err(format!(
                    "red-label SETAB clear chunk 0x{cursor:04X} exceeds table end 0x{:04X}",
                    object_erase_table.end
                ));
            }
            for offset in [0u16, 2, 4, 6] {
                let screen_address = self.read_word(cursor + offset)?;
                self.write_word(screen_address, 0)?;
                cleared = cleared.wrapping_add(1);
            }
            cursor = chunk_end;
        }
        Ok(cleared)
    }

    pub(super) fn clear_previous_scanner_player_blip(
        &mut self,
        setend: u16,
    ) -> Result<Option<u16>, String> {
        let previous_screen_address = self.read_word(setend)?;
        if previous_screen_address == 0 {
            return Ok(None);
        }

        self.write_word(previous_screen_address, 0)?;
        self.write_byte(screen_offset(previous_screen_address, 2)?, 0)?;
        self.write_word(previous_screen_address.wrapping_sub(0x0100), 0)?;
        Ok(Some(previous_screen_address))
    }

    pub(super) fn draw_scanner_terrain_blips(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        scan_left: u16,
    ) -> Result<Vec<RedLabelScannerTerrainBlip>, String> {
        let terrain_erase_table = field_range(layout, "scanner_terrain_erase", "STETAB")?;
        let mini_terrain = red_label_terrain_data_table("MTERR")?;
        let first_record = usize::from(scan_left.to_be_bytes()[0] >> 2);
        let start = first_record * 3;
        let required_end = start + 64 * 3;
        if required_end > mini_terrain.bytes.len() {
            return Err(format!(
                "red-label SCNRV MTERR slice {}..{} exceeds {} byte(s)",
                start,
                required_end,
                mini_terrain.bytes.len()
            ));
        }

        let mut blips = Vec::with_capacity(64);
        let mut row = RED_LABEL_SCANNER_ADDRESS.to_be_bytes()[0];
        for index in 0..64usize {
            let erase_table_address = terrain_erase_table.start + index as u16 * 2;
            let old_screen_address = self.read_word(erase_table_address)?;
            self.write_word(old_screen_address, 0)?;

            let record = start + index * 3;
            let screen_address = u16::from_be_bytes([row, mini_terrain.bytes[record]]);
            let word = u16::from_be_bytes([
                mini_terrain.bytes[record + 1],
                mini_terrain.bytes[record + 2],
            ]);
            self.write_word(erase_table_address, screen_address)?;
            self.write_word(screen_address, word)?;
            blips.push(RedLabelScannerTerrainBlip {
                erase_table_address,
                old_screen_address,
                screen_address,
                word,
            });
            row = row.wrapping_add(1);
        }
        Ok(blips)
    }

    pub(super) fn draw_scanner_bezel(&mut self) -> Result<(), String> {
        let upper_left = u16::from(RED_LABEL_SCANNER_HEIGHT) + 0x4C01;
        self.write_word(upper_left, 0x9090)?;
        self.write_word(screen_offset(upper_left, 0x1D)?, 0x9090)?;

        let lower_left = u16::from(RED_LABEL_SCANNER_HEIGHT) + 0x5301;
        self.write_word(lower_left, 0x0909)?;
        self.write_word(screen_offset(lower_left, 0x1D)?, 0x0909)
    }

    pub(super) fn draw_scanner_object_blips(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        list_field: &str,
        scan_left: u16,
        erase_cursor: &mut u16,
    ) -> Result<Vec<RedLabelScannerObjectBlip>, String> {
        let object_erase_table = field_range(layout, "scanner_object_erase", "SETAB")?;
        let object_table = table_descriptor(layout, "object")?;
        let mut object_address = self.read_field_word(layout, "runtime_pointers", list_field)?;
        let mut blips = Vec::new();

        for _ in 0..object_table.entries {
            if object_address == 0 {
                return Ok(blips);
            }
            object_table_for_address(layout, object_address)?;
            let next_cursor = erase_cursor
                .checked_add(2)
                .ok_or_else(|| String::from("red-label SETAB object cursor overflowed"))?;
            if next_cursor > object_erase_table.end {
                return Err(format!(
                    "red-label SETAB cannot store scanner blip for object 0x{object_address:04X}"
                ));
            }

            let screen_address =
                self.scanner_object_screen_address(layout, object_address, scan_left)?;
            let color = self.read_object_word(layout, object_address, "OBJCOL")?;
            self.write_word(*erase_cursor, screen_address)?;
            self.write_word(screen_address, color)?;
            blips.push(RedLabelScannerObjectBlip {
                object_address,
                erase_table_address: *erase_cursor,
                screen_address,
                color,
            });
            *erase_cursor = next_cursor;
            object_address = self.read_object_word(layout, object_address, "OLINK")?;
        }

        Err(format!(
            "red-label {list_field} object list did not terminate within object table size"
        ))
    }

    pub(super) fn scanner_object_screen_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        scan_left: u16,
    ) -> Result<u16, String> {
        let x_delta = self
            .read_object_word(layout, object_address, "OX16")?
            .wrapping_sub(scan_left);
        let x_byte = x_delta.to_be_bytes()[0] >> 2;
        let y_byte = self
            .read_object_word(layout, object_address, "OY16")?
            .to_be_bytes()[0]
            >> 3;
        Ok(u16::from_be_bytes([x_byte, y_byte]).wrapping_add(RED_LABEL_SCANNER_ADDRESS - 1))
    }

    pub(super) fn draw_scanner_player_blip(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        erase_table_address: u16,
        previous_screen_address: Option<u16>,
    ) -> Result<RedLabelScannerPlayerBlip, String> {
        let object_erase_table = field_range(layout, "scanner_object_erase", "SETAB")?;
        if erase_table_address + 2 > object_erase_table.end {
            return Err(format!(
                "red-label SETAB cannot store scanner player blip at 0x{erase_table_address:04X}"
            ));
        }

        let [x, y] = self
            .read_field_word(layout, "base_page", "PLAXC")?
            .to_be_bytes();
        let screen_address = u16::from_be_bytes([x >> 4, y >> 3])
            .wrapping_add(0x4B00 + u16::from(RED_LABEL_SCANNER_HEIGHT) - 1);
        self.write_word(erase_table_address, screen_address)?;
        self.write_word(screen_address, 0x9099)?;
        self.write_byte(screen_offset(screen_address, 2)?, 0x90)?;
        self.write_byte(screen_address.wrapping_sub(0x00FF), 0x09)?;

        Ok(RedLabelScannerPlayerBlip {
            erase_table_address,
            previous_screen_address,
            screen_address,
        })
    }

    /// Source-shaped visible `PLSTRT` entry after the machine-level `INIT20`
    /// color/RNG refresh: reset object lists through `OINIT`, kill
    /// non-current/non-coin processes, set gameplay status, then either wait
    /// for coin counters or run the source `PINIT`/`NEWP PLSTR3` handoff.
    /// The full `INIT20` ordering remains owned by the higher-level machine
    /// wrapper because it consumes `RAND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1208-L1226>.
    pub fn start_player_start_current_process(&mut self) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_address = self.current_process_address(&layout)?;
        self.initialize_object_lists(&layout, &lists)?;
        self.finish_player_start_entry_after_init20(&layout, &lists, process_address)
    }

    pub(super) fn initialize_object_lists_from_embedded_layout(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        self.initialize_object_lists(&layout, &lists)
    }

    pub(super) fn start_player_start_after_init20_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_address = self.current_process_address(&layout)?;
        self.finish_player_start_entry_after_init20(&layout, &lists, process_address)
    }

    pub(super) fn finish_player_start_entry_after_init20(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
        process_address: u16,
    ) -> Result<RedLabelPlayerStart, String> {
        let genocide = self.genocide_other_processes()?;
        let status = 0x7F;
        self.write_field_byte(layout, "base_page", "STATUS", status)?;
        self.finish_player_start_entry_after_coin_counters(
            layout,
            lists,
            process_address,
            status,
            genocide.killed_processes,
        )
    }

    /// Source-shaped `PLST1A`: keep sleeping while either coin counter is
    /// active, otherwise run the `PINIT`/`NEWP PLSTR3` handoff.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1220-L1226>.
    pub fn continue_player_start_after_coin_counters_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_address = self.current_process_address(&layout)?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        self.finish_player_start_entry_after_coin_counters(
            &layout,
            &lists,
            process_address,
            status,
            Vec::new(),
        )
    }

    /// Source-shaped visible `PLSTR3` / `PLSTR5` runtime initialization:
    /// select the cocktail screen hook when `PIA3` says the cabinet is flipped,
    /// initialize current-player runtime bytes, redraw the top display, create
    /// the source support processes, then sleep either for the two-player
    /// prompt or the post-`SCLR1` game-entry delay.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1227-L1311>.
    pub fn start_player_runtime_current_process(&mut self) -> Result<RedLabelPlayerStart, String> {
        self.start_player_runtime_current_process_with_pia3(0)
    }

    pub fn start_player_runtime_current_process_with_pia3(
        &mut self,
        pia3_input: u8,
    ) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 0)?;
        let screen_switch = self.player_runtime_screen_switch(&layout, pia3_input)?;
        let mut runtime = self.initialize_current_player_runtime_state(&layout)?;
        runtime.screen_switch = screen_switch;

        let mut support_processes = Vec::with_capacity(6);
        for routine in ["SCPROC", "COLR", "FLPUP", "THPROC", "CBOMB", "TIECOL"] {
            support_processes.push(self.make_process(
                red_label_routine_address(routine)?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?);
        }

        let pdf_flag = self.read_field_byte(&layout, "base_page", "PDFLG")?;
        let player_count = self.read_field_byte(&layout, "base_page", "PLRCNT")?;
        if pdf_flag != 0 && player_count.wrapping_sub(1) != 0 {
            let prompt_message = if runtime.current_player == 2 {
                red_label_message("PLYR2")?
            } else {
                red_label_message("PLYR1")?
            };
            self.write_message_text_block(
                &layout,
                RED_LABEL_PLAYER_START_PROMPT_SCREEN,
                prompt_message,
            )?;
            let wakeup_address = red_label_routine_address("PLS01")?;
            self.sleep_current_process(0x80, wakeup_address)?;
            return Ok(RedLabelPlayerStart::RuntimeSleeping {
                process_address,
                runtime,
                support_processes,
                prompt_display_required: true,
                prompt: Some(RedLabelBonusTextCall {
                    vector_address: prompt_message.vector_address,
                    screen_address: RED_LABEL_PLAYER_START_PROMPT_SCREEN,
                }),
                screen_clear: None,
                target_count: None,
                status: None,
                wakeup_address,
                sleep_time: 0x80,
            });
        }

        let screen = self.sleep_player_start_screen_current_process(&layout, process_address)?;
        let RedLabelPlayerStart::ScreenClearedSleeping {
            screen_clear,
            target_count,
            status,
            wakeup_address,
            ..
        } = screen
        else {
            unreachable!("PLS01 helper only returns ScreenClearedSleeping");
        };
        Ok(RedLabelPlayerStart::RuntimeSleeping {
            process_address,
            runtime,
            support_processes,
            prompt_display_required: false,
            prompt: None,
            screen_clear: Some(screen_clear),
            target_count: Some(target_count),
            status: Some(status),
            wakeup_address,
            sleep_time: 0x60,
        })
    }

    pub(super) fn player_runtime_screen_switch(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        pia3_input: u8,
    ) -> Result<Option<RedLabelScreenSwitch>, String> {
        if pia3_input & RED_LABEL_PIA3_COCKTAIL_BIT == 0 {
            return Ok(None);
        }

        let screen_clear = self.clear_screen_ram()?;
        let current_player = self.read_field_byte(layout, "base_page", "CURPLR")?;
        let mut screen_switch = if current_player == 1 {
            self.screen_switch_player_one_with_context(
                layout,
                pia3_input,
                Some(screen_clear),
                true,
            )?
        } else {
            self.screen_switch_player_two_with_context(
                layout,
                pia3_input,
                Some(screen_clear),
                true,
            )?
        };
        self.write_field_byte(layout, "base_page", "PIA21", 0xFF)?;
        self.write_field_byte(layout, "base_page", "PIA22", 0xFF)?;
        screen_switch.stick_history_reset = true;
        Ok(Some(screen_switch))
    }

    pub fn screen_switch_player_one(&mut self) -> Result<RedLabelScreenSwitch, String> {
        let layout = red_label_ram_layout()?;
        self.screen_switch_player_one_with_context(&layout, 0, None, false)
    }

    pub fn screen_switch_player_two(
        &mut self,
        pia3_input: u8,
    ) -> Result<RedLabelScreenSwitch, String> {
        let layout = red_label_ram_layout()?;
        self.screen_switch_player_two_with_context(&layout, pia3_input, None, false)
    }

    pub(super) fn screen_switch_player_one_with_context(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        pia3_input: u8,
        screen_clear: Option<RedLabelScreenClear>,
        stick_history_reset: bool,
    ) -> Result<RedLabelScreenSwitch, String> {
        let map_before = self.read_field_byte(layout, "base_page", "MAPCR")?;
        let hardware_map_before = self.hardware_map;
        self.apply_screen_switch(
            layout,
            RedLabelScreenSwitchContext {
                routine: RedLabelScreenSwitchRoutine::PlayerOne,
                pia3_input,
                cocktail_detected: false,
                screen_clear,
                stick_history_reset,
                map_before,
                hardware_map_before,
                hardware_map_writes: Vec::new(),
            },
        )
    }

    pub(super) fn screen_switch_player_two_with_context(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        pia3_input: u8,
        screen_clear: Option<RedLabelScreenClear>,
        stick_history_reset: bool,
    ) -> Result<RedLabelScreenSwitch, String> {
        let map_before = self.read_field_byte(layout, "base_page", "MAPCR")?;
        let hardware_map_before = self.hardware_map;
        self.write_field_byte(layout, "base_page", "MAPCR", 0)?;
        let mut pre_read_writes = Vec::new();
        self.write_hardware_map(0, &mut pre_read_writes);
        self.apply_screen_switch(
            layout,
            RedLabelScreenSwitchContext {
                routine: RedLabelScreenSwitchRoutine::PlayerTwo,
                pia3_input,
                cocktail_detected: pia3_input & RED_LABEL_PIA3_COCKTAIL_BIT != 0,
                screen_clear,
                stick_history_reset,
                map_before,
                hardware_map_before,
                hardware_map_writes: pre_read_writes,
            },
        )
    }

    pub(super) fn apply_screen_switch(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        mut context: RedLabelScreenSwitchContext,
    ) -> Result<RedLabelScreenSwitch, String> {
        let (mode, irq_hook_target, pia3_control, watchdog_value) = if context.cocktail_detected {
            (
                RedLabelIrqMode::Inverted,
                RED_LABEL_IRQB_ADDRESS,
                RED_LABEL_P2SW_PIA3_CONTROL,
                RED_LABEL_INVERTED_WATCHDOG_DATA,
            )
        } else {
            (
                RedLabelIrqMode::Normal,
                RED_LABEL_IRQ_ADDRESS,
                RED_LABEL_P1SW_PIA3_CONTROL,
                RED_LABEL_NORMAL_WATCHDOG_DATA,
            )
        };

        let irq_hook = field_range(layout, "base_page", "IRQHK")?;
        self.write_word(irq_hook.start + 1, irq_hook_target)?;
        self.write_field_byte(layout, "base_page", "MAPCR", 0)?;
        self.write_hardware_map(0, &mut context.hardware_map_writes);
        self.write_byte(irq_hook.start, RED_LABEL_IRQ_JUMP_OPCODE)?;
        self.write_field_byte(layout, "base_page", "MAPCR", context.map_before)?;
        self.write_hardware_map(context.map_before, &mut context.hardware_map_writes);

        let map_after = self.read_field_byte(layout, "base_page", "MAPCR")?;
        Ok(RedLabelScreenSwitch {
            routine: context.routine,
            pia3_input: context.pia3_input,
            cocktail_detected: context.cocktail_detected,
            mode,
            irq_hook_opcode: RED_LABEL_IRQ_JUMP_OPCODE,
            irq_hook_target,
            pia3_control,
            watchdog_value,
            map_before: context.map_before,
            map_after,
            hardware_map_before: context.hardware_map_before,
            hardware_map_writes: context.hardware_map_writes,
            hardware_map_after: self.hardware_map,
            screen_clear: context.screen_clear,
            stick_history_reset: context.stick_history_reset,
        })
    }

    /// Source-shaped `PLS01`: clear active screen, update terrain/player
    /// status from the current player's target count, then sleep to `PLS1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1304-L1311>.
    pub fn continue_player_start_screen_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.sleep_player_start_screen_current_process(&layout, process_address)
    }

    /// Source-shaped visible `PLS1`: run the RAM-visible `PLRES` body, run
    /// `STCHK`, clear `PDFLG`, and return to the game executive.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1312-L1324>.
    pub fn finish_player_start_current_process(&mut self) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let entry_registers = red_label_source_entry_registers_for_routine(
            process_address,
            red_label_routine_address("PLS1")?,
        )?;
        self.finish_player_start_current_process_with_entry_registers(entry_registers)
    }

    pub(super) fn finish_player_start_current_process_with_entry_registers(
        &mut self,
        entry_registers: RedLabelCpuRegisters,
    ) -> Result<RedLabelPlayerStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        if entry_registers.u != Some(process_address) {
            return Err(format!(
                "red-label PLS1 entry U {:?} does not match CRPROC 0x{process_address:04X}",
                entry_registers.u
            ));
        }
        let restore = self.restore_player_world_current_process(&layout, entry_registers)?;
        let status = self.write_terrain_status(&layout, 0)?;
        let pdf_flag_before = self.read_field_byte(&layout, "base_page", "PDFLG")?;
        self.write_field_byte(&layout, "base_page", "PDFLG", 0)?;
        self.write_process_word(
            &layout,
            process_address,
            "PADDR",
            red_label_routine_address("GEXEC")?,
        )?;
        self.write_process_byte(&layout, process_address, "PTIME", 1)?;
        Ok(RedLabelPlayerStart::GameExecReady {
            process_address,
            restore,
            status,
            pdf_flag_before,
        })
    }

    /// Source-shaped `BMBOUT`: erase the old shell footprint and draw the
    /// current six-byte bomb image selected by `BAX` and `OX16+1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2645-L2662>.
    pub fn output_bomb_shell(
        &mut self,
        shell_address: u16,
        old_screen_address: u16,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, shell_address)?;
        let image_pointer = self.bomb_image_pointer_for_shell(&layout, shell_address)?;
        self.clear_shell_video_footprint(old_screen_address)?;
        let new_screen_address = self.read_object_screen_address(&layout, shell_address)?;

        self.write_byte(
            new_screen_address,
            self.read_byte_at_offset(image_pointer, 0)?,
        )?;
        self.write_byte(
            screen_offset(new_screen_address, 1)?,
            self.read_byte_at_offset(image_pointer, 1)?,
        )?;
        self.write_byte(
            screen_offset(new_screen_address, 2)?,
            self.read_byte_at_offset(image_pointer, 2)?,
        )?;
        self.write_byte(
            screen_offset(new_screen_address, 0x100)?,
            self.read_byte_at_offset(image_pointer, 3)?,
        )?;
        self.write_byte(
            screen_offset(new_screen_address, 0x101)?,
            self.read_byte_at_offset(image_pointer, 4)?,
        )?;
        self.write_byte(
            screen_offset(new_screen_address, 0x102)?,
            self.read_byte_at_offset(image_pointer, 5)?,
        )
    }

    /// Source-shaped `FBOUT`: erase the old shell footprint and draw the
    /// four-byte fireball slice selected by `FBX` and `OX16+1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2671-L2698>.
    pub fn output_fireball_shell(
        &mut self,
        shell_address: u16,
        old_screen_address: u16,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, shell_address)?;
        let image_pointer = self.read_field_word(&layout, "base_page", "FBX")?;
        let flavor = self
            .read_object_word(&layout, shell_address, "OX16")?
            .to_be_bytes()[1];
        self.clear_shell_video_footprint(old_screen_address)?;
        let new_screen_address = self.read_object_screen_address(&layout, shell_address)?;
        let first = self.read_byte_at_offset(image_pointer, 0)?;
        let second = self.read_byte_at_offset(image_pointer, 1)?;
        let third = self.read_byte_at_offset(image_pointer, 2)?;
        let fourth = self.read_byte_at_offset(image_pointer, 3)?;

        if flavor & 0x80 == 0 {
            self.write_byte(new_screen_address, first & 0x0F)?;
            self.write_byte(screen_offset(new_screen_address, 1)?, second)?;
            self.write_byte(screen_offset(new_screen_address, 2)?, third & 0x0F)?;
            self.write_byte(screen_offset(new_screen_address, 0x101)?, fourth & 0xF0)
        } else {
            self.write_byte(screen_offset(new_screen_address, 1)?, second & 0x0F)?;
            self.write_byte(screen_offset(new_screen_address, 0x102)?, first & 0xF0)?;
            self.write_byte(screen_offset(new_screen_address, 0x100)?, third & 0xF0)?;
            self.write_byte(screen_offset(new_screen_address, 0x101)?, fourth)
        }
    }

    /// Dispatches the `OBJCOL` routine address returned by `SHELL` to the
    /// translated output callback with the same entry point.
    pub fn dispatch_shell_output_step(
        &mut self,
        step: RedLabelShellStep,
    ) -> Result<Option<RedLabelShellOutputRoutine>, String> {
        let RedLabelShellStep::Output {
            shell_address,
            old_screen_address,
            output_routine_address,
            ..
        } = step
        else {
            return Ok(None);
        };

        if output_routine_address == red_label_routine_address("BMBOUT")? {
            self.output_bomb_shell(shell_address, old_screen_address)?;
            return Ok(Some(RedLabelShellOutputRoutine::Bomb));
        }
        if output_routine_address == red_label_routine_address("FBOUT")? {
            self.output_fireball_shell(shell_address, old_screen_address)?;
            return Ok(Some(RedLabelShellOutputRoutine::Fireball));
        }

        Err(format!(
            "red-label shell OBJCOL routine 0x{output_routine_address:04X} for shell 0x{shell_address:04X} is not translated"
        ))
    }

    pub fn dispatch_shell_output_steps(
        &mut self,
        steps: &[RedLabelShellStep],
    ) -> Result<Vec<RedLabelShellOutputRoutine>, String> {
        let mut routines = Vec::new();
        for step in steps.iter().copied() {
            if let Some(routine) = self.dispatch_shell_output_step(step)? {
                routines.push(routine);
            }
        }
        Ok(routines)
    }

    /// Source-shaped `SCORE`: add a packed BCD score word to the current
    /// player's `PSCOR` field, refresh the visible score digits through the
    /// `SCRTRN` tail, and redraw stock icons before loading the replay sound
    /// when the replay threshold awards an extra ship and smart bomb.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L474-L535>.
    pub fn score_current_player(
        &mut self,
        score_word: u16,
    ) -> Result<RedLabelScoreOutcome, String> {
        let layout = red_label_ram_layout()?;
        let player_index = self.current_player_entry_index(&layout)?;
        let player_number = if player_index == 0 { 1 } else { 2 };
        let score_range = ram_field(&layout, "player", "PSCOR")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSCOR range is invalid"))?;
        let (addend, score_offset) = score_addend(score_word)?;

        let score_flag = self.read_field_byte(&layout, "base_page", "SCRFLG")?;
        self.write_field_byte(
            &layout,
            "base_page",
            "SCRFLG",
            score_flag.wrapping_shl(1) | 1,
        )?;
        self.write_field_word(&layout, "base_page", "XTEMP", addend)?;
        self.add_bcd_word_to_score(score_range.start, score_offset, addend)?;
        let bonus_awarded = self.apply_score_replay_award(&layout, player_index, score_range)?;
        let sound_loaded = if bonus_awarded {
            self.display_laser_stocks(&layout)?;
            self.display_smart_bomb_stocks(&layout)?;
            self.load_sound_table_by_label("RPSND")?
        } else {
            None
        };
        self.transfer_score_digits(&layout, player_number)?;

        Ok(RedLabelScoreOutcome {
            player_number,
            bonus_awarded,
            sound_loaded,
        })
    }

    pub fn player_score_value(&self, player_number: u8) -> Result<u32, String> {
        let layout = red_label_ram_layout()?;
        let player_index = match player_number {
            1 => 0,
            2 => 1,
            other => {
                return Err(format!(
                    "red-label player number {other} is outside the player table"
                ));
            }
        };
        let score_range = ram_field(&layout, "player", "PSCOR")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSCOR range is invalid"))?;
        let digits = self.read_fixed_bytes::<3>(score_range.start + 1)?;
        Ok(bcd_digits_to_u32(&digits))
    }

    pub(super) fn write_player_score_value(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_index: u16,
        score: u32,
    ) -> Result<(), String> {
        let score_digits = high_score_bcd_bytes(score.min(RED_LABEL_HIGH_SCORE_MAX_SCORE))?;
        self.write_field(
            layout,
            "player",
            "PSCOR",
            player_index,
            &[0, score_digits[0], score_digits[1], score_digits[2]],
        )
    }

    pub(super) fn write_player_runtime_snapshot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_index: u16,
        player: PlayerState,
        wave: u8,
    ) -> Result<(), String> {
        self.write_field(layout, "player", "PWAV", player_index, &[wave])?;
        self.write_field(layout, "player", "PLAS", player_index, &[player.lives])?;
        self.write_field(
            layout,
            "player",
            "PSBC",
            player_index,
            &[player.smart_bombs],
        )?;
        self.write_field_word(layout, "base_page", "PLAX16", fixed16_source_word(player.x))?;
        self.write_field_word(layout, "base_page", "PLAY16", fixed16_source_word(player.y))?;
        let x_velocity = fixed16_source_24(player.xv).to_be_bytes();
        self.write_field(
            layout,
            "base_page",
            "PLAXV",
            0,
            &[x_velocity[1], x_velocity[2], x_velocity[3]],
        )?;
        self.write_field_word(layout, "base_page", "PLAYV", fixed16_source_word(player.yv))?;
        self.write_field_word(
            layout,
            "base_page",
            "PLADIR",
            match player.facing {
                Facing::Right => 0x0300,
                Facing::Left => 0xFD00,
            },
        )
    }

    pub(super) fn player_trace_score_value(&self, player_number: u8) -> Result<u32, String> {
        let layout = red_label_ram_layout()?;
        let player_index = match player_number {
            1 => 0,
            2 => 1,
            other => {
                return Err(format!(
                    "red-label player number {other} is outside the player table"
                ));
            }
        };
        let score_range = ram_field(&layout, "player", "PSCOR")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSCOR range is invalid"))?;
        let digits = self.read_fixed_bytes::<4>(score_range.start)?;
        Ok(bcd_digits_to_u32(&digits))
    }

    pub fn player_smart_bombs_value(&self, player_number: u8) -> Result<u8, String> {
        let layout = red_label_ram_layout()?;
        let player_index = match player_number {
            1 => 0,
            2 => 1,
            other => {
                return Err(format!(
                    "red-label player number {other} is outside the player table"
                ));
            }
        };
        let smart_bomb_range = ram_field(&layout, "player", "PSBC")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSBC range is invalid"))?;
        self.read_byte(smart_bomb_range.start)
    }

    pub fn trace_state(&self) -> Result<RedLabelTraceState, String> {
        let layout = red_label_ram_layout()?;
        let player_one_index = 0;
        Ok(RedLabelTraceState {
            player_one_score: self.player_trace_score_value(1)?,
            player_two_score: self.player_trace_score_value(2)?,
            wave: self.read_player_field_byte(&layout, player_one_index, "PWAV")?,
            lives: self.read_player_field_byte(&layout, player_one_index, "PLAS")?,
            smart_bombs: self.read_player_field_byte(&layout, player_one_index, "PSBC")?,
            seed: self.read_field_byte(&layout, "base_page", "SEED")?,
            hseed: self.read_field_byte(&layout, "base_page", "HSEED")?,
            lseed: self.read_field_byte(&layout, "base_page", "LSEED")?,
        })
    }

    /// Source-shaped `BKIL`: score a bomb collision, unlink the shell from
    /// `SPTR`, erase its 2x3 footprint, convert its coordinates to explosion
    /// coordinates, attach `BXPIC`, and load `AHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2700-L2719>.
    pub fn kill_bomb_shell_collision(
        &mut self,
        shell_address: u16,
    ) -> Result<RedLabelBombCollision, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, shell_address)?;
        let old_screen_address = self.read_object_screen_address(&layout, shell_address)?;
        let score = self.score_current_player(0x0025)?;
        let shell_count = self.read_field_byte(&layout, "base_page", "BMBCNT")?;
        self.write_field_byte(&layout, "base_page", "BMBCNT", shell_count.wrapping_sub(1))?;
        self.kill_shell_cell(shell_address)?;
        self.clear_shell_video_footprint(old_screen_address)?;

        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let explosion_x = (self.read_object_word(&layout, shell_address, "OX16")? >> 2)
            .wrapping_add(background_left);
        self.write_object_word(&layout, shell_address, "OX16", explosion_x)?;

        let [y, fraction] = self
            .read_object_word(&layout, shell_address, "OY16")?
            .to_be_bytes();
        self.write_object_word(
            &layout,
            shell_address,
            "OY16",
            u16::from_be_bytes([y.wrapping_sub(2), fraction]),
        )?;

        let explosion_picture_address = red_label_object_picture_address("BXPIC")?;
        self.write_object_word(&layout, shell_address, "OPICT", explosion_picture_address)?;
        let sound_loaded = self.load_sound_table_by_label("AHSND")?;

        Ok(RedLabelBombCollision {
            shell_address,
            old_screen_address,
            explosion_picture_address,
            score,
            sound_loaded,
        })
    }

    /// Source-shaped `SCZKIL`: decrement `SCZCNT`, then run the `KILP`
    /// score/explosion/sound path with score `$0115` and `SCHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L624-L626>.
    pub fn kill_schizoid_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelEnemyKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let count = self
            .read_field_byte(&layout, "enemy_runtime", "SCZCNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "SCZCNT", count)?;
        self.kill_positioned_object_with_process_score_and_sound(object_address, 0x0115, "SCHSND")
    }

    /// Source-shaped `UFOKIL`: decrement `UFOCNT`, then run the `KILP`
    /// score/explosion/sound path with score `$0120` and `UFHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L80-L82>.
    pub fn kill_ufo_collision(&mut self, object_address: u16) -> Result<RedLabelEnemyKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let count = self
            .read_field_byte(&layout, "enemy_runtime", "UFOCNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "UFOCNT", count)?;
        self.kill_positioned_object_with_process_score_and_sound(object_address, 0x0120, "UFHSND")
    }

    /// Source-shaped normal `LKILL`: decrement `LNDCNT`, then run the `KILP`
    /// score/explosion/sound path with score `$0115` and `LHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L920-L922>.
    pub fn kill_lander_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelEnemyKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let count = self
            .read_field_byte(&layout, "enemy_runtime", "LNDCNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "LNDCNT", count)?;
        self.kill_positioned_object_with_process_score_and_sound(object_address, 0x0115, "LHSND")
    }

    /// Source-shaped kidnapping `LKIL1`: if the lander still has a passenger,
    /// start an `AFALL` process for that astronaut, load `ASCSND`, then fall
    /// through the normal `LKILL` lander kill path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L905-L922>.
    pub fn kill_kidnapping_lander_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelKidnappingLanderKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let lander_process_address = self.read_object_word(&layout, object_address, "OBJID")?;
        process_table_for_address(&layout, lander_process_address)?;
        let target_slot_address =
            self.read_process_data_word(&layout, lander_process_address, "PD4")?;
        let passenger_release = if self.read_word(target_slot_address)? == 0 {
            None
        } else {
            let falling_process = self.make_process(
                red_label_routine_address("AFALL")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            let passenger_object_address =
                self.read_process_data_word(&layout, lander_process_address, "PD2")?;
            object_table_for_address(&layout, passenger_object_address)?;
            self.write_process_data_word(
                &layout,
                falling_process.process_address,
                "PD",
                passenger_object_address,
            )?;
            let sound_loaded = self.load_sound_table_by_label("ASCSND")?;
            self.write_object_word(&layout, passenger_object_address, "OYV", 0)?;
            self.write_object_word(
                &layout,
                passenger_object_address,
                "OBJID",
                falling_process.process_address,
            )?;
            Some(RedLabelKidnappingPassengerRelease {
                falling_process,
                passenger_object_address,
                target_slot_address,
                sound_loaded,
            })
        };

        let lander = self.kill_lander_collision(object_address)?;
        Ok(RedLabelKidnappingLanderKill {
            passenger_release,
            lander,
        })
    }

    /// Source-shaped `TIEKIL`: run `KILO`, decrement `TIECNT`, clear the
    /// object's squad slot in its owning super-process, decrement `PD+8`, and
    /// kill that super-process when the squad count reaches zero.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1120-L1128>.
    pub fn kill_tie_collision(&mut self, object_address: u16) -> Result<RedLabelTieKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let process_address = self.read_object_word(&layout, object_address, "OBJID")?;
        let object =
            self.kill_positioned_object_score_and_sound(object_address, 0x0125, "TIHSND")?;

        let active_count = self
            .read_field_byte(&layout, "enemy_runtime", "TIECNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "TIECNT", active_count)?;

        let process_table = process_table_for_address(&layout, process_address)?;
        let data_range =
            process_field_range_for_address(&layout, process_table, process_address, "PDATA")?;
        let mut cleared_squad_slot_address = None;
        for slot_index in 0..4 {
            let slot_address = data_range.start + slot_index * 2;
            if self.read_word(slot_address)? == object_address {
                self.write_word(slot_address, 0)?;
                cleared_squad_slot_address = Some(slot_address);
                break;
            }
        }
        let cleared_squad_slot_address = cleared_squad_slot_address.ok_or_else(|| {
            format!(
                "red-label TIEKIL object 0x{object_address:04X} was not in process 0x{process_address:04X} squad slots"
            )
        })?;

        let squad_count_address = data_range.start + 8;
        let squad_remaining = self.read_byte(squad_count_address)?.wrapping_sub(1);
        self.write_byte(squad_count_address, squad_remaining)?;
        let killed_process = if squad_remaining == 0 {
            Some(RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address: self.kill_process(process_address)?,
            })
        } else {
            None
        };

        Ok(RedLabelTieKill {
            object,
            object_process_address: process_address,
            active_count,
            cleared_squad_slot_address,
            squad_remaining,
            killed_process,
        })
    }

    /// Source-shaped `PRBKIL`: run `KILO`, choose a bounded mini-swarmer count
    /// through `RMAX`, spawn mini swarmers through `MMSW`, then decrement
    /// `PRBCNT`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L119-L125>.
    pub fn kill_probe_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelProbeKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let object =
            self.kill_positioned_object_score_and_sound(object_address, 0x0210, "PRHSND")?;
        let requested_swarmer_count = self.advance_red_label_rmax(&layout, 6)?;
        let spawned_swarmers =
            self.make_mini_swarmers_from_center(&layout, object_address, requested_swarmer_count)?;
        let active_probe_count = self
            .read_field_byte(&layout, "enemy_runtime", "PRBCNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "PRBCNT", active_probe_count)?;

        Ok(RedLabelProbeKill {
            object,
            requested_swarmer_count,
            spawned_swarmers,
            active_probe_count,
        })
    }

    /// Source-shaped `MSWKIL`: decrement `SWCNT`, run `KILOFF`, kill the
    /// owning process, convert the object to `SWXP1`, start `XSVCT`, score, and
    /// load `SWHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L176-L193>.
    pub fn kill_mini_swarmer_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelMiniSwarmerKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let active_swarmer_count = self
            .read_field_byte(&layout, "enemy_runtime", "SWCNT")?
            .wrapping_sub(1);
        self.write_field_byte(&layout, "enemy_runtime", "SWCNT", active_swarmer_count)?;
        let previous_object_link_address = self.kill_object_cell_offscreen(object_address)?;
        let process_address = self.read_object_word(&layout, object_address, "OBJID")?;
        let previous_process_link_address = self.kill_process(process_address)?;

        let adjusted_x16 = self
            .read_object_word(&layout, object_address, "OX16")?
            .wrapping_sub(0x0040);
        self.write_object_word(&layout, object_address, "OX16", adjusted_x16)?;
        let [y, fraction] = self
            .read_object_word(&layout, object_address, "OY16")?
            .to_be_bytes();
        let adjusted_y16 = u16::from_be_bytes([y.wrapping_sub(2), fraction]);
        self.write_object_word(&layout, object_address, "OY16", adjusted_y16)?;
        self.write_object_word(
            &layout,
            object_address,
            "OPICT",
            red_label_object_picture_address("SWXP1")?,
        )?;
        let explosion = self.start_explosion_for_object(object_address)?;
        let score = self.score_current_player(0x0115)?;
        let sound_loaded = self.load_sound_table_by_label("SWHSND")?;

        Ok(RedLabelMiniSwarmerKill {
            object_address,
            active_swarmer_count,
            previous_object_link_address,
            killed_process: RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address: previous_process_link_address,
            },
            adjusted_x16,
            adjusted_y16,
            explosion,
            score,
            sound_loaded,
        })
    }

    /// Source-shaped `ASTKIL`: ignore player collisions while `PCFLG` is set,
    /// otherwise clear the astronaut from `TLIST`, unlink and erase the object
    /// through `KILOFF`, switch it to `ASXP1`, start `XSVCT`, and load
    /// `AHSND`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L384-L436>.
    pub fn kill_astronaut_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelAstronautKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        if self.read_field_byte(&layout, "base_page", "PCFLG")? != 0 {
            return Ok(RedLabelAstronautKill::IgnoredPlayerCollision { object_address });
        }

        self.kill_astronaut_unchecked(&layout, object_address)
    }

    /// Source-shaped `AKIL1`: when a kidnapped/falling astronaut collides with
    /// the player, switch the owning fall process to `AFALL2`, optionally start
    /// the `P500` score-popup process, and load `ACSND`; when hit by a shot,
    /// run `ASTK1` and kill the owning fall process.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L398-L417>.
    pub fn collide_captured_astronaut(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelCapturedAstronautCollision, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let falling_process_address = self.read_object_word(&layout, object_address, "OBJID")?;
        if falling_process_address == 0 {
            return self
                .kill_astronaut_collision(object_address)
                .map(RedLabelCapturedAstronautCollision::NormalAstronaut);
        }
        process_table_for_address(&layout, falling_process_address)?;

        if self.read_field_byte(&layout, "base_page", "PCFLG")? == 0 {
            let astronaut_kill = self.kill_astronaut_unchecked(&layout, object_address)?;
            let previous_link_address = self.kill_process(falling_process_address)?;
            return Ok(RedLabelCapturedAstronautCollision::ShotKilled {
                object_address,
                falling_process_address,
                astronaut_kill,
                killed_process: RedLabelKilledProcess {
                    killed_process_address: falling_process_address,
                    previous_link_address,
                },
            });
        }

        let already_carried = self.read_process_word(&layout, falling_process_address, "PADDR")?
            == red_label_routine_address("AFALL2")?;
        let (score_process, sound_loaded) = if already_carried {
            (None, None)
        } else {
            let sound_loaded = self.load_sound_table_by_label("ACSND")?;
            let score_process = self.make_process(
                red_label_routine_address("P500")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            self.write_process_data_word(
                &layout,
                score_process.process_address,
                "PD",
                object_address,
            )?;
            (Some(score_process), sound_loaded)
        };
        self.write_process_word(
            &layout,
            falling_process_address,
            "PADDR",
            red_label_routine_address("AFALL2")?,
        )?;
        Ok(RedLabelCapturedAstronautCollision::PlayerCaught {
            object_address,
            falling_process_address,
            score_process,
            sound_loaded,
        })
    }

    pub(super) fn kill_astronaut_unchecked(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<RedLabelAstronautKill, String> {
        let (cleared_target_slot_address, active_astronaut_count, terrain_blow_process) =
            self.clear_astronaut_target_slot(layout, object_address)?;
        let previous_object_link_address = self.kill_object_cell_offscreen(object_address)?;
        self.write_object_word(
            layout,
            object_address,
            "OPICT",
            red_label_object_picture_address("ASXP1")?,
        )?;
        let adjusted_x16 = self
            .read_object_word(layout, object_address, "OX16")?
            .wrapping_sub(0x0040);
        self.write_object_word(layout, object_address, "OX16", adjusted_x16)?;
        let explosion = self.start_explosion_for_object(object_address)?;
        let sound_loaded = self.load_sound_table_by_label("AHSND")?;

        Ok(RedLabelAstronautKill::Killed {
            object_address,
            cleared_target_slot_address,
            active_astronaut_count,
            terrain_blow_process,
            previous_object_link_address,
            adjusted_x16,
            explosion,
            sound_loaded,
        })
    }

    pub(super) fn clear_astronaut_target_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<(u16, u8, Option<RedLabelCreatedProcess>), String> {
        let target_list = field_range(layout, "target_list", "TLIST")?;
        let mut slot_address = target_list.start;
        while slot_address + 1 < target_list.end {
            if self.read_word(slot_address)? == object_address {
                self.write_word(slot_address, 0)?;
                let active_count = self
                    .read_field_byte(layout, "base_page", "ASTCNT")?
                    .wrapping_sub(1);
                self.write_field_byte(layout, "base_page", "ASTCNT", active_count)?;
                let terrain_blow_process = if active_count == 0 {
                    Some(self.make_process(
                        red_label_routine_address("TERBLO")?,
                        RED_LABEL_SYSTEM_PROCESS_TYPE,
                    )?)
                } else {
                    None
                };
                return Ok((slot_address, active_count, terrain_blow_process));
            }
            slot_address = slot_address.wrapping_add(2);
        }

        Err(format!(
            "red-label ASTCLR object 0x{object_address:04X} was not found in TLIST"
        ))
    }

    /// Source-shaped `MMSW`: create up to the requested mini-swarmer count,
    /// stopping when `SWCNT` would exceed 20.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L144-L174>.
    pub(super) fn make_mini_swarmers_from_center(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        center_object_address: u16,
        requested_count: u8,
    ) -> Result<Vec<RedLabelMiniSwarmerSpawn>, String> {
        object_table_for_address(layout, center_object_address)?;
        let xtemp = field_range(layout, "base_page", "XTEMP")?.start;
        self.write_byte(xtemp, requested_count)?;
        let mut spawned = Vec::new();

        while self.read_byte(xtemp)? != 0 {
            let next_swarmer_count = self
                .read_field_byte(layout, "enemy_runtime", "SWCNT")?
                .wrapping_add(1);
            if next_swarmer_count > 20 {
                break;
            }
            self.write_field_byte(layout, "enemy_runtime", "SWCNT", next_swarmer_count)?;

            let process = self.make_process(
                red_label_routine_address("MSWM")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("SWPIC1")?,
                collision_vector_address: red_label_routine_address("MSWKIL")?,
                scanner_color: 0x2424,
            };
            let object = self.init_object_cell(process.process_address, descriptor)?;
            let x16 = self.read_object_word(layout, center_object_address, "OX16")?;
            let y16 = self.read_object_word(layout, center_object_address, "OY16")?;
            self.write_object_word(layout, object.object_address, "OX16", x16)?;
            self.write_object_word(layout, object.object_address, "OY16", y16)?;
            self.write_process_data_word(
                layout,
                process.process_address,
                "PD",
                object.object_address,
            )?;
            self.write_object_word(
                layout,
                object.object_address,
                "OBJID",
                process.process_address,
            )?;

            let velocity_rand = self.advance_red_label_rand(layout)?;
            let yv = sign_extend_u8_to_u16(velocity_rand.seed).wrapping_shl(1);
            let xv = sign_extend_u8_to_u16((velocity_rand.lseed & 0x3F).wrapping_sub(0x20));
            self.write_object_word(layout, object.object_address, "OYV", yv)?;
            self.write_object_word(layout, object.object_address, "OXV", xv)?;

            let swac = self.read_field_byte(layout, "enemy_runtime", "SWAC")?;
            let acceleration = velocity_rand.lseed & swac;
            self.write_process_byte(layout, process.process_address, "PD2", acceleration)?;
            let sleep_time = velocity_rand.hseed & 0x1F;
            self.write_process_byte(layout, process.process_address, "PTIME", sleep_time)?;
            let shot_timer_rand = self.advance_red_label_rand(layout)?;
            let shot_timer_max = self.read_field_byte(layout, "enemy_runtime", "SWSTIM")?;
            let shot_timer = rmax(shot_timer_max, shot_timer_rand.seed);
            self.write_process_byte(layout, process.process_address, "PD4", shot_timer)?;
            self.activate_object_cell(object.object_address)?;

            spawned.push(RedLabelMiniSwarmerSpawn {
                process,
                object,
                x16,
                y16,
                xv,
                yv,
                acceleration,
                sleep_time,
                shot_timer,
                velocity_rand,
                shot_timer_rand,
            });

            let remaining = self.read_byte(xtemp)?.wrapping_sub(1);
            self.write_byte(xtemp, remaining)?;
        }

        Ok(spawned)
    }

    /// Source-shaped `KILOS`: unlink the object, score the ROM-supplied BCD
    /// word, start `EXST`, and load the ROM-supplied sound table.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1174-L1182>.
    pub(super) fn kill_positioned_object_score_and_sound(
        &mut self,
        object_address: u16,
        score_word: u16,
        sound_label: &str,
    ) -> Result<RedLabelPositionedObjectKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let previous_object_link_address = self.kill_object_cell(object_address)?;
        let score = self.score_current_player(score_word)?;
        let explosion = self.start_explosion_for_object(object_address)?;
        let sound_loaded = self.load_sound_table_by_label(sound_label)?;
        Ok(RedLabelPositionedObjectKill {
            object_address,
            previous_object_link_address,
            score,
            explosion,
            sound_loaded,
        })
    }

    /// Source-shaped `KILPOS`: kill the object's owning process, unlink the
    /// object, score the ROM-supplied BCD word, start `EXST`, and load the
    /// ROM-supplied sound table.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1164-L1182>.
    pub(super) fn kill_positioned_object_with_process_score_and_sound(
        &mut self,
        object_address: u16,
        score_word: u16,
        sound_label: &str,
    ) -> Result<RedLabelEnemyKill, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let process_address = self.read_object_word(&layout, object_address, "OBJID")?;
        let previous_process_link_address = self.kill_process(process_address)?;
        let object =
            self.kill_positioned_object_score_and_sound(object_address, score_word, sound_label)?;
        Ok(RedLabelEnemyKill {
            object_address,
            killed_process: RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address: previous_process_link_address,
            },
            previous_object_link_address: object.previous_object_link_address,
            score: object.score,
            explosion: object.explosion,
            sound_loaded: object.sound_loaded,
        })
    }

    /// Source-shaped `JSR [OCVECT,X]` boundary used by `COLIDE` and `SBOMB`.
    /// Only translated collision vectors are dispatched here; unknown vectors
    /// stay explicit gaps.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3008-L3009>.
    pub fn dispatch_object_collision_vector(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelObjectCollision, String> {
        let layout = red_label_ram_layout()?;
        object_table_for_address(&layout, object_address)?;
        let collision_vector = self.read_object_word(&layout, object_address, "OCVECT")?;
        if collision_vector == red_label_routine_address("BKIL")? {
            return self
                .kill_bomb_shell_collision(object_address)
                .map(RedLabelObjectCollision::BombShell);
        }
        if collision_vector == red_label_routine_address("NOKILL")? {
            return Ok(RedLabelObjectCollision::NoKill { object_address });
        }
        if collision_vector == red_label_routine_address("SCZKIL")? {
            return self
                .kill_schizoid_collision(object_address)
                .map(RedLabelObjectCollision::EnemyKilled);
        }
        if collision_vector == red_label_routine_address("UFOKIL")? {
            return self
                .kill_ufo_collision(object_address)
                .map(RedLabelObjectCollision::UfoKilled);
        }
        if collision_vector == red_label_routine_address("LKILL")? {
            return self
                .kill_lander_collision(object_address)
                .map(RedLabelObjectCollision::LanderKilled);
        }
        if collision_vector == red_label_routine_address("LKIL1")? {
            return self
                .kill_kidnapping_lander_collision(object_address)
                .map(RedLabelObjectCollision::KidnappingLanderKilled);
        }
        if collision_vector == red_label_routine_address("PRBKIL")? {
            return self
                .kill_probe_collision(object_address)
                .map(RedLabelObjectCollision::ProbeKilled);
        }
        if collision_vector == red_label_routine_address("MSWKIL")? {
            return self
                .kill_mini_swarmer_collision(object_address)
                .map(RedLabelObjectCollision::MiniSwarmerKilled);
        }
        if collision_vector == red_label_routine_address("ASTKIL")? {
            return self
                .kill_astronaut_collision(object_address)
                .map(RedLabelObjectCollision::AstronautKilled);
        }
        if collision_vector == red_label_routine_address("AKIL1")? {
            return self
                .collide_captured_astronaut(object_address)
                .map(RedLabelObjectCollision::CapturedAstronaut);
        }
        if collision_vector == red_label_routine_address("TIEKIL")? {
            return self
                .kill_tie_collision(object_address)
                .map(RedLabelObjectCollision::TieKilled);
        }

        Err(format!(
            "red-label object OCVECT routine 0x{collision_vector:04X} for object 0x{object_address:04X} is not translated"
        ))
    }

    /// Source-shaped RAM-visible side of `LCOL`: select character map 2 in
    /// `MAPCR`, use the short-form laser picture descriptor, then enter
    /// `COLIDE` through the active object list. The matching hardware `MAPC`
    /// write is a board integration concern outside this RAM-only core.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2775-L2788>.
    pub fn collide_laser_with_active_objects(
        &mut self,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.collide_picture_with_active_objects(
            red_label_object_picture_address("LASP1")?,
            upper_left,
        )
    }

    /// Source-shaped `CRINIT`: copy the 16 bytes of `CRTAB` into `PCRAM`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1057-L1065>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1874-L1890>.
    pub fn initialize_color_ram_from_crtab(&mut self) -> Result<RedLabelColorRamInit, String> {
        let layout = red_label_ram_layout()?;
        let table = red_label_color_ram_table("CRTAB")?;
        let target = field_range(&layout, "base_page", "PCRAM")?;
        let expected_len = usize::from(target.end - target.start);
        if table.bytes.len() != expected_len {
            return Err(format!(
                "red-label CRINIT requires {expected_len} CRTAB byte(s), got {}",
                table.bytes.len()
            ));
        }

        let target_start = target.start;
        let target_end = target.end;
        self.write_range(target, &table.bytes)?;
        Ok(RedLabelColorRamInit {
            source_label: table.label.clone(),
            source_address: table.address,
            target_start,
            target_end,
            bytes: table.bytes.clone(),
        })
    }

    /// Source-shaped `COLR` / `COLRLP`: cycle the laser pseudo-color byte
    /// through ROM `COLTAB`, reset at the zero terminator, then sleep for two
    /// ticks back to `COLRLP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3024-L3042>.
    pub fn start_laser_color_current_process(
        &mut self,
        reset_index: bool,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let color_table = red_label_color_cycle_table("COLTAB")?;
        let lcolrx = ram_field(&layout, "base_page", "LCOLRX")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label LCOLRX range is invalid"))?
            .start;
        if reset_index {
            self.write_byte(lcolrx, 0)?;
        }

        let mut color_index = self.read_byte(lcolrx)?;
        let mut color = color_table
            .bytes
            .get(usize::from(color_index))
            .copied()
            .ok_or_else(|| {
                format!(
                    "red-label COLR index {color_index} is outside embedded COLTAB at 0x{:04X}",
                    color_table.address
                )
            })?;
        if color == 0 {
            color_index = 0;
            self.write_byte(lcolrx, 0)?;
            color = color_table.bytes.first().copied().ok_or_else(|| {
                String::from("red-label COLTAB must contain at least one color byte")
            })?;
            if color == 0 {
                return Err(String::from("red-label COLTAB starts with terminator"));
            }
        }

        self.write_byte(lcolrx, color_index.wrapping_add(1))?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram + 1, color)?;
        let wakeup_address = red_label_routine_address("COLRLP")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::LaserColorSleeping {
            process_address,
            color_index,
            color,
            wakeup_address,
        })
    }

    /// Source-shaped `TIECOL`: seed process `PD` with ROM `TCTAB`, then sleep
    /// for six ticks to the table-writing body.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1195-L1205>.
    pub fn start_tie_color_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let table = red_label_color_cycle_table("TCTAB")?;
        self.write_process_data_word(&layout, process_address, "PD", table.address)?;
        let wakeup_address = red_label_routine_address("TIECL")?;
        self.sleep_current_process(6, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::TieColorPrimedSleeping {
            process_address,
            table_pointer: table.address,
            wakeup_address,
        })
    }

    /// Source-shaped `TIECL`: copy the next three ROM `TCTAB` bytes into
    /// `PCRAM+$0D..$0F`, advance process `PD`, and wrap through `TIECOL` after
    /// the third triplet.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1198-L1209>.
    pub fn continue_tie_color_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let table = red_label_color_cycle_table("TCTAB")?;
        if table.bytes.len() != 9 {
            return Err(format!(
                "red-label TCTAB requires 9 byte(s), got {}",
                table.bytes.len()
            ));
        }

        let table_pointer = self.read_process_data_word(&layout, process_address, "PD")?;
        let offset = table_pointer.checked_sub(table.address).ok_or_else(|| {
            format!(
                "red-label TIECL PD 0x{table_pointer:04X} precedes TCTAB at 0x{:04X}",
                table.address
            )
        })?;
        let end_offset = offset + 3;
        if end_offset > table.bytes.len() as u16 {
            return Err(format!(
                "red-label TIECL PD 0x{table_pointer:04X} is outside embedded TCTAB"
            ));
        }
        let colors = [
            table.bytes[usize::from(offset)],
            table.bytes[usize::from(offset + 1)],
            table.bytes[usize::from(offset + 2)],
        ];
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram + 0x0D, colors[0])?;
        self.write_byte(pcram + 0x0E, colors[1])?;
        self.write_byte(pcram + 0x0F, colors[2])?;

        let table_end = table.address + table.bytes.len() as u16;
        let next_table_pointer = if table_pointer + 3 < table_end {
            table_pointer + 3
        } else {
            table.address
        };
        self.write_process_data_word(&layout, process_address, "PD", next_table_pointer)?;
        let wakeup_address = red_label_routine_address("TIECL")?;
        self.sleep_current_process(6, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::TieColorWrittenSleeping {
            process_address,
            table_pointer,
            colors,
            next_table_pointer,
            wakeup_address,
        })
    }

    /// Source-shaped `CBOMB`: prime the bomb pseudo-color pair and sleep for
    /// three ticks to `CBMB1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1213-L1217>.
    pub fn start_bomb_color_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram + 0x0A, 0xFF)?;
        self.write_byte(pcram + 0x0C, 0)?;
        let wakeup_address = red_label_routine_address("CBMB1")?;
        self.sleep_current_process(3, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::BombColorPrimedSleeping {
            process_address,
            wakeup_address,
        })
    }

    /// Source-shaped `CBMB1`: select a bomb color from `COLTAB` using
    /// `SEED&$1F`, write the two pseudo-color slots, toggle `BAX` between the
    /// ROM bomb images, then sleep for six ticks to `CBOMB`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1217-L1228>.
    pub fn continue_bomb_color_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let color_table = red_label_color_cycle_table("COLTAB")?;
        let seed = self.read_field_byte(&layout, "base_page", "SEED")?;
        let color_index = seed & 0x1F;
        let color = color_table
            .bytes
            .get(usize::from(color_index))
            .copied()
            .ok_or_else(|| {
                format!(
                    "red-label CBMB1 index {color_index} is outside embedded COLTAB at 0x{:04X}",
                    color_table.address
                )
            })?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram + 0x0A, color)?;
        self.write_byte(pcram + 0x0C, color)?;

        let previous_bomb_image = self.read_field_word(&layout, "base_page", "BAX")?;
        let bmbd10 = red_label_object_image_address("BMBD10")?;
        let bmbd20 = red_label_object_image_address("BMBD20")?;
        let next_bomb_image = if previous_bomb_image == bmbd10 {
            bmbd20
        } else {
            bmbd10
        };
        self.write_field_word(&layout, "base_page", "BAX", next_bomb_image)?;

        let wakeup_address = red_label_routine_address("CBOMB")?;
        self.sleep_current_process(6, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::BombColorWrittenSleeping {
            process_address,
            seed,
            color_index,
            color,
            previous_bomb_image,
            next_bomb_image,
            wakeup_address,
        })
    }

    /// Source-shaped `HOFST`: decrement the initials-entry stall timer once per
    /// source second and sleep back to itself.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L310-L313>.
    pub fn step_hall_of_fame_stall_timer_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let stall_before = self.read_byte(RED_LABEL_HOF_STALL_TIMER_RAM)?;
        let stall_after = stall_before.wrapping_sub(1);
        self.write_byte(RED_LABEL_HOF_STALL_TIMER_RAM, stall_after)?;
        let wakeup_address = red_label_routine_address("HOFST")?;
        self.sleep_current_process(60, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::HallOfFameStallSleeping {
            process_address,
            stall_before,
            stall_after,
            wakeup_address,
        })
    }

    /// Source-shaped `HOFBL`: toggle the blinking initials color between zero
    /// and the normal letter color, then sleep for the source blink period.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L315-L323>.
    pub fn step_hall_of_fame_blink_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        let normal_color = self.read_byte(pcram + 1)?;
        let blink_color_before = self.read_byte(pcram + 0x0D)?;
        let blink_color_after = if blink_color_before == 0 {
            normal_color
        } else {
            0
        };
        self.write_byte(pcram + 0x0D, blink_color_after)?;
        let wakeup_address = red_label_routine_address("HOFBL")?;
        self.sleep_current_process(15, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::HallOfFameBlinkSleeping {
            process_address,
            blink_color_before,
            normal_color,
            blink_color_after,
            wakeup_address,
        })
    }

    /// Source-shaped `ATTR`: select bank 1 and jump through the attract vector
    /// into `HALLOF`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1083-L1086>.
    pub fn start_attract_vector_current_process(
        &mut self,
    ) -> Result<RedLabelAttractVector, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let selected_map = 1;
        self.write_field_byte(&layout, "base_page", "MAPCR", selected_map)?;
        let entry = self.start_hall_of_fame_entry_current_process()?;
        Ok(RedLabelAttractVector {
            process_address,
            selected_map,
            attract_vector_address: RED_LABEL_ATTRACT_VECTOR_ADDRESS,
            entry,
        })
    }

    /// Source-shaped `HALLOF`: clear transient processes, reinitialize stars,
    /// clear attract credit/entry flags, and either branch to `AMODES` during the
    /// power-on pass or fall through into `HALL1` for player-one score
    /// qualification.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L121-L135>.
    pub fn start_hall_of_fame_entry_current_process(
        &mut self,
    ) -> Result<RedLabelHallOfFameEntryDispatch, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.live_defender_wordmark_coalesced = false;
        let genocide = self.genocide_other_processes()?;
        self.write_field_byte(
            &layout,
            "base_page",
            "STATUS",
            RED_LABEL_HALL_OF_FAME_ENTRY_STATUS,
        )?;
        let stars = self.initialize_star_table_from_runtime_rand(&layout)?;
        let credit = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        let old_credit_before = self.read_byte(RED_LABEL_ATTRACT_OLD_CREDIT_RAM)?;
        self.write_byte(RED_LABEL_ATTRACT_OLD_CREDIT_RAM, credit)?;
        let credit_increase_flag_before =
            self.read_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM)?;
        self.write_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM, 0)?;
        let entry_flag_before = self.read_byte(RED_LABEL_ATTRACT_ENTRY_FLAG_RAM)?;
        self.write_byte(RED_LABEL_ATTRACT_ENTRY_FLAG_RAM, 0)?;
        let power_flag = self.read_field_byte(&layout, "base_page", "PWRFLG")?;
        let setup = RedLabelHallOfFameEntrySetup {
            process_address,
            genocide,
            status: RED_LABEL_HALL_OF_FAME_ENTRY_STATUS,
            stars,
            credit,
            old_credit_before,
            old_credit_after: credit,
            credit_increase_flag_before,
            entry_flag_before,
            power_flag,
        };

        if power_flag == 0 {
            let target_address = red_label_routine_address("AMODES")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHallOfFameEntryDispatch::PowerOnWilliamsJump {
                setup,
                target_address,
            });
        }

        let credits_process = self.make_process(
            red_label_routine_address("CREDS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let player = 1;
        let score_pointer = self.player_score_pointer_for_high_score_player(&layout, player)?;
        self.write_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM, player)?;
        self.write_word(RED_LABEL_HOF_PLAYER_SCORE_POINTER_RAM, score_pointer)?;
        let qualification = self.start_high_score_qualification_current_process()?;
        Ok(RedLabelHallOfFameEntryDispatch::PlayerOneQualification {
            setup,
            credits_process,
            player,
            score_pointer,
            qualification,
        })
    }

    /// Source-shaped `HALL1`: qualify the current player's score against
    /// today's table, build the initials-entry screen and support processes, then
    /// fall through into the `HALL3A` fire-switch wait.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L135-L189>.
    pub fn start_high_score_qualification_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreQualification, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player = self.read_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM)?;
        let score_pointer = self.read_word(RED_LABEL_HOF_PLAYER_SCORE_POINTER_RAM)?;
        self.player_score_pointer_for_high_score_player(&layout, player)?;
        let score = self.read_high_score_pointer_score("HALL1", score_pointer)?;

        let Some(rank) =
            self.high_score_qualifying_rank(RuntimeHighScoreTable::TodaysGreatest, score)?
        else {
            let handoff =
                self.advance_high_score_submission_handoff(&layout, process_address, player)?;
            return Ok(RedLabelHighScoreQualification::NotQualified {
                process_address,
                player,
                score_pointer,
                score,
                handoff,
            });
        };

        let entry_flag_before = self.read_byte(RED_LABEL_ATTRACT_ENTRY_FLAG_RAM)?;
        let entry_flag_after = entry_flag_before.wrapping_add(1);
        self.write_byte(RED_LABEL_ATTRACT_ENTRY_FLAG_RAM, entry_flag_after)?;
        let state = HighScoreEntryState {
            score,
            rank,
            initials: [
                b'A',
                RED_LABEL_HOF_BLANK_INITIAL_BYTE,
                RED_LABEL_HOF_BLANK_INITIAL_BYTE,
            ],
            cursor: 0,
        };
        let display = self.write_high_score_entry_display(player, state)?;
        let top_todays_score = self
            .high_score_entry(RuntimeHighScoreTable::TodaysGreatest, 0)?
            .score;
        let sound_port_b = if score > top_todays_score { 0x3D } else { 0x3E };
        let sound_command = SoundCommand::from_main_board_pia_port_b(sound_port_b);
        self.write_byte(
            RED_LABEL_HOF_STALL_TIMER_RAM,
            RED_LABEL_HOF_FIRST_INITIAL_STALL_TICKS,
        )?;
        let support_processes = vec![
            self.make_process(
                red_label_routine_address("HOFST")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("HOFBL")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("HOFUD")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
        ];
        self.write_byte(RED_LABEL_HOF_INIT_INDEX_RAM, 0)?;
        let fire_switch = self.start_high_score_fire_switch_current_process()?;

        Ok(RedLabelHighScoreQualification::Qualified(
            RedLabelHighScoreEntryStart {
                process_address,
                player,
                score_pointer,
                score,
                rank,
                entry_flag_before,
                entry_flag_after,
                sound_port_b,
                sound_command,
                display,
                support_processes,
                stall_ticks: RED_LABEL_HOF_FIRST_INITIAL_STALL_TICKS,
                fire_switch,
            },
        ))
    }

    /// Source-shaped `HOFUD`: clear the remembered direction, then run the
    /// first `HOFUD1` up/down initials handler tick.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L325-L371>.
    pub fn start_hall_of_fame_initials_input_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        self.write_byte(RED_LABEL_HOF_INITIAL_DIRECTION_RAM, 0)?;
        self.step_hall_of_fame_initials_input_current_process(true)
    }

    /// Source-shaped `HOFUD1`: sample up/down bits, update source delay
    /// counters, wrap initials through the source blank byte/`Z`, redraw
    /// `HOFIN`, and sleep.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L328-L371>.
    pub fn continue_hall_of_fame_initials_input_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        self.step_hall_of_fame_initials_input_current_process(false)
    }

    pub(super) fn step_hall_of_fame_initials_input_current_process(
        &mut self,
        initialized: bool,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pia21 = self.read_field_byte(&layout, "base_page", "PIA21")?;
        let pia31 = self.read_field_byte(&layout, "base_page", "PIA31")?;
        let direction_before = self.read_byte(RED_LABEL_HOF_INITIAL_DIRECTION_RAM)?;
        let delay_before = self.read_byte(RED_LABEL_HOF_UP_DOWN_DELAY_RAM)?;
        let count_before = self.read_byte(RED_LABEL_HOF_UP_DOWN_COUNT_RAM)?;
        let input_direction = if pia21 & 0x80 != 0 {
            0xFF
        } else if pia31 & 0x01 != 0 {
            1
        } else {
            0
        };

        let mut direction_after = direction_before;
        let mut delay_after = delay_before;
        let mut count_after = count_before;
        let mut update = None;
        if input_direction == 0 {
            direction_after = 0;
            self.write_byte(RED_LABEL_HOF_INITIAL_DIRECTION_RAM, direction_after)?;
        } else if input_direction != direction_before {
            direction_after = input_direction;
            delay_after = 55;
            count_after = 3;
            self.write_byte(RED_LABEL_HOF_INITIAL_DIRECTION_RAM, direction_after)?;
            self.write_byte(RED_LABEL_HOF_UP_DOWN_DELAY_RAM, delay_after)?;
            self.write_byte(RED_LABEL_HOF_UP_DOWN_COUNT_RAM, count_after)?;
        } else {
            count_after = count_before.wrapping_sub(1);
            self.write_byte(RED_LABEL_HOF_UP_DOWN_COUNT_RAM, count_after)?;
            if count_after == 0 {
                let initial_index = self.read_byte(RED_LABEL_HOF_INIT_INDEX_RAM)?;
                let initial_address = RED_LABEL_HOF_INITS_RAM + u16::from(initial_index) * 2;
                let initial_before = self.read_byte(initial_address)?;
                let mut initial_after = initial_before.wrapping_add(input_direction);
                if initial_after == 0x3F {
                    initial_after = b'Z';
                }
                if initial_after == 0x5B {
                    initial_after = RED_LABEL_HOF_BLANK_INITIAL_BYTE;
                }
                self.write_byte(initial_address, initial_after)?;
                delay_after = (delay_before >> 1).wrapping_add(5);
                count_after = delay_after;
                self.write_byte(RED_LABEL_HOF_UP_DOWN_DELAY_RAM, delay_after)?;
                self.write_byte(RED_LABEL_HOF_UP_DOWN_COUNT_RAM, count_after)?;
                let display =
                    self.write_high_score_initials_display_from_current_ram(initial_index)?;
                update = Some(RedLabelHighScoreInitialUpdate {
                    initial_index,
                    initial_address,
                    initial_before,
                    initial_after,
                    display,
                });
            }
        }

        let wakeup_address = red_label_routine_address("HOFUD1")?;
        self.sleep_current_process(1, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::HallOfFameInitialsSleeping {
            process_address,
            initialized,
            pia21,
            pia31,
            input_direction,
            direction_before,
            direction_after,
            delay_before,
            delay_after,
            count_before,
            count_after,
            update,
            wakeup_address,
        })
    }

    /// Source-shaped `HALL3A`: clear the fire-open debounce state and sleep
    /// for one tick to the fire-switch sampler.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L189-L193>.
    pub fn start_high_score_fire_switch_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreFireSwitch, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let fire_flag_before = self.read_byte(RED_LABEL_HOF_FIRE_FLAG_RAM)?;
        let fire_count_before = self.read_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM)?;
        self.write_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM, 0)?;
        self.write_byte(RED_LABEL_HOF_FIRE_FLAG_RAM, 0)?;
        let wakeup_address = red_label_routine_address("HALL4")?;
        self.sleep_current_process(RED_LABEL_HOF_FIRE_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelHighScoreFireSwitch::ResetSleeping {
            process_address,
            fire_flag_before,
            fire_count_before,
            wakeup_address,
        })
    }

    /// Source-shaped `HALL4`: sample the fire switch, require a five-tick open
    /// debounce before accepting a close, or jump to `HALL6` when the stall
    /// timer expires.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L192-L203>.
    pub fn continue_high_score_fire_switch_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreFireSwitch, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pia21 = self.read_field_byte(&layout, "base_page", "PIA21")?;
        if pia21 & RED_LABEL_HOF_FIRE_SWITCH_MASK != 0 {
            return self.advance_high_score_fire_switch_closed(&layout, process_address, pia21);
        }

        let stall_timer = self.read_byte(RED_LABEL_HOF_STALL_TIMER_RAM)?;
        if stall_timer == 0 {
            let target_address = red_label_routine_address("HALL6")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHighScoreFireSwitch::StallExpiredSubmitJump {
                process_address,
                pia21,
                stall_timer,
                target_address,
            });
        }

        let fire_count_before = self.read_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM)?;
        let fire_flag_before = self.read_byte(RED_LABEL_HOF_FIRE_FLAG_RAM)?;
        let fire_count_after = fire_count_before.wrapping_add(1);
        self.write_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM, fire_count_after)?;
        let fire_flag_after = if fire_count_after == RED_LABEL_HOF_FIRE_OPEN_COUNT_READY {
            self.write_byte(RED_LABEL_HOF_FIRE_FLAG_RAM, fire_count_after)?;
            fire_count_after
        } else {
            fire_flag_before
        };
        let wakeup_address = red_label_routine_address("HALL4")?;
        self.sleep_current_process(RED_LABEL_HOF_FIRE_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelHighScoreFireSwitch::OpenSleeping {
            process_address,
            pia21,
            stall_timer,
            fire_count_before,
            fire_count_after,
            fire_flag_before,
            fire_flag_after,
            wakeup_address,
        })
    }

    /// Source-shaped `HALL5`: accept a debounced fire close, reset the stall
    /// timer, advance the active initials cursor, redraw `HOFUL`, and either
    /// return through `HALL3A` or jump to `HALL6`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L204-L214>.
    pub fn advance_high_score_fire_switch_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreFireSwitch, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pia21 = self.read_field_byte(&layout, "base_page", "PIA21")?;
        self.advance_high_score_fire_switch_closed(&layout, process_address, pia21)
    }

    pub(super) fn advance_high_score_fire_switch_closed(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        pia21: u8,
    ) -> Result<RedLabelHighScoreFireSwitch, String> {
        self.write_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM, 0)?;
        let fire_flag_before = self.read_byte(RED_LABEL_HOF_FIRE_FLAG_RAM)?;
        if fire_flag_before == 0 {
            let wakeup_address = red_label_routine_address("HALL4")?;
            self.sleep_current_process(RED_LABEL_HOF_FIRE_SLEEP_TICKS, wakeup_address)?;
            return Ok(RedLabelHighScoreFireSwitch::ClosedIgnoredSleeping {
                process_address,
                pia21,
                fire_flag_before,
                wakeup_address,
            });
        }

        self.write_byte(
            RED_LABEL_HOF_STALL_TIMER_RAM,
            RED_LABEL_HOF_NEXT_INITIAL_STALL_TICKS,
        )?;
        let initial_before = self.read_byte(RED_LABEL_HOF_INIT_INDEX_RAM)?;
        let initial_after = initial_before.wrapping_add(1);
        self.write_byte(RED_LABEL_HOF_INIT_INDEX_RAM, initial_after)?;
        let underline_words = self.write_high_score_initial_underlines(initial_after)?;
        if initial_after == RED_LABEL_INITIALS_ENTRY_CHARS as u8 {
            let target_address = red_label_routine_address("HALL6")?;
            self.write_process_word(layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHighScoreFireSwitch::InitialAdvancedSubmitJump {
                process_address,
                pia21,
                initial_before,
                initial_after,
                stall_timer: RED_LABEL_HOF_NEXT_INITIAL_STALL_TICKS,
                underline_words,
                target_address,
            });
        }

        self.write_byte(RED_LABEL_HOF_FIRE_OPEN_COUNT_RAM, 0)?;
        self.write_byte(RED_LABEL_HOF_FIRE_FLAG_RAM, 0)?;
        let wakeup_address = red_label_routine_address("HALL4")?;
        self.sleep_current_process(RED_LABEL_HOF_FIRE_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelHighScoreFireSwitch::InitialAdvancedSleeping {
            process_address,
            pia21,
            initial_before,
            initial_after,
            stall_timer: RED_LABEL_HOF_NEXT_INITIAL_STALL_TICKS,
            underline_words,
            wakeup_address,
        })
    }

    /// Source-shaped `HALL6` / `HALL12`: kill entry helper processes, commit
    /// the current player's score and initials to the source high-score tables,
    /// then either continue to player two or hand off to `HALL13`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L214-L232>.
    pub fn submit_high_score_initials_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreSubmission, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let genocide = self.genocide_other_processes()?;
        let player = self.read_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM)?;
        let score_pointer = self.read_word(RED_LABEL_HOF_PLAYER_SCORE_POINTER_RAM)?;
        let score = self.read_high_score_pointer_score("HALL6", score_pointer)?;
        let initials = self.read_high_score_submission_initials()?;
        let todays_rank =
            self.insert_high_score(RuntimeHighScoreTable::TodaysGreatest, score, initials)?;
        let all_time_rank =
            self.insert_high_score(RuntimeHighScoreTable::AllTime, score, initials)?;
        let handoff =
            self.advance_high_score_submission_handoff(&layout, process_address, player)?;

        Ok(RedLabelHighScoreSubmission {
            process_address,
            genocide,
            player,
            score_pointer,
            score,
            initials,
            todays_rank,
            all_time_rank,
            handoff,
        })
    }

    pub(super) fn read_high_score_pointer_score(
        &self,
        routine_label: &'static str,
        score_pointer: u16,
    ) -> Result<u32, String> {
        let score_digits = self.read_fixed_bytes::<3>(score_pointer.wrapping_add(1))?;
        if score_digits.iter().any(|byte| !is_bcd_byte(*byte)) {
            return Err(format!(
                "red-label {routine_label} score at 0x{score_pointer:04X} is not valid BCD"
            ));
        }
        Ok(bcd_digits_to_u32(&score_digits))
    }

    pub(super) fn read_high_score_submission_initials(
        &self,
    ) -> Result<[u8; RED_LABEL_INITIALS_ENTRY_CHARS], String> {
        let mut initials = [0; RED_LABEL_INITIALS_ENTRY_CHARS];
        for (index, initial) in initials.iter_mut().enumerate() {
            let address = RED_LABEL_HOF_INITS_RAM
                + u16::try_from(index * 2).expect("initial index should fit in u16");
            *initial = self.read_byte(address)?;
        }
        if !red_label_high_score_initials_are_valid(&initials) {
            return Err(String::from(
                "red-label HALL6 initials must be uppercase ASCII or source blank bytes",
            ));
        }
        Ok(initials)
    }

    pub(super) fn advance_high_score_submission_handoff(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        player: u8,
    ) -> Result<RedLabelHighScoreSubmissionHandoff, String> {
        let next_player = player.wrapping_add(1);
        if next_player != 3 {
            let target_address = red_label_routine_address("HALL1")?;
            let next_player_score_pointer =
                self.player_score_pointer_for_high_score_player(layout, next_player)?;
            self.write_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM, next_player)?;
            self.write_word(
                RED_LABEL_HOF_PLAYER_SCORE_POINTER_RAM,
                next_player_score_pointer,
            )?;
            self.write_process_word(layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHighScoreSubmissionHandoff::NextPlayerJump {
                next_player,
                next_player_score_pointer,
                target_address,
            });
        }

        let entry_flag = self.read_byte(RED_LABEL_ATTRACT_ENTRY_FLAG_RAM)?;
        if entry_flag != 0 {
            let target_address = red_label_routine_address("HALL13")?;
            self.write_process_word(layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHighScoreSubmissionHandoff::HallOfFameJump {
                entry_flag,
                target_address,
            });
        }

        let wakeup_address = red_label_routine_address("HALL13")?;
        self.sleep_current_process(RED_LABEL_HALL_OF_FAME_NO_ENTRY_DELAY_TICKS, wakeup_address)?;
        Ok(RedLabelHighScoreSubmissionHandoff::NoEntryDelaySleeping {
            entry_flag,
            sleep_ticks: RED_LABEL_HALL_OF_FAME_NO_ENTRY_DELAY_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `HALL12`: advance to player two, jump to `HALDIS`, or
    /// sleep before `HALDIS` without replaying the `HALL6` insertion work.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L224-L231>.
    pub fn advance_high_score_handoff_current_process(
        &mut self,
    ) -> Result<RedLabelHighScoreHandoff, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player = self.read_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM)?;
        let handoff =
            self.advance_high_score_submission_handoff(&layout, process_address, player)?;
        Ok(RedLabelHighScoreHandoff {
            process_address,
            player,
            handoff,
        })
    }

    pub(super) fn player_score_pointer_for_high_score_player(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        player: u8,
    ) -> Result<u16, String> {
        let player_index = match player {
            1 | 2 => u16::from(player - 1),
            other => {
                return Err(format!(
                    "red-label HALL12 player {other} is outside the high-score player range"
                ));
            }
        };
        ram_field(layout, "player", "PSCOR")?
            .field_range_for_entry(player_index)
            .map(|range| range.start)
            .ok_or_else(|| String::from("red-label player.PSCOR range is invalid"))
    }

    /// Source-shaped `FLPUP`: if score transfer is active, die; otherwise
    /// blank the current player's score area and sleep to `FLP2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1232-L1243>.
    pub fn start_player_up_flash_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        if self.read_field_byte(&layout, "base_page", "SCRFLG")? != 0 {
            let killed = self.kill_current_process(&layout)?;
            return Ok(RedLabelSupportProcessStep::PlayerUpCompleted(killed));
        }

        let player_number = self.read_field_byte(&layout, "base_page", "CURPLR")?;
        let screen_address = if player_number.wrapping_sub(1) == 0 {
            RED_LABEL_P1_SCORE_DISPLAY
        } else {
            RED_LABEL_P2_SCORE_DISPLAY
        };
        let block_clear = self.block_clear(screen_address, 24, 8)?;
        let wakeup_address = red_label_routine_address("FLP2")?;
        self.sleep_current_process(8, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::PlayerUpBlankedSleeping {
            process_address,
            player_number,
            block_clear,
            wakeup_address,
        })
    }

    /// Source-shaped `FLP2`: run the current-player score transfer body and
    /// sleep for twelve ticks back to `FLPUP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1242-L1243>.
    pub fn continue_player_up_flash_current_process(
        &mut self,
    ) -> Result<RedLabelSupportProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player_number = self.read_field_byte(&layout, "base_page", "CURPLR")?;
        let score_transfer = self.transfer_score_digits(&layout, player_number)?;
        let wakeup_address = red_label_routine_address("FLPUP")?;
        self.sleep_current_process(12, wakeup_address)?;
        Ok(RedLabelSupportProcessStep::PlayerUpRedrawnSleeping {
            process_address,
            score_transfer,
            wakeup_address,
        })
    }

    /// Source-shaped bank-7 `BGALT` / `ALINIT`: copy the ROM `TDATA` bitstream
    /// into the 0x400-byte altitude table at `ALTTBL`, preserving the right-side
    /// `RFONR1` rotate/refill behavior and its visible terrain work bytes.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L375-L399>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L510-L525>.
    pub fn initialize_altitude_table_from_tdata(
        &mut self,
    ) -> Result<RedLabelAltitudeTableInit, String> {
        let layout = red_label_ram_layout()?;
        let table = red_label_terrain_data_table("TDATA")?;
        if table.bytes.len() != RED_LABEL_TERRAIN_DATA_BYTES {
            return Err(format!(
                "red-label ALINIT requires {RED_LABEL_TERRAIN_DATA_BYTES} TDATA byte(s), got {}",
                table.bytes.len()
            ));
        }

        let target = field_range(&layout, "terrain_altitude", "ALTTBL")?;
        let expected_len = u16::try_from(RED_LABEL_ALTITUDE_TABLE_BYTES)
            .map_err(|_| String::from("red-label altitude table length is outside u16 range"))?;
        if target.end - target.start != expected_len {
            return Err(format!(
                "red-label ALTTBL layout must be {expected_len} byte(s), got {}",
                target.end - target.start
            ));
        }

        let mut data_index = 0usize;
        let mut data_pointer = table.address;
        let mut data_byte = table.bytes[0];
        let mut bit_counter = 7u8;
        let mut offset = 0xE0u8;
        let mut altitude = Vec::with_capacity(RED_LABEL_ALTITUDE_TABLE_BYTES);
        for _ in 0..RED_LABEL_ALTITUDE_TABLE_BYTES {
            altitude.push(offset);
            offset = terrain_altitude_step(offset, data_byte);
            advance_terrain_right_data(
                &table.bytes,
                table.address,
                &mut data_index,
                &mut data_pointer,
                &mut data_byte,
                &mut bit_counter,
            );
            offset = terrain_altitude_step(offset, data_byte);
            advance_terrain_right_data(
                &table.bytes,
                table.address,
                &mut data_index,
                &mut data_pointer,
                &mut data_byte,
                &mut bit_counter,
            );
        }

        let table_start = target.start;
        let table_end = target.end;
        self.write_range(target, &altitude)?;
        self.write_field_word(&layout, "terrain_runtime", "RTPTR", data_pointer)?;
        self.write_field_byte(&layout, "terrain_runtime", "RTBYTE", data_byte)?;
        self.write_field_byte(&layout, "terrain_runtime", "RTCNT", bit_counter)?;
        self.write_field_byte(&layout, "terrain_runtime", "ROFF", offset)?;

        Ok(RedLabelAltitudeTableInit {
            source_label: table.label.clone(),
            source_address: table.address,
            table_start,
            table_end,
            final_data_pointer: data_pointer,
            final_data_byte: data_byte,
            final_bit_counter: bit_counter,
            final_offset: offset,
            table_crc32: crc32(&altitude),
        })
    }

    /// Source-shaped bank-7 `BGINIT`: position the `TDATA` terrain stream from
    /// `BGL`, fill the mirrored `TERTF0`/`TERTF1` terrain flavor tables, restore
    /// the right-edge terrain cursor, and zero `STBL`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L95-L149>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L235-L303>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L458-L506>.
    pub fn initialize_terrain_tables_from_bgl(
        &mut self,
    ) -> Result<RedLabelTerrainTablesInit, String> {
        let layout = red_label_ram_layout()?;
        let table = red_label_terrain_data_table("TDATA")?;
        if table.bytes.len() != RED_LABEL_TERRAIN_DATA_BYTES {
            return Err(format!(
                "red-label BGINIT requires {RED_LABEL_TERRAIN_DATA_BYTES} TDATA byte(s), got {}",
                table.bytes.len()
            ));
        }

        let flavor_0 = field_range(&layout, "terrain_flavor_0", "TERTF0")?;
        let flavor_1 = field_range(&layout, "terrain_flavor_1", "TERTF1")?;
        let expected_flavor_len = RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES * 2;
        if flavor_0.end - flavor_0.start != expected_flavor_len {
            return Err(format!(
                "red-label TERTF0 layout must be {expected_flavor_len} byte(s), got {}",
                flavor_0.end - flavor_0.start
            ));
        }
        if flavor_1.end - flavor_1.start != expected_flavor_len {
            return Err(format!(
                "red-label TERTF1 layout must be {expected_flavor_len} byte(s), got {}",
                flavor_1.end - flavor_1.start
            ));
        }
        let screen_table = field_range(&layout, "terrain_screen_table", "STBL")?;
        if screen_table.end - screen_table.start != RED_LABEL_TERRAIN_SCREEN_TABLE_BYTES {
            return Err(format!(
                "red-label STBL layout must be {RED_LABEL_TERRAIN_SCREEN_TABLE_BYTES} byte(s), got {}",
                screen_table.end - screen_table.start
            ));
        }

        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let terrain_left = background_left & 0xFFE0;
        let mut generation_left = terrain_left.wrapping_add(0x2610);
        let mut left = TerrainBitState {
            data_index: table.bytes.len() - 1,
            data_pointer: table.address.wrapping_sub(1),
            data_byte: 0,
            bit_counter: 0,
        };
        let mut left_offset = 0xE0;
        advance_terrain_right_state(&mut left, &table.bytes, table.address);

        let mut scan_x = 0x0010;
        for _ in 0..=0x0800 {
            if scan_x == generation_left {
                break;
            }
            left_offset = terrain_altitude_step(left_offset, left.data_byte);
            advance_terrain_right_state(&mut left, &table.bytes, table.address);
            scan_x = scan_x.wrapping_add(0x20);
        }
        if scan_x != generation_left {
            return Err(format!(
                "red-label BGINIT could not align terrain stream to BGLX 0x{generation_left:04X}"
            ));
        }

        let saved_right = left;
        let saved_right_offset = left_offset;
        let right_pointer = self.read_field_word(&layout, "terrain_runtime", "RTPTR")?;
        let right = TerrainBitState {
            data_index: terrain_data_index_for_pointer(
                right_pointer,
                table.address,
                table.bytes.len(),
            )?,
            data_pointer: right_pointer,
            data_byte: self.read_field_byte(&layout, "terrain_runtime", "RTBYTE")?,
            bit_counter: self.read_field_byte(&layout, "terrain_runtime", "RTCNT")?,
        };
        if right.bit_counter > 7 {
            return Err(format!(
                "red-label BGINIT found RTCNT {} outside 0..=7",
                right.bit_counter
            ));
        }

        let mut state = TerrainTableGenerationState {
            left,
            right,
            left_offset,
            right_offset: self.read_field_byte(&layout, "terrain_runtime", "ROFF")?,
            background_left: generation_left,
            terrain_left,
            flavor_0_pointer: flavor_0.start,
            flavor_1_pointer: flavor_1.start,
        };
        let mut left_pixels_generated = 0u16;
        loop {
            generation_left = generation_left.wrapping_sub(0x20);
            state.background_left = generation_left;
            if generation_left.wrapping_sub(state.terrain_left) & 0x8000 != 0 {
                break;
            }
            self.add_left_terrain_pixel(
                &mut state,
                &table.bytes,
                table.address,
                flavor_0.start,
                flavor_1.start,
            )?;
            left_pixels_generated = left_pixels_generated.wrapping_add(1);
            if left_pixels_generated > 0x0800 {
                return Err(String::from(
                    "red-label BGINIT terrain generation did not converge",
                ));
            }
        }

        state.right = saved_right;
        state.right_offset = saved_right_offset;
        self.clear_range(screen_table.clone())?;
        self.write_field_word(&layout, "terrain_runtime", "TEMP2B", terrain_left)?;
        self.write_field_word(&layout, "terrain_runtime", "BGLX", state.background_left)?;
        self.write_field_word(
            &layout,
            "terrain_runtime",
            "TEMP2A",
            saved_right.data_pointer,
        )?;
        self.write_field_byte(
            &layout,
            "terrain_runtime",
            "TEMP1A",
            saved_right.bit_counter,
        )?;
        self.write_field_byte(&layout, "terrain_runtime", "TEMP1B", saved_right.data_byte)?;
        self.write_field_byte(&layout, "terrain_runtime", "TEMP1C", saved_right_offset)?;
        self.write_field_word(&layout, "terrain_runtime", "TTBLP0", state.flavor_0_pointer)?;
        self.write_field_word(&layout, "terrain_runtime", "TTBLP1", state.flavor_1_pointer)?;
        self.write_field_word(&layout, "terrain_runtime", "LTPTR", state.left.data_pointer)?;
        self.write_field_byte(&layout, "terrain_runtime", "LTBYTE", state.left.data_byte)?;
        self.write_field_byte(&layout, "terrain_runtime", "LTCNT", state.left.bit_counter)?;
        self.write_field_byte(&layout, "terrain_runtime", "LOFF", state.left_offset)?;
        self.write_field_word(
            &layout,
            "terrain_runtime",
            "RTPTR",
            state.right.data_pointer,
        )?;
        self.write_field_byte(&layout, "terrain_runtime", "RTBYTE", state.right.data_byte)?;
        self.write_field_byte(&layout, "terrain_runtime", "RTCNT", state.right.bit_counter)?;
        self.write_field_byte(&layout, "terrain_runtime", "ROFF", state.right_offset)?;

        let flavor_0_bytes = self
            .ram_range(flavor_0.clone())
            .ok_or_else(|| String::from("red-label TERTF0 range is outside RAM"))?;
        let flavor_1_bytes = self
            .ram_range(flavor_1.clone())
            .ok_or_else(|| String::from("red-label TERTF1 range is outside RAM"))?;
        Ok(RedLabelTerrainTablesInit {
            background_left,
            terrain_left,
            terrain_generation_left: state.background_left,
            left_data_pointer: state.left.data_pointer,
            left_data_byte: state.left.data_byte,
            left_bit_counter: state.left.bit_counter,
            left_offset: state.left_offset,
            right_data_pointer: state.right.data_pointer,
            right_data_byte: state.right.data_byte,
            right_bit_counter: state.right.bit_counter,
            right_offset: state.right_offset,
            flavor_0_pointer: state.flavor_0_pointer,
            flavor_1_pointer: state.flavor_1_pointer,
            flavor_0_crc32: crc32(flavor_0_bytes),
            flavor_1_crc32: crc32(flavor_1_bytes),
            screen_table_start: screen_table.start,
            screen_table_end: screen_table.end,
            left_pixels_generated,
        })
    }

    /// Source-shaped `BGI`: select bank/map 7 through `MAPCH7`, then jump to
    /// the bank-7 `BGINIT` terrain table generator.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1287-L1289>.
    pub fn initialize_background_from_bgi(&mut self) -> Result<RedLabelBackgroundInit, String> {
        let layout = red_label_ram_layout()?;
        let selected_map = 7;
        self.write_field_byte(&layout, "base_page", "MAPCR", selected_map)?;
        let terrain_tables = self.initialize_terrain_tables_from_bgl()?;
        Ok(RedLabelBackgroundInit {
            bgi_address: red_label_routine_address("BGI")?,
            selected_map,
            terrain_tables,
        })
    }

    /// Source-shaped `SCINIT`: prepare the attract instruction-page world by
    /// resetting objects, clearing video RAM, zeroing terrain scroll, refreshing
    /// the `ALINIT` terrain stream for `BGI`, reloading `CRTAB`, and centering
    /// the scanner player blip.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L525-L540>.
    pub fn initialize_attract_scene_from_scinit(
        &mut self,
    ) -> Result<RedLabelAttractSceneInit, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let initial_status = 0xFF;
        self.write_field_byte(&layout, "base_page", "STATUS", initial_status)?;
        self.initialize_object_lists(&layout, &lists)?;
        let screen_clear = self.clear_screen_ram()?;
        self.write_field_word(&layout, "base_page", "BGL", 0)?;
        self.write_field_word(&layout, "base_page", "BGLX", 0)?;
        let altitude_table = self.initialize_altitude_table_from_tdata()?;
        let background = self.initialize_background_from_bgi()?;
        let color_ram = self.initialize_color_ram_from_crtab()?;
        let final_status = 0xDB;
        self.write_field_byte(&layout, "base_page", "STATUS", final_status)?;
        let player_scanner_center = 0x1030;
        self.write_field_word(&layout, "base_page", "PLAXC", player_scanner_center)?;

        Ok(RedLabelAttractSceneInit {
            initial_status,
            screen_clear,
            altitude_table,
            background,
            color_ram,
            final_status,
            player_scanner_center,
        })
    }

    /// Source-shaped `LEDRET`: start the attract instruction page by
    /// preparing the world, display state, support processes, object cells, and
    /// the initial lander appearance before sleeping to `AMODE1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L477-L514>.
    pub fn start_attract_instruction_page_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let genocide = self.genocide_other_processes()?;
        let scene = self.initialize_attract_scene_from_scinit()?;
        self.write_field_byte(
            &layout,
            "base_page",
            "STATUS",
            RED_LABEL_ATTRACT_INSTRUCTION_STATUS,
        )?;
        let score_transfers = self.display_player_scores(&layout)?;
        let credits_process = self.make_process(
            red_label_routine_address("CREDS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let border = self.draw_top_display_border()?;
        let text_pointer = RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE.wrapping_add(2);
        self.write_word(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_POINTER_RAM, text_pointer)?;
        let support_processes = vec![
            self.make_process(
                red_label_routine_address("COLR")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("CBOMB")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("TIECOL")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("SCPROC")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("TEXTP")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
        ];

        let astronaut_object = self.init_attract_instruction_object_cell(
            &layout,
            "ASTP1",
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_X16,
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_Y16,
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_COLOR,
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM,
        )?;
        let player_object = self.init_attract_instruction_object_cell(
            &layout,
            "PLAPIC",
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_X16,
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_Y16,
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_COLOR,
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM,
        )?;
        let enemy_object = self.get_object_cell()?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OPICT",
            red_label_object_picture_address("LNDP1")?,
        )?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OX16",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_X16,
        )?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OY16",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_Y16,
        )?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_Y_VELOCITY,
        )?;
        self.write_object_word(&layout, enemy_object, "OXV", 0)?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OBJCOL",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_COLOR,
        )?;
        let enemy_appearance = self.start_appearance_for_object(enemy_object)?;
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_POINTER_RAM,
            enemy_object,
        )?;
        let wakeup_address = red_label_routine_address("AMODE1")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionStart {
            process_address,
            genocide,
            scene,
            status: RED_LABEL_ATTRACT_INSTRUCTION_STATUS,
            score_transfers,
            credits_process,
            border,
            text_pointer,
            support_processes,
            astronaut_object,
            player_object,
            enemy_object,
            enemy_appearance,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_ENTRY_SLEEP_TICKS,
            wakeup_address,
        })
    }

    pub(super) fn init_attract_instruction_object_cell(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        picture_label: &'static str,
        x16: u16,
        y16: u16,
        color: u16,
        pointer_ram: u16,
    ) -> Result<u16, String> {
        let object_address = self.get_object_cell()?;
        self.write_object_word(layout, object_address, "OXV", 0)?;
        self.write_object_word(layout, object_address, "OYV", 0)?;
        self.write_object_word(layout, object_address, "OX16", x16)?;
        self.write_object_word(layout, object_address, "OY16", y16)?;
        self.write_object_word(
            layout,
            object_address,
            "OPICT",
            red_label_object_picture_address(picture_label)?,
        )?;
        self.activate_object_cell(object_address)?;
        self.write_object_word(layout, object_address, "OBJCOL", color)?;
        self.write_word(pointer_ram, object_address)?;
        Ok(object_address)
    }

    /// Source-shaped `AMODE1`: send the lander and man upward, then sleep to
    /// `AMODE2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L515-L520>.
    pub fn raise_attract_instruction_objects_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionAscent, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let enemy_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_POINTER_RAM)?;
        let astronaut_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM)?;
        self.write_object_word(
            &layout,
            enemy_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_ASCENT_Y_VELOCITY,
        )?;
        self.write_object_word(
            &layout,
            astronaut_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_ASCENT_Y_VELOCITY,
        )?;
        let wakeup_address = red_label_routine_address("AMODE2")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_ASCENT_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionAscent {
            process_address,
            enemy_object,
            astronaut_object,
            y_velocity: RED_LABEL_ATTRACT_INSTRUCTION_ASCENT_Y_VELOCITY,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_ASCENT_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMODE2`: create the laser process, save it in `LASPRC`,
    /// then sleep to `AMODE3`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L521-L523>.
    pub fn start_attract_instruction_laser_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionLaserStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let laser_process = self.make_process(
            red_label_routine_address("LASRS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_LASER_PROCESS_RAM,
            laser_process.process_address,
        )?;
        let wakeup_address = red_label_routine_address("AMODE3")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_LASER_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionLaserStart {
            process_address,
            laser_process,
            laser_process_pointer_address: RED_LABEL_ATTRACT_INSTRUCTION_LASER_PROCESS_RAM,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_LASER_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `LASRS` entry: seed the laser process data from the
    /// player ship screen coordinate, then fall through the first `LAS0` loop.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L668-L701>.
    pub fn start_attract_instruction_laser_step_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionLaserStep, String> {
        self.step_attract_instruction_laser_current_process(true)
    }

    /// Source-shaped `LAS0`: draw one instruction-page laser slice and sleep
    /// back for the next frame.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L675-L701>.
    pub fn continue_attract_instruction_laser_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionLaserStep, String> {
        self.step_attract_instruction_laser_current_process(false)
    }

    pub(super) fn step_attract_instruction_laser_current_process(
        &mut self,
        initialize: bool,
    ) -> Result<RedLabelAttractInstructionLaserStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let ship_object = if initialize {
            self.attract_instruction_laser_addresses.clear();
            let ship_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM)?;
            let ship_screen_address = self.read_object_screen_address(&layout, ship_object)?;
            let laser_start = ship_screen_address
                .wrapping_add(RED_LABEL_ATTRACT_INSTRUCTION_LASER_START_SCREEN_OFFSET);
            self.write_process_data_word(&layout, process_address, "PD", laser_start)?;
            self.write_process_data_word(&layout, process_address, "PD2", laser_start)?;
            self.write_word(RED_LABEL_ATTRACT_INSTRUCTION_LASER_STATE_RAM, laser_start)?;
            Some(ship_object)
        } else {
            None
        };

        let laser_start = self.read_process_data_word(&layout, process_address, "PD")?;
        let tip_address = self.draw_laser_body(RedLabelLaserDirection::Right, laser_start)?;
        self.record_attract_instruction_laser_path(laser_start, tip_address);
        self.write_process_data_word(&layout, process_address, "PD", tip_address)?;
        let fizzle_target = self.read_process_data_word(&layout, process_address, "PD2")?;
        let (fizzle_source_next, fizzle_target_next) =
            self.draw_laser_fizzle(&layout, RedLabelLaserDirection::Right, fizzle_target)?;
        self.record_attract_instruction_laser_path_exclusive(fizzle_target, fizzle_target_next);
        self.write_process_data_word(&layout, process_address, "PD2", fizzle_target_next)?;
        let erase_address = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_LASER_STATE_RAM)?;
        self.write_byte(erase_address, 0)?;
        let erase_next = step_laser_address(RedLabelLaserDirection::Right, erase_address);
        self.write_word(RED_LABEL_ATTRACT_INSTRUCTION_LASER_STATE_RAM, erase_next)?;
        let wakeup_address = red_label_routine_address("LAS0")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_LASER_STEP_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionLaserStep {
            process_address,
            initialized: initialize,
            ship_object,
            laser_start,
            tip_address,
            fizzle_source_next,
            fizzle_target_next,
            erase_address,
            erase_next,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_LASER_STEP_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMODE3`: kill the instruction-page laser, free/explode
    /// the lander, start the ship/man rescue motion, then fall through the
    /// first `AMODE4` free-fall frame.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L549-L568>.
    pub fn start_attract_instruction_rescue_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionRescueStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let laser_kill = self.kill_attract_instruction_laser_process(&layout)?;
        let enemy_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_POINTER_RAM)?;
        let enemy_previous_link_address = self.kill_object_cell(enemy_object)?;
        let enemy_explosion = self.start_explosion_for_object(enemy_object)?;
        let ship_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM)?;
        self.write_object_word(
            &layout,
            ship_object,
            "OXV",
            RED_LABEL_ATTRACT_INSTRUCTION_RESCUE_SHIP_X_VELOCITY,
        )?;
        self.write_object_word(
            &layout,
            ship_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_RESCUE_SHIP_Y_VELOCITY,
        )?;
        self.write_byte(
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_FREE_FALL_RAM,
            RED_LABEL_ATTRACT_INSTRUCTION_RESCUE_FREE_FALL_TICKS,
        )?;
        let astronaut_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM)?;
        self.write_object_word(&layout, astronaut_object, "OYV", 0)?;
        let free_fall = self.run_attract_instruction_free_fall_frame(&layout)?;

        Ok(RedLabelAttractInstructionRescueStart {
            process_address,
            laser_kill,
            enemy_object,
            enemy_previous_link_address,
            enemy_explosion,
            ship_object,
            astronaut_object,
            free_fall,
        })
    }

    /// Source-shaped `AMODE4`: accelerate the man downward until the source
    /// free-fall counter reaches the `AMODE5` intersection fallthrough.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L563-L579>.
    pub fn continue_attract_instruction_free_fall_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionFreeFall, String> {
        let layout = red_label_ram_layout()?;
        self.run_attract_instruction_free_fall_frame(&layout)
    }

    /// Source-shaped `AMODE5`: run the rescue intersection fallthrough reached
    /// after the `AMODE4` free-fall counter expires.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L574-L589>.
    pub fn start_attract_instruction_intersection_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionIntersection, String> {
        let layout = red_label_ram_layout()?;
        self.start_attract_instruction_intersection_current_process_with_layout(&layout)
    }

    pub(super) fn kill_attract_instruction_laser_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelAttractInstructionLaserKill, String> {
        let laser_process_address =
            self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_LASER_PROCESS_RAM)?;
        let laser_state_start = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_LASER_STATE_RAM)?;
        let laser_state_end = self.read_process_data_word(layout, laser_process_address, "PD")?;
        let mut cleared_addresses = Vec::new();
        self.clear_attract_instruction_laser_path(
            laser_state_start,
            laser_state_end,
            &mut cleared_addresses,
        )?;
        self.clear_recorded_attract_instruction_laser_addresses(&mut cleared_addresses)?;
        let previous_link_address = self.kill_process(laser_process_address)?;
        let killed_process = RedLabelKilledProcess {
            killed_process_address: laser_process_address,
            previous_link_address,
        };
        Ok(RedLabelAttractInstructionLaserKill {
            laser_process_address,
            laser_state_start,
            laser_state_end,
            cleared_addresses,
            killed_process,
        })
    }

    pub(super) fn clear_attract_instruction_laser_path(
        &mut self,
        start: u16,
        end: u16,
        cleared_addresses: &mut Vec<u16>,
    ) -> Result<(), String> {
        let mut address = start;
        for _ in 0..0x100 {
            self.write_byte(address, 0)?;
            if !cleared_addresses.contains(&address) {
                cleared_addresses.push(address);
            }
            let next = address.wrapping_add(0x0100);
            if next > end {
                break;
            }
            address = next;
        }
        Ok(())
    }

    pub(super) fn record_attract_instruction_laser_path(&mut self, start: u16, end: u16) {
        let mut address = start;
        for _ in 0..0x100 {
            self.record_attract_instruction_laser_address(address);
            let next = address.wrapping_add(0x0100);
            if next > end {
                break;
            }
            address = next;
        }
    }

    pub(super) fn record_attract_instruction_laser_path_exclusive(&mut self, start: u16, end: u16) {
        let mut address = start;
        for _ in 0..0x100 {
            if address == end {
                break;
            }
            self.record_attract_instruction_laser_address(address);
            address = address.wrapping_add(0x0100);
        }
    }

    pub(super) fn record_attract_instruction_laser_address(&mut self, address: u16) {
        if !self.attract_instruction_laser_addresses.contains(&address) {
            self.attract_instruction_laser_addresses.push(address);
        }
    }

    pub(super) fn clear_recorded_attract_instruction_laser_addresses(
        &mut self,
        cleared_addresses: &mut Vec<u16>,
    ) -> Result<(), String> {
        let recorded_addresses = std::mem::take(&mut self.attract_instruction_laser_addresses);
        for address in recorded_addresses {
            self.write_byte(address, 0)?;
            if !cleared_addresses.contains(&address) {
                cleared_addresses.push(address);
            }
        }
        Ok(())
    }

    pub(super) fn run_attract_instruction_free_fall_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelAttractInstructionFreeFall, String> {
        let process_address = self.current_process_address(layout)?;
        let astronaut_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM)?;
        let y_velocity_before = self.read_object_word(layout, astronaut_object, "OYV")?;
        let y_velocity_after =
            y_velocity_before.wrapping_add(RED_LABEL_ATTRACT_INSTRUCTION_FREE_FALL_ACCELERATION);
        self.write_object_word(layout, astronaut_object, "OYV", y_velocity_after)?;
        let free_fall_before = self.read_byte(RED_LABEL_ATTRACT_INSTRUCTION_MAN_FREE_FALL_RAM)?;
        let free_fall_after = free_fall_before.wrapping_sub(1);
        self.write_byte(
            RED_LABEL_ATTRACT_INSTRUCTION_MAN_FREE_FALL_RAM,
            free_fall_after,
        )?;
        if free_fall_after == 0 {
            return self
                .start_attract_instruction_intersection_current_process_with_layout(layout)
                .map(RedLabelAttractInstructionFreeFall::Intersection);
        }

        let wakeup_address = red_label_routine_address("AMODE4")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_FREE_FALL_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractInstructionFreeFall::Sleeping {
            process_address,
            astronaut_object,
            y_velocity_before,
            y_velocity_after,
            free_fall_before,
            free_fall_after,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_FREE_FALL_SLEEP_TICKS,
            wakeup_address,
        })
    }

    pub(super) fn start_attract_instruction_intersection_current_process_with_layout(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelAttractInstructionIntersection, String> {
        let process_address = self.current_process_address(layout)?;
        let points_object = self.init_attract_instruction_object_cell(
            layout,
            "C5P1",
            RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_X16,
            RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_Y16,
            0,
            RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_OBJECT_RAM,
        )?;
        let ship_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM)?;
        self.write_object_word(layout, ship_object, "OXV", 0)?;
        self.write_object_word(
            layout,
            ship_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_SHIP_Y_VELOCITY,
        )?;
        let astronaut_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM)?;
        self.write_object_word(
            layout,
            astronaut_object,
            "OX16",
            RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_MAN_X16,
        )?;
        self.write_object_word(
            layout,
            astronaut_object,
            "OY16",
            RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_MAN_Y16,
        )?;
        self.write_object_word(
            layout,
            astronaut_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_SHIP_Y_VELOCITY,
        )?;
        let wakeup_address = red_label_routine_address("AMODE7")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionIntersection {
            process_address,
            points_object,
            ship_object,
            astronaut_object,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_INTERSECTION_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMODE7`: move the 500-point object down, stop the man,
    /// turn the ship around, and sleep to `AMODE8`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L594-L604>.
    pub fn start_attract_instruction_ship_return_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionShipReturn, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let points_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_OBJECT_RAM)?;
        self.write_object_word(
            &layout,
            points_object,
            "OY16",
            RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_RETURN_Y16,
        )?;
        self.write_object_word(
            &layout,
            points_object,
            "OX16",
            RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_RETURN_X16,
        )?;
        let astronaut_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_MAN_POINTER_RAM)?;
        self.write_object_word(&layout, astronaut_object, "OYV", 0)?;
        let ship_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM)?;
        self.write_object_word(
            &layout,
            ship_object,
            "OPICT",
            red_label_object_picture_address("PLBPIC")?,
        )?;
        self.write_object_word(
            &layout,
            ship_object,
            "OXV",
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_RETURN_X_VELOCITY,
        )?;
        self.write_object_word(
            &layout,
            ship_object,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_RETURN_Y_VELOCITY,
        )?;
        let wakeup_address = red_label_routine_address("AMODE8")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_SHIP_RETURN_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionShipReturn {
            process_address,
            points_object,
            ship_object,
            astronaut_object,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_SHIP_RETURN_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMODE8`: restore the ship, remove the 500-point object,
    /// then fall through the first `AMOD12` enemy-table spawn.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L606-L638>.
    pub fn start_attract_instruction_enemy_table_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionEnemyTableStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let ship_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_SHIP_POINTER_RAM)?;
        self.write_object_word(
            &layout,
            ship_object,
            "OPICT",
            red_label_object_picture_address("PLAPIC")?,
        )?;
        self.write_object_word(&layout, ship_object, "OXV", 0)?;
        self.write_object_word(&layout, ship_object, "OYV", 0)?;
        let points_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_500_POINT_OBJECT_RAM)?;
        let points_previous_link_address = self.kill_object_cell_offscreen(points_object)?;
        let enemy = self.spawn_attract_instruction_enemy_from_table(
            &layout,
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_ADDRESS,
        )?;

        Ok(RedLabelAttractInstructionEnemyTableStart {
            process_address,
            points_object,
            points_previous_link_address,
            ship_object,
            enemy,
        })
    }

    pub(super) fn spawn_attract_instruction_enemy_from_table(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table_pointer_before: u16,
    ) -> Result<RedLabelAttractInstructionEnemySpawn, String> {
        let table_index = self.attract_instruction_enemy_table_index(table_pointer_before)?;
        let index = usize::from(table_index);
        let object_address = self.get_object_cell()?;
        let picture_address = red_label_object_picture_address(
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_PICTURES[index],
        )?;
        self.write_object_word(layout, object_address, "OPICT", picture_address)?;
        self.write_object_word(
            layout,
            object_address,
            "OBJCOL",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_BLIPS[index],
        )?;
        self.write_object_word(
            layout,
            object_address,
            "OX16",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_X16,
        )?;
        self.write_object_word(
            layout,
            object_address,
            "OY16",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y16_LOW,
        )?;
        self.write_object_word(
            layout,
            object_address,
            "OYV",
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y_VELOCITY,
        )?;
        self.write_object_word(layout, object_address, "OXV", 0)?;
        let appearance = self.start_appearance_for_object(object_address)?;
        let table_pointer_after = table_pointer_before;
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_OBJECT_TABLE_POINTER_RAM,
            table_pointer_after,
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_POINTER_RAM,
            object_address,
        )?;
        let wakeup_address = red_label_routine_address("AMOD10")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionEnemySpawn {
            table_pointer_before,
            table_pointer_after,
            table_index,
            object_address,
            picture_address,
            x16: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_X16,
            y16: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y16_LOW,
            source_table_y16: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y[index],
            y_velocity: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y_VELOCITY,
            color: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_BLIPS[index],
            appearance,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped direct `AMOD12`: spawn the enemy pointed at by `OTABPT`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L625-L638>.
    pub fn spawn_attract_instruction_enemy_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionEnemySpawn, String> {
        let layout = red_label_ram_layout()?;
        let table_pointer =
            self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_OBJECT_TABLE_POINTER_RAM)?;
        self.spawn_attract_instruction_enemy_from_table(&layout, table_pointer)
    }

    /// Source-shaped `AMOD10`: start the per-enemy laser and sleep to
    /// `AMOD11`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L639-L641>.
    pub fn start_attract_instruction_table_laser_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionLaserStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let laser_process = self.make_process(
            red_label_routine_address("LASRS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_LASER_PROCESS_RAM,
            laser_process.process_address,
        )?;
        let wakeup_address = red_label_routine_address("AMOD11")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_TABLE_LASER_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionLaserStart {
            process_address,
            laser_process,
            laser_process_pointer_address: RED_LABEL_ATTRACT_INSTRUCTION_LASER_PROCESS_RAM,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_TABLE_LASER_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMOD11`: kill the laser, explode the active enemy, reuse
    /// its object cell for the source table X/Y position, and sleep to
    /// `BMODE2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L642-L657>.
    pub fn resolve_attract_instruction_enemy_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionEnemyResolve, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let laser_kill = self.kill_attract_instruction_laser_process(&layout)?;
        let enemy_object = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_POINTER_RAM)?;
        let enemy_previous_link_address = self.kill_object_cell(enemy_object)?;
        let _reused_object = self.get_object_cell()?;
        let enemy_explosion = self.start_explosion_for_object(enemy_object)?;
        let table_pointer_before =
            self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_OBJECT_TABLE_POINTER_RAM)?;
        let table_index = self.attract_instruction_enemy_table_index(table_pointer_before)?;
        let index = usize::from(table_index);
        let x16 = RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_X[index];
        let y16 = RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_Y[index];
        self.write_object_word(&layout, enemy_object, "OX16", x16)?;
        self.write_object_word(&layout, enemy_object, "OY16", y16)?;
        self.write_object_word(&layout, enemy_object, "OYV", 0)?;
        self.write_object_word(&layout, enemy_object, "OXV", 0)?;
        let appearance = self.start_appearance_for_object(enemy_object)?;
        let table_pointer_after = table_pointer_before.wrapping_add(2);
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_OBJECT_TABLE_POINTER_RAM,
            table_pointer_after,
        )?;
        let wakeup_address = red_label_routine_address("BMODE2")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_RESOLVE_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionEnemyResolve {
            process_address,
            laser_kill,
            table_pointer_before,
            table_pointer_after,
            table_index,
            enemy_object,
            enemy_previous_link_address,
            enemy_explosion,
            x16,
            y16,
            appearance,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_RESOLVE_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `BMODE2`: advance `TEXPTR` and sleep to `BMODE3`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L658-L661>.
    pub fn advance_attract_instruction_text_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionTextAdvance, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let text_pointer_before = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_POINTER_RAM)?;
        let text_pointer_after = text_pointer_before.wrapping_add(2);
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_TEXT_POINTER_RAM,
            text_pointer_after,
        )?;
        let wakeup_address = red_label_routine_address("BMODE3")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_TEXT_ADVANCE_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionTextAdvance {
            process_address,
            text_pointer_before,
            text_pointer_after,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_TEXT_ADVANCE_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `TEXTP`: write the first instruction-page text line,
    /// store `TEXTMP`, and sleep to `TEXTP2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L703-L708>.
    pub fn start_attract_instruction_text_process_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionTextProcess, String> {
        self.write_attract_instruction_text_process_line(
            RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE,
            false,
        )
    }

    /// Source-shaped `TEXTP2`: resume from `TEXTMP` while it is below
    /// `TEXPTR`; otherwise branch directly back to `TEXTP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L709-L712>.
    pub fn continue_attract_instruction_text_process_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionTextProcess, String> {
        let text_pointer = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TEMP_RAM)?;
        let text_pointer_limit = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_POINTER_RAM)?;
        let (table_pointer, restarted) = if text_pointer == text_pointer_limit {
            (RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE, true)
        } else {
            (text_pointer, false)
        };
        self.write_attract_instruction_text_process_line(table_pointer, restarted)
    }

    pub(super) fn write_attract_instruction_text_process_line(
        &mut self,
        table_pointer_before: u16,
        restarted: bool,
    ) -> Result<RedLabelAttractInstructionTextProcess, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let table_index = self.attract_instruction_text_table_index(table_pointer_before)?;
        let index = usize::from(table_index);
        let message_label = RED_LABEL_ATTRACT_INSTRUCTION_TEXT_LABELS[index];
        let message = red_label_message(message_label)?;
        let message_screen_address = RED_LABEL_ATTRACT_INSTRUCTION_TEXT_SCREEN_ADDRESSES[index];
        let cursor_after =
            self.write_message_text_block(&layout, message_screen_address, message)?;
        let table_pointer_after =
            table_pointer_before.wrapping_add(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_ENTRY_SIZE);
        self.write_word(
            RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TEMP_RAM,
            table_pointer_after,
        )?;
        let text_pointer_limit = self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_POINTER_RAM)?;
        let wakeup_address = red_label_routine_address("TEXTP2")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_TEXT_SLEEP_TICKS,
            wakeup_address,
        )?;

        Ok(RedLabelAttractInstructionTextProcess {
            process_address,
            restarted,
            table_pointer_before,
            table_pointer_after,
            text_pointer_limit,
            message_vector_address: message.vector_address,
            message_screen_address,
            message_label,
            cursor_after,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_TEXT_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `BMODE3`: continue with `AMOD12` until `OTABND`, then
    /// sleep to `AMOD13`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L662-L665>.
    pub fn decide_attract_instruction_table_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionTableDecision, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let table_pointer =
            self.read_word(RED_LABEL_ATTRACT_INSTRUCTION_OBJECT_TABLE_POINTER_RAM)?;
        if table_pointer != RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_END {
            return self
                .spawn_attract_instruction_enemy_from_table(&layout, table_pointer)
                .map(RedLabelAttractInstructionTableDecision::NextEnemy);
        }

        let wakeup_address = red_label_routine_address("AMOD13")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_TABLE_END_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractInstructionTableDecision::TableEnded {
            process_address,
            table_pointer,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_TABLE_END_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `AMOD13`: sleep before restarting the Williams page.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L666-L666>.
    pub fn restart_attract_instruction_current_process(
        &mut self,
    ) -> Result<RedLabelAttractInstructionRestart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let wakeup_address = red_label_routine_address("AMODES")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_INSTRUCTION_RESTART_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractInstructionRestart {
            process_address,
            sleep_ticks: RED_LABEL_ATTRACT_INSTRUCTION_RESTART_SLEEP_TICKS,
            wakeup_address,
        })
    }

    pub(super) fn attract_instruction_enemy_table_index(
        &self,
        table_pointer: u16,
    ) -> Result<u8, String> {
        if !(RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_ADDRESS
            ..RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_END)
            .contains(&table_pointer)
            || !(table_pointer - RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_ADDRESS)
                .is_multiple_of(2)
        {
            return Err(format!(
                "red-label attract enemy table pointer 0x{table_pointer:04X} is invalid"
            ));
        }
        Ok(((table_pointer - RED_LABEL_ATTRACT_INSTRUCTION_ENEMY_TABLE_ADDRESS) / 2) as u8)
    }

    pub(super) fn attract_instruction_text_table_index(
        &self,
        table_pointer: u16,
    ) -> Result<u8, String> {
        if table_pointer < RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE
            || table_pointer
                >= RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE
                    .wrapping_add(RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE_BYTES)
            || !(table_pointer - RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE).is_multiple_of(2)
        {
            return Err(format!(
                "red-label attract instruction text table pointer 0x{table_pointer:04X} is invalid"
            ));
        }
        Ok(((table_pointer - RED_LABEL_ATTRACT_INSTRUCTION_TEXT_TABLE) / 2) as u8)
    }

    /// Source-shaped `CREDS`: refresh the attract credits label and BCD
    /// number, flag newly-increased credit counts, then sleep back to `CREDS`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L978-L990>.
    pub fn display_attract_credits_current_process(
        &mut self,
    ) -> Result<RedLabelAttractCredits, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let credit = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        let old_credit_before = self.read_byte(RED_LABEL_ATTRACT_OLD_CREDIT_RAM)?;
        let credit_increase_flag_before =
            self.read_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM)?;
        let mut old_credit_after = old_credit_before;
        let mut credit_increase_flag_after = credit_increase_flag_before;
        if credit > old_credit_before {
            self.write_byte(RED_LABEL_ATTRACT_OLD_CREDIT_RAM, credit)?;
            old_credit_after = credit;
            credit_increase_flag_after = credit_increase_flag_after.wrapping_add(1);
            self.write_byte(
                RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM,
                credit_increase_flag_after,
            )?;
        }

        let text = Some(self.write_attract_credits_text(&layout, credit)?);
        let wakeup_address = red_label_routine_address("CREDS")?;
        self.sleep_current_process(RED_LABEL_ATTRACT_CREDIT_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelAttractCredits {
            process_address,
            credit,
            old_credit_before,
            old_credit_after,
            credit_increase_flag_before,
            credit_increase_flag_after,
            text,
            sleep_ticks: RED_LABEL_ATTRACT_CREDIT_SLEEP_TICKS,
            wakeup_address,
        })
    }

    pub(super) fn write_attract_credits_text(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        credit: u8,
    ) -> Result<RedLabelAttractCreditsText, String> {
        let message = red_label_message("CREDV")?;
        self.write_message_text_block(layout, RED_LABEL_ATTRACT_CREDITS_SCREEN, message)?;
        self.write_field_word(
            layout,
            "base_page",
            "CURSER",
            RED_LABEL_ATTRACT_CREDIT_NUMBER_SCREEN,
        )?;
        self.write_message_number_block(layout, credit)?;
        Ok(RedLabelAttractCreditsText {
            message_vector_address: message.vector_address,
            message_screen_address: RED_LABEL_ATTRACT_CREDITS_SCREEN,
            number_screen_address: RED_LABEL_ATTRACT_CREDIT_NUMBER_SCREEN,
            displayed_credit_bcd: credit,
        })
    }

    /// Source-shaped `AMODES`: prepare the Williams logo page before entering
    /// `LOGO`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L717-L730>.
    pub fn start_attract_williams_page_current_process(
        &mut self,
    ) -> Result<RedLabelAttractWilliamsPage, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let genocide = self.genocide_other_processes()?;
        self.write_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM, 0)?;
        self.write_field_byte(
            &layout,
            "base_page",
            "STATUS",
            RED_LABEL_ATTRACT_WILLIAMS_STATUS,
        )?;
        let screen_clear = self.clear_screen_ram()?;
        self.write_byte(RED_LABEL_ATTRACT_MESSAGE_POINTER_RAM, 0)?;
        self.write_word(RED_LABEL_ATTRACT_NUMBER_RAM, 0xFFFF)?;
        let support_processes = vec![
            self.make_process(
                red_label_routine_address("COLR")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
            self.make_process(
                red_label_routine_address("TIECOL")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?,
        ];
        let logo_color_address = field_range(&layout, "base_page", "PCRAM")?.start + 0x0C;
        self.write_byte(logo_color_address, RED_LABEL_ATTRACT_WILLIAMS_LOGO_COLOR)?;
        let logo_address = red_label_routine_address("LOGO")?;
        self.write_process_word(&layout, process_address, "PADDR", logo_address)?;
        Ok(RedLabelAttractWilliamsPage {
            process_address,
            genocide,
            status: RED_LABEL_ATTRACT_WILLIAMS_STATUS,
            screen_clear,
            message_pointer_high_address: RED_LABEL_ATTRACT_MESSAGE_POINTER_RAM,
            number_address: RED_LABEL_ATTRACT_NUMBER_RAM,
            support_processes,
            logo_color_address,
            logo_color: RED_LABEL_ATTRACT_WILLIAMS_LOGO_COLOR,
            logo_address,
        })
    }

    /// Source-shaped `LOGO`: expand the later `DEFENDER` picture data, seed the
    /// Williams-logo table walker, and run the first table slice.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L735-L796>.
    pub fn start_attract_logo_current_process(&mut self) -> Result<RedLabelAttractLogo, String> {
        let logo = expanded_defender_logo_image();
        let logo_ram_range = checked_defender_logo_ram_range(self.ram.len())?;
        self.ram[logo_ram_range].copy_from_slice(&logo);
        self.write_byte(
            RED_LABEL_ATTRACT_LOGO_FLAG_RAM,
            RED_LABEL_ATTRACT_LOGO_INITIAL_BYTES_PER_SLICE,
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_LOGO_POINTER_RAM,
            RED_LABEL_ATTRACT_LOGO_TABLE_ADDRESS,
        )?;
        self.step_attract_logo_table_current_process_with_table(
            true,
            Some(crc32(&logo)),
            RED_LABEL_ATTRACT_LOGO_TABLE_ADDRESS,
            &RED_LABEL_ATTRACT_LOGO_TABLE,
        )
    }

    /// Source-shaped `LOGO0`: consume the configured number of `LGOTAB`
    /// entries, write pixels through the ROM cursor format, and sleep back to
    /// `LOGO0`. Hitting `QUIT` switches the first pass to the fast rate and
    /// starts `PRES` before continuing from the table start.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L740-L796>.
    pub fn step_attract_logo_table_current_process(
        &mut self,
    ) -> Result<RedLabelAttractLogo, String> {
        self.step_attract_logo_table_current_process_with_table(
            false,
            None,
            RED_LABEL_ATTRACT_LOGO_TABLE_ADDRESS,
            &RED_LABEL_ATTRACT_LOGO_TABLE,
        )
    }

    pub(super) fn step_attract_logo_table_current_process_with_table(
        &mut self,
        initialized: bool,
        defender_logo_ram_crc32: Option<u32>,
        table_base: u16,
        table: &[u8],
    ) -> Result<RedLabelAttractLogo, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let table_pointer_before = self.read_word(RED_LABEL_ATTRACT_LOGO_POINTER_RAM)?;
        let cursor_before = self.read_word(RED_LABEL_ATTRACT_LOGO_CURSOR_RAM)?;
        let mut table_pointer = table_pointer_before;
        let mut cursor = cursor_before;
        let mut bytes_per_slice = self.read_byte(RED_LABEL_ATTRACT_LOGO_FLAG_RAM)?;
        let mut table_operations = 0u16;
        let mut pixel_writes = Vec::new();
        let mut first_pass_completed = false;
        let mut presents_process = None;
        let mut last_screen_address = attract_logo_screen_address(cursor).0;

        for _ in 0..1024 {
            let mut bytes_remaining = bytes_per_slice;
            self.write_byte(RED_LABEL_ATTRACT_LOGO_TEMP_B_RAM, bytes_remaining)?;
            loop {
                let opcode = attract_logo_table_byte(table_base, table, table_pointer)?;
                table_pointer = table_pointer.wrapping_add(1);
                if opcode > 0xAA {
                    let complemented = !opcode;
                    if complemented == 0 {
                        continue;
                    }
                    if complemented.wrapping_sub(1) != 0 {
                        self.write_word(
                            RED_LABEL_ATTRACT_LOGO_CURSOR_END_RAM,
                            last_screen_address,
                        )?;
                        if bytes_per_slice == RED_LABEL_ATTRACT_LOGO_INITIAL_BYTES_PER_SLICE {
                            bytes_per_slice = RED_LABEL_ATTRACT_LOGO_FAST_BYTES_PER_SLICE;
                            self.write_byte(RED_LABEL_ATTRACT_LOGO_FLAG_RAM, bytes_per_slice)?;
                            presents_process = Some(self.make_process(
                                red_label_routine_address("PRES")?,
                                RED_LABEL_SYSTEM_PROCESS_TYPE,
                            )?);
                            first_pass_completed = true;
                        }
                        table_pointer = table_base;
                        self.write_word(RED_LABEL_ATTRACT_LOGO_POINTER_RAM, table_pointer)?;
                        break;
                    }
                    let cursor_high = attract_logo_table_byte(table_base, table, table_pointer)?;
                    table_pointer = table_pointer.wrapping_add(1);
                    let cursor_low = attract_logo_table_byte(table_base, table, table_pointer)?;
                    table_pointer = table_pointer.wrapping_add(1);
                    cursor = u16::from_be_bytes([cursor_high, cursor_low]);
                    self.write_word(RED_LABEL_ATTRACT_LOGO_CURSOR_RAM, cursor)?;
                    self.write_byte(RED_LABEL_ATTRACT_LOGO_TEMP_A_RAM, 0)?;
                    let write = self.write_attract_logo_pixel(cursor)?;
                    last_screen_address = write.screen_address;
                    pixel_writes.push(write);
                } else {
                    let mut accumulator = opcode;
                    loop {
                        accumulator =
                            attract_logo_apply_horizontal_bit(accumulator, &mut cursor, true);
                        accumulator =
                            attract_logo_apply_horizontal_bit(accumulator, &mut cursor, false);
                        accumulator =
                            attract_logo_apply_vertical_bit(accumulator, &mut cursor, true);
                        accumulator =
                            attract_logo_apply_vertical_bit(accumulator, &mut cursor, false);
                        self.write_word(RED_LABEL_ATTRACT_LOGO_CURSOR_RAM, cursor)?;
                        self.write_byte(RED_LABEL_ATTRACT_LOGO_TEMP_A_RAM, accumulator)?;
                        let write = self.write_attract_logo_pixel(cursor)?;
                        last_screen_address = write.screen_address;
                        pixel_writes.push(write);
                        if accumulator == 0 {
                            break;
                        }
                    }
                }

                table_operations = table_operations.wrapping_add(1);
                bytes_remaining = bytes_remaining.wrapping_sub(1);
                self.write_byte(RED_LABEL_ATTRACT_LOGO_TEMP_B_RAM, bytes_remaining)?;
                if bytes_remaining != 0 {
                    continue;
                }

                self.write_word(RED_LABEL_ATTRACT_LOGO_POINTER_RAM, table_pointer)?;
                let wakeup_address = red_label_routine_address("LOGO0")?;
                self.sleep_current_process(RED_LABEL_ATTRACT_LOGO_SLEEP_TICKS, wakeup_address)?;
                return Ok(RedLabelAttractLogo {
                    process_address,
                    initialized,
                    defender_logo_ram_crc32,
                    table_pointer_before,
                    table_pointer_after: table_pointer,
                    cursor_before,
                    cursor_after: self.read_word(RED_LABEL_ATTRACT_LOGO_CURSOR_RAM)?,
                    bytes_per_slice,
                    table_operations,
                    pixel_writes,
                    first_pass_completed,
                    presents_process,
                    sleep_ticks: RED_LABEL_ATTRACT_LOGO_SLEEP_TICKS,
                    wakeup_address,
                });
            }
        }

        Err(String::from(
            "red-label LOGO table walker did not reach a sleep point",
        ))
    }

    pub(super) fn write_attract_logo_pixel(
        &mut self,
        cursor: u16,
    ) -> Result<RedLabelAttractLogoPixel, String> {
        let (screen_address, pixel_mask) = attract_logo_screen_address(cursor);
        let byte_before = self.read_byte(screen_address)?;
        let byte_after = byte_before | pixel_mask;
        self.write_byte(screen_address, byte_after)?;
        Ok(RedLabelAttractLogoPixel {
            cursor,
            screen_address,
            pixel_mask,
            byte_before,
            byte_after,
        })
    }

    /// Source-shaped `PRES`: create `DEFEND`, write the `ELECV` electronics /
    /// presents text block, then sleep back to `PRES1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L800-L806>.
    pub fn start_attract_presents_current_process(
        &mut self,
    ) -> Result<RedLabelAttractPresents, String> {
        let defender_process = self.make_process(
            red_label_routine_address("DEFEND")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        self.write_attract_presents_text(Some(defender_process))
    }

    /// Source-shaped `PRES1`: redraw the `ELECV` block and sleep back to itself.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L801-L806>.
    pub fn refresh_attract_presents_current_process(
        &mut self,
    ) -> Result<RedLabelAttractPresents, String> {
        self.write_attract_presents_text(None)
    }

    pub(super) fn write_attract_presents_text(
        &mut self,
        defender_process: Option<RedLabelCreatedProcess>,
    ) -> Result<RedLabelAttractPresents, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let message = red_label_message("ELECV")?;
        self.write_text_bytes_with_space(
            &layout,
            RED_LABEL_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            b"ELECTRONICS INC.",
        )?;
        self.write_text_bytes_with_space(
            &layout,
            RED_LABEL_ATTRACT_PRESENTS_TEXT_SCREEN,
            b"PRESENTS",
        )?;

        let wakeup_address = red_label_routine_address("PRES1")?;
        self.sleep_current_process(RED_LABEL_ATTRACT_PRESENTS_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelAttractPresents {
            process_address,
            defender_process,
            message_vector_address: message.vector_address,
            electronics_screen_address: RED_LABEL_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            presents_screen_address: RED_LABEL_ATTRACT_PRESENTS_TEXT_SCREEN,
            cursor_after: RED_LABEL_ATTRACT_PRESENTS_CURSOR_AFTER,
            sleep_ticks: RED_LABEL_ATTRACT_PRESENTS_SLEEP_TICKS,
            wakeup_address,
        })
    }

    pub(super) fn write_trace_partial_attract_presents_text(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.write_text_bytes_with_space(
            &layout,
            RED_LABEL_ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
            b"ELECTRONICS INC.",
        )?;
        let mut cursor = RED_LABEL_ATTRACT_PRESENTS_TEXT_SCREEN;
        self.write_field_word(&layout, "base_page", "CURSER", cursor)?;
        for byte in b"PRESENT" {
            cursor = self.write_text_byte(cursor, *byte)?;
        }
        self.write_field_word(&layout, "base_page", "CURSER", cursor)
    }

    pub(super) fn finish_trace_partial_attract_presents_text(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.write_text_bytes_with_space(
            &layout,
            RED_LABEL_ATTRACT_PRESENTS_TEXT_SCREEN,
            b"PRESENTS",
        )?;
        Ok(())
    }

    /// Source-shaped `DEFEND`: wait before the Defender wordmark appearance
    /// objects are started.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L810-L810>.
    pub fn delay_attract_defender_current_process(
        &mut self,
    ) -> Result<RedLabelAttractDefenderDelay, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let wakeup_address = red_label_routine_address("DEFENS")?;
        self.sleep_current_process(RED_LABEL_ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelAttractDefenderDelay {
            process_address,
            sleep_ticks: RED_LABEL_ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `DEFENS`: build the compact `DEFRAM` object blocks and
    /// `DEFPIC` descriptors, start one `APVCT` appearance per 4x12 logo slice,
    /// then sleep to `DEF33`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L811-L845>.
    pub fn start_attract_defender_appearances_current_process(
        &mut self,
    ) -> Result<RedLabelAttractDefenderAppearances, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let descriptor_pointer_start = RED_LABEL_ATTRACT_DEFENDER_PICTURES;
        let data_pointer_start = RED_LABEL_ATTRACT_DEFENDER_DATA;
        let object_pointer_start = RED_LABEL_ATTRACT_DEFENDER_OBJECTS;
        let bgl = 0;
        let initial_x16 = RED_LABEL_ATTRACT_DEFENDER_INITIAL_X16;

        self.write_word(
            RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR_POINTER_RAM,
            descriptor_pointer_start,
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_DEFENDER_DATA_POINTER_RAM,
            data_pointer_start,
        )?;
        self.write_field_word(&layout, "base_page", "BGL", bgl)?;
        self.write_word(RED_LABEL_ATTRACT_DEFENDER_X_POINTER_RAM, initial_x16)?;
        self.write_word(
            RED_LABEL_ATTRACT_DEFENDER_OBJECT_POINTER_RAM,
            object_pointer_start,
        )?;

        let mut descriptor_pointer = descriptor_pointer_start;
        let mut data_pointer = data_pointer_start;
        let mut object_pointer = object_pointer_start;
        let mut x16 = initial_x16;
        let mut objects = Vec::with_capacity(usize::from(RED_LABEL_ATTRACT_DEFENDER_OBJECT_COUNT));
        for index in 0..RED_LABEL_ATTRACT_DEFENDER_OBJECT_COUNT {
            self.write_word(
                descriptor_pointer,
                u16::from_be_bytes([
                    RED_LABEL_ATTRACT_DEFENDER_PICTURE_WIDTH,
                    RED_LABEL_ATTRACT_DEFENDER_PICTURE_HEIGHT,
                ]),
            )?;
            self.write_word(descriptor_pointer + 2, data_pointer)?;
            data_pointer = data_pointer.wrapping_add(RED_LABEL_ATTRACT_DEFENDER_PICTURE_DATA_STEP);
            self.write_word(RED_LABEL_ATTRACT_DEFENDER_DATA_POINTER_RAM, data_pointer)?;

            self.write_word(
                object_pointer + RED_LABEL_ATTRACT_DEFENDER_OBJECT_PICTURE_OFFSET,
                descriptor_pointer,
            )?;
            self.write_word(
                object_pointer + RED_LABEL_ATTRACT_DEFENDER_OBJECT_X16_OFFSET,
                x16,
            )?;
            x16 = x16.wrapping_add(RED_LABEL_ATTRACT_DEFENDER_X16_STEP);
            self.write_word(RED_LABEL_ATTRACT_DEFENDER_X_POINTER_RAM, x16)?;
            self.write_word(
                object_pointer + RED_LABEL_ATTRACT_DEFENDER_OBJECT_Y16_OFFSET,
                RED_LABEL_ATTRACT_DEFENDER_Y16,
            )?;

            let appearance = self.start_appearance_for_raw_object_cell(object_pointer)?;
            objects.push(RedLabelAttractDefenderObject {
                index,
                object_address: object_pointer,
                picture_descriptor_address: descriptor_pointer,
                picture_data_address: data_pointer
                    .wrapping_sub(RED_LABEL_ATTRACT_DEFENDER_PICTURE_DATA_STEP),
                x16: x16.wrapping_sub(RED_LABEL_ATTRACT_DEFENDER_X16_STEP),
                y16: RED_LABEL_ATTRACT_DEFENDER_Y16,
                appearance,
            });

            object_pointer = object_pointer.wrapping_add(RED_LABEL_ATTRACT_DEFENDER_OBJECT_BYTES);
            self.write_word(
                RED_LABEL_ATTRACT_DEFENDER_OBJECT_POINTER_RAM,
                object_pointer,
            )?;
            descriptor_pointer =
                descriptor_pointer.wrapping_add(RED_LABEL_ATTRACT_DEFENDER_PICTURE_BYTES);
            self.write_word(
                RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR_POINTER_RAM,
                descriptor_pointer,
            )?;
        }

        let wakeup_address = red_label_routine_address("DEF33")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractDefenderAppearances {
            process_address,
            descriptor_pointer_start,
            data_pointer_start,
            object_pointer_start,
            bgl,
            initial_x16,
            objects,
            descriptor_pointer_after: descriptor_pointer,
            data_pointer_after: data_pointer,
            object_pointer_after: object_pointer,
            x16_after: x16,
            sleep_ticks: RED_LABEL_ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `DEF33`: build the whole `DEFENDER` descriptor, create
    /// the `DEF50` refresh process, then sleep to `DEF44`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L846-L852>.
    pub fn start_attract_defender_restore_current_process(
        &mut self,
    ) -> Result<RedLabelAttractDefenderRestoreStart, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_word(
            RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR,
            u16::from_be_bytes([
                RED_LABEL_ATTRACT_DEFENDER_WHOLE_WIDTH,
                RED_LABEL_ATTRACT_DEFENDER_WHOLE_HEIGHT,
            ]),
        )?;
        self.write_word(
            RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR + 2,
            RED_LABEL_ATTRACT_DEFENDER_DATA,
        )?;
        let restore_process = self.make_process(
            red_label_routine_address("DEF50")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let wakeup_address = red_label_routine_address("DEF44")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_DEFENDER_RESTORE_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractDefenderRestoreStart {
            process_address,
            descriptor_address: RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR,
            picture_address: RED_LABEL_ATTRACT_DEFENDER_DATA,
            width: RED_LABEL_ATTRACT_DEFENDER_WHOLE_WIDTH,
            height: RED_LABEL_ATTRACT_DEFENDER_WHOLE_HEIGHT,
            restore_process,
            sleep_ticks: RED_LABEL_ATTRACT_DEFENDER_RESTORE_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `DEF44`: start the color-bomb process, then fall through
    /// into `COPYRT` for the copyright bitmap and wait gate.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L853-L895>.
    pub fn start_attract_copyright_current_process(
        &mut self,
    ) -> Result<RedLabelAttractCopyright, String> {
        self.write_attract_copyright_current_process_with_bomb(true)
    }

    /// Source-shaped `COPYRT`: expand the copyright bitmap, set the
    /// Williams/power flags, start credits, then run the first `CPR55` wait
    /// check.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L857-L895>.
    pub fn write_attract_copyright_current_process(
        &mut self,
    ) -> Result<RedLabelAttractCopyright, String> {
        self.write_attract_copyright_current_process_with_bomb(false)
    }

    pub(super) fn write_attract_copyright_current_process_with_bomb(
        &mut self,
        start_bomb_process: bool,
    ) -> Result<RedLabelAttractCopyright, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let bomb_process = if start_bomb_process {
            Some(self.make_process(
                red_label_routine_address("CBOMB")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?)
        } else {
            None
        };

        self.write_attract_copyright_bitmap()?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        let williams_color_address =
            pcram.wrapping_add(RED_LABEL_ATTRACT_COPYRIGHT_WILLIAMS_COLOR_INDEX);
        let williams_color = self.read_byte(williams_color_address)?;
        let williams_cursor = self.read_word(RED_LABEL_ATTRACT_LOGO_CURSOR_END_RAM)?;
        let williams_cursor_transferred = (!williams_color) & 0x07 == 0;
        if williams_cursor_transferred {
            self.write_field_word(&layout, "base_page", "WCURS", williams_cursor)?;
        }

        let power_flag = 1;
        self.write_field_byte(&layout, "base_page", "PWRFLG", power_flag)?;
        let credits_process = self.make_process(
            red_label_routine_address("CREDS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        self.write_byte(
            RED_LABEL_HOF_STALL_TIMER_RAM,
            RED_LABEL_ATTRACT_COPYRIGHT_STALL_TICKS,
        )?;
        let wait = self.continue_attract_copyright_wait_current_process()?;
        Ok(RedLabelAttractCopyright {
            process_address,
            bomb_process,
            copyright_screen_address: RED_LABEL_ATTRACT_COPYRIGHT_SCREEN,
            copyright_data_address: RED_LABEL_ATTRACT_COPYRIGHT_DATA_ADDRESS,
            rows: RED_LABEL_ATTRACT_COPYRIGHT_ROWS,
            row_width: RED_LABEL_ATTRACT_COPYRIGHT_ROW_WIDTH,
            williams_color_address,
            williams_color,
            williams_cursor,
            williams_cursor_transferred,
            power_flag,
            credits_process,
            stall_ticks: RED_LABEL_ATTRACT_COPYRIGHT_STALL_TICKS,
            wait,
        })
    }

    /// Source-shaped `CPR55` / `CPR56`: wait for credit input or timeout,
    /// then jump to instructions or hall-of-fame display.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L891-L896>.
    pub fn continue_attract_copyright_wait_current_process(
        &mut self,
    ) -> Result<RedLabelAttractCopyrightWait, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let credit_increase_flag = self.read_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM)?;
        if credit_increase_flag != 0 {
            let target_address = red_label_routine_address("LEDRET")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelAttractCopyrightWait::CreditIncreaseJump {
                process_address,
                credit_increase_flag,
                target_address,
            });
        }

        let stall_before = self.read_byte(RED_LABEL_HOF_STALL_TIMER_RAM)?;
        let stall_after = stall_before.wrapping_sub(1);
        self.write_byte(RED_LABEL_HOF_STALL_TIMER_RAM, stall_after)?;
        if stall_after == 0 {
            let target_address = red_label_routine_address("HALDIS")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelAttractCopyrightWait::HallOfFameJump {
                process_address,
                credit_increase_flag,
                stall_before,
                stall_after,
                target_address,
            });
        }

        let wakeup_address = red_label_routine_address("CPR55")?;
        self.sleep_current_process(RED_LABEL_ATTRACT_COPYRIGHT_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelAttractCopyrightWait::Sleeping {
            process_address,
            credit_increase_flag,
            stall_before,
            stall_after,
            sleep_ticks: RED_LABEL_ATTRACT_COPYRIGHT_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `DEF50`: redraw the whole `DEFENDER` descriptor until the
    /// first two appearance slots are clear, then create `WILLIR` and suicide.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L899-L909>.
    pub fn refresh_attract_defender_current_process(
        &mut self,
    ) -> Result<RedLabelAttractDefenderRefresh, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let picture = self.write_ram_picture_descriptor_cwrit(
            RED_LABEL_ATTRACT_DEFENDER_RESTORE_SCREEN,
            RED_LABEL_ATTRACT_DEFENDER_DESCRIPTOR,
        )?;
        let first_appearance_size = self.read_word(0x9C00)?;
        let second_appearance_size = self.read_word(0x9C40)?;
        if first_appearance_size == 0 && second_appearance_size == 0 {
            let restore_process = self.make_process(
                red_label_routine_address("WILLIR")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            let killed_process = self.kill_current_process(&layout)?;
            return Ok(RedLabelAttractDefenderRefresh::Completed {
                process_address,
                picture,
                first_appearance_size,
                second_appearance_size,
                restore_process,
                killed_process,
            });
        }

        let wakeup_address = red_label_routine_address("DEF50")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_DEFENDER_REFRESH_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractDefenderRefresh::Refreshing {
            process_address,
            picture,
            first_appearance_size,
            second_appearance_size,
            sleep_ticks: RED_LABEL_ATTRACT_DEFENDER_REFRESH_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `DEF51`: sleep one tick, then re-enter `DEF50`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L909-L909>.
    pub fn delay_attract_defender_refresh_current_process(
        &mut self,
    ) -> Result<RedLabelAttractDefenderRefresh, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let wakeup_address = red_label_routine_address("DEF50")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_DEFENDER_REFRESH_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractDefenderRefresh::DelaySleeping {
            process_address,
            sleep_ticks: RED_LABEL_ATTRACT_DEFENDER_REFRESH_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `WILLIR`: force the Williams logo restore to the fastest
    /// rate, then sleep to `WILR1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L994-L996>.
    pub fn start_attract_williams_restore_current_process(
        &mut self,
    ) -> Result<RedLabelAttractWilliamsRestore, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_byte(
            RED_LABEL_ATTRACT_LOGO_FLAG_RAM,
            RED_LABEL_ATTRACT_WILLIAMS_FAST_LOGO_RATE,
        )?;
        let wakeup_address = red_label_routine_address("WILR1")?;
        self.sleep_current_process(
            RED_LABEL_ATTRACT_WILLIAMS_RESTORE_SLEEP_TICKS,
            wakeup_address,
        )?;
        Ok(RedLabelAttractWilliamsRestore::StartedSleeping {
            process_address,
            logo_rate: RED_LABEL_ATTRACT_WILLIAMS_FAST_LOGO_RATE,
            sleep_ticks: RED_LABEL_ATTRACT_WILLIAMS_RESTORE_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `WILR1`: restore the normal logo walker rate and suicide.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L997-L999>.
    pub fn finish_attract_williams_restore_current_process(
        &mut self,
    ) -> Result<RedLabelAttractWilliamsRestore, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_byte(
            RED_LABEL_ATTRACT_LOGO_FLAG_RAM,
            RED_LABEL_ATTRACT_WILLIAMS_NORMAL_LOGO_RATE,
        )?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelAttractWilliamsRestore::Finished {
            process_address,
            logo_rate: RED_LABEL_ATTRACT_WILLIAMS_NORMAL_LOGO_RATE,
            killed_process,
        })
    }

    pub(super) fn write_attract_copyright_bitmap(&mut self) -> Result<(), String> {
        for (row, pair) in RED_LABEL_ATTRACT_COPYRIGHT_DATA.chunks_exact(2).enumerate() {
            let row_offset = u16::try_from(row)
                .expect("red-label CPRTAB row count fits in u16")
                .wrapping_shl(8);
            for bit in 0..RED_LABEL_ATTRACT_COPYRIGHT_ROW_WIDTH {
                let mask = 1u8.wrapping_shl(u32::from(bit));
                let mut output = 0;
                if pair[0] & mask != 0 {
                    output |= 0x10;
                }
                if pair[1] & mask != 0 {
                    output |= 0x01;
                }
                let address = screen_offset(
                    RED_LABEL_ATTRACT_COPYRIGHT_SCREEN,
                    row_offset + u16::from(bit),
                )?;
                self.write_byte(address, output)?;
            }
        }
        Ok(())
    }

    pub(super) fn write_ram_picture_descriptor_cwrit(
        &mut self,
        screen_address: u16,
        descriptor_address: u16,
    ) -> Result<RedLabelPictureWrite, String> {
        let layout = red_label_ram_layout()?;
        let width = self.read_byte(descriptor_address)?;
        let height = self.read_byte(descriptor_address + 1)?;
        let image_address = self.read_word(descriptor_address + 2)?;
        let previous_map = self.read_field_byte(&layout, "base_page", "MAPCR")?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let result = (|| {
            for column in 0..width {
                let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
                let source_column = image_address + u16::from(column) * u16::from(height);
                for row in 0..height {
                    self.write_byte(
                        screen_offset(column_address, u16::from(row))?,
                        self.read_byte(source_column + u16::from(row))?,
                    )?;
                }
            }
            Ok(RedLabelPictureWrite {
                screen_address,
                picture_address: descriptor_address,
                width,
                height,
            })
        })();
        self.write_field_byte(&layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    pub(super) fn start_appearance_for_raw_object_cell(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelAppearanceStart, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let original_picture_address =
            self.read_word(object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_PICTURE_OFFSET)?;
        let null_picture_address = red_label_object_picture_address("NULOB")?;
        self.write_word(
            object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_PICTURE_OFFSET,
            null_picture_address,
        )?;
        self.write_word(
            linked_list(&lists, "active_object")?.head_address,
            object_address,
        )?;

        let relative_x = self
            .read_word(object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_X16_OFFSET)?
            .wrapping_sub(self.read_field_word(&layout, "base_page", "BGL")?);
        if relative_x > RED_LABEL_APPEARANCE_ON_SCREEN_LIMIT {
            self.write_word(
                object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_PICTURE_OFFSET,
                original_picture_address,
            )?;
            return Ok(RedLabelAppearanceStart {
                object_address,
                original_picture_address,
                final_picture_address: original_picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                sound_loaded: None,
            });
        }

        let Some((slot_address, erased_previous_slot)) = self.allocate_appearance_slot(&layout)?
        else {
            self.write_word(
                object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_PICTURE_OFFSET,
                original_picture_address,
            )?;
            return Ok(RedLabelAppearanceStart {
                object_address,
                original_picture_address,
                final_picture_address: original_picture_address,
                relative_x,
                slot_address: None,
                erased_previous_slot: false,
                sound_loaded: None,
            });
        };

        let sound_loaded = if self.read_field_byte(&layout, "base_page", "STATUS")? & 0x80 == 0 {
            self.load_sound_table_by_label("APSND")?
        } else {
            None
        };
        let object_type =
            self.read_byte(object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_TYPE_OFFSET)?;
        self.write_byte(
            object_address + RED_LABEL_ATTRACT_DEFENDER_OBJECT_TYPE_OFFSET,
            object_type | 0x02,
        )?;
        self.write_appearance_word(
            &layout,
            slot_address,
            "RSIZE",
            RED_LABEL_APPEARANCE_INITIAL_SIZE,
        )?;
        self.write_appearance_word(&layout, slot_address, "OBDESC", original_picture_address)?;
        self.write_appearance_word(
            &layout,
            slot_address,
            "ERASES",
            slot_address.wrapping_add(0x40),
        )?;
        self.write_appearance_word(&layout, slot_address, "OBJPTR", object_address)?;

        Ok(RedLabelAppearanceStart {
            object_address,
            original_picture_address,
            final_picture_address: null_picture_address,
            relative_x,
            slot_address: Some(slot_address),
            erased_previous_slot,
            sound_loaded,
        })
    }

    pub(super) fn add_left_terrain_pixel(
        &mut self,
        state: &mut TerrainTableGenerationState,
        data: &[u8],
        data_base_address: u16,
        flavor_0_start: u16,
        flavor_1_start: u16,
    ) -> Result<(), String> {
        advance_terrain_left_state(&mut state.right, data, data_base_address);
        state.right_offset = if state.right.data_byte & 0x80 == 0 {
            state.right_offset.wrapping_sub(1)
        } else {
            state.right_offset.wrapping_add(1)
        };

        let flavor_0_selected = state.background_left.to_be_bytes()[1] & 0x20 != 0;
        let record_address = if flavor_0_selected {
            state.flavor_0_pointer
        } else {
            state.flavor_1_pointer
        };

        advance_terrain_left_state(&mut state.left, data, data_base_address);
        let (offset, pattern) = if state.left.data_byte & 0x80 == 0 {
            state.left_offset = state.left_offset.wrapping_sub(1);
            (state.left_offset, 0x7007)
        } else {
            let offset = state.left_offset;
            state.left_offset = state.left_offset.wrapping_add(1);
            (offset, 0x0770)
        };
        self.write_terrain_flavor_record(record_address, offset, pattern)?;

        let table_start = if flavor_0_selected {
            flavor_0_start
        } else {
            flavor_1_start
        };
        let mut next_record = record_address.wrapping_add(3);
        if next_record == table_start.wrapping_add(RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES) {
            next_record = table_start;
        }
        if flavor_0_selected {
            state.flavor_0_pointer = next_record;
        } else {
            state.flavor_1_pointer = next_record;
        }
        Ok(())
    }

    pub(super) fn add_right_terrain_pixel(
        &mut self,
        state: &mut TerrainTableGenerationState,
        data: &[u8],
        data_base_address: u16,
        flavor_0_start: u16,
        flavor_1_start: u16,
    ) -> Result<(), String> {
        state.left_offset = if state.left.data_byte & 0x80 == 0 {
            state.left_offset.wrapping_add(1)
        } else {
            state.left_offset.wrapping_sub(1)
        };
        advance_terrain_right_state(&mut state.left, data, data_base_address);

        let flavor_0_selected = state.background_left.to_be_bytes()[1] & 0x20 == 0;
        let table_start = if flavor_0_selected {
            flavor_0_start
        } else {
            flavor_1_start
        };
        let current_pointer = if flavor_0_selected {
            state.flavor_0_pointer
        } else {
            state.flavor_1_pointer
        };
        let mut record_address = current_pointer.wrapping_sub(3);
        if record_address == table_start.wrapping_sub(3) {
            record_address = table_start.wrapping_add(RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES - 3);
        }

        let (offset, pattern) = if state.right.data_byte & 0x80 == 0 {
            let offset = state.right_offset;
            state.right_offset = state.right_offset.wrapping_add(1);
            (offset, 0x7007)
        } else {
            state.right_offset = state.right_offset.wrapping_sub(1);
            (state.right_offset, 0x0770)
        };
        self.write_terrain_flavor_record(record_address, offset, pattern)?;
        advance_terrain_right_state(&mut state.right, data, data_base_address);

        if flavor_0_selected {
            state.flavor_0_pointer = record_address;
        } else {
            state.flavor_1_pointer = record_address;
        }
        Ok(())
    }

    pub(super) fn write_terrain_flavor_record(
        &mut self,
        address: u16,
        offset: u8,
        pattern: u16,
    ) -> Result<(), String> {
        self.write_byte(address, offset)?;
        self.write_word(address.wrapping_add(1), pattern)?;
        self.write_byte(
            address.wrapping_add(RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES),
            offset,
        )?;
        self.write_word(
            address
                .wrapping_add(RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES)
                .wrapping_add(1),
            pattern,
        )
    }

    /// Source-shaped bank-7 `BGOUT`: roll `TERTF0`/`TERTF1` to the current
    /// `BGL`, then draw the selected 152 terrain words through `STBL`.
    ///
    /// The original routine saves the live 6809 stack pointer into `SSTACK`
    /// before reusing `S` as a terrain-table reader. This translation requires
    /// the caller to pass that stack pointer explicitly rather than inventing a
    /// CPU context.
    ///
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L155-L233>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L235-L370>.
    pub fn output_terrain_from_bgl(
        &mut self,
        hardware_stack_pointer: u16,
    ) -> Result<RedLabelTerrainOutput, String> {
        let layout = red_label_ram_layout()?;
        let table = red_label_terrain_data_table("TDATA")?;
        let flavor_0 = field_range(&layout, "terrain_flavor_0", "TERTF0")?;
        let flavor_1 = field_range(&layout, "terrain_flavor_1", "TERTF1")?;
        let screen_table = field_range(&layout, "terrain_screen_table", "STBL")?;
        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let masked_background_left = background_left & 0xFFE0;
        let previous_generation_left = self.read_field_word(&layout, "terrain_runtime", "BGLX")?;

        let mut state = self.read_terrain_generation_state_from_runtime(
            &layout,
            table,
            masked_background_left,
        )?;
        let shifted_delta = masked_background_left
            .wrapping_sub(previous_generation_left)
            .wrapping_shl(3);
        let mut update_counter = shifted_delta.to_be_bytes()[0];
        let update_steps = update_counter as i8;
        if update_counter & 0x80 == 0 {
            while update_counter != 0 {
                state.background_left = state.background_left.wrapping_add(0x20);
                self.add_right_terrain_pixel(
                    &mut state,
                    &table.bytes,
                    table.address,
                    flavor_0.start,
                    flavor_1.start,
                )?;
                update_counter = update_counter.wrapping_sub(1);
            }
        } else {
            while update_counter != 0 {
                state.background_left = state.background_left.wrapping_sub(0x20);
                self.add_left_terrain_pixel(
                    &mut state,
                    &table.bytes,
                    table.address,
                    flavor_0.start,
                    flavor_1.start,
                )?;
                update_counter = update_counter.wrapping_add(1);
            }
        }

        state.background_left = masked_background_left;
        self.write_terrain_generation_state_to_runtime(&layout, state, hardware_stack_pointer)?;
        self.write_field_word(&layout, "terrain_runtime", "TEMP2B", masked_background_left)?;
        self.write_field_byte(&layout, "terrain_runtime", "TEMP1A", update_steps as u8)?;

        let (selected_flavor_start, selected_flavor_pointer) =
            if masked_background_left.to_be_bytes()[1] & 0x20 == 0 {
                (flavor_1.start, state.flavor_1_pointer)
            } else {
                (flavor_0.start, state.flavor_0_pointer)
            };
        validate_terrain_flavor_pointer(
            selected_flavor_pointer,
            selected_flavor_start,
            "BGOUT selected flavor pointer",
        )?;

        let mut first_screen_address = 0;
        for entry_index in 0..0x98u16 {
            let screen_table_entry = screen_table.start.wrapping_add(entry_index * 2);
            let old_screen_address = self.read_word(screen_table_entry)?;
            if old_screen_address != 0 {
                self.write_word(old_screen_address, 0)?;
            }
            let record_address = selected_flavor_pointer.wrapping_add(entry_index * 3);
            let screen_address = u16::from_be_bytes([
                0x98u8.wrapping_sub(entry_index as u8),
                self.read_byte(record_address)?,
            ]);
            let terrain_word = self.read_word(record_address.wrapping_add(1))?;
            if entry_index == 0 {
                first_screen_address = screen_address;
            }
            self.write_word(screen_table_entry, screen_address)?;
            self.write_word(screen_address, terrain_word)?;
        }

        Ok(RedLabelTerrainOutput {
            background_left,
            previous_generation_left,
            terrain_generation_left: masked_background_left,
            update_steps,
            selected_flavor_start,
            selected_flavor_pointer,
            screen_table_start: screen_table.start,
            screen_table_end: screen_table.end,
            screen_entries: 0x98,
            first_screen_address,
            stack_pointer_saved: hardware_stack_pointer,
        })
    }

    pub(super) fn read_terrain_generation_state_from_runtime(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        table: &RedLabelTerrainDataTable,
        terrain_left: u16,
    ) -> Result<TerrainTableGenerationState, String> {
        let left_pointer = self.read_field_word(layout, "terrain_runtime", "LTPTR")?;
        let right_pointer = self.read_field_word(layout, "terrain_runtime", "RTPTR")?;
        let left = TerrainBitState {
            data_index: terrain_data_index_for_pointer(
                left_pointer,
                table.address,
                table.bytes.len(),
            )?,
            data_pointer: left_pointer,
            data_byte: self.read_field_byte(layout, "terrain_runtime", "LTBYTE")?,
            bit_counter: self.read_field_byte(layout, "terrain_runtime", "LTCNT")?,
        };
        let right = TerrainBitState {
            data_index: terrain_data_index_for_pointer(
                right_pointer,
                table.address,
                table.bytes.len(),
            )?,
            data_pointer: right_pointer,
            data_byte: self.read_field_byte(layout, "terrain_runtime", "RTBYTE")?,
            bit_counter: self.read_field_byte(layout, "terrain_runtime", "RTCNT")?,
        };
        validate_terrain_bit_counter(left.bit_counter, "LTCNT")?;
        validate_terrain_bit_counter(right.bit_counter, "RTCNT")?;

        let flavor_0 = field_range(layout, "terrain_flavor_0", "TERTF0")?;
        let flavor_1 = field_range(layout, "terrain_flavor_1", "TERTF1")?;
        let flavor_0_pointer = self.read_field_word(layout, "terrain_runtime", "TTBLP0")?;
        let flavor_1_pointer = self.read_field_word(layout, "terrain_runtime", "TTBLP1")?;
        validate_terrain_flavor_pointer(flavor_0_pointer, flavor_0.start, "TTBLP0")?;
        validate_terrain_flavor_pointer(flavor_1_pointer, flavor_1.start, "TTBLP1")?;

        Ok(TerrainTableGenerationState {
            left,
            right,
            left_offset: self.read_field_byte(layout, "terrain_runtime", "LOFF")?,
            right_offset: self.read_field_byte(layout, "terrain_runtime", "ROFF")?,
            background_left: self.read_field_word(layout, "terrain_runtime", "BGLX")?,
            terrain_left,
            flavor_0_pointer,
            flavor_1_pointer,
        })
    }

    pub(super) fn write_terrain_generation_state_to_runtime(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        state: TerrainTableGenerationState,
        hardware_stack_pointer: u16,
    ) -> Result<(), String> {
        self.write_field_word(layout, "terrain_runtime", "TTBLP0", state.flavor_0_pointer)?;
        self.write_field_word(layout, "terrain_runtime", "TTBLP1", state.flavor_1_pointer)?;
        self.write_field_word(layout, "terrain_runtime", "LTPTR", state.left.data_pointer)?;
        self.write_field_word(layout, "terrain_runtime", "RTPTR", state.right.data_pointer)?;
        self.write_field_byte(layout, "terrain_runtime", "LTBYTE", state.left.data_byte)?;
        self.write_field_byte(layout, "terrain_runtime", "RTBYTE", state.right.data_byte)?;
        self.write_field_byte(layout, "terrain_runtime", "LTCNT", state.left.bit_counter)?;
        self.write_field_byte(layout, "terrain_runtime", "RTCNT", state.right.bit_counter)?;
        self.write_field_byte(layout, "terrain_runtime", "LOFF", state.left_offset)?;
        self.write_field_byte(layout, "terrain_runtime", "ROFF", state.right_offset)?;
        self.write_field_word(layout, "terrain_runtime", "SSTACK", hardware_stack_pointer)?;
        self.write_field_word(layout, "terrain_runtime", "BGLX", state.background_left)
    }

    /// Source-shaped bank-7 `BGERAS`: walk the 0x130-byte `STBL` screen
    /// address table and store a zero word at each indirect screen address.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L405-L409>.
    pub fn erase_terrain_from_screen_table(&mut self) -> Result<RedLabelTerrainErase, String> {
        let layout = red_label_ram_layout()?;
        let table = field_range(&layout, "terrain_screen_table", "STBL")?;
        if !(table.end - table.start).is_multiple_of(2) {
            return Err(format!(
                "red-label STBL range 0x{:04X}..0x{:04X} must contain word entries",
                table.start, table.end
            ));
        }

        let mut cursor = table.start;
        while cursor != table.end {
            let screen_address = self.read_word(cursor)?;
            self.write_word(screen_address, 0)?;
            cursor = cursor.wrapping_add(2);
        }

        Ok(RedLabelTerrainErase {
            table_start: table.start,
            table_end: table.end,
            entries: (table.end - table.start) / 2,
        })
    }

    /// Source-shaped `TERBLO` scanner terrain clear: walk the 64-word
    /// `STETAB` table and store a zero word at each indirect screen address.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L447-L451>.
    pub fn erase_scanner_terrain_from_erase_table(
        &mut self,
    ) -> Result<RedLabelScannerTerrainErase, String> {
        let layout = red_label_ram_layout()?;
        let table = field_range(&layout, "scanner_terrain_erase", "STETAB")?;
        if !(table.end - table.start).is_multiple_of(2) {
            return Err(format!(
                "red-label STETAB range 0x{:04X}..0x{:04X} must contain word entries",
                table.start, table.end
            ));
        }

        let mut cursor = table.start;
        while cursor != table.end {
            let screen_address = self.read_word(cursor)?;
            self.write_word(screen_address, 0)?;
            cursor = cursor.wrapping_add(2);
        }

        Ok(RedLabelScannerTerrainErase {
            table_start: table.start,
            table_end: table.end,
            entries: (table.end - table.start) / 2,
        })
    }

    /// Source-shaped `FISS`: point `FISX` at `FISTAB`, then fill the 32-byte
    /// fizzle table from caller-supplied `RAND` results.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2889-L2902>.
    pub fn initialize_laser_fizzle_table_from_rand_values(
        &mut self,
        rand_values: &[u8],
    ) -> Result<RedLabelLaserFizzleInit, String> {
        let layout = red_label_ram_layout()?;
        let fizzle_range = field_range(&layout, "laser_fizzle", "FISTAB")?;
        let expected_len = usize::from(fizzle_range.end - fizzle_range.start);
        if rand_values.len() != expected_len {
            return Err(format!(
                "red-label FISS requires {expected_len} RAND byte(s), got {}",
                rand_values.len()
            ));
        }

        self.write_field_word(&layout, "base_page", "FISX", fizzle_range.start)?;
        for (offset, rand_value) in rand_values.iter().copied().enumerate() {
            self.write_byte(
                fizzle_range.start + offset as u16,
                laser_fizzle_byte(rand_value),
            )?;
        }

        Ok(RedLabelLaserFizzleInit {
            table_start: fizzle_range.start,
            table_end: fizzle_range.end,
            fizzle_index: fizzle_range.start,
        })
    }

    /// Source-shaped `STINIT`: initialize the 16-entry star table from
    /// caller-supplied `RAND` results, preserving the source rejection loops for
    /// X and Y coordinates.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2073-L2094>.
    pub fn initialize_star_table_from_rand_values(
        &mut self,
        rand_values: &[u8],
    ) -> Result<RedLabelStarTableInit, String> {
        let (stars, rand_values_consumed) =
            star_table_from_rand_values(rand_values.iter().copied())?;
        self.write_star_table(&stars, rand_values_consumed)
    }

    pub(super) fn initialize_star_table_from_runtime_rand(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelStarTableInit, String> {
        let mut stars = Vec::with_capacity(16);
        let mut consumed = 0usize;
        let mut color = 0u8;

        for _ in 0..16 {
            let x = loop {
                let state = self.advance_red_label_rand(layout)?;
                consumed += 1;
                if state.seed < 0x9C {
                    break state.seed;
                }
            };
            let y = loop {
                let state = self.advance_red_label_rand(layout)?;
                consumed += 1;
                if state.seed <= 0xA8 && state.seed > RED_LABEL_Y_MIN {
                    break state.seed;
                }
            };
            stars.push(RedLabelStar { x, y, color });
            color = color.wrapping_add(0x11) & 0x77;
        }

        self.write_star_table(&stars, consumed)
    }

    pub(super) fn write_star_table(
        &mut self,
        stars: &[RedLabelStar],
        rand_values_consumed: usize,
    ) -> Result<RedLabelStarTableInit, String> {
        let layout = red_label_ram_layout()?;
        let star_range = table_descriptor(&layout, "star_map")?
            .table_range()
            .ok_or_else(|| String::from("red-label star_map table range is invalid"))?;
        self.write_field_byte(&layout, "base_page", "STRCNT", stars.len() as u8)?;

        for (entry_index, star) in stars.iter().copied().enumerate() {
            let entry_index = entry_index as u16;
            let x_range = ram_field(&layout, "star_map", "SX")?
                .field_range_for_entry(entry_index)
                .ok_or_else(|| format!("red-label star SX entry {entry_index} is invalid"))?;
            let y_range = ram_field(&layout, "star_map", "SY")?
                .field_range_for_entry(entry_index)
                .ok_or_else(|| format!("red-label star SY entry {entry_index} is invalid"))?;
            let color_range = ram_field(&layout, "star_map", "SCOL")?
                .field_range_for_entry(entry_index)
                .ok_or_else(|| format!("red-label star SCOL entry {entry_index} is invalid"))?;
            self.write_byte(x_range.start, star.x)?;
            self.write_byte(y_range.start, star.y)?;
            self.write_byte(color_range.start, star.color)?;
        }

        Ok(RedLabelStarTableInit {
            table_start: star_range.start,
            table_end: star_range.end,
            star_count: stars.len() as u8,
            rand_values_consumed,
        })
    }

    /// Source-shaped `STOUT` star output, including its fall-through into
    /// `SBLNK`. This routine mutates only the RAM/video bytes named by
    /// `phr6.src`; interrupt timing and scanline ownership remain external.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2097-L2199>.
    pub fn output_stars(&mut self) -> Result<RedLabelStarOutput, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x20 != 0 {
            return Ok(RedLabelStarOutput::StatusBlocked { status });
        }

        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let previous_background_left = self.read_field_word(&layout, "base_page", "BGLX")?;
        let background_phase = background_left & 0xFF80;
        let movement = star_output_movement(background_left, previous_background_left);
        let phase_mask = if background_left.to_be_bytes()[1] & 0x40 == 0 {
            0x0F
        } else {
            0xF0
        };
        let itemp = field_range(&layout, "base_page", "ITEMP")?;
        self.write_word(itemp.start, background_phase)?;
        self.write_byte(itemp.start, movement)?;
        self.write_byte(
            field_range(&layout, "base_page", "ITEMP2")?.start,
            phase_mask,
        )?;

        let star_table = table_descriptor(&layout, "star_map")?;
        for entry_index in 0..star_table.entries {
            let screen_address = self.read_star_screen_address(&layout, entry_index)?;
            self.write_byte(screen_address, 0)?;
        }

        let star_count = self.read_field_byte(&layout, "base_page", "STRCNT")?;
        if star_count == 0 || u16::from(star_count) > star_table.entries {
            return Err(format!(
                "red-label STOUT source-valid STRCNT must be 1..={}, got {star_count}",
                star_table.entries
            ));
        }

        for entry_index in 0..u16::from(star_count) {
            let x_range = star_field_range(&layout, entry_index, "SX")?;
            let color_range = star_field_range(&layout, entry_index, "SCOL")?;
            let x = star_output_next_x(self.read_byte(x_range.start)?, movement);
            self.write_byte(x_range.start, x)?;

            let screen_address = self.read_star_screen_address(&layout, entry_index)?;
            let color = self.read_byte(color_range.start)? & phase_mask;
            self.write_byte(screen_address, color)?;
        }

        let blink = self.blink_star(&layout)?;
        Ok(RedLabelStarOutput::Updated {
            movement,
            phase_mask,
            star_count,
            blink,
        })
    }

    pub(super) fn read_star_screen_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        entry_index: u16,
    ) -> Result<u16, String> {
        let x_range = star_field_range(layout, entry_index, "SX")?;
        self.read_word(x_range.start)
    }

    pub(super) fn blink_star(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelStarBlink, String> {
        let star_table = table_descriptor(layout, "star_map")?;
        let seed = self.read_field_byte(layout, "base_page", "SEED")?;
        let selected_offset = seed & 0x3C;
        let selected_table_address = star_table
            .base
            .checked_add(u16::from(selected_offset))
            .ok_or_else(|| String::from("red-label SBLNK selected star address overflows"))?;
        let table_range = star_table
            .table_range()
            .ok_or_else(|| String::from("red-label star_map table range is invalid"))?;
        if selected_table_address + 2 >= table_range.end {
            return Err(format!(
                "red-label SBLNK selected star 0x{selected_table_address:04X} is outside SMAP"
            ));
        }

        let color_address = selected_table_address + 2;
        let color = self.read_byte(color_address)?.wrapping_add(0x11) & 0x77;
        self.write_byte(color_address, color)?;

        let hyper = if seed & 0x01 == 0 {
            Some(self.blink_hyperspace_star(layout, selected_table_address, seed)?)
        } else {
            None
        };

        Ok(RedLabelStarBlink {
            selected_table_address,
            selected_offset,
            color,
            hyper,
        })
    }

    pub(super) fn blink_hyperspace_star(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        selected_table_address: u16,
        seed: u8,
    ) -> Result<RedLabelStarHyper, String> {
        let mut target_table_address = selected_table_address;
        let mut x = seed;
        let mut ram_blast = None;

        if seed >= 0x98 {
            let wcurs = self.read_field_word(layout, "base_page", "WCURS")?;
            let status = self.read_field_byte(layout, "base_page", "STATUS")?;
            if wcurs != 0x6245 && status & 0x80 == 0 && seed == 0xA0 {
                let lseed = self.read_field_byte(layout, "base_page", "LSEED")?;
                let value = self.read_field_byte(layout, "base_page", "HSEED")?;
                target_table_address = u16::from_be_bytes([seed, lseed]);
                self.write_byte(target_table_address, value)?;
                ram_blast = Some(RedLabelStarRamBlast {
                    address: target_table_address,
                    value,
                });
            }
            x = seed.wrapping_sub(0x84);
        }

        let screen_clear_address = self.read_word(target_table_address)?;
        self.write_byte(screen_clear_address, 0)?;
        self.write_byte(target_table_address, x)?;

        let status = self.read_field_byte(layout, "base_page", "STATUS")?;
        let y = if status & 0x02 == 0 {
            None
        } else {
            let target_y_address = target_table_address.checked_add(1).ok_or_else(|| {
                format!("red-label SBLNK Y write 0x{target_table_address:04X}+1 overflows")
            })?;
            let lseed = self.read_field_byte(layout, "base_page", "LSEED")?;
            let y = star_hyperspace_y(lseed);
            self.write_byte(target_y_address, y)?;
            Some(y)
        };

        Ok(RedLabelStarHyper {
            target_table_address,
            screen_clear_address,
            x,
            y,
            ram_blast,
        })
    }

    /// Source-shaped `FBINIT`: point `FBX` at `FBTAB`, then fill the 32-byte
    /// fireball table from caller-supplied `RAND` results.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2722-L2738>.
    pub fn initialize_fireball_table_from_rand_values(
        &mut self,
        rand_values: &[u8],
    ) -> Result<RedLabelFireballTableInit, String> {
        let layout = red_label_ram_layout()?;
        let fireball_range = field_range(&layout, "fireball_table", "FBTAB")?;
        let expected_len = usize::from(fireball_range.end - fireball_range.start);
        if rand_values.len() != expected_len {
            return Err(format!(
                "red-label FBINIT requires {expected_len} RAND byte(s), got {}",
                rand_values.len()
            ));
        }

        self.write_field_word(&layout, "base_page", "FBX", fireball_range.start)?;
        for (offset, rand_value) in rand_values.iter().copied().enumerate() {
            self.write_byte(
                fireball_range.start + offset as u16,
                fireball_table_byte(rand_value),
            )?;
        }

        Ok(RedLabelFireballTableInit {
            table_start: fireball_range.start,
            table_end: fireball_range.end,
            fireball_index: fireball_range.start,
        })
    }

    /// Source-shaped `THINIT`: point `THX` at `THTAB`, then fill the primary
    /// thrust table and its source-overlapping mirror. The original loop writes
    /// 33 RAND bytes and the final mirror byte lands on the first byte of
    /// `FBTAB`; this is preserved rather than normalized.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2201-L2210>.
    pub fn initialize_thrust_table_from_rand_values(
        &mut self,
        rand_values: &[u8],
    ) -> Result<RedLabelThrustTableInit, String> {
        let layout = red_label_ram_layout()?;
        let thrust_range = field_range(&layout, "thrust_table", "THTAB")?;
        let expected_len = 33;
        if rand_values.len() != expected_len {
            return Err(format!(
                "red-label THINIT requires {expected_len} RAND byte(s), got {}",
                rand_values.len()
            ));
        }

        self.write_field_word(&layout, "base_page", "THX", thrust_range.start)?;
        for (offset, rand_value) in rand_values.iter().copied().enumerate() {
            let source_offset = offset as u16;
            self.write_byte(thrust_range.start + 32 + source_offset, rand_value)?;
            self.write_byte(thrust_range.start + source_offset, rand_value)?;
        }

        Ok(RedLabelThrustTableInit {
            table_start: thrust_range.start,
            table_end: thrust_range.end,
            mirror_end: thrust_range.start + 65,
            thrust_index: thrust_range.start,
        })
    }

    /// Source-shaped `THPROC`: advance `THX` through `THTAB..THTAB+32`, advance
    /// `FBX` through `FBTAB..FBTAB+24`, then sleep four ticks back to `THPROC`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3280-L3294>.
    pub fn step_thrust_process_current_process(
        &mut self,
    ) -> Result<RedLabelThrustProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let thrust_range = field_range(&layout, "thrust_table", "THTAB")?;
        let fireball_range = field_range(&layout, "fireball_table", "FBTAB")?;

        let previous_thrust_pointer = self.read_field_word(&layout, "base_page", "THX")?;
        let mut next_thrust_pointer = previous_thrust_pointer.wrapping_add(1);
        if next_thrust_pointer > thrust_range.start + 32 {
            next_thrust_pointer = thrust_range.start;
        }
        self.write_field_word(&layout, "base_page", "THX", next_thrust_pointer)?;

        let previous_fireball_pointer = self.read_field_word(&layout, "base_page", "FBX")?;
        let mut next_fireball_pointer = previous_fireball_pointer.wrapping_add(1);
        if next_fireball_pointer > fireball_range.start + 24 {
            next_fireball_pointer = fireball_range.start;
        }
        self.write_field_word(&layout, "base_page", "FBX", next_fireball_pointer)?;

        let wakeup_address = red_label_routine_address("THPROC")?;
        self.sleep_current_process(4, wakeup_address)?;
        Ok(RedLabelThrustProcessStep {
            process_address,
            previous_thrust_pointer,
            next_thrust_pointer,
            previous_fireball_pointer,
            next_fireball_pointer,
            wakeup_address,
        })
    }

    /// Source-shaped `LFIRE` entry: enforce the four-laser cap, load `LASSND`,
    /// seed the current process `PD` words, then report the selected
    /// `LASR0`/`LASL0` continuation for the translated laser loops.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2761-L2773>.
    pub fn start_laser_fire_current_process(&mut self) -> Result<RedLabelLaserFire, String> {
        let layout = red_label_ram_layout()?;
        let active_lasers = self.read_field_byte(&layout, "base_page", "LFLG")?;
        if active_lasers >= 4 {
            return self
                .kill_current_process(&layout)
                .map(RedLabelLaserFire::Capped);
        }

        let process_address = self.current_process_address(&layout)?;
        let laser_count = active_lasers.wrapping_add(1);
        self.write_field_byte(&layout, "base_page", "LFLG", laser_count)?;
        let sound_loaded = self.load_sound_table_by_label("LASSND")?;

        let player_upper_left = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        let player_direction = self.read_field_word(&layout, "base_page", "NPLAD")?;
        let (direction, start_address, next_routine_address) = if player_direction & 0x8000 == 0 {
            (
                RedLabelLaserDirection::Right,
                player_upper_left.wrapping_add(0x0704),
                red_label_routine_address("LASR0")?,
            )
        } else {
            (
                RedLabelLaserDirection::Left,
                player_upper_left.wrapping_add(4),
                red_label_routine_address("LASL0")?,
            )
        };

        self.write_process_data_word(&layout, process_address, "PD", start_address)?;
        self.write_process_data_word(&layout, process_address, "PD2", start_address)?;
        self.write_process_data_word(&layout, process_address, "PD4", start_address)?;

        Ok(RedLabelLaserFire::Started {
            process_address,
            direction,
            laser_count,
            start_address,
            next_routine_address,
            sound_loaded,
        })
    }

    /// Source-shaped full `LFIRE` process dispatch: execute the entry path,
    /// then fall through the `LASR` / `LASL` setup labels into the first
    /// `LASR0` / `LASL0` loop body before returning at `NAP` or `SUCIDE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2761-L2773>.
    pub fn dispatch_laser_fire_current_process(
        &mut self,
    ) -> Result<RedLabelLaserFireDispatch, String> {
        let fire = self.start_laser_fire_current_process()?;
        match fire {
            RedLabelLaserFire::Started { direction, .. } => {
                let first_step = self.step_laser_current_process(direction)?;
                Ok(RedLabelLaserFireDispatch::Started { fire, first_step })
            }
            RedLabelLaserFire::Capped(killed_process) => {
                Ok(RedLabelLaserFireDispatch::Capped(killed_process))
            }
        }
    }

    /// Source-shaped `LASR`: seed the right-moving laser process data from
    /// `NPLAXC`, then fall through the first `LASR0` loop body.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2790-L2837>.
    pub fn start_right_laser_current_process(&mut self) -> Result<RedLabelLaserStep, String> {
        self.start_laser_current_process(RedLabelLaserDirection::Right)
    }

    /// Source-shaped `LASL`: seed the left-moving laser process data from
    /// `NPLAXC`, then fall through the first `LASL0` loop body.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2839-L2884>.
    pub fn start_left_laser_current_process(&mut self) -> Result<RedLabelLaserStep, String> {
        self.start_laser_current_process(RedLabelLaserDirection::Left)
    }

    pub(super) fn start_laser_current_process(
        &mut self,
        direction: RedLabelLaserDirection,
    ) -> Result<RedLabelLaserStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player_upper_left = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        let start_address = match direction {
            RedLabelLaserDirection::Right => player_upper_left.wrapping_add(0x0704),
            RedLabelLaserDirection::Left => player_upper_left.wrapping_add(4),
        };
        self.write_process_data_word(&layout, process_address, "PD", start_address)?;
        self.write_process_data_word(&layout, process_address, "PD2", start_address)?;
        self.write_process_data_word(&layout, process_address, "PD4", start_address)?;
        self.step_laser_current_process(direction)
    }

    /// Source-shaped visible `LASD` tail: decrement `LFLG`, then `SUCIDE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2885-L2886>.
    pub fn finish_laser_fire_current_process(&mut self) -> Result<RedLabelKilledProcess, String> {
        let layout = red_label_ram_layout()?;
        let active_lasers = self.read_field_byte(&layout, "base_page", "LFLG")?;
        self.write_field_byte(&layout, "base_page", "LFLG", active_lasers.wrapping_sub(1))?;
        self.kill_current_process(&layout)
    }

    /// Source-shaped visible `LASR0` loop body: draw one right-moving laser
    /// step, advance fizzle bytes, clear the tail byte, run `LCOL`, then either
    /// sleep for one tick or erase through `LASD`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2796-L2837>.
    pub fn step_right_laser_current_process(&mut self) -> Result<RedLabelLaserStep, String> {
        self.step_laser_current_process(RedLabelLaserDirection::Right)
    }

    /// Source-shaped visible `LASL0` loop body. See `step_right_laser_current_process`
    /// for the shared control flow.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2845-L2884>.
    pub fn step_left_laser_current_process(&mut self) -> Result<RedLabelLaserStep, String> {
        self.step_laser_current_process(RedLabelLaserDirection::Left)
    }

    /// Source-shaped `COLIDE`: scan active objects for an exact non-zero
    /// picture-byte intersection, store `CENTMP`, and dispatch the hit
    /// object's `OCVECT`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2904-L3020>.
    pub fn collide_picture_with_active_objects(
        &mut self,
        picture_address: u16,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        self.collide_picture_with_list("active_object", picture_address, upper_left)
    }

    /// Source-shaped `COL0` variant used by `COLCHK` for the `SPTR` shell list.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3144-L3145>.
    pub fn collide_picture_with_shells(
        &mut self,
        picture_address: u16,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        self.collide_picture_with_list("shell_object", picture_address, upper_left)
    }

    /// Source-shaped visible side of `COLCHK`: skip when player collision is
    /// disabled, test active objects and shells against the current player
    /// picture, create the `PLEND` process on hit, set the status death bit,
    /// and clear `PCFLG` before return.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3130-L3153>.
    pub fn check_player_collision(&mut self) -> Result<Option<RedLabelPlayerCollision>, String> {
        let layout = red_label_ram_layout()?;
        if self.read_field_byte(&layout, "base_page", "STATUS")? & 0x10 != 0 {
            self.write_field_byte(&layout, "base_page", "PCFLG", 0)?;
            return Ok(None);
        }

        let player_upper_left = self.read_field_word(&layout, "base_page", "PLAXC")?;
        let player_direction =
            self.read_byte(field_range(&layout, "base_page", "PLADIR")?.start)?;
        let player_picture = if player_direction & 0x80 == 0 {
            red_label_object_picture_address("PLAPIC")?
        } else {
            red_label_object_picture_address("PLBPIC")?
        };

        let pcflg = self.read_field_byte(&layout, "base_page", "PCFLG")?;
        self.write_field_byte(&layout, "base_page", "PCFLG", pcflg.wrapping_add(1))?;
        let collision: Result<Option<RedLabelObjectCollision>, String> = (|| {
            let mut collision =
                self.collide_picture_with_active_objects(player_picture, player_upper_left)?;
            if collision.is_none() {
                collision = self.collide_picture_with_shells(player_picture, player_upper_left)?;
            }
            Ok(collision)
        })();
        self.write_field_byte(&layout, "base_page", "PCFLG", 0)?;

        let Some(collision) = collision? else {
            return Ok(None);
        };

        let death_process = self.make_process(
            red_label_routine_address("PLEND")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        self.write_field_byte(&layout, "base_page", "STATUS", status | 0x08)?;
        Ok(Some(RedLabelPlayerCollision {
            collision,
            death_process,
        }))
    }

    /// Source-shaped entry side of `SBOMB`: guard on `SBFLG`/`PSBC`, decrement
    /// the current player's smart-bomb count, load `SBSND`, walk active
    /// objects through `OCVECT`, seed the four screen-flash half-cycles in the
    /// current process `PD` byte, then enter the first `SBMBX1` sleep.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3173-L3209>.
    pub fn start_smart_bomb_current_player(&mut self) -> Result<Option<RedLabelSmartBomb>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        if self.read_field_byte(&layout, "base_page", "SBFLG")? != 0 {
            return Ok(None);
        }

        let (player_number, smart_bomb_address) =
            self.current_player_pointer_and_smart_bomb_address(&layout)?;
        let smart_bombs = self.read_byte(smart_bomb_address)?;
        if smart_bombs == 0 {
            return Ok(None);
        }

        self.write_field_byte(&layout, "base_page", "SBFLG", 1)?;
        let remaining_smart_bombs = smart_bombs.wrapping_sub(1);
        self.write_byte(smart_bomb_address, remaining_smart_bombs)?;
        let sound_loaded = self.load_sound_table_by_label("SBSND")?;
        let collisions = self.dispatch_smart_bomb_active_collisions(&layout, &lists)?;
        let flash_count = 4;
        self.write_current_process_data_byte(&layout, "PD", flash_count)?;
        self.start_smart_bomb_flash_sleep(&layout)?;

        Ok(Some(RedLabelSmartBomb {
            player_number,
            remaining_smart_bombs,
            sound_loaded,
            collisions,
            flash_count,
        }))
    }

    /// Resume at `SBMBX1`: decrement the flash counter and either toggle
    /// `PCRAM` for another two-tick sleep or enter the ten-tick debounce wait.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3199-L3204>.
    pub fn continue_smart_bomb_flash_tail(&mut self) -> Result<RedLabelSmartBombTail, String> {
        let layout = red_label_ram_layout()?;
        let current_process = self.current_process_address(&layout)?;
        let remaining_flash_count = self
            .read_process_byte(&layout, current_process, "PD")?
            .wrapping_sub(1);
        self.write_process_byte(&layout, current_process, "PD", remaining_flash_count)?;

        if remaining_flash_count != 0 {
            let wakeup_address = self.start_smart_bomb_flash_sleep(&layout)?;
            return Ok(RedLabelSmartBombTail::FlashSleeping {
                remaining_flash_count,
                wakeup_address,
            });
        }

        let wakeup_address = red_label_routine_address("SBX1A")?;
        self.sleep_current_process(10, wakeup_address)?;
        Ok(RedLabelSmartBombTail::DebounceSleeping { wakeup_address })
    }

    /// Resume at `SBX1A`: poll the smart-bomb switch and either repeat the
    /// debounce sleep while it is still active or arm the release sleep.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3203-L3208>.
    pub fn continue_smart_bomb_debounce_tail(&mut self) -> Result<RedLabelSmartBombTail, String> {
        let layout = red_label_ram_layout()?;
        if self.read_field_byte(&layout, "base_page", "PIA21")? & RED_LABEL_SMART_BOMB_SWITCH_BIT
            != 0
        {
            let wakeup_address = red_label_routine_address("SBX1A")?;
            self.sleep_current_process(10, wakeup_address)?;
            return Ok(RedLabelSmartBombTail::DebounceSleeping { wakeup_address });
        }

        let wakeup_address = red_label_routine_address("SBX2A")?;
        self.sleep_current_process(10, wakeup_address)?;
        Ok(RedLabelSmartBombTail::ReleaseSleeping { wakeup_address })
    }

    /// Resume at `SBX2A`: clear `SBFLG` and perform the visible `SUCIDE`
    /// process-list mutation.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3207-L3209>.
    pub fn finish_smart_bomb_tail(&mut self) -> Result<RedLabelSmartBombTail, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "SBFLG", 0)?;
        self.suicide_current_process(&layout)
    }

    /// Source-shaped `HYPER` entry: allow hyperspace only when every status bit
    /// masked by `$FD` is clear, otherwise jump to `HYPX`/`SUCIDE`. The
    /// translated body covers the visible `STATUS` write, `SCLR1`, and
    /// `NAP 15,HYP02`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3213-L3219>.
    pub fn start_hyperspace_current_process(&mut self) -> Result<RedLabelHyperspace, String> {
        let layout = red_label_ram_layout()?;
        let previous_status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if previous_status & 0xFD != 0 {
            return self
                .kill_current_process(&layout)
                .map(RedLabelHyperspace::Suppressed);
        }

        let process_address = self.current_process_address(&layout)?;
        self.write_field_byte(&layout, "base_page", "STATUS", 0x77)?;
        let screen_clear = self.clear_active_screen_ram()?;
        let wakeup_address = red_label_routine_address("HYP02")?;
        self.sleep_current_process(15, wakeup_address)?;

        Ok(RedLabelHyperspace::StartedSleeping {
            process_address,
            previous_status,
            screen_clear,
            wakeup_address,
        })
    }

    /// Source-shaped visible `HYP02` rematerialization slice: clear shell
    /// objects through `KILSHL`, seed background/player RAM from `SEED` and
    /// `HSEED`, create the phony player object through `OBINIT`, run the
    /// source `APVCT` appearance start, and sleep until `HYP2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3220-L3271>.
    pub fn continue_hyperspace_current_process(&mut self) -> Result<RedLabelHyperspace, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_address = self.current_process_address(&layout)?;
        let shell_head = linked_list(&lists, "shell_object")?.head_address;
        let mut killed_shells = 0u8;
        loop {
            let shell_address = self.read_word(shell_head)?;
            if shell_address == 0 {
                break;
            }
            self.kill_shell_cell(shell_address)?;
            killed_shells = killed_shells.wrapping_add(1);
        }

        self.write_field_byte(&layout, "base_page", "BMBCNT", 0)?;
        let seed = self.read_field_byte(&layout, "base_page", "SEED")?;
        let hseed = self.read_field_byte(&layout, "base_page", "HSEED")?;
        let seed_word = u16::from_be_bytes([seed, hseed]);
        self.write_field_word(&layout, "base_page", "BGL", seed_word)?;
        self.write_field_word(&layout, "base_page", "BGLX", seed_word)?;

        let (player_x16, player_direction) = if hseed & 1 != 0 {
            (0x2000, 0x0300)
        } else {
            (0x7000, 0xFD00)
        };
        self.write_field_word(&layout, "base_page", "PLAX16", player_x16)?;
        self.write_field_word(&layout, "base_page", "NPLAD", player_direction)?;

        let player_y = (hseed >> 1).wrapping_add(RED_LABEL_Y_MIN);
        let play16 = field_range(&layout, "base_page", "PLAY16")?;
        self.write_byte(play16.start, player_y)?;
        let player_y16 = self.read_field_word(&layout, "base_page", "PLAY16")?;
        let player_upper_left = u16::from_be_bytes([player_x16.to_be_bytes()[0], player_y]);
        self.write_field_word(&layout, "base_page", "NPLAXC", player_upper_left)?;
        self.write_field(&layout, "base_page", "PLAXV", 0, &[0, 0, 0])?;
        self.write_field_word(&layout, "base_page", "PLAYV", 0)?;

        self.write_status_from_astcnt(&layout, 0x50)?;

        let phony_picture_address = if player_direction & 0x8000 == 0 {
            red_label_object_picture_address("PLAPIC")?
        } else {
            red_label_object_picture_address("PLBPIC")?
        };
        let created_object = self.init_object_cell(
            process_address,
            RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("PLAPIC")?,
                collision_vector_address: red_label_routine_address("NOKILL")?,
                scanner_color: 0,
            },
        )?;
        let phony_object_address = created_object.object_address;
        self.write_object_word(&layout, phony_object_address, "OXV", 0)?;
        self.write_object_word(&layout, phony_object_address, "OYV", 0)?;
        self.write_object_word(&layout, phony_object_address, "OY16", player_y16)?;
        let object_x16 = (player_x16 >> 2).wrapping_add(seed_word);
        self.write_object_word(&layout, phony_object_address, "OX16", object_x16)?;
        if player_direction & 0x8000 != 0 {
            self.write_object_word(
                &layout,
                phony_object_address,
                "OPICT",
                phony_picture_address,
            )?;
        }
        self.write_process_data_word(&layout, process_address, "PD", phony_object_address)?;
        let appearance = self.start_appearance_for_object(phony_object_address)?;
        let wakeup_address = red_label_routine_address("HYP2")?;
        self.sleep_current_process(0x28, wakeup_address)?;

        Ok(RedLabelHyperspace::RematerializingSleeping {
            process_address,
            killed_shells,
            seed_word,
            player_x16,
            player_y16,
            player_upper_left,
            player_direction,
            phony_object_address,
            phony_picture_address,
            appearance,
            wakeup_address,
        })
    }

    /// Source-shaped visible `HYP2` tail: kill the phony player object through
    /// `KILOFF`, reset `STATUS` through `STCHK`, then either branch into the
    /// untranslated `PLEND` death path when `LSEED > 192` or run
    /// `HYPX`/`SUCIDE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3272-L3278>.
    pub fn finish_hyperspace_current_process(&mut self) -> Result<RedLabelHyperspace, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let phony_object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        let previous_object_link_address = self.kill_object_cell_offscreen(phony_object_address)?;
        let status = self.write_status_from_astcnt(&layout, 0)?;
        let lseed = self.read_field_byte(&layout, "base_page", "LSEED")?;

        if lseed > 192 {
            return Ok(RedLabelHyperspace::DeathRisk {
                process_address,
                phony_object_address,
                previous_object_link_address,
                status,
                lseed,
                plend_address: red_label_routine_address("PLEND")?,
            });
        }

        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelHyperspace::Completed {
            process_address,
            phony_object_address,
            previous_object_link_address,
            status,
            lseed,
            killed_process,
        })
    }

    /// Source-shaped visible `PLEND` / `PDTHL` entry: update `STATUS` through
    /// `STCHK0`, freeze scroll, clear the current player block, save the
    /// current player target/enemy counters through `PLSAV`, load `PDSND`,
    /// build the monochrome player picture in `MONOTB`, blank it once through
    /// `COFF`, then sleep to `PDTH2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1328-L1358>.
    pub fn start_player_death_current_process(&mut self) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let status = self.write_status_from_astcnt(&layout, 0x58)?;
        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        self.write_field_word(&layout, "base_page", "BGLX", background_left)?;

        let player_screen_address = self.read_field_word(&layout, "base_page", "PLAXC")?;
        self.clear_screen_block(player_screen_address, 8, 6)?;
        self.save_current_player_state_for_death(&layout)?;
        let sound_loaded = self.load_sound_table_by_label("PDSND")?;

        let next_player_direction = self.read_field_word(&layout, "base_page", "NPLAD")?;
        let player_picture_address = if next_player_direction & 0x8000 == 0 {
            red_label_object_picture_address("PLAPIC")?
        } else {
            red_label_object_picture_address("PLBPIC")?
        };
        let mono_picture_address =
            self.write_monochrome_player_picture(&layout, player_picture_address)?;
        let glow_table = red_label_player_death_table("PXCTB")?;
        self.write_process_data_word(&layout, process_address, "PD", glow_table.address)?;
        self.write_process_data_word(&layout, process_address, "PD4", mono_picture_address)?;

        let next_player_screen_address = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        self.clear_screen_block(next_player_screen_address, 8, 6)?;
        let wakeup_address = red_label_routine_address("PDTH2")?;
        self.sleep_current_process(2, wakeup_address)?;

        Ok(RedLabelPlayerDeath::StartedSleeping {
            process_address,
            status,
            background_left,
            player_screen_address,
            next_player_screen_address,
            player_picture_address,
            glow_table_address: glow_table.address,
            mono_picture_address,
            sound_loaded,
            wakeup_address,
        })
    }

    /// Resume at `PDTHL`: blank the current monochrome player frame and sleep
    /// back to `PDTH2` for the next glow write.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1352-L1357>.
    pub fn blank_player_death_current_process(&mut self) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player_screen_address = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        let mono_picture_address = self.read_process_data_word(&layout, process_address, "PD4")?;
        self.clear_ram_picture_descriptor_footprint(player_screen_address, mono_picture_address)?;
        let wakeup_address = red_label_routine_address("PDTH2")?;
        self.sleep_current_process(2, wakeup_address)?;

        Ok(RedLabelPlayerDeath::GlowBlankedSleeping {
            process_address,
            player_screen_address,
            mono_picture_address,
            wakeup_address,
        })
    }

    /// Resume at `PDTH2`: draw the monochrome player frame, consume one glow
    /// color from `PXCTB`, and either sleep back to `PDTHL` or enter the final
    /// `PDTH4` flash.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1358-L1369>.
    pub fn continue_player_death_glow_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player_screen_address = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        let mono_picture_address = self.read_process_data_word(&layout, process_address, "PD4")?;
        self.write_ram_picture_descriptor(player_screen_address, mono_picture_address)?;

        let glow_table_address = self.read_process_data_word(&layout, process_address, "PD")?;
        let glow_value = red_label_player_death_byte(glow_table_address)?.ok_or_else(|| {
            format!("red-label player-death asset has no byte at 0x{glow_table_address:04X}")
        })?;
        if glow_value == 0 {
            return self.finish_player_death_glow_current_process();
        }

        let next_glow_table_address = glow_table_address.wrapping_add(1);
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram + 0x0B, glow_value)?;
        self.write_byte(pcram, 0)?;
        self.write_process_data_word(&layout, process_address, "PD", next_glow_table_address)?;
        let wakeup_address = red_label_routine_address("PDTHL")?;
        self.sleep_current_process(2, wakeup_address)?;

        Ok(RedLabelPlayerDeath::GlowWrittenSleeping {
            process_address,
            player_screen_address,
            mono_picture_address,
            glow_value,
            next_glow_table_address,
            wakeup_address,
        })
    }

    /// Source-shaped `PDTH4`: mark the player inactive/game-over-style,
    /// whiten pseudo-color RAM, and sleep into the larger `PDTH5` tail.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1370-L1376>.
    pub fn finish_player_death_glow_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let player_screen_address = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        let mono_picture_address = self.read_process_data_word(&layout, process_address, "PD4")?;
        let status = 0x7F;
        let pseudo_color = 0xFF;
        self.write_field_byte(&layout, "base_page", "STATUS", status)?;
        self.write_byte(
            field_range(&layout, "base_page", "PCRAM")?.start,
            pseudo_color,
        )?;
        let wakeup_address = red_label_routine_address("PDTH5")?;
        self.sleep_current_process(2, wakeup_address)?;

        Ok(RedLabelPlayerDeath::FinalFlashSleeping {
            process_address,
            player_screen_address,
            mono_picture_address,
            status,
            pseudo_color,
            wakeup_address,
        })
    }

    /// Source-shaped `PDTH5` entry through the long-sleeping bank-7 `PXVCT`
    /// player explosion vector. The continuation after `PXVCT` returns is
    /// modeled as `PDTH5R`, the exact return address after the source `JSR`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1377-L1383>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L564-L670>.
    pub fn start_player_death_tail_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_byte(field_range(&layout, "base_page", "PCRAM")?.start, 0)?;
        let genocide = self.genocide_other_processes()?;
        let player_center = self
            .read_field_word(&layout, "base_page", "NPLAXC")?
            .wrapping_add(0x0403);
        self.write_field_byte(&layout, "base_page", "MAPCR", 7)?;
        let return_address = red_label_routine_address("PDTH5R")?;
        let wakeup_address =
            self.start_player_explosion_current_process(&layout, player_center, return_address)?;

        Ok(RedLabelPlayerDeath::ExplosionStartedSleeping {
            process_address,
            player_center,
            return_address,
            wakeup_address,
            killed_processes: genocide.killed_processes.len(),
        })
    }

    /// Source-shaped `PX1A`: advance one long-sleep player-explosion frame.
    /// When the zero terminator in `PXCOL` is reached, the source jumps
    /// through `PD2`; this dispatcher immediately runs the translated `PDTH5R`
    /// continuation instead of inventing a synthetic frame delay.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/blk71.src#L626-L670>.
    pub fn continue_player_explosion_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let color_pointer = self.read_field_word(&layout, "player_explosion_state", "PCOLP")?;
        let color = red_label_player_death_byte(color_pointer)?.ok_or_else(|| {
            format!("red-label player-explosion asset has no color at 0x{color_pointer:04X}")
        })?;
        self.write_byte(
            field_range(&layout, "base_page", "PCRAM")?.start + 0x0B,
            color,
        )?;
        if color == 0 {
            return self.continue_player_death_after_explosion_current_process();
        }

        let table = table_descriptor(&layout, "player_explosion_table")?;
        let mut live_pieces = 0u8;
        for entry_index in 0..table.entries {
            let piece_address = table.base + entry_index * table.entry_size;
            self.erase_player_explosion_piece(&layout, piece_address)?;

            let next_y = self
                .read_player_explosion_word(&layout, piece_address, "PYVELT")?
                .wrapping_add(self.read_player_explosion_word(&layout, piece_address, "PYPOST")?);
            if next_y.to_be_bytes()[0] < RED_LABEL_Y_MIN {
                continue;
            }
            self.write_player_explosion_word(&layout, piece_address, "PYPOST", next_y)?;

            let next_x = self
                .read_player_explosion_word(&layout, piece_address, "PXVELT")?
                .wrapping_add(self.read_player_explosion_word(&layout, piece_address, "PXPOST")?);
            if next_x.to_be_bytes()[0] > 0x98 {
                continue;
            }
            self.write_player_explosion_word(&layout, piece_address, "PXPOST", next_x)?;

            let screen_address =
                u16::from_be_bytes([next_x.to_be_bytes()[0], next_y.to_be_bytes()[0]]);
            self.write_player_explosion_word(&layout, piece_address, "PSCR", screen_address)?;
            self.write_player_explosion_piece(screen_address, next_x.to_be_bytes()[1])?;
            live_pieces = live_pieces.wrapping_add(1);
        }

        let color_count = self
            .read_field_byte(&layout, "player_explosion_state", "PCOLC")?
            .wrapping_sub(1);
        if color_count == 0 {
            self.write_field_word(
                &layout,
                "player_explosion_state",
                "PCOLP",
                color_pointer.wrapping_add(1),
            )?;
            self.write_field_byte(&layout, "player_explosion_state", "PCOLC", 4)?;
        } else {
            self.write_field_byte(&layout, "player_explosion_state", "PCOLC", color_count)?;
        }

        let wakeup_address = red_label_routine_address("PX1A")?;
        self.sleep_current_process(1, wakeup_address)?;

        Ok(RedLabelPlayerDeath::ExplosionFrameSleeping {
            process_address,
            color,
            color_counter: self.read_field_byte(&layout, "player_explosion_state", "PCOLC")?,
            color_pointer: self.read_field_word(&layout, "player_explosion_state", "PCOLP")?,
            live_pieces,
            wakeup_address,
        })
    }

    /// Source-shaped `PDTH5` continuation after `PXVCT`: turn off the current
    /// sound sequence, output the source sound-board stop command, check
    /// wave completion, then branch to either respawn/player-switch/game-over.
    /// The zero-enemy branch now enters the translated scheduler-visible
    /// `BONUS` body, including the bank-2 `MESS`/`WNBV` text and number
    /// writes used by the survivor bonus screen.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1383-L1432>.
    pub fn continue_player_death_after_explosion_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.write_field_byte(&layout, "base_page", "SNDTMR", 0)?;
        let sound_command = red_label_sound_output_command(RED_LABEL_PLAYER_END_SOUND_STOP_NUMBER);

        if self.wave_enemy_total()? == 0 {
            let _ = sound_command;
            return self.start_player_death_bonus_current_process();
        }

        self.finish_player_death_ship_branch(&layout, process_address, sound_command)
    }

    /// Source-shaped `BONUS`: clear pseudo-color RAM, save the subroutine
    /// return, run the active-screen clear, write the bank-2 text/number
    /// blocks, then either draw/score the first survivor icon and sleep to
    /// `BC1` or go straight to the wave-advance sleep.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1788-L1838>.
    pub fn start_player_death_bonus_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let return_address = red_label_routine_address("PDTH5SCLR")?;
        self.start_bonus_current_process_with_return(&layout, process_address, return_address)
    }

    pub(super) fn start_bonus_current_process_with_return(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        return_address: u16,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.write_byte(field_range(layout, "base_page", "PCRAM")?.start, 0)?;
        self.write_process_data_word(layout, process_address, "PD6", return_address)?;
        let screen_clear = self.clear_active_screen_ram()?;
        let text = self.player_death_bonus_text_plan(layout)?;
        let astronaut_counter = self.read_field_byte(layout, "base_page", "ASTCNT")?;
        self.write_process_byte(layout, process_address, "PD2", astronaut_counter)?;

        if astronaut_counter == 0 {
            return self.sleep_player_death_bonus_wave_advance(
                layout,
                process_address,
                return_address,
                astronaut_counter,
            );
        }

        self.draw_bonus_astronaut_and_sleep(
            layout,
            process_address,
            return_address,
            RedLabelBonusIntro {
                screen_clear: Some(screen_clear),
                text: Some(text),
            },
            RED_LABEL_BONUS_FIRST_ASTRO_SCREEN,
            astronaut_counter,
        )
    }

    /// Source-shaped `BC1`: reload the next survivor-icon screen address,
    /// decrement the source `PD2` counter, and either draw/score the next
    /// survivor or fall into `BC2`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1838-L1842>.
    pub fn continue_player_death_bonus_astronaut_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let return_address = self.read_process_data_word(&layout, process_address, "PD6")?;
        let astronaut_screen_address =
            self.read_process_data_word(&layout, process_address, "PD")?;
        let astronaut_counter = self
            .read_process_byte(&layout, process_address, "PD2")?
            .wrapping_sub(1);
        self.write_process_byte(&layout, process_address, "PD2", astronaut_counter)?;

        if astronaut_counter == 0 {
            return self.sleep_player_death_bonus_wave_advance(
                &layout,
                process_address,
                return_address,
                astronaut_counter,
            );
        }

        self.draw_bonus_astronaut_and_sleep(
            &layout,
            process_address,
            return_address,
            RedLabelBonusIntro {
                screen_clear: None,
                text: None,
            },
            astronaut_screen_address,
            astronaut_counter,
        )
    }

    /// Source-shaped `BC2`: refresh wave parameters through `GETWV` and sleep
    /// to the `BC3` return trampoline.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1842-L1845>.
    pub fn advance_player_death_bonus_wave_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let return_address = self.read_process_data_word(&layout, process_address, "PD6")?;
        let astronaut_counter = self.read_process_byte(&layout, process_address, "PD2")?;
        self.sleep_player_death_bonus_wave_advance(
            &layout,
            process_address,
            return_address,
            astronaut_counter,
        )
    }

    /// Source-shaped `BC3` plus the `PDTH5` return site: jump through saved
    /// `PD+6`, run the post-bonus `SCLR1`, then continue into the normal
    /// player-switch/respawn/game-over branch.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1844-L1845>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1394-L1432>.
    pub fn finish_player_death_bonus_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let return_address = self.read_process_data_word(&layout, process_address, "PD6")?;
        let expected_return = red_label_routine_address("PDTH5SCLR")?;
        if return_address != expected_return {
            return Err(format!(
                "red-label BC3 return 0x{return_address:04X} is not translated"
            ));
        }
        self.clear_active_screen_ram()?;
        self.finish_player_death_ship_branch(
            &layout,
            process_address,
            red_label_sound_output_command(RED_LABEL_PLAYER_END_SOUND_STOP_NUMBER),
        )
    }

    /// Source-shaped `PLE02`: choose the next player with remaining ships,
    /// set `PDFLG`, and jump to the untranslated `PLSTRT` respawn path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1413-L1422>.
    pub fn continue_player_death_player_switch_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.jump_to_next_player_with_ships(&layout, process_address)
    }

    /// Source-shaped `PLE3`: the game-over sleep resumes by jumping into
    /// `ATTR`, which selects map 1 and jumps to the bank-1 attract vector.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1429-L1432>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1083-L1086>.
    pub fn jump_player_death_game_over_to_attract_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let attract_address = red_label_routine_address("ATTR")?;
        self.write_process_word(&layout, process_address, "PADDR", attract_address)?;
        let selected_map = 1;
        self.write_field_byte(&layout, "base_page", "MAPCR", selected_map)?;
        Ok(RedLabelPlayerDeath::AttractJump {
            process_address,
            attract_address,
            selected_map,
            attract_vector_address: RED_LABEL_ATTRACT_VECTOR_ADDRESS,
        })
    }

    /// Source-shaped `HALL13`: after `HALLOF` finds no qualifying initials
    /// entry, the source sleeps and then jumps into `HALDIS`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L229-L232>.
    pub fn display_hall_of_fame_from_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        self.genocide_other_processes()?;
        self.make_process(
            red_label_routine_address("CREDS")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let display = self.write_hall_of_fame_display()?;
        self.make_process(
            red_label_routine_address("COLR")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        self.write_process_word(
            &layout,
            process_address,
            "PADDR",
            red_label_routine_address("HALD3")?,
        )?;
        Ok(RedLabelPlayerDeath::HallOfFameDisplayed {
            process_address,
            stall_ticks: display.stall_ticks,
        })
    }

    /// Source-shaped `HALD3`: wait on the hall-of-fame table until credits,
    /// reset, or the display timer hands off to the instruction page.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L428-L435>.
    pub fn continue_hall_of_fame_display_wait_current_process(
        &mut self,
    ) -> Result<RedLabelHallOfFameDisplayWait, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let credit_increase_flag = self.read_byte(RED_LABEL_ATTRACT_CREDIT_INCREASE_FLAG_RAM)?;
        if credit_increase_flag != 0 {
            let target_address = red_label_routine_address("LEDRET")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(
                RedLabelHallOfFameDisplayWait::CreditIncreaseInstructionJump {
                    process_address,
                    credit_increase_flag,
                    target_address,
                },
            );
        }

        let high_score_reset_flag = self.read_byte(RED_LABEL_HOF_RESET_FLAG_RAM)?;
        if high_score_reset_flag != 0 {
            let target_address = red_label_routine_address("HALDIS")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHallOfFameDisplayWait::HighScoreResetRedisplayJump {
                process_address,
                high_score_reset_flag,
                target_address,
            });
        }

        let stall_before = self.read_byte(RED_LABEL_HOF_STALL_TIMER_RAM)?;
        let stall_after = stall_before.wrapping_sub(1);
        self.write_byte(RED_LABEL_HOF_STALL_TIMER_RAM, stall_after)?;
        if stall_after == 0 {
            let target_address = red_label_routine_address("LEDRET")?;
            self.write_process_word(&layout, process_address, "PADDR", target_address)?;
            return Ok(RedLabelHallOfFameDisplayWait::TimeoutInstructionJump {
                process_address,
                credit_increase_flag,
                high_score_reset_flag,
                stall_before,
                stall_after,
                target_address,
            });
        }

        let wakeup_address = red_label_routine_address("HALD3")?;
        self.sleep_current_process(RED_LABEL_HALL_OF_FAME_WAIT_SLEEP_TICKS, wakeup_address)?;
        Ok(RedLabelHallOfFameDisplayWait::Sleeping {
            process_address,
            credit_increase_flag,
            high_score_reset_flag,
            stall_before,
            stall_after,
            sleep_ticks: RED_LABEL_HALL_OF_FAME_WAIT_SLEEP_TICKS,
            wakeup_address,
        })
    }

    /// Source-shaped `SHELL` front half: apply scroll and velocity to every
    /// `SPTR` shell, mark out-of-bounds shells dead, clear their old video
    /// footprint, and return live output callbacks that still need rendering.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2609-L2669>.
    pub fn step_shell_output(&mut self) -> Result<Vec<RedLabelShellStep>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x20 != 0 {
            return Ok(Vec::new());
        }

        let background_left = self.read_field_word(&layout, "base_page", "BGL")? & 0xFFE0;
        let previous_background_left = self.read_field_word(&layout, "base_page", "BGLX")? & 0xFFE0;
        let scroll_delta = previous_background_left
            .wrapping_sub(background_left)
            .wrapping_shl(2);
        self.write_field_word(&layout, "base_page", "SHTEMP", scroll_delta)?;

        let object = table_descriptor(&layout, "object")?;
        let mut link_address = linked_list(&lists, "shell_object")?.head_address;
        let mut steps = Vec::new();
        let mut scanned = 0;
        loop {
            let shell_address = self.read_word(link_address)?;
            if shell_address == 0 {
                return Ok(steps);
            }
            if scanned >= object.entries {
                return Err(String::from(
                    "red-label shell list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, shell_address)?;

            let old_screen_address = self.read_object_screen_address(&layout, shell_address)?;
            let new_y = self
                .read_object_word(&layout, shell_address, "OYV")?
                .wrapping_add(self.read_object_word(&layout, shell_address, "OY16")?);
            if new_y.to_be_bytes()[0] <= RED_LABEL_Y_MIN {
                self.mark_shell_dead(&layout, shell_address)?;
                self.clear_shell_video_footprint(old_screen_address)?;
                steps.push(RedLabelShellStep::MarkedDead {
                    shell_address,
                    old_screen_address,
                });
                link_address =
                    object_field_range_for_address(&layout, shell_address, "OLINK")?.start;
                continue;
            }
            self.write_object_word(&layout, shell_address, "OY16", new_y)?;

            let new_x = self
                .read_object_word(&layout, shell_address, "OXV")?
                .wrapping_add(scroll_delta)
                .wrapping_add(self.read_object_word(&layout, shell_address, "OX16")?);
            if new_x.to_be_bytes()[0] >= 0x98 {
                self.mark_shell_dead(&layout, shell_address)?;
                self.clear_shell_video_footprint(old_screen_address)?;
                steps.push(RedLabelShellStep::MarkedDead {
                    shell_address,
                    old_screen_address,
                });
                link_address =
                    object_field_range_for_address(&layout, shell_address, "OLINK")?.start;
                continue;
            }
            self.write_object_word(&layout, shell_address, "OX16", new_x)?;

            let new_screen_address =
                u16::from_be_bytes([new_x.to_be_bytes()[0], new_y.to_be_bytes()[0]]);
            self.write_object_screen_address(&layout, shell_address, new_screen_address)?;
            steps.push(RedLabelShellStep::Output {
                shell_address,
                old_screen_address,
                new_screen_address,
                output_routine_address: self.read_object_word(&layout, shell_address, "OBJCOL")?,
            });

            link_address = object_field_range_for_address(&layout, shell_address, "OLINK")?.start;
        }
    }

    /// Source-shaped `MKPROC`: take a regular cell from `FREE`, initialize
    /// `PADDR`/`PTYPE`/`PTIME`/`PCOD`, and splice it after `[CRPROC]`.
    pub fn make_process(
        &mut self,
        routine_address: u16,
        process_type: u8,
    ) -> Result<RedLabelCreatedProcess, String> {
        self.make_process_from_free_list(
            "process",
            "free_process",
            RedLabelProcessClass::Regular,
            routine_address,
            process_type,
            0,
        )
    }

    /// Source-shaped `MSPROC`: same insertion rule as `MKPROC`, but consumes
    /// `SPFREE` and marks `PCOD` as a super-process cell.
    pub fn make_super_process(
        &mut self,
        routine_address: u16,
        process_type: u8,
    ) -> Result<RedLabelCreatedProcess, String> {
        self.make_process_from_free_list(
            "super_process",
            "free_super_process",
            RedLabelProcessClass::Super,
            routine_address,
            process_type,
            1,
        )
    }

    /// Source-shaped translated-control slice of red-label `SSCAN`: keep the
    /// two-frame `PIA21`/`PIA22` history, store current IN0/IN1 bytes in
    /// `PIA21`/`PIA31`, read the complete asset-backed `SWTAB`, and queue
    /// translated routine bodies when a supported bit rises after two clear
    /// samples.
    pub fn scan_translated_player_switches(
        &mut self,
        input_ports: DefenderInputPorts,
    ) -> Result<RedLabelSwitchScan, String> {
        let layout = red_label_ram_layout()?;
        let previous_pia21 = self.read_field_byte(&layout, "base_page", "PIA21")?;
        let previous_pia22 = self.read_field_byte(&layout, "base_page", "PIA22")?;
        let current_pia21 = input_ports.in0;
        let current_pia31 = input_ports.in1;
        let triggered_bits = current_pia21 & !(previous_pia21 | previous_pia22);

        self.write_field_byte(&layout, "base_page", "PIA22", previous_pia21)?;
        self.write_field_byte(&layout, "base_page", "PIA21", current_pia21)?;
        self.write_field_byte(&layout, "base_page", "PIA31", current_pia31)?;

        let queued = translated_player_switch_process(triggered_bits)?;

        if let Some(process) = queued {
            self.queue_switch_process(&layout, process)?;
        }

        Ok(RedLabelSwitchScan {
            previous_pia21,
            previous_pia22,
            current_pia21,
            current_pia31,
            triggered_bits,
            queued,
        })
    }

    /// Source-shaped red-label `CSCAN`: keep the two-frame `PIA01`/`PIA02`
    /// coin-door history, mask the current IN2 coin/admin byte like `ANDB #$3F`,
    /// perform the source double-check with the same input sample, and queue the
    /// first source `SWTAB1` process whose bit survives both history checks.
    pub fn scan_translated_coin_switches(
        &mut self,
        input_ports: DefenderInputPorts,
    ) -> Result<RedLabelCoinSwitchScan, String> {
        let layout = red_label_ram_layout()?;
        let previous_pia01 = self.read_field_byte(&layout, "base_page", "PIA01")?;
        let previous_pia02 = self.read_field_byte(&layout, "base_page", "PIA02")?;
        let history_clear_mask = !(previous_pia01 | previous_pia02);
        let current_pia01 = input_ports.in2 & RED_LABEL_COIN_SCAN_MASK;
        let triggered_bits = current_pia01 & history_clear_mask;

        self.write_field_byte(&layout, "base_page", "PIA02", previous_pia01)?;
        self.write_field_byte(&layout, "base_page", "PIA01", current_pia01)?;

        let confirmed_bits = if triggered_bits == 0 {
            0
        } else {
            let confirmed_pia01 = input_ports.in2 & current_pia01;
            self.write_field_byte(&layout, "base_page", "PIA01", confirmed_pia01)?;
            confirmed_pia01 & history_clear_mask
        };
        let queued = translated_coin_switch_process(confirmed_bits)?;

        if let Some(process) = queued {
            self.queue_switch_process(&layout, process)?;
        }

        Ok(RedLabelCoinSwitchScan {
            previous_pia01,
            previous_pia02,
            current_pia01,
            triggered_bits,
            confirmed_bits,
            queued,
        })
    }

    /// Source-shaped `SSCAN` coin/slam debounce maintenance before switch
    /// scans: seed `SLMCNT` from tilt, then decrement every nonzero slam/coin
    /// counter once.
    pub fn tick_coin_slam_debouncers(
        &mut self,
        input_ports: DefenderInputPorts,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        if input_ports.in2 & RED_LABEL_SLAM_SWITCH_BIT != 0 {
            self.write_field_byte(
                &layout,
                "base_page",
                "SLMCNT",
                RED_LABEL_SLAM_DEBOUNCE_COUNT,
            )?;
        }

        for field in ["SLMCNT", "LCCNT", "CCCNT", "RCCNT"] {
            let value = self.read_field_byte(&layout, "base_page", field)?;
            if value != 0 {
                self.write_field_byte(&layout, "base_page", field, value.wrapping_sub(1))?;
            }
        }
        Ok(())
    }

    /// Source-shaped `LCOIN` / `RCOIN` / `CCOIN` entry: reject slam/debounce
    /// state, arm the selected coin counter, stash the fixed-bank vector in
    /// `PD2`, then sleep to `CN1`.
    pub fn start_coin_process_current_process(
        &mut self,
        slot: RedLabelCoinSlot,
    ) -> Result<RedLabelCoinProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let slam_counter = self.read_field_byte(&layout, "base_page", "SLMCNT")?;
        if slam_counter != 0 {
            let killed_process = self.kill_current_process(&layout)?;
            return Ok(RedLabelCoinProcessStep::Slammed {
                slot: Some(slot),
                process_address,
                slam_counter,
                killed_process,
            });
        }

        let counter_before = self.read_field_byte(&layout, "base_page", slot.counter_field())?;
        if counter_before != 0 {
            let killed_process = self.kill_current_process(&layout)?;
            return Ok(RedLabelCoinProcessStep::DebounceBlocked {
                slot,
                process_address,
                counter: counter_before,
                killed_process,
            });
        }

        self.write_field_byte(
            &layout,
            "base_page",
            slot.counter_field(),
            RED_LABEL_COIN_DEBOUNCE_COUNT,
        )?;
        let vector_address = slot.vector_address();
        self.write_process_data_word(&layout, process_address, "PD2", vector_address)?;
        let wakeup_address = red_label_routine_address("CN1")?;
        self.sleep_current_process(RED_LABEL_COIN_SLEEP_TICKS, wakeup_address)?;

        Ok(RedLabelCoinProcessStep::DebounceSleeping {
            slot,
            process_address,
            slam_counter,
            counter_before,
            counter_after: RED_LABEL_COIN_DEBOUNCE_COUNT,
            vector_address,
            wakeup_address,
        })
    }

    /// Source-shaped `CN1`: reject slam tilt, load `CNSND`, run the fixed-bank
    /// coinage vector selected in `PD2`, then suicide.
    pub fn continue_coin_process_current_process(
        &mut self,
    ) -> Result<RedLabelCoinProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let vector_address = self.read_process_data_word(&layout, process_address, "PD2")?;
        let slot = RedLabelCoinSlot::from_vector_address(vector_address).ok_or_else(|| {
            format!("red-label coin process vector 0x{vector_address:04X} is not translated")
        })?;
        let slam_counter = self.read_field_byte(&layout, "base_page", "SLMCNT")?;
        if slam_counter != 0 {
            let killed_process = self.kill_current_process(&layout)?;
            return Ok(RedLabelCoinProcessStep::Slammed {
                slot: Some(slot),
                process_address,
                slam_counter,
                killed_process,
            });
        }

        let sound_loaded = self.load_sound_table_by_label("CNSND")?;
        let coin_credit = self.apply_coin_slot_credit(slot)?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelCoinProcessStep::Completed {
            slot,
            process_address,
            vector_address,
            sound_loaded,
            coin_credit,
            killed_process,
        })
    }

    /// Source-shaped `SWP` over the two `SWPROC` slots: create every queued
    /// process whose status mask is clear, and clear the routine word for each
    /// consumed slot just as the source does.
    pub fn dispatch_switch_processes(&mut self) -> Result<Vec<RedLabelCreatedProcess>, String> {
        let layout = red_label_ram_layout()?;
        let mut created = Vec::new();

        loop {
            let Some(process) = self.take_next_switch_process(&layout)? else {
                return Ok(created);
            };
            let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
            if process.status_mask & status != 0 {
                continue;
            }
            created.push(self.make_process(process.routine_address, process.process_type)?);
        }
    }

    /// Source-shaped `REV` entry: ignore a second reverse process while the
    /// debounce flag is live, otherwise negate `PLADIR` into `NPLAD` and sleep
    /// until the `REV2` switch-hold check.
    pub fn start_reverse_current_process(&mut self) -> Result<RedLabelReverse, String> {
        let layout = red_label_ram_layout()?;
        if self.read_field_byte(&layout, "base_page", "REVFLG")? != 0 {
            return self
                .kill_current_process(&layout)
                .map(RedLabelReverse::Suppressed);
        }

        let process_address = self.current_process_address(&layout)?;
        self.write_field_byte(&layout, "base_page", "REVFLG", 1)?;
        let previous_direction = self.read_field_word(&layout, "base_page", "PLADIR")?;
        let new_direction = 0u16.wrapping_sub(previous_direction);
        self.write_field_word(&layout, "base_page", "NPLAD", new_direction)?;
        let wakeup_address = red_label_routine_address("REV2")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(RedLabelReverse::StartedSleeping {
            process_address,
            previous_direction,
            new_direction,
            wakeup_address,
        })
    }

    /// Source-shaped `REV2`: while reverse remains held, sleep for two ticks
    /// and re-check; after release, sleep for five ticks before clearing
    /// `REVFLG` and dying at `REVX1`.
    pub fn continue_reverse_current_process(&mut self) -> Result<RedLabelReverse, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        if self.read_field_byte(&layout, "base_page", "PIA21")? & RED_LABEL_REVERSE_SWITCH_BIT != 0
        {
            let wakeup_address = red_label_routine_address("REV2")?;
            self.sleep_current_process(2, wakeup_address)?;
            return Ok(RedLabelReverse::PressedSleeping {
                process_address,
                wakeup_address,
            });
        }

        let wakeup_address = red_label_routine_address("REVX1")?;
        self.sleep_current_process(5, wakeup_address)?;
        Ok(RedLabelReverse::ReleaseSleeping {
            process_address,
            wakeup_address,
        })
    }

    /// Source-shaped `REVX1`: clear `REVFLG`, then run the `REVX`/`SUCIDE`
    /// tail for the current process.
    pub fn finish_reverse_current_process(&mut self) -> Result<RedLabelReverse, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "REVFLG", 0)?;
        self.kill_current_process(&layout)
            .map(RedLabelReverse::Completed)
    }

    /// Source-shaped `SLEEP`: rewrite the current `CRPROC` cell's timer and
    /// wake address. The executive scheduler resumes from this process's
    /// `PLINK`, matching the ROM's `JMP DISP2` continuation.
    pub fn sleep_current_process(
        &mut self,
        sleep_time: u8,
        wakeup_address: u16,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let current_process = self.current_process_address(&layout)?;
        self.write_process_byte(&layout, current_process, "PTIME", sleep_time)?;
        self.write_process_word(&layout, current_process, "PADDR", wakeup_address)
    }

    /// Source-shaped `KILL`: unlink an active process and push it onto the
    /// regular or super free list selected by `PCOD`.
    pub fn kill_process(&mut self, process_address: u16) -> Result<u16, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process = table_descriptor(&layout, "process")?;
        let super_process = table_descriptor(&layout, "super_process")?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let max_links = process.entries + super_process.entries;
        let mut previous_link_address = active_head;

        for _ in 0..max_links {
            let current = self.read_word(previous_link_address)?;
            if current == 0 {
                return Err(format!(
                    "red-label process 0x{process_address:04X} was not in ACTIVE"
                ));
            }

            let current_table = process_table_for_address(&layout, current)?;
            let current_plink = self.read_process_word(&layout, current, "PLINK")?;
            if current == process_address {
                self.write_word(previous_link_address, current_plink)?;
                let pcod = self.read_process_byte(&layout, current, "PCOD")?;
                let free_list = if pcod == 0 {
                    "free_process"
                } else {
                    "free_super_process"
                };
                let free_head = linked_list(&lists, free_list)?.head_address;
                let old_free = self.read_word(free_head)?;
                self.write_word(free_head, current)?;
                self.write_process_word(&layout, current, "PLINK", old_free)?;
                return Ok(previous_link_address);
            }

            previous_link_address =
                process_field_range_for_address(&layout, current_table, current, "PLINK")?.start;
        }

        Err(String::from(
            "red-label ACTIVE process list did not terminate while killing process",
        ))
    }

    /// Source-shaped `GNCIDE`: kill every active process except the current
    /// process and coin processes (`CTYPE`).
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L91-L102>.
    pub fn genocide_other_processes(&mut self) -> Result<RedLabelGenocide, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process = table_descriptor(&layout, "process")?;
        let super_process = table_descriptor(&layout, "super_process")?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let current_process_address = self.current_process_address(&layout)?;
        let max_links = process.entries + super_process.entries;
        let mut previous_link_address = active_head;
        let mut killed_processes = Vec::new();

        for _ in 0..max_links {
            let process_address = self.read_word(previous_link_address)?;
            if process_address == 0 {
                return Ok(RedLabelGenocide {
                    current_process_address,
                    killed_processes,
                });
            }

            let table = process_table_for_address(&layout, process_address)?;
            let process_type = self.read_process_byte(&layout, process_address, "PTYPE")?;
            if process_address == current_process_address
                || process_type == RED_LABEL_COIN_PROCESS_TYPE
            {
                previous_link_address =
                    process_field_range_for_address(&layout, table, process_address, "PLINK")?
                        .start;
                continue;
            }

            let previous_link_address_after_kill = self.kill_process(process_address)?;
            killed_processes.push(RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address: previous_link_address_after_kill,
            });
            previous_link_address = previous_link_address_after_kill;
        }

        Err(String::from(
            "red-label ACTIVE process list did not terminate during GNCIDE",
        ))
    }

    /// Source-shaped `WVCHK`: return the wrapping sum of active and reserve
    /// enemy counts used by end-of-wave checks.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1748-L1755>.
    pub fn wave_enemy_total(&self) -> Result<u8, String> {
        let layout = red_label_ram_layout()?;
        let mut total = self.read_field_byte(&layout, "enemy_runtime", "LNDCNT")?;
        for field in ["LNDRES", "TIECNT", "PRBCNT", "SWCNT", "SCZCNT", "SCZRES"] {
            total = total.wrapping_add(self.read_field_byte(&layout, "enemy_runtime", field)?);
        }
        Ok(total)
    }

    /// Source-shaped `GETWV`: increment the player's wave, refresh `PENEMY`
    /// from source-order `WVTAB`, reset `PTARG` on the configured restore-wave
    /// cadence, and apply inter-wall `WDELT` difficulty deltas.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1846-L1900>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1908-L1924>.
    pub fn get_new_wave_parameters_for_player_address(
        &mut self,
        player_address: u16,
    ) -> Result<RedLabelWaveParameters, String> {
        let layout = red_label_ram_layout()?;
        let player_table = table_descriptor(&layout, "player")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let wave_range = player_field_range_for_entry(&layout, player_index, "PWAV")?;
        let target_range = player_field_range_for_entry(&layout, player_index, "PTARG")?;
        let enemy_range = player_field_range_for_entry(&layout, player_index, "PENEMY")?;
        let enemy_len = usize::from(enemy_range.end - enemy_range.start);
        if enemy_len < RED_LABEL_WDELT_RECORD_COUNT {
            return Err(format!(
                "red-label player.PENEMY has {enemy_len} byte(s), expected at least {RED_LABEL_WDELT_RECORD_COUNT}"
            ));
        }

        let mut previous_values = [0; RED_LABEL_WDELT_RECORD_COUNT];
        for (offset, value) in previous_values.iter_mut().enumerate() {
            *value = self.read_byte(enemy_range.start + offset as u16)?;
        }

        let previous_wave = self.read_byte(wave_range.start)?;
        let wave = previous_wave.wrapping_add(1);
        self.write_byte(wave_range.start, wave)?;

        let restore_wave = self.read_cmos_byte_by_symbol("GA1+6")?;
        let restored_humans = getwv_restore_wave_hits(wave, restore_wave);
        if restored_humans {
            self.write_byte(target_range.start, RED_LABEL_START_HUMAN_COUNT)?;
        }

        let mut values = red_label_wave_table().getwv_base_values(wave)?;
        let difficulty_initial = bcd_byte_to_u16(self.read_cmos_byte_by_symbol("GA1")?) as u8;
        let difficulty_ceiling = bcd_byte_to_u16(self.read_cmos_byte_by_symbol("GA1+2")?) as u8;
        let inter_wall_delta_iterations =
            getwv_inter_wall_delta_iterations(wave, difficulty_initial, difficulty_ceiling);
        for _ in 0..inter_wall_delta_iterations {
            red_label_wave_table().apply_inter_wall_deltas(&mut values)?;
        }

        let parameter_range =
            enemy_range.start..enemy_range.start + RED_LABEL_WDELT_RECORD_COUNT as u16;
        self.write_range(parameter_range.clone(), &values)?;
        let bytes_changed = previous_values
            .iter()
            .zip(values.iter())
            .filter(|(previous, value)| previous != value)
            .count();

        Ok(RedLabelWaveParameters {
            player_address,
            player_index,
            previous_wave,
            wave,
            restore_wave,
            restored_humans,
            difficulty_initial,
            difficulty_ceiling,
            inter_wall_delta_iterations,
            parameter_delta: RedLabelWaveDelta {
                start_address: parameter_range.start,
                bytes_updated: RED_LABEL_WDELT_RECORD_COUNT,
                bytes_changed,
            },
        })
    }

    /// Source-shaped `GEXEC` entry: initialize the process delta counter and
    /// wave pacing timers, then fall through into `GEX0`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1647-L1731>.
    pub fn start_game_exec_current_process(&mut self) -> Result<RedLabelGameExec, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let delta_counter = 40;
        self.write_process_byte(&layout, process_address, "PD", delta_counter)?;
        let ufo_timer = self.read_field_byte(&layout, "enemy_runtime", "UFOTIM")?;
        self.write_field_byte(&layout, "enemy_runtime", "UFOTMR", ufo_timer)?;
        let wave_timer = 1;
        self.write_field_byte(&layout, "enemy_runtime", "WAVTMR", wave_timer)?;
        let entry = RedLabelGameExecEntry {
            delta_counter,
            ufo_timer,
            wave_timer,
        };
        self.step_game_exec_current_process_with_layout(&layout, Some(entry))
    }

    /// Source-shaped `GEX0` game executive pass. The zero-enemy branch now
    /// runs the ROM `BONUS` body with the assembled return site after
    /// `JSR BONUS`, so `BC3` can resume through the `PLSTR0` restart path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1654-L1731>.
    pub fn step_game_exec_current_process(&mut self) -> Result<RedLabelGameExec, String> {
        let layout = red_label_ram_layout()?;
        self.step_game_exec_current_process_with_layout(&layout, None)
    }

    pub(super) fn step_game_exec_current_process_with_layout(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        entry: Option<RedLabelGameExecEntry>,
    ) -> Result<RedLabelGameExec, String> {
        let process_address = self.current_process_address(layout)?;
        let status = self.read_field_byte(layout, "base_page", "STATUS")?;
        let mut enemy_total = None;
        let mut ufo = None;
        let mut lander = None;

        if status & 0x08 == 0 {
            let total = self.wave_enemy_total()?;
            enemy_total = Some(total);
            if total == 0 {
                return self.start_game_exec_wave_clear_bonus_current_process(
                    layout,
                    process_address,
                    status,
                    total,
                );
            }

            ufo = Some(self.advance_game_exec_ufo_pacing(layout, total)?);
            lander = Some(self.advance_game_exec_lander_pacing(layout)?);
        }

        let star_time = self.advance_game_exec_star_time()?;
        let wakeup_address = red_label_routine_address("GEX0")?;
        self.sleep_current_process(15, wakeup_address)?;
        Ok(RedLabelGameExec::Running(RedLabelGameExecRun {
            process_address,
            entry,
            status,
            enemy_total,
            ufo,
            lander,
            star_time,
            wakeup_address,
        }))
    }

    pub(super) fn start_game_exec_wave_clear_bonus_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        previous_status: u8,
        enemy_total: u8,
    ) -> Result<RedLabelGameExec, String> {
        let status = 0x77;
        self.write_field_byte(layout, "base_page", "STATUS", status)?;
        let genocide = self.genocide_other_processes()?;
        self.save_current_player_state_for_death(layout)?;
        let return_address = red_label_routine_address("GEXBON")?;
        let bonus =
            self.start_bonus_current_process_with_return(layout, process_address, return_address)?;

        Ok(RedLabelGameExec::WaveClearBonusSleeping(
            RedLabelGameExecWaveClearBonusSleeping {
                process_address,
                previous_status,
                status,
                enemy_total,
                genocide,
                return_address,
                bonus,
            },
        ))
    }

    pub(super) fn advance_game_exec_ufo_pacing(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        enemy_total: u8,
    ) -> Result<RedLabelGameExecUfoPacing, String> {
        let previous_timer = self.read_field_byte(layout, "enemy_runtime", "UFOTMR")?;
        let mut current_timer = previous_timer;
        let mut accelerated_timer = None;
        if enemy_total <= 8 {
            let mut target_timer = self
                .read_field_byte(layout, "enemy_runtime", "UFOTIM")?
                .wrapping_shr(1);
            if enemy_total <= 3 {
                target_timer = target_timer.wrapping_shr(1);
            }
            target_timer = target_timer.wrapping_add(1);
            if target_timer < current_timer {
                current_timer = target_timer;
                self.write_field_byte(layout, "enemy_runtime", "UFOTMR", current_timer)?;
                accelerated_timer = Some(current_timer);
            }
        }

        let decremented_timer = current_timer.wrapping_sub(1);
        self.write_field_byte(layout, "enemy_runtime", "UFOTMR", decremented_timer)?;
        let mut reset_timer = None;
        let mut ufo_count_before = None;
        let mut ufo_count_after = None;
        let mut started = None;
        if decremented_timer == 0 {
            let mut next_timer = self.read_field_byte(layout, "enemy_runtime", "UFOTIM")?;
            if enemy_total < 4 {
                next_timer = self.advance_red_label_rmax(layout, next_timer.wrapping_shr(2))?;
            }
            self.write_field_byte(layout, "enemy_runtime", "UFOTMR", next_timer)?;
            reset_timer = Some(next_timer);

            let count_before = self.read_field_byte(layout, "enemy_runtime", "UFOCNT")?;
            ufo_count_before = Some(count_before);
            if count_before < 12 {
                let ufo_start = self.start_ufo_process()?;
                let count_after = count_before.wrapping_add(1);
                self.write_field_byte(layout, "enemy_runtime", "UFOCNT", count_after)?;
                ufo_count_after = Some(count_after);
                started = Some(ufo_start);
            } else {
                ufo_count_after = Some(count_before);
            }
        }

        Ok(RedLabelGameExecUfoPacing {
            previous_timer,
            accelerated_timer,
            decremented_timer,
            reset_timer,
            ufo_count_before,
            ufo_count_after,
            started,
        })
    }

    pub(super) fn advance_game_exec_lander_pacing(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelGameExecLanderPacing, String> {
        let previous_timer = self.read_field_byte(layout, "enemy_runtime", "WAVTMR")?;
        let decremented_timer = previous_timer.wrapping_sub(1);
        self.write_field_byte(layout, "enemy_runtime", "WAVTMR", decremented_timer)?;

        if decremented_timer != 0 {
            let active_count = self.read_field_byte(layout, "enemy_runtime", "LNDCNT")?;
            if active_count != 0 {
                return Ok(RedLabelGameExecLanderPacing {
                    previous_timer,
                    decremented_timer,
                    reset_timer: None,
                    active_count_before: Some(active_count),
                    reserve_count_before: None,
                    requested_count: None,
                    reserve_count_after: None,
                    started: None,
                });
            }
        }

        let reset_timer = self.read_field_byte(layout, "enemy_runtime", "WAVTIM")?;
        self.write_field_byte(layout, "enemy_runtime", "WAVTMR", reset_timer)?;
        let reserve_count = self.read_field_byte(layout, "enemy_runtime", "LNDRES")?;
        if reserve_count == 0 {
            return Ok(RedLabelGameExecLanderPacing {
                previous_timer,
                decremented_timer,
                reset_timer: Some(reset_timer),
                active_count_before: None,
                reserve_count_before: Some(reserve_count),
                requested_count: None,
                reserve_count_after: Some(reserve_count),
                started: None,
            });
        }

        let active_count = self.read_field_byte(layout, "enemy_runtime", "LNDCNT")?;
        if active_count >= 8 {
            return Ok(RedLabelGameExecLanderPacing {
                previous_timer,
                decremented_timer,
                reset_timer: Some(reset_timer),
                active_count_before: Some(active_count),
                reserve_count_before: Some(reserve_count),
                requested_count: None,
                reserve_count_after: Some(reserve_count),
                started: None,
            });
        }

        let wave_size = self.read_field_byte(layout, "enemy_runtime", "WAVSIZ")?;
        let requested_count = if wave_size <= reserve_count {
            wave_size
        } else {
            reserve_count
        };
        let started = self.start_lander_processes(requested_count)?;
        let reserve_count_after = reserve_count.wrapping_sub(requested_count);
        self.write_field_byte(layout, "enemy_runtime", "LNDRES", reserve_count_after)?;

        Ok(RedLabelGameExecLanderPacing {
            previous_timer,
            decremented_timer,
            reset_timer: Some(reset_timer),
            active_count_before: Some(active_count),
            reserve_count_before: Some(reserve_count),
            requested_count: Some(requested_count),
            reserve_count_after: Some(reserve_count_after),
            started: Some(started),
        })
    }

    /// Source-shaped `GEXEC` tail slice through `GEX6`: restore one active star
    /// per pass until `STRCNT` reaches 16, advance `GTIME`, then decrement
    /// `PD` and run the source `WDELT` intra-wall update on `ELIST` every 40
    /// passes.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1711-L1731>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1908-L1924>.
    pub fn advance_game_exec_star_time(&mut self) -> Result<RedLabelGameExecStarTime, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let previous_star_count = self.read_field_byte(&layout, "base_page", "STRCNT")?;
        let star_count = if previous_star_count < 16 {
            previous_star_count.wrapping_add(1)
        } else {
            previous_star_count
        };
        if star_count != previous_star_count {
            self.write_field_byte(&layout, "base_page", "STRCNT", star_count)?;
        }

        let previous_game_time = self.read_field_byte(&layout, "base_page", "GTIME")?;
        let incremented_game_time = previous_game_time.wrapping_add(1);
        let (game_time, audit_meter) = if incremented_game_time > 240 {
            (0, Some(0x06))
        } else {
            (incremented_game_time, None)
        };
        self.write_field_byte(&layout, "base_page", "GTIME", game_time)?;

        let previous_delta_counter = self.read_process_byte(&layout, process_address, "PD")?;
        let mut delta_counter = previous_delta_counter.wrapping_sub(1);
        self.write_process_byte(&layout, process_address, "PD", delta_counter)?;
        let wave_delta = if delta_counter == 0 {
            let elist = field_range(&layout, "enemy_runtime", "ELIST")?;
            let elist_len = usize::from(elist.end - elist.start);
            if elist_len != RED_LABEL_WDELT_RECORD_COUNT {
                return Err(format!(
                    "red-label ELIST has {elist_len} byte(s), expected {RED_LABEL_WDELT_RECORD_COUNT}"
                ));
            }
            let mut values = [0; RED_LABEL_WDELT_RECORD_COUNT];
            for (offset, value) in values.iter_mut().enumerate() {
                *value = self.read_byte(elist.start + offset as u16)?;
            }
            let bytes_changed = red_label_wave_table().apply_intra_wall_deltas(&mut values)?;
            self.write_range(elist, &values)?;
            delta_counter = 40;
            self.write_process_byte(&layout, process_address, "PD", delta_counter)?;
            Some(RedLabelWaveDelta {
                start_address: field_range(&layout, "enemy_runtime", "ELIST")?.start,
                bytes_updated: RED_LABEL_WDELT_RECORD_COUNT,
                bytes_changed,
            })
        } else {
            None
        };

        Ok(RedLabelGameExecStarTime {
            process_address,
            previous_star_count,
            star_count,
            previous_game_time,
            game_time,
            audit_meter,
            previous_delta_counter,
            delta_counter,
            wave_delta,
        })
    }

    /// Source-shaped `PLAYER` IRQ motion update: apply 24-bit horizontal
    /// damping, thrust acceleration from `PIA21`, position/scroll correction,
    /// absolute X calculation, and vertical velocity from `PIA31`/`PIA21`.
    /// Ship-body and thrust-flame byte rendering are owned by the translated
    /// `PRDISP` / `POUT` display slice; scanline scheduling remains separate.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2339-L2476>.
    pub fn update_player_motion_from_pia(&mut self) -> Result<RedLabelPlayerMotion, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x40 != 0 {
            return Ok(RedLabelPlayerMotion::StatusBlocked { status });
        }

        let pia21 = self.read_field_byte(&layout, "base_page", "PIA21")?;
        let pia31 = self.read_field_byte(&layout, "base_page", "PIA31")?;

        self.apply_player_horizontal_damping(&layout)?;
        let direction = self.read_field_word(&layout, "base_page", "PLADIR")?;
        if pia21 & 0x02 != 0 {
            self.add_signed_word_to_player_x_velocity(&layout, direction)?;
        }

        let player_calculated_x =
            self.update_player_calculated_x_from_velocity(&layout, direction)?;
        let previous_player_x16 = self.read_field_word(&layout, "base_page", "PLAX16")?;
        let (player_x16, background_delta) =
            player_scroll_adjusted_x(previous_player_x16, player_calculated_x);
        self.write_field_word(&layout, "base_page", "BGDELT", background_delta)?;
        self.write_field_word(&layout, "base_page", "PLAX16", player_x16)?;
        self.write_byte(
            field_range(&layout, "base_page", "NPLAXC")?.start,
            player_x16.to_be_bytes()[0],
        )?;

        let previous_background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        self.write_field_word(&layout, "base_page", "BGLX", previous_background_left)?;
        let velocity_range = field_range(&layout, "base_page", "PLAXV")?;
        let clamped_velocity =
            clamp_player_x_velocity_high_word(self.read_word(velocity_range.start)?);
        self.write_word(velocity_range.start, clamped_velocity)?;
        let background_left = previous_background_left
            .wrapping_add(clamped_velocity)
            .wrapping_sub(background_delta);
        self.write_field_word(&layout, "base_page", "BGL", background_left)?;

        let absolute_x = player_absolute_x(player_x16, background_left);
        self.write_field_word(&layout, "base_page", "PLABX", absolute_x)?;

        let player_y16 = self.read_field_word(&layout, "base_page", "PLAY16")?;
        let player_y_screen = player_y16.to_be_bytes()[0];
        let vertical_action = player_vertical_action(pia21, pia31);
        let Some(player_y_velocity) =
            self.next_player_y_velocity(&layout, player_y_screen, vertical_action)?
        else {
            return Ok(RedLabelPlayerMotion::VerticalLimitBlocked {
                status,
                pia21,
                pia31,
                player_velocity: self.read_fixed_bytes(velocity_range.start)?,
                player_calculated_x,
                player_x16,
                background_delta,
                background_left,
                absolute_x,
                player_y_velocity: self.read_field_word(&layout, "base_page", "PLAYV")?,
                player_y16,
                next_player_x: player_x16.to_be_bytes()[0],
            });
        };

        self.write_field_word(&layout, "base_page", "PLAYV", player_y_velocity)?;
        let next_player_y16 = player_y_velocity.wrapping_add(player_y16);
        self.write_field_word(&layout, "base_page", "PLAY16", next_player_y16)?;
        self.write_field_byte(
            &layout,
            "base_page",
            "NPLAYC",
            next_player_y16.to_be_bytes()[0],
        )?;
        let next_player_screen = u16::from_be_bytes([
            player_x16.to_be_bytes()[0],
            next_player_y16.to_be_bytes()[0],
        ]);

        Ok(RedLabelPlayerMotion::Updated {
            status,
            pia21,
            pia31,
            player_velocity: self.read_fixed_bytes(velocity_range.start)?,
            player_calculated_x,
            player_x16,
            background_delta,
            background_left,
            absolute_x,
            player_y_velocity,
            player_y16: next_player_y16,
            next_player_screen,
        })
    }

    /// Source-shaped `PRDISP` player picture slice: when the player Y position
    /// is inside the caller's scanline band, erase the old 8x6 player
    /// footprint through `OFF86`, copy `NPLAD` into `PLADIR`, then draw
    /// `PLAPIC`/`PLBPIC` through the `ON86` flavor selected by `PLAX16+1`.
    /// The adjacent `THOUT` / `THOFF` thrust-flame byte writes are handled by
    /// the same source-shaped display slice.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2304-L2334>.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L374-L430>.
    pub fn display_player_picture_in_band(
        &mut self,
        upper_bound: u8,
        lower_bound: u8,
    ) -> Result<RedLabelPlayerDisplay, String> {
        let layout = red_label_ram_layout()?;
        self.write_byte(
            field_range(&layout, "base_page", "TEMP48")?.start,
            upper_bound,
        )?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x10 != 0 {
            return Ok(RedLabelPlayerDisplay::StatusBlocked { status });
        }

        let player_y = self.read_field_byte(&layout, "base_page", "PLAYC")?;
        if upper_bound <= player_y || lower_bound > player_y {
            return Ok(RedLabelPlayerDisplay::OutsideBand {
                status,
                upper_bound,
                lower_bound,
                player_y,
            });
        }

        let old_screen_address = self.read_field_word(&layout, "base_page", "PLAXC")?;
        let old_direction = self.read_field_word(&layout, "base_page", "PLADIR")?;
        self.clear_player_on86_footprint(old_screen_address)?;
        if old_direction & 0x8000 == 0 {
            self.clear_player_thrust_forward(old_screen_address)?;
        } else {
            self.clear_player_thrust_backward(old_screen_address)?;
        }

        let new_direction = self.read_field_word(&layout, "base_page", "NPLAD")?;
        self.write_field_word(&layout, "base_page", "PLADIR", new_direction)?;
        let new_screen_address = self.read_field_word(&layout, "base_page", "NPLAXC")?;
        self.write_field_word(&layout, "base_page", "PLAXC", new_screen_address)?;
        let picture_address = if new_direction & 0x8000 == 0 {
            red_label_object_picture_address("PLAPIC")?
        } else {
            red_label_object_picture_address("PLBPIC")?
        };
        let alternate_flavor =
            self.read_byte(field_range(&layout, "base_page", "PLAX16")?.start + 1)? & 0x80 != 0;
        self.write_player_on86_picture(new_screen_address, picture_address, alternate_flavor)?;
        if new_direction & 0x8000 == 0 {
            self.write_player_thrust_forward(&layout, new_screen_address)?;
        } else {
            self.write_player_thrust_backward(&layout, new_screen_address)?;
        }

        Ok(RedLabelPlayerDisplay::Updated {
            status,
            upper_bound,
            lower_bound,
            old_screen_address,
            old_direction,
            erased_old_picture: true,
            new_direction,
            new_screen_address,
            picture_address,
            alternate_flavor,
        })
    }

    /// Source-shaped `OPROC`: walk `OPTR`, erase an object's old descriptor
    /// picture only when its saved screen Y lies in the caller's scanline band,
    /// then draw the current object descriptor when its new Y and BGL-relative
    /// X pass the ROM bounds checks. The caller is responsible for selecting
    /// the character map, matching the IRQ path that writes `MAPC=2` before
    /// entering `OPROC`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2501-L2546>.
    pub fn process_active_objects_in_band(
        &mut self,
        upper_bound: u8,
        lower_bound: u8,
    ) -> Result<RedLabelObjectDisplayBand, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x20 != 0 {
            return Ok(RedLabelObjectDisplayBand::StatusBlocked {
                status,
                upper_bound,
                lower_bound,
            });
        }

        let lists = red_label_linked_lists()?;
        let object_table = table_descriptor(&layout, "object")?;
        let mut object_address =
            self.read_word(linked_list(&lists, "active_object")?.head_address)?;
        let background_left = self.read_field_word(&layout, "base_page", "BGL")?;
        let mut objects = Vec::new();
        let mut scanned = 0;

        loop {
            if object_address == 0 {
                return Ok(RedLabelObjectDisplayBand::Updated {
                    status,
                    upper_bound,
                    lower_bound,
                    objects,
                });
            }
            if scanned >= object_table.entries {
                return Err(String::from(
                    "red-label active object list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, object_address)?;

            let next_object = self.read_object_word(&layout, object_address, "OLINK")?;
            let picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
            let mut display = RedLabelObjectDisplay {
                object_address,
                picture_address,
                old_screen_address: None,
                erased_old_picture: None,
                relative_x: None,
                new_screen_address: None,
                alternate_flavor: None,
                written_picture: None,
            };

            let old_screen_address = self.read_object_screen_address(&layout, object_address)?;
            if old_screen_address != 0 {
                let old_y = old_screen_address.to_be_bytes()[1];
                if !object_display_y_in_band(old_y, upper_bound, lower_bound) {
                    object_address = next_object;
                    continue;
                }

                display.old_screen_address = Some(old_screen_address);
                display.erased_old_picture =
                    self.erase_object_picture_by_descriptor(old_screen_address, picture_address)?;
                self.write_object_screen_address(&layout, object_address, 0)?;
            }

            let object_y = self
                .read_object_word(&layout, object_address, "OY16")?
                .to_be_bytes()[0];
            if object_display_y_in_band(object_y, upper_bound, lower_bound) {
                let relative_x = self
                    .read_object_word(&layout, object_address, "OX16")?
                    .wrapping_sub(background_left);
                display.relative_x = Some(relative_x);

                if relative_x < RED_LABEL_OBJECT_SCREEN_RANGE {
                    let shifted = relative_x.wrapping_shl(2);
                    let [screen_x, phase] = shifted.to_be_bytes();
                    let picture = red_label_object_picture(picture_address)?;

                    if screen_x.wrapping_add(picture.width) <= 0x9C {
                        let alternate_flavor = phase & 0x80 != 0;
                        let new_screen_address = u16::from_be_bytes([screen_x, object_y]);
                        self.write_object_screen_address(
                            &layout,
                            object_address,
                            new_screen_address,
                        )?;
                        display.new_screen_address = Some(new_screen_address);
                        display.alternate_flavor = Some(alternate_flavor);
                        display.written_picture = self.output_object_picture_by_descriptor(
                            new_screen_address,
                            picture_address,
                            alternate_flavor,
                        )?;
                    }
                }
            }

            if display.old_screen_address.is_some() || display.new_screen_address.is_some() {
                objects.push(display);
            }
            object_address = next_object;
        }
    }

    pub fn process_live_active_objects_full_frame(
        &mut self,
    ) -> Result<RedLabelObjectDisplayBand, String> {
        self.clear_recorded_live_object_addresses()?;
        let display = self.process_active_objects_in_band(0xFF, 0)?;
        self.record_current_live_object_footprints()?;
        Ok(display)
    }

    pub(super) fn clear_recorded_live_object_addresses(&mut self) -> Result<(), String> {
        let recorded_addresses = std::mem::take(&mut self.live_object_addresses);
        for address in recorded_addresses {
            self.write_byte(address, 0)?;
        }
        Ok(())
    }

    pub(super) fn record_current_live_object_footprints(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object_table = table_descriptor(&layout, "object")?;
        let mut object_address =
            self.read_word(linked_list(&lists, "active_object")?.head_address)?;
        for _ in 0..object_table.entries {
            if object_address == 0 {
                return Ok(());
            }
            if object_table_for_address(&layout, object_address).is_err() {
                return Ok(());
            }
            let next_object = self.read_object_word(&layout, object_address, "OLINK")?;
            let screen_address = self.read_object_screen_address(&layout, object_address)?;
            if screen_address != 0 {
                let picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
                let picture = red_label_object_picture(picture_address)?;
                for column in 0..picture.width {
                    let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
                    for row in 0..picture.height {
                        let address = screen_offset(column_address, u16::from(row))?;
                        if !self.live_object_addresses.contains(&address) {
                            self.live_object_addresses.push(address);
                        }
                    }
                }
            }
            object_address = next_object;
        }

        Err(String::from(
            "red-label active object list did not terminate while recording live footprints",
        ))
    }

    pub fn redraw_live_laser_beams(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_table = table_descriptor(&layout, "process")?;
        let lasr0 = red_label_routine_address("LASR0")?;
        let lasl0 = red_label_routine_address("LASL0")?;
        let mut process_address =
            self.read_word(linked_list(&lists, "active_process")?.head_address)?;

        for _ in 0..process_table.entries {
            if process_address == 0 {
                return Ok(());
            }
            process_table_for_address(&layout, process_address)?;
            let next_process = self.read_process_word(&layout, process_address, "PLINK")?;
            let routine_address = self.read_process_word(&layout, process_address, "PADDR")?;
            let direction = if routine_address == lasr0 {
                Some(RedLabelLaserDirection::Right)
            } else if routine_address == lasl0 {
                Some(RedLabelLaserDirection::Left)
            } else {
                None
            };
            if let Some(direction) = direction {
                let tail = self.read_process_data_word(&layout, process_address, "PD4")?;
                let tip = self.read_process_data_word(&layout, process_address, "PD")?;
                self.draw_live_laser_beam_span(direction, tail, tip)?;
            }
            process_address = next_process;
        }

        Err(String::from(
            "red-label active process list did not terminate while redrawing live laser beams",
        ))
    }

    pub(super) fn draw_live_laser_beam_span(
        &mut self,
        direction: RedLabelLaserDirection,
        tail: u16,
        tip: u16,
    ) -> Result<(), String> {
        let mut address = tail;
        for _ in 0..=u8::MAX {
            self.write_byte(address, 0x99)?;
            let next = step_laser_address(direction, address);
            let keep_drawing = match direction {
                RedLabelLaserDirection::Right => next <= tip,
                RedLabelLaserDirection::Left => next >= tip,
            };
            if !keep_drawing {
                self.write_byte(tip, 0x99)?;
                return Ok(());
            }
            address = next;
        }

        Err(format!(
            "red-label {:?} live laser redraw did not reach PD 0x{tip:04X}",
            direction
        ))
    }

    pub fn redraw_live_defender_wordmark_gap(&mut self) -> Result<bool, String> {
        let visible_wordmark_bytes = self.defender_wordmark_video_byte_count()?;
        if visible_wordmark_bytes >= RED_LABEL_LIVE_DEFENDER_WORDMARK_COALESCED_BYTES {
            self.live_defender_wordmark_coalesced = true;
            return Ok(false);
        }

        let def33 = red_label_routine_address("DEF33")?;
        if !self.live_defender_wordmark_coalesced || !self.active_process_has_routine(&[def33])? {
            return Ok(false);
        }
        if visible_wordmark_bytes >= RED_LABEL_LIVE_DEFENDER_WORDMARK_BLANK_BYTES {
            return Ok(false);
        }

        self.write_live_defender_wordmark()?;
        Ok(true)
    }

    pub(super) fn defender_wordmark_video_byte_count(&self) -> Result<usize, String> {
        let mut count = 0;
        for column in 0..RED_LABEL_ATTRACT_DEFENDER_WHOLE_WIDTH {
            let column_address = screen_offset(
                RED_LABEL_ATTRACT_DEFENDER_RESTORE_SCREEN,
                u16::from(column) << 8,
            )?;
            for row in 0..RED_LABEL_ATTRACT_DEFENDER_WHOLE_HEIGHT {
                let address = screen_offset(column_address, u16::from(row))?;
                if self.read_byte(address)? != 0 {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    pub(super) fn write_live_defender_wordmark(&mut self) -> Result<(), String> {
        for column in 0..RED_LABEL_ATTRACT_DEFENDER_WHOLE_WIDTH {
            let column_address = screen_offset(
                RED_LABEL_ATTRACT_DEFENDER_RESTORE_SCREEN,
                u16::from(column) << 8,
            )?;
            let source_column = RED_LABEL_ATTRACT_DEFENDER_DATA
                + u16::from(column) * u16::from(RED_LABEL_ATTRACT_DEFENDER_WHOLE_HEIGHT);
            for row in 0..RED_LABEL_ATTRACT_DEFENDER_WHOLE_HEIGHT {
                let address = screen_offset(column_address, u16::from(row))?;
                self.write_byte(address, self.read_byte(source_column + u16::from(row))?)?;
            }
        }

        Ok(())
    }

    /// Source-shaped `VELO`: walk `OPTR`, add each active object's velocity to
    /// `OX16` / `OY16`, and wrap the high byte of Y through the cabinet
    /// `YMIN` / `YMAX` bounds.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2478-L2499>.
    pub fn advance_active_object_velocities(
        &mut self,
    ) -> Result<RedLabelObjectVelocityUpdate, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x20 != 0 {
            return Ok(RedLabelObjectVelocityUpdate::StatusBlocked { status });
        }

        let lists = red_label_linked_lists()?;
        let object_table = table_descriptor(&layout, "object")?;
        let mut object_address =
            self.read_word(linked_list(&lists, "active_object")?.head_address)?;
        let mut objects = Vec::new();
        let mut scanned = 0;

        loop {
            if object_address == 0 {
                return Ok(RedLabelObjectVelocityUpdate::Updated { status, objects });
            }
            if scanned >= object_table.entries {
                return Err(String::from(
                    "red-label active object list did not terminate within object table size",
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, object_address)?;

            let previous_x16 = self.read_object_word(&layout, object_address, "OX16")?;
            let x_velocity = self.read_object_word(&layout, object_address, "OXV")?;
            let x16 = previous_x16.wrapping_add(x_velocity);
            self.write_object_word(&layout, object_address, "OX16", x16)?;

            let previous_y16 = self.read_object_word(&layout, object_address, "OY16")?;
            let y_velocity = self.read_object_word(&layout, object_address, "OYV")?;
            let y16 = active_object_next_y(previous_y16, y_velocity);
            self.write_object_word(&layout, object_address, "OY16", y16)?;
            self.write_object_bytes(&layout, object_address, "OBJY", &[y16.to_be_bytes()[0]])?;

            objects.push(RedLabelObjectVelocityStep {
                object_address,
                previous_x16,
                x_velocity,
                x16,
                previous_y16,
                y_velocity,
                y16,
            });

            object_address = self.read_object_word(&layout, object_address, "OLINK")?;
        }
    }

    /// Source-shaped normal-screen `IRQ` upper object-band tail after the
    /// caller has updated `XXX2`: select the object map, run `OPROC`, then
    /// `PRDISP`, then the full translated `SHELL` visible dispatch path.
    /// Hardware entry, `VERTCT`/`IFLG`, `SNDSEQ`, `PLAYER`, `STOUT`, and map
    /// register restoration remain owned by the full IRQ scheduler.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1951-L1958>.
    pub fn run_normal_irq_upper_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        let (upper_bound, lower_bound) = self.irq_object_band_bounds_from("XXX2")?;
        let mut steps = Vec::new();
        steps.push(RedLabelIrqObjectBandStep::Objects(
            self.process_active_objects_in_band(upper_bound, lower_bound)?,
        ));
        steps.push(RedLabelIrqObjectBandStep::Player(
            self.display_player_picture_in_band(upper_bound, lower_bound)?,
        ));
        let shell_steps = self.step_shell_output()?;
        let output_routines = self.dispatch_shell_output_steps(&shell_steps)?;
        steps.push(RedLabelIrqObjectBandStep::Shells {
            steps: shell_steps,
            output_routines,
        });

        Ok(RedLabelIrqObjectBandPass {
            phase: RedLabelIrqObjectBandPhase::NormalUpper,
            upper_bound,
            lower_bound,
            steps,
        })
    }

    /// Source-shaped normal-screen `IRQ` lower object-band tail: load the
    /// scanline bounds from `XXX1`, run `PRDISP`, then `OPROC`, then `VELO`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1988-L1994>.
    pub fn run_normal_irq_lower_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        let (upper_bound, lower_bound) = self.irq_object_band_bounds_from("XXX1")?;
        let steps = vec![
            RedLabelIrqObjectBandStep::Player(
                self.display_player_picture_in_band(upper_bound, lower_bound)?,
            ),
            RedLabelIrqObjectBandStep::Objects(
                self.process_active_objects_in_band(upper_bound, lower_bound)?,
            ),
            RedLabelIrqObjectBandStep::Velocities(self.advance_active_object_velocities()?),
        ];

        Ok(RedLabelIrqObjectBandPass {
            phase: RedLabelIrqObjectBandPhase::NormalLower,
            upper_bound,
            lower_bound,
            steps,
        })
    }

    /// Source-shaped flipped-screen `IRQB` upper object-band tail: load the
    /// scanline bounds from `XXX1`, then run `PRDISP` and `OPROC`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2030-L2035>.
    pub fn run_inverted_irq_upper_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        let (upper_bound, lower_bound) = self.irq_object_band_bounds_from("XXX1")?;
        let steps = vec![
            RedLabelIrqObjectBandStep::Player(
                self.display_player_picture_in_band(upper_bound, lower_bound)?,
            ),
            RedLabelIrqObjectBandStep::Objects(
                self.process_active_objects_in_band(upper_bound, lower_bound)?,
            ),
        ];

        Ok(RedLabelIrqObjectBandPass {
            phase: RedLabelIrqObjectBandPhase::InvertedUpper,
            upper_bound,
            lower_bound,
            steps,
        })
    }

    /// Source-shaped flipped-screen `IRQB` lower object-band tail: load the
    /// scanline bounds from `XXX2`, run `PRDISP`, `OPROC`, full translated
    /// `SHELL` visible dispatch, then `VELO`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2061-L2068>.
    pub fn run_inverted_irq_lower_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        let (upper_bound, lower_bound) = self.irq_object_band_bounds_from("XXX2")?;
        let mut steps = Vec::new();
        steps.push(RedLabelIrqObjectBandStep::Player(
            self.display_player_picture_in_band(upper_bound, lower_bound)?,
        ));
        steps.push(RedLabelIrqObjectBandStep::Objects(
            self.process_active_objects_in_band(upper_bound, lower_bound)?,
        ));
        let shell_steps = self.step_shell_output()?;
        let output_routines = self.dispatch_shell_output_steps(&shell_steps)?;
        steps.push(RedLabelIrqObjectBandStep::Shells {
            steps: shell_steps,
            output_routines,
        });
        steps.push(RedLabelIrqObjectBandStep::Velocities(
            self.advance_active_object_velocities()?,
        ));

        Ok(RedLabelIrqObjectBandPass {
            phase: RedLabelIrqObjectBandPhase::InvertedLower,
            upper_bound,
            lower_bound,
            steps,
        })
    }

    /// Source-scheduled live video slice for the upright red-label `IRQ`
    /// frame work after the separate live sound sequencer tick has run. This
    /// uses the same `VERTCT`/`IFLG` gate as the source IRQ path for map
    /// selection, palette copy, terrain, object bands, and velocity updates
    /// while leaving the already-run sound cadence to the audio phase.
    pub fn run_normal_live_irq_video_frame(&mut self) -> Result<RedLabelLiveVideoFrame, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "XXX1", 0xFF)?;
        self.write_field_byte(&layout, "base_page", "XXX3", 0)?;
        let schedule = RedLabelLiveIrqFrameSchedule::for_mode(RedLabelIrqMode::Normal);

        let context =
            RedLabelIrqSchedulerContext::source_irq_after_sound_step(DefenderInputPorts::EMPTY);
        let upper_scanline = self.run_irq_scanline_object_phase_with_context(
            RedLabelIrqMode::Normal,
            schedule.upper_vertical_counter,
            context,
        )?;
        let player_motion =
            Self::live_player_motion_from_pre_tail_steps(&upper_scanline.pre_tail_steps)?;
        let star_output =
            Self::live_star_output_from_pre_tail_steps(&upper_scanline.pre_tail_steps)?;
        let upper_object_band = Self::live_object_band_from_scheduler_step(
            &upper_scanline,
            RedLabelIrqObjectBandPhase::NormalUpper,
        )?;

        let lower_scanline = self.run_irq_scanline_object_phase_with_context(
            RedLabelIrqMode::Normal,
            schedule.lower_vertical_counter,
            context,
        )?;
        let terrain_output =
            Self::live_terrain_output_from_pre_tail_steps(&lower_scanline.pre_tail_steps);
        let lower_object_band = Self::live_object_band_from_scheduler_step(
            &lower_scanline,
            RedLabelIrqObjectBandPhase::NormalLower,
        )?;

        Ok(RedLabelLiveVideoFrame {
            mode: RedLabelIrqMode::Normal,
            upper_scanline,
            lower_scanline,
            player_motion,
            star_output,
            upper_object_band,
            terrain_output,
            lower_object_band,
        })
    }

    /// Source-ordered live video slice for the flipped red-label `IRQB` frame
    /// work. `IRQB` draws the terrain and upper object band first, then runs
    /// `PLAYER` / `STOUT` before the lower object band and velocity update.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L2008-L2068>.
    pub fn run_inverted_live_irq_video_frame(&mut self) -> Result<RedLabelLiveVideoFrame, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "XXX1", 0xFF)?;
        self.write_field_byte(&layout, "base_page", "XXX3", 0)?;
        let schedule = RedLabelLiveIrqFrameSchedule::for_mode(RedLabelIrqMode::Inverted);

        let context =
            RedLabelIrqSchedulerContext::source_irq_after_sound_step(DefenderInputPorts::EMPTY);
        let upper_scanline = self.run_irq_scanline_object_phase_with_context(
            RedLabelIrqMode::Inverted,
            schedule.upper_vertical_counter,
            context,
        )?;
        let terrain_output =
            Self::live_terrain_output_from_pre_tail_steps(&upper_scanline.pre_tail_steps);
        let upper_object_band = Self::live_object_band_from_scheduler_step(
            &upper_scanline,
            RedLabelIrqObjectBandPhase::InvertedUpper,
        )?;

        let lower_scanline = self.run_irq_scanline_object_phase_with_context(
            RedLabelIrqMode::Inverted,
            schedule.lower_vertical_counter,
            context,
        )?;
        let player_motion =
            Self::live_player_motion_from_pre_tail_steps(&lower_scanline.pre_tail_steps)?;
        let star_output =
            Self::live_star_output_from_pre_tail_steps(&lower_scanline.pre_tail_steps)?;
        let lower_object_band = Self::live_object_band_from_scheduler_step(
            &lower_scanline,
            RedLabelIrqObjectBandPhase::InvertedLower,
        )?;

        Ok(RedLabelLiveVideoFrame {
            mode: RedLabelIrqMode::Inverted,
            upper_scanline,
            lower_scanline,
            player_motion,
            star_output,
            upper_object_band,
            terrain_output,
            lower_object_band,
        })
    }

    pub(super) fn live_player_motion_from_pre_tail_steps(
        steps: &[RedLabelIrqPreTailStep],
    ) -> Result<RedLabelPlayerMotion, String> {
        steps
            .iter()
            .find_map(|step| match step {
                RedLabelIrqPreTailStep::PlayerMotion(motion) => Some(*motion),
                _ => None,
            })
            .ok_or_else(|| String::from("live IRQ video step did not run PLAYER"))
    }

    pub(super) fn live_star_output_from_pre_tail_steps(
        steps: &[RedLabelIrqPreTailStep],
    ) -> Result<RedLabelStarOutput, String> {
        steps
            .iter()
            .find_map(|step| match step {
                RedLabelIrqPreTailStep::StarOutput(output) => Some(*output),
                _ => None,
            })
            .ok_or_else(|| String::from("live IRQ video step did not run STOUT"))
    }

    pub(super) fn live_terrain_output_from_pre_tail_steps(
        steps: &[RedLabelIrqPreTailStep],
    ) -> Option<RedLabelTerrainOutput> {
        steps.iter().find_map(|step| match step {
            RedLabelIrqPreTailStep::TerrainOutput(output) => Some(*output),
            _ => None,
        })
    }

    pub(super) fn live_object_band_from_scheduler_step(
        step: &RedLabelIrqSchedulerStep,
        expected_phase: RedLabelIrqObjectBandPhase,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        let object_band = step.object_band.clone().ok_or_else(|| {
            format!(
                "live IRQ video step {:?} did not run an object band",
                step.phase
            )
        })?;
        if object_band.phase != expected_phase {
            return Err(format!(
                "live IRQ video step {:?} ran {:?}, expected {:?}",
                step.phase, object_band.phase, expected_phase
            ));
        }
        Ok(object_band)
    }

    pub fn live_irq_mode(&self) -> Result<RedLabelIrqMode, String> {
        let layout = red_label_ram_layout()?;
        let irq_hook = field_range(&layout, "base_page", "IRQHK")?;
        let opcode = self.read_byte(irq_hook.start)?;
        let target = self.read_word(irq_hook.start + 1)?;
        if opcode == RED_LABEL_IRQ_JUMP_OPCODE && target == RED_LABEL_IRQB_ADDRESS {
            Ok(RedLabelIrqMode::Inverted)
        } else {
            Ok(RedLabelIrqMode::Normal)
        }
    }

    pub fn live_irq_frame_schedule(&self) -> Result<RedLabelLiveIrqFrameSchedule, String> {
        Ok(RedLabelLiveIrqFrameSchedule::for_mode(
            self.live_irq_mode()?,
        ))
    }

    pub fn run_live_irq_video_frame(&mut self) -> Result<RedLabelLiveVideoFrame, String> {
        match self.live_irq_mode()? {
            RedLabelIrqMode::Normal => self.run_normal_live_irq_video_frame(),
            RedLabelIrqMode::Inverted => self.run_inverted_live_irq_video_frame(),
        }
    }

    /// Source-shaped RAM-visible scanline gate for the red-label `IRQ` and
    /// `IRQB` object phases. This mutates `IFLG`, `TIMER`, `XXX2`, and the
    /// color-RAM palette copy, reports the source watchdog byte, runs
    /// translated `PLAYER` and `STOUT` where the source branches call them, and
    /// then runs only the already translated object-band tail reached by the
    /// source branch. When the caller supplies live cabinet input, the source
    /// `CSCAN` branch keeps its two-frame coin-door history and queues source
    /// switch processes; when the caller supplies the live 6809 stack pointer,
    /// the terrain branch can run translated `BGOUT`; otherwise it reports that
    /// `BGOUT` is due. The source `MAPC` clear/select/restore writes are
    /// captured in `hardware_map_writes` and leave hardware map selection
    /// restored from `MAPCR` at `IRQX`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1931-L2069>.
    pub fn run_irq_scanline_object_phase(
        &mut self,
        mode: RedLabelIrqMode,
        vertical_counter: u8,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        self.run_irq_scanline_object_phase_with_context(
            mode,
            vertical_counter,
            RedLabelIrqSchedulerContext::default(),
        )
    }

    pub fn run_irq_scanline_object_phase_with_context(
        &mut self,
        mode: RedLabelIrqMode,
        vertical_counter: u8,
        context: RedLabelIrqSchedulerContext,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        let layout = red_label_ram_layout()?;
        let previous_iflg = self.read_field_byte(&layout, "base_page", "IFLG")?;

        match mode {
            RedLabelIrqMode::Normal => self.run_normal_irq_scanline_object_phase(
                &layout,
                vertical_counter,
                previous_iflg,
                context,
            ),
            RedLabelIrqMode::Inverted => self.run_inverted_irq_scanline_object_phase(
                &layout,
                vertical_counter,
                previous_iflg,
                context,
            ),
        }
    }

    pub(super) fn run_normal_irq_scanline_object_phase(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        vertical_counter: u8,
        previous_iflg: u8,
        context: RedLabelIrqSchedulerContext,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        let (hardware_map_before, mut hardware_map_writes) = self.begin_irq_hardware_map_sequence();
        if vertical_counter >= RED_LABEL_NORMAL_IRQ_UPPER_THRESHOLD {
            if previous_iflg != 0 {
                let hardware_map_after =
                    self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
                return Ok(RedLabelIrqSchedulerStep {
                    mode: RedLabelIrqMode::Normal,
                    vertical_counter,
                    phase: RedLabelIrqSchedulerPhase::Idle,
                    hardware_map_before,
                    hardware_map_writes,
                    hardware_map_after,
                    previous_iflg,
                    iflg: previous_iflg,
                    timer_before: None,
                    timer_after: None,
                    watchdog_value: None,
                    palette_copy: None,
                    xxx2: None,
                    pre_tail_steps: Vec::new(),
                    object_band: None,
                });
            }

            self.write_field_byte(layout, "base_page", "IFLG", 1)?;
            let xxx2 = vertical_counter
                .wrapping_sub(8)
                .min(RED_LABEL_NORMAL_IRQ_MAX_XXX2);
            self.write_field_byte(layout, "base_page", "XXX2", xxx2)?;
            let pre_tail_steps = self.irq_sound_player_star_pre_tail_steps(context)?;
            self.write_hardware_map(2, &mut hardware_map_writes);
            let object_band = self.run_normal_irq_upper_object_band_pass()?;
            let hardware_map_after =
                self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
            return Ok(RedLabelIrqSchedulerStep {
                mode: RedLabelIrqMode::Normal,
                vertical_counter,
                phase: RedLabelIrqSchedulerPhase::NormalUpper,
                hardware_map_before,
                hardware_map_writes,
                hardware_map_after,
                previous_iflg,
                iflg: 1,
                timer_before: None,
                timer_after: None,
                watchdog_value: None,
                palette_copy: None,
                xxx2: Some(xxx2),
                pre_tail_steps,
                object_band: Some(object_band),
            });
        }

        if previous_iflg == 0 {
            let hardware_map_after =
                self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
            return Ok(RedLabelIrqSchedulerStep {
                mode: RedLabelIrqMode::Normal,
                vertical_counter,
                phase: RedLabelIrqSchedulerPhase::Idle,
                hardware_map_before,
                hardware_map_writes,
                hardware_map_after,
                previous_iflg,
                iflg: previous_iflg,
                timer_before: None,
                timer_after: None,
                watchdog_value: None,
                palette_copy: None,
                xxx2: None,
                pre_tail_steps: Vec::new(),
                object_band: None,
            });
        }

        self.write_field_byte(layout, "base_page", "IFLG", 0)?;
        let timer_before = self.read_field_byte(layout, "base_page", "TIMER")?;
        let timer_after = timer_before.wrapping_add(1);
        self.write_field_byte(layout, "base_page", "TIMER", timer_after)?;
        let palette_copy = if vertical_counter <= RED_LABEL_NORMAL_IRQ_PALETTE_COPY_LIMIT {
            Some(self.copy_color_mapping_to_palette_ram(layout)?)
        } else {
            None
        };
        self.write_hardware_map(7, &mut hardware_map_writes);
        let pre_tail_steps = self.irq_coin_terrain_pre_tail_steps(layout, context)?;
        self.write_hardware_map(2, &mut hardware_map_writes);
        let object_band = self.run_normal_irq_lower_object_band_pass()?;
        let hardware_map_after =
            self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
        Ok(RedLabelIrqSchedulerStep {
            mode: RedLabelIrqMode::Normal,
            vertical_counter,
            phase: RedLabelIrqSchedulerPhase::NormalLower,
            hardware_map_before,
            hardware_map_writes,
            hardware_map_after,
            previous_iflg,
            iflg: 0,
            timer_before: Some(timer_before),
            timer_after: Some(timer_after),
            watchdog_value: Some(RED_LABEL_NORMAL_WATCHDOG_DATA),
            palette_copy,
            xxx2: None,
            pre_tail_steps,
            object_band: Some(object_band),
        })
    }

    pub(super) fn run_inverted_irq_scanline_object_phase(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        vertical_counter: u8,
        previous_iflg: u8,
        context: RedLabelIrqSchedulerContext,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        let (hardware_map_before, mut hardware_map_writes) = self.begin_irq_hardware_map_sequence();
        if vertical_counter >= RED_LABEL_INVERTED_IRQ_UPPER_THRESHOLD {
            if previous_iflg != 0 {
                let hardware_map_after =
                    self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
                return Ok(RedLabelIrqSchedulerStep {
                    mode: RedLabelIrqMode::Inverted,
                    vertical_counter,
                    phase: RedLabelIrqSchedulerPhase::Idle,
                    hardware_map_before,
                    hardware_map_writes,
                    hardware_map_after,
                    previous_iflg,
                    iflg: previous_iflg,
                    timer_before: None,
                    timer_after: None,
                    watchdog_value: None,
                    palette_copy: None,
                    xxx2: None,
                    pre_tail_steps: Vec::new(),
                    object_band: None,
                });
            }

            self.write_field_byte(layout, "base_page", "IFLG", 1)?;
            let xxx2 = !vertical_counter;
            self.write_field_byte(layout, "base_page", "XXX2", xxx2)?;
            self.write_hardware_map(7, &mut hardware_map_writes);
            let pre_tail_steps = self.irq_coin_terrain_pre_tail_steps(layout, context)?;
            self.write_hardware_map(2, &mut hardware_map_writes);
            let object_band = self.run_inverted_irq_upper_object_band_pass()?;
            let hardware_map_after =
                self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
            return Ok(RedLabelIrqSchedulerStep {
                mode: RedLabelIrqMode::Inverted,
                vertical_counter,
                phase: RedLabelIrqSchedulerPhase::InvertedUpper,
                hardware_map_before,
                hardware_map_writes,
                hardware_map_after,
                previous_iflg,
                iflg: 1,
                timer_before: None,
                timer_after: None,
                watchdog_value: None,
                palette_copy: None,
                xxx2: Some(xxx2),
                pre_tail_steps,
                object_band: Some(object_band),
            });
        }

        if previous_iflg == 0 {
            let hardware_map_after =
                self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
            return Ok(RedLabelIrqSchedulerStep {
                mode: RedLabelIrqMode::Inverted,
                vertical_counter,
                phase: RedLabelIrqSchedulerPhase::Idle,
                hardware_map_before,
                hardware_map_writes,
                hardware_map_after,
                previous_iflg,
                iflg: previous_iflg,
                timer_before: None,
                timer_after: None,
                watchdog_value: None,
                palette_copy: None,
                xxx2: None,
                pre_tail_steps: Vec::new(),
                object_band: None,
            });
        }

        self.write_field_byte(layout, "base_page", "IFLG", 0)?;
        let timer_before = self.read_field_byte(layout, "base_page", "TIMER")?;
        let timer_after = timer_before.wrapping_add(1);
        self.write_field_byte(layout, "base_page", "TIMER", timer_after)?;
        let palette_copy = if vertical_counter <= RED_LABEL_INVERTED_IRQ_PALETTE_COPY_LIMIT {
            Some(self.copy_color_mapping_to_palette_ram(layout)?)
        } else {
            None
        };
        let pre_tail_steps = self.irq_sound_player_star_pre_tail_steps(context)?;
        self.write_hardware_map(2, &mut hardware_map_writes);
        let object_band = self.run_inverted_irq_lower_object_band_pass()?;
        let hardware_map_after =
            self.finish_irq_hardware_map_sequence(layout, &mut hardware_map_writes)?;
        Ok(RedLabelIrqSchedulerStep {
            mode: RedLabelIrqMode::Inverted,
            vertical_counter,
            phase: RedLabelIrqSchedulerPhase::InvertedLower,
            hardware_map_before,
            hardware_map_writes,
            hardware_map_after,
            previous_iflg,
            iflg: 0,
            timer_before: Some(timer_before),
            timer_after: Some(timer_after),
            watchdog_value: Some(RED_LABEL_INVERTED_WATCHDOG_DATA),
            palette_copy,
            xxx2: None,
            pre_tail_steps,
            object_band: Some(object_band),
        })
    }

    pub(super) fn irq_sound_player_star_pre_tail_steps(
        &mut self,
        context: RedLabelIrqSchedulerContext,
    ) -> Result<Vec<RedLabelIrqPreTailStep>, String> {
        let mut steps = Vec::with_capacity(if context.sound_sequence_already_stepped {
            2
        } else {
            3
        });
        if !context.sound_sequence_already_stepped {
            steps.push(RedLabelIrqPreTailStep::SoundSequence(
                self.step_sound_sequence()?,
            ));
        }
        steps.push(RedLabelIrqPreTailStep::PlayerMotion(
            self.update_player_motion_from_pia()?,
        ));
        steps.push(RedLabelIrqPreTailStep::StarOutput(self.output_stars()?));
        Ok(steps)
    }

    pub(super) fn irq_coin_terrain_pre_tail_steps(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        context: RedLabelIrqSchedulerContext,
    ) -> Result<Vec<RedLabelIrqPreTailStep>, String> {
        let mut steps = vec![RedLabelIrqPreTailStep::CoinScan(
            self.scan_translated_coin_switches(context.input_ports)?,
        )];
        let status = self.read_field_byte(layout, "base_page", "STATUS")?;
        if status & 0x02 == 0 {
            if let Some(stack_pointer) = context.terrain_stack_pointer {
                steps.push(RedLabelIrqPreTailStep::TerrainOutput(
                    self.output_terrain_from_bgl(stack_pointer)?,
                ));
            } else {
                steps.push(RedLabelIrqPreTailStep::TerrainOutputDue);
            }
        }
        Ok(steps)
    }

    pub(super) fn irq_object_band_bounds_from(
        &self,
        first_byte_field: &str,
    ) -> Result<(u8, u8), String> {
        let layout = red_label_ram_layout()?;
        let start = field_range(&layout, "base_page", first_byte_field)?.start;
        Ok((self.read_byte(start)?, self.read_byte(start + 1)?))
    }

    /// Applies the visible player-table writes made by the red-label `START`
    /// path before the player process bodies are translated.
    pub fn start_one_player_tables(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let player_table = table_descriptor(&layout, "player")?;
        let player_range = player_table.table_range().ok_or_else(|| {
            String::from("red-label player table range overflows main RAM address space")
        })?;
        self.clear_range(player_range)?;

        let nship = self.read_cmos_byte_by_symbol("NSHIP")?;
        let replay_word = self.read_cmos_word_by_symbol("REPLAY")?;
        let replay = replay_word.to_be_bytes();
        self.write_field_byte(&layout, "base_page", "CURPLR", 1)?;
        self.write_field_byte(&layout, "base_page", "SCRFLG", 0)?;
        self.write_field_byte(&layout, "base_page", "PLRCNT", 1)?;
        self.write_field_word(&layout, "base_page", "PLRX", player_table.base)?;
        self.write_field_byte(&layout, "base_page", "SBFLG", 0)?;
        self.write_field_word(&layout, "base_page", "REPLA", replay_word)?;

        self.write_player_entry_start_defaults(&layout, 0, nship, replay.as_slice())?;
        let first_player = table_entry_range(player_table, 0)?;
        self.get_new_wave_parameters_for_player_address(first_player.start)?;
        let second_player = table_entry_range(player_table, 1)?;
        let source = self
            .ram_range(first_player)
            .ok_or_else(|| String::from("red-label player 1 table range is outside RAM"))?
            .to_vec();
        self.write_range(second_player, &source)?;
        self.initialize_one_player_runtime_state(&layout, player_table)?;
        Ok(())
    }

    /// Source-shaped `FPLAY`: read the `FREEPL` CMOS adjustment and seed two
    /// credits only when the adjustment byte is exactly one. Paid-credit start
    /// handling remains in the separate `ST1`/`ST2` bodies.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1090-L1097>.
    pub fn apply_free_play_credit(&mut self) -> Result<RedLabelFreePlayCredit, String> {
        self.apply_free_play_credit_value(self.read_cmos_byte_by_symbol("FREEPL")?)
    }

    pub(super) fn apply_free_play_credit_value(
        &mut self,
        free_play: u8,
    ) -> Result<RedLabelFreePlayCredit, String> {
        let layout = red_label_ram_layout()?;
        let credit_before = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        if free_play.wrapping_sub(1) == 0 {
            self.write_field_byte(&layout, "base_page", "CREDIT", 2)?;
        }
        let credit_after = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        Ok(RedLabelFreePlayCredit {
            free_play,
            credit_before,
            credit_after,
        })
    }

    /// Source-shaped visible `START2` credit tail: increment `PLRCNT`, subtract
    /// one BCD credit with the source `ADDA #$99` / `DAA` sequence, and write
    /// the packed CMOS credit backup through the source `WCMOSA CREDST` path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1163-L1175>.
    pub fn advance_start_credit_tail(&mut self) -> Result<RedLabelStartCredit, String> {
        let layout = red_label_ram_layout()?;
        let player_count_before = self.read_field_byte(&layout, "base_page", "PLRCNT")?;
        let player_count_after = player_count_before.wrapping_add(1);
        self.write_field_byte(&layout, "base_page", "PLRCNT", player_count_after)?;

        let credit_before = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        let (credit_after, _) = bcd_add_byte(credit_before, 0x99, false);
        self.write_field_byte(&layout, "base_page", "CREDIT", credit_after)?;
        let credit_cmos_backup = self.write_cmos_byte_by_symbol("CREDST", credit_after)?;
        let top_display_required = player_count_after.wrapping_sub(1) != 0;
        if top_display_required {
            self.top_display()?;
        }

        Ok(RedLabelStartCredit {
            player_count_before,
            player_count_after,
            credit_before,
            credit_after,
            credit_cmos_backup,
            top_display_required,
        })
    }

    /// Source-shaped `START` body through the `START2` credit tail. `SCRCLR`,
    /// `WCMOSA CREDST`, and the conditional `TDISP` top-display redraw are
    /// translated here.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1124-L1175>.
    pub fn start_game_from_credit(&mut self) -> Result<RedLabelStartGame, String> {
        let layout = red_label_ram_layout()?;
        self.write_field_byte(&layout, "base_page", "CUNITS", 0)?;

        let power_flag = self.read_field_byte(&layout, "base_page", "PWRFLG")?;
        if power_flag == 0 {
            return Ok(RedLabelStartGame::PowerPageBlocked { power_flag });
        }

        let status_before = self.read_field_byte(&layout, "base_page", "STATUS")?;
        let initialized = if status_before & 0x80 != 0 {
            Some(self.initialize_start_game_tables(&layout)?)
        } else {
            None
        };
        let credit = self.advance_start_credit_tail()?;

        Ok(RedLabelStartGame::Updated {
            power_flag,
            status_before,
            initialized,
            credit,
        })
    }

    /// Source-shaped `ST1`: require attract/game-over status, apply `FPLAY`,
    /// load `ST1SND`, run `START`, then execute the `DIE` macro.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1100-L1108>.
    pub fn dispatch_start_one_current_process(&mut self) -> Result<RedLabelStartSwitch, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            return Ok(RedLabelStartSwitch::StatusBlocked {
                players: 1,
                status,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let free_play = self.apply_free_play_credit()?;
        let credit = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        if credit == 0 {
            return Ok(RedLabelStartSwitch::InsufficientCredit {
                players: 1,
                free_play,
                credit,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let sound_loaded = self.load_sound_table_by_label("ST1SND")?;
        let start = self.start_game_from_credit()?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelStartSwitch::StartedOne {
            free_play,
            sound_loaded,
            start,
            killed_process,
        })
    }

    /// Source-shaped `ST2`: require two credits, run `START` once, load
    /// `ST2SND`, run the shared `ST09` sound/start tail, then `DIE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1112-L1120>.
    pub fn dispatch_start_two_current_process(&mut self) -> Result<RedLabelStartSwitch, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            return Ok(RedLabelStartSwitch::StatusBlocked {
                players: 2,
                status,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let free_play = self.apply_free_play_credit()?;
        let credit = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        if credit < 2 {
            return Ok(RedLabelStartSwitch::InsufficientCredit {
                players: 2,
                free_play,
                credit,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let first_start = self.start_game_from_credit()?;
        let sound_loaded = self.load_sound_table_by_label("ST2SND")?;
        let second_start = self.start_game_from_credit()?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelStartSwitch::StartedTwo {
            free_play,
            first_start,
            sound_loaded,
            second_start,
            killed_process,
        })
    }

    /// Source-shaped `HSRES`: only run in attract/game-over, reset today's
    /// high scores, increment the aliased `HSRFLG` byte, then `SUCIDE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1081-L1087>.
    pub fn dispatch_high_score_reset_current_process(
        &mut self,
    ) -> Result<RedLabelAdminSwitch, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            return Ok(RedLabelAdminSwitch::HighScoreResetStatusBlocked {
                status,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let map_after = 3;
        self.write_field_byte(&layout, "base_page", "MAPCR", map_after)?;
        let defaults = red_label_cmos_defaults()?;
        self.apply_todays_high_score_defaults(&defaults)?;
        let hsrflg_address = RED_LABEL_HOF_RESET_FLAG_RAM;
        let hsrflg_before = self.read_byte(hsrflg_address)?;
        let hsrflg_after = hsrflg_before.wrapping_add(1);
        self.write_byte(hsrflg_address, hsrflg_after)?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelAdminSwitch::HighScoreReset {
            status,
            map_after,
            hsrflg_before,
            hsrflg_after,
            killed_process,
        })
    }

    /// Source-shaped `ADVSW`: only run in attract/game-over, select the manual
    /// diagnostic or automatic audit vector from `PIA01` bit 0, and retire the
    /// cabinet switch process after recording the target.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L1091-L1102>.
    pub fn dispatch_advance_switch_current_process(
        &mut self,
    ) -> Result<RedLabelAdminSwitch, String> {
        let layout = red_label_ram_layout()?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            return Ok(RedLabelAdminSwitch::AdvanceStatusBlocked {
                status,
                killed_process: self.kill_current_process(&layout)?,
            });
        }

        let map_after = 3;
        self.write_field_byte(&layout, "base_page", "MAPCR", map_after)?;
        let pia01 = self.read_field_byte(&layout, "base_page", "PIA01")?;
        let target = if pia01 & 0x01 == 0 {
            RedLabelAdvanceSwitchTarget::Diagnostics
        } else {
            RedLabelAdvanceSwitchTarget::Audits
        };
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelAdminSwitch::AdvanceJump {
            status,
            pia01,
            target,
            map_after,
            killed_process,
        })
    }

    pub(super) fn apply_coin_slot_credit(
        &mut self,
        slot: RedLabelCoinSlot,
    ) -> Result<RedLabelCoinCredit, String> {
        let layout = red_label_ram_layout()?;
        let slot_audit = self.add_bcd_cmos_word_by_symbol(slot.audit_symbol(), 0x01)?;
        let multiplier_units =
            bcd_byte_to_u16(self.read_cmos_byte_by_symbol(slot.multiplier_symbol())?) as u8;
        let minimum_units = bcd_byte_to_u16(self.read_cmos_byte_by_symbol("MINUNT")?) as u8;
        let units_per_credit = bcd_byte_to_u16(self.read_cmos_byte_by_symbol("CUNITC")?) as u8;
        let bonus_units_per_credit =
            bcd_byte_to_u16(self.read_cmos_byte_by_symbol("CUNITB")?) as u8;

        let bunits_before = self.read_field_byte(&layout, "base_page", "BUNITS")?;
        let cunits_before = self.read_field_byte(&layout, "base_page", "CUNITS")?;
        let mut bunits_after = bunits_before.wrapping_add(multiplier_units);
        let mut cunits_after = cunits_before.wrapping_add(multiplier_units);
        self.write_field_byte(&layout, "base_page", "BUNITS", bunits_after)?;
        self.write_field_byte(&layout, "base_page", "CUNITS", cunits_after)?;

        let mut paid_credit_audit = None;
        let mut paid_credits = 0;
        let mut bonus_credits = 0;
        let mut credits_awarded = 0;
        let credit_before = self.read_field_byte(&layout, "base_page", "CREDIT")?;
        let mut credit_after = credit_before;
        let mut credit_cmos_backup = None;

        if cunits_after >= minimum_units {
            let (credits, remainder) = red_label_divide_coin_units(cunits_after, units_per_credit);
            paid_credits = credits;
            cunits_after = remainder;
            self.write_field_byte(&layout, "base_page", "CUNITS", cunits_after)?;

            let (bonus, _) = red_label_divide_coin_units(bunits_after, bonus_units_per_credit);
            bonus_credits = bonus;
            if bonus_credits != 0 {
                cunits_after = 0;
                bunits_after = 0;
                self.write_field_byte(&layout, "base_page", "CUNITS", 0)?;
                self.write_field_byte(&layout, "base_page", "BUNITS", 0)?;
            }

            credits_awarded = bcd_add_byte(bonus_credits, paid_credits, false).0;
            paid_credit_audit = Some(self.add_bcd_cmos_word_by_symbol("TOTPDC", credits_awarded)?);
            let (added_credit, carry) = bcd_add_byte(credit_before, credits_awarded, false);
            credit_after = if carry { 0x99 } else { added_credit };
            self.write_field_byte(&layout, "base_page", "CREDIT", credit_after)?;
            credit_cmos_backup = Some(self.write_cmos_byte_by_symbol("CREDST", credit_after)?);
        }

        Ok(RedLabelCoinCredit {
            slot,
            slot_audit,
            paid_credit_audit,
            multiplier_units,
            minimum_units,
            units_per_credit,
            bonus_units_per_credit,
            cunits_before,
            cunits_after,
            bunits_before,
            bunits_after,
            paid_credits,
            bonus_credits,
            credits_awarded,
            credit_before,
            credit_after,
            credit_cmos_backup,
        })
    }

    pub(super) fn add_bcd_cmos_word_by_symbol(
        &mut self,
        symbol: &'static str,
        addend: u8,
    ) -> Result<RedLabelCmosWordWrite, String> {
        let current = self.read_cmos_word_by_symbol(symbol)?;
        let [high, low] = current.to_be_bytes();
        let (new_low, carry) = bcd_add_byte(low, addend, false);
        let (new_high, _) = bcd_add_byte(high, 0, carry);
        self.write_cmos_word_by_symbol(symbol, u16::from_be_bytes([new_high, new_low]))
    }

    /// Source-shaped `BLKCLR`: clear a screen-format rectangular block at X
    /// with D carrying width in A and height in B.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1411-L1418>.
    pub fn block_clear(
        &mut self,
        screen_address: u16,
        width: u8,
        height: u8,
    ) -> Result<RedLabelBlockClear, String> {
        self.clear_screen_block(screen_address, width, height)?;
        Ok(RedLabelBlockClear {
            screen_address,
            width,
            height,
        })
    }

    /// Source-shaped `CWRIT`: switch character map 2 in, copy the primary
    /// picture image described by Y to screen address D, then restore `MAPCR`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1317-L1366>.
    pub fn write_object_picture_cwrit(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<RedLabelPictureWrite, String> {
        let layout = red_label_ram_layout()?;
        let previous_map = self.read_field_byte(&layout, "base_page", "MAPCR")?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let result = (|| {
            let picture = red_label_object_picture(picture_address)?;
            self.write_object_picture_image(screen_address, picture, picture.primary_image)
        })();
        self.write_field_byte(&layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    /// Source-shaped `COFF`: switch character map 2 in, clear the descriptor
    /// footprint at screen address D, then restore `MAPCR`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1368-L1409>.
    pub fn erase_object_picture_coff(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<RedLabelBlockClear, String> {
        let layout = red_label_ram_layout()?;
        let previous_map = self.read_field_byte(&layout, "base_page", "MAPCR")?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let result = (|| {
            let picture = red_label_object_picture(picture_address)?;
            self.clear_screen_block(screen_address, picture.width, picture.height)?;
            Ok(RedLabelBlockClear {
                screen_address,
                width: picture.width,
                height: picture.height,
            })
        })();
        self.write_field_byte(&layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    /// Source-shaped object descriptor output dispatch: run the routine pointer
    /// from `OBJOUT,Y`, using carry-set flavor selection for the ONxx routines.
    /// `DRTS` is preserved as the null-object no-op.
    pub fn output_object_picture_by_descriptor(
        &mut self,
        screen_address: u16,
        picture_address: u16,
        alternate_flavor: bool,
    ) -> Result<Option<RedLabelPictureWrite>, String> {
        let picture = red_label_object_picture(picture_address)?;
        let routine_address = picture.output_routine.ok_or_else(|| {
            format!(
                "red-label picture `{}` has no descriptor output routine",
                picture.label
            )
        })?;
        if routine_address == red_label_routine_address("DRTS")? {
            return Ok(None);
        }
        if routine_address == red_label_routine_address("CWRIT")? {
            return self
                .write_object_picture_cwrit(screen_address, picture_address)
                .map(Some);
        }
        if !red_label_routine_address_matches_any(
            routine_address,
            &RED_LABEL_OBJECT_OUTPUT_ROUTINES,
        )? {
            return Err(format!(
                "red-label picture `{}` uses unimplemented output routine 0x{routine_address:04X}",
                picture.label
            ));
        }

        let image_address = if alternate_flavor {
            picture.alternate_image.ok_or_else(|| {
                format!(
                    "red-label picture `{}` has no alternate image for output routine 0x{routine_address:04X}",
                    picture.label
                )
            })?
        } else {
            picture.primary_image
        };
        self.write_object_picture_image(screen_address, picture, image_address)
            .map(Some)
    }

    /// Source-shaped object descriptor erase dispatch: run the routine pointer
    /// from `OBJDEL,Y`. ON/OFF picture erasers depend on the caller-selected
    /// character map; `COFF` performs its own map save/restore.
    pub fn erase_object_picture_by_descriptor(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<Option<RedLabelBlockClear>, String> {
        let picture = red_label_object_picture(picture_address)?;
        let routine_address = picture.erase_routine.ok_or_else(|| {
            format!(
                "red-label picture `{}` has no descriptor erase routine",
                picture.label
            )
        })?;
        if routine_address == red_label_routine_address("DRTS")? {
            return Ok(None);
        }
        if routine_address == red_label_routine_address("COFF")? {
            return self
                .erase_object_picture_coff(screen_address, picture_address)
                .map(Some);
        }
        if !red_label_routine_address_matches_any(
            routine_address,
            &RED_LABEL_OBJECT_ERASE_ROUTINES,
        )? {
            return Err(format!(
                "red-label picture `{}` uses unimplemented erase routine 0x{routine_address:04X}",
                picture.label
            ));
        }

        let erase_height = if red_label_routine_address_matches_any(
            routine_address,
            &["OFF28", "OFF48", "OFF58"],
        )? {
            picture.height.saturating_add(1)
        } else {
            picture.height
        };
        self.clear_screen_block(screen_address, picture.width, erase_height)?;
        Ok(Some(RedLabelBlockClear {
            screen_address,
            width: picture.width,
            height: erase_height,
        }))
    }

    /// Source-shaped `TDISP`: clear and redraw the scanner/top display,
    /// redraw laser and smart-bomb stocks, then transfer visible score digits.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L862-L920>.
    pub fn top_display(&mut self) -> Result<RedLabelTopDisplay, String> {
        let layout = red_label_ram_layout()?;
        let player_count = self.read_field_byte(&layout, "base_page", "PLRCNT")?;
        if player_count == 0 {
            return Err(String::from("red-label TDISP requires at least one player"));
        }

        let scanner_clear = self.block_clear(RED_LABEL_SCANNER_ADDRESS, 0x40, 0x20)?;
        let border = self.draw_top_display_border()?;
        let laser_displays = self.display_laser_stocks(&layout)?;
        let smart_bomb_displays = self.display_smart_bomb_stocks(&layout)?;

        let mut score_transfers = Vec::with_capacity(usize::from(player_count));
        let mut player_number = player_count;
        loop {
            score_transfers.push(self.transfer_score_digits(&layout, player_number)?);
            player_number = player_number.wrapping_sub(1);
            if player_number == 0 {
                break;
            }
        }

        Ok(RedLabelTopDisplay {
            scanner_clear,
            border,
            laser_displays,
            smart_bomb_displays,
            score_transfers,
        })
    }

    pub(super) fn display_laser_stocks(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Vec<RedLabelStockDisplay>, String> {
        let player_count = self.read_field_byte(layout, "base_page", "PLRCNT")?;
        let player_table = table_descriptor(layout, "player")?;
        let mut displays = Vec::with_capacity(2);
        displays.push(self.display_stock_icons(
            layout,
            player_table,
            StockDisplaySpec {
                player_number: 1,
                screen_address: RED_LABEL_P1_LASER_DISPLAY,
                stock_field: "PLAS",
                display_limit: 5,
                picture_address: red_label_object_picture_address("PLAMIN")?,
                step: StockIconStep::Horizontal(6),
                clear_size: (0x20, 0x06),
            },
        )?);
        if player_count.wrapping_sub(1) != 0 {
            displays.push(self.display_stock_icons(
                layout,
                player_table,
                StockDisplaySpec {
                    player_number: 2,
                    screen_address: RED_LABEL_P2_LASER_DISPLAY,
                    stock_field: "PLAS",
                    display_limit: 5,
                    picture_address: red_label_object_picture_address("PLAMIN")?,
                    step: StockIconStep::Horizontal(6),
                    clear_size: (0x20, 0x06),
                },
            )?);
        }
        Ok(displays)
    }

    pub(super) fn display_smart_bomb_stocks(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Vec<RedLabelStockDisplay>, String> {
        let player_count = self.read_field_byte(layout, "base_page", "PLRCNT")?;
        let player_table = table_descriptor(layout, "player")?;
        let mut displays = Vec::with_capacity(2);
        displays.push(self.display_stock_icons(
            layout,
            player_table,
            StockDisplaySpec {
                player_number: 1,
                screen_address: RED_LABEL_P1_SMART_BOMB_DISPLAY,
                stock_field: "PSBC",
                display_limit: 3,
                picture_address: red_label_object_picture_address("SBPIC")?,
                step: StockIconStep::Vertical(4),
                clear_size: (0x03, 0x0B),
            },
        )?);
        if player_count.wrapping_sub(1) != 0 {
            displays.push(self.display_stock_icons(
                layout,
                player_table,
                StockDisplaySpec {
                    player_number: 2,
                    screen_address: RED_LABEL_P2_SMART_BOMB_DISPLAY,
                    stock_field: "PSBC",
                    display_limit: 3,
                    picture_address: red_label_object_picture_address("SBPIC")?,
                    step: StockIconStep::Vertical(4),
                    clear_size: (0x03, 0x0B),
                },
            )?);
        }
        Ok(displays)
    }

    pub(super) fn display_stock_icons(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_table: &RedLabelRamLayoutEntry,
        spec: StockDisplaySpec,
    ) -> Result<RedLabelStockDisplay, String> {
        let player_index = u16::from(spec.player_number.checked_sub(1).ok_or_else(|| {
            format!(
                "red-label stock display player {} is invalid",
                spec.player_number
            )
        })?);
        let stock_range = player_table
            .field_range_for_entry(player_index)
            .ok_or_else(|| {
                format!(
                    "red-label player table has no player {}",
                    spec.player_number
                )
            })?
            .start
            + ram_field(layout, "player", spec.stock_field)?.offset;
        let available = self.read_byte(stock_range)?;
        let displayed = available.min(spec.display_limit);
        let block_clear =
            self.block_clear(spec.screen_address, spec.clear_size.0, spec.clear_size.1)?;
        let mut icons = Vec::with_capacity(usize::from(displayed));
        let mut icon_address = spec.screen_address;
        for _ in 0..displayed {
            icons.push(self.write_object_picture_primary(icon_address, spec.picture_address)?);
            icon_address = spec.step.apply(icon_address);
        }
        Ok(RedLabelStockDisplay {
            player_number: spec.player_number,
            available,
            displayed,
            block_clear,
            icons,
        })
    }

    pub(super) fn transfer_score_digits(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_number: u8,
    ) -> Result<RedLabelScoreTransfer, String> {
        let player_index = u16::from(
            player_number
                .checked_sub(1)
                .ok_or_else(|| format!("red-label SCRTR0 player {player_number} is invalid"))?,
        );
        let player_table = table_descriptor(layout, "player")?;
        if player_index >= player_table.entries {
            return Err(format!(
                "red-label SCRTR0 player {player_number} exceeds player table"
            ));
        }
        let score_field = ram_field(layout, "player", "PSCOR")?;
        let score_range = score_field
            .field_range_for_entry(player_index)
            .ok_or_else(|| {
                format!("red-label PSCOR range for player {player_number} is invalid")
            })?;
        let mut score_address = score_range.start + 1;
        let display_address = match player_number {
            1 => RED_LABEL_P1_SCORE_DISPLAY,
            2 => RED_LABEL_P2_SCORE_DISPLAY,
            _ => {
                return Err(format!(
                    "red-label SCRTR0 has no display address for player {player_number}"
                ));
            }
        };
        let previous_map = self.read_field_byte(layout, "base_page", "MAPCR")?;
        self.write_field_byte(layout, "base_page", "MAPCR", 2)?;
        let result = (|| {
            let mut blanking_started = false;
            let mut current_display_address = display_address;
            let mut digits = Vec::with_capacity(6);
            let mut counter = 6u8;
            while counter != 0 {
                let mut score_byte = self.read_byte(score_address)?;
                if counter & 1 == 0 {
                    score_byte >>= 4;
                } else {
                    score_address = score_address.wrapping_add(1);
                }
                let digit = score_byte & 0x0F;
                let transfer = if digit == 0 && counter > 2 && !blanking_started {
                    self.clear_score_digit_picture(current_display_address)?;
                    RedLabelScoreDigitTransfer {
                        digit_index: 6 - counter,
                        screen_address: current_display_address,
                        digit: None,
                        picture_address: None,
                    }
                } else {
                    blanking_started = true;
                    let image = red_label_score_digit_image(digit)?;
                    self.write_score_digit_image(current_display_address, image)?;
                    RedLabelScoreDigitTransfer {
                        digit_index: 6 - counter,
                        screen_address: current_display_address,
                        digit: Some(digit),
                        picture_address: Some(image.address),
                    }
                };
                digits.push(transfer);
                current_display_address =
                    screen_offset(current_display_address, RED_LABEL_SCORE_DIGIT_LENGTH)?;
                counter = counter.wrapping_sub(1);
            }
            Ok(RedLabelScoreTransfer {
                player_number,
                display_address,
                score_address: score_range.start + 1,
                digits,
            })
        })();
        self.write_field_byte(layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    pub(super) fn draw_top_display_border(&mut self) -> Result<RedLabelBorder, String> {
        let mut bottom_line_words = 0u16;
        let mut address = u16::from(RED_LABEL_SCANNER_HEIGHT) + 0x20;
        while address < RED_LABEL_SCREEN_CLEAR_END {
            self.write_word(address, 0x5555)?;
            bottom_line_words = bottom_line_words.wrapping_add(1);
            address = screen_offset(address, 0x0100)?;
        }

        let mut side_boundary_words = 0u8;
        address = RED_LABEL_SCANNER_ADDRESS - 0x0100;
        while address != RED_LABEL_SCANNER_ADDRESS - 0x0100 + 0x20 {
            self.write_word(screen_offset(address, 0x4100)?, 0x5555)?;
            self.write_word(address, 0x5555)?;
            side_boundary_words = side_boundary_words.wrapping_add(2);
            address = screen_offset(address, 2)?;
        }

        let mut top_boundary_bytes = 0u16;
        address = RED_LABEL_SCANNER_ADDRESS - 0x0101;
        let top_boundary_end = RED_LABEL_SCANNER_ADDRESS - 1 + 0x4100;
        while address != top_boundary_end {
            self.write_byte(address, 0x55)?;
            top_boundary_bytes = top_boundary_bytes.wrapping_add(1);
            address = screen_offset(address, 0x0100)?;
        }

        let mut scanner_marker_words = 0u8;
        address = 0x4C00 + u16::from(RED_LABEL_SCANNER_HEIGHT) - 1;
        let marker_end = 0x5400 + u16::from(RED_LABEL_SCANNER_HEIGHT) - 1;
        while address != marker_end {
            self.write_word(address, 0x9999)?;
            self.write_word(screen_offset(address, 0x21)?, 0x9999)?;
            scanner_marker_words = scanner_marker_words.wrapping_add(2);
            address = screen_offset(address, 0x0100)?;
        }

        Ok(RedLabelBorder {
            bottom_line_words,
            side_boundary_words,
            top_boundary_bytes,
            scanner_marker_words,
        })
    }

    /// Source-shaped visible `EXEC0` / `EXEC1` pre-dispatch slice: clear
    /// `TIMER`, update overload throttling, optionally demote one regular
    /// active object to `IPTR`, select map 2, run translated `COLCHK`, call
    /// translated `XUVCT` / `EXPU`, advance `RAND`, and drain queued `SWPROC`
    /// entries through `SWP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L3044-L3124>.
    pub fn run_exec_pre_dispatch_visible_slice(
        &mut self,
    ) -> Result<RedLabelExecPreDispatch, String> {
        let layout = red_label_ram_layout()?;
        let timer_before = self.read_field_byte(&layout, "base_page", "TIMER")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")?;
        let overload_counter_before = self.read_field_byte(&layout, "base_page", "OVCNT")?;

        let mut overload_counter_raw = None;
        let mut star_count_after = None;
        let mut overloaded_object = None;
        let overload_counter_after = if status & 0x7D != 0 {
            self.write_field_byte(&layout, "base_page", "OVCNT", 0)?;
            0
        } else {
            let raw = timer_before
                .wrapping_shl(1)
                .wrapping_add(overload_counter_before)
                .wrapping_sub(4);
            overload_counter_raw = Some(raw);
            let mut counter = if raw & 0x80 == 0 { raw } else { 0 };
            self.write_field_byte(&layout, "base_page", "OVCNT", counter)?;
            if counter >= 2 {
                self.write_field_byte(&layout, "base_page", "STRCNT", 3)?;
                star_count_after = Some(3);
                if counter > 2 {
                    counter = 2;
                    self.write_field_byte(&layout, "base_page", "OVCNT", counter)?;
                    overloaded_object =
                        self.move_first_exec_overload_object_to_inactive(&layout)?;
                }
            }
            counter
        };

        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let player_collision = self.check_player_collision()?;
        self.advance_active_object_velocities()?;
        let expanded_updates = self.update_expanded_objects()?;
        let rand_state = self.advance_red_label_rand(&layout)?;
        let switch_processes = self.dispatch_switch_processes()?;

        Ok(RedLabelExecPreDispatch {
            timer_before,
            status,
            overload_counter_before,
            overload_counter_raw,
            overload_counter_after,
            star_count_after,
            overloaded_object,
            map_after: 2,
            player_collision,
            expanded_updates,
            rand_state,
            switch_processes,
        })
    }

    pub(super) fn run_trace_power_on_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0..RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_DRAWN_SLOTS {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_PARTIAL_SLOT
                .wrapping_mul(table.entry_size),
        );
        let size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.write_expanded_slot(&layout, partial_slot, false)?;
        let slot_end = partial_slot.wrapping_add(table.entry_size);
        let skipped_erase_start = partial_slot + 0x28;
        let skipped_erase_end = skipped_erase_start
            + RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_ERASE_SKIP_BYTES;
        let mut skipped_erase = skipped_erase_start;
        while skipped_erase < skipped_erase_end {
            let screen_address = self.read_word(skipped_erase)?;
            self.write_word(screen_address, 0)?;
            skipped_erase = skipped_erase.wrapping_add(2);
        }
        self.write_appearance_word(&layout, partial_slot, "ERASES", slot_end)?;
        self.write_range(
            skipped_erase_start..skipped_erase_end,
            &[0; RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_ERASE_SKIP_BYTES as usize],
        )?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(table.entry_size);
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 2u16..12 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(12u16.wrapping_mul(table.entry_size));
        let size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        for (erase_address, screen_address) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTH_APPEARANCE_INACTIVE_ERASE_WORDS
        {
            self.write_word(erase_address, screen_address)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..7 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        let resumed_slot = table
            .base
            .wrapping_add(12u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 13u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        for screen_address in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTH_APPEARANCE_ERASED_WORDS {
            self.write_word(screen_address, 0)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..2 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(2u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 8u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        for (erase_address, screen_address) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTH_APPEARANCE_INACTIVE_ERASE_WORDS
        {
            self.write_word(erase_address, screen_address)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTH_APPEARANCE_FINAL_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(2u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 3u16..13 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        for (erase_address, screen_address) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINTH_APPEARANCE_INACTIVE_ERASE_WORDS
        {
            self.write_word(erase_address, screen_address)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_tenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..7 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.write_appearance_word(
            &layout,
            partial_slot,
            "ERASES",
            partial_slot.wrapping_add(table.entry_size),
        )?;

        let resumed_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, false)?;

        let final_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let final_size = self.read_appearance_word(&layout, final_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, final_slot, final_size)?;
        self.erase_expanded_slot(&layout, final_slot)?;
        self.write_expanded_slot(&layout, final_slot, false)?;
        self.apply_trace_power_on_tenth_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_tenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_TENTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, byte) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_TENTH_APPEARANCE_MID_WRITE_BYTES
        {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_eleventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 8u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_twelfth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        self.apply_trace_power_on_twelfth_defender_appearance_video_boundary()?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_twelfth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, value)?;
        }
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_MID_WRITE_WORDS {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_MID_WRITE_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, value)?;
        }

        for entry_index in 5u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_thirteenth_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_MID_WRITE_WORDS {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_MID_WRITE_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fourteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..9 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(9u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_fourteenth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fourteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTEENTH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTEENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        let resumed_slot = table.base.wrapping_add(9u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 10u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_fifteenth_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTEENTH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_sixteenth_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTEENTH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTEENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_seventeenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..9 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(9u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_seventeenth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventeenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTEENTH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTEENTH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_eighteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        let resumed_slot = table.base.wrapping_add(9u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 10u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_eighteenth_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTEENTH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTEENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_nineteenth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_nineteenth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_nineteenth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETEENTH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twentieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        let first_size = self.read_appearance_word(&layout, first_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, first_slot, first_size)?;

        let partial_slot = table.base.wrapping_add(table.entry_size);
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_twentieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTIETH_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(table.entry_size);
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 2u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.write_expanded_slot(&layout, partial_slot, false)?;
        self.write_appearance_word(
            &layout,
            partial_slot,
            "ERASES",
            partial_slot.wrapping_add(table.entry_size),
        )?;
        self.apply_trace_power_on_twenty_first_defender_appearance_video_boundary()?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_first_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIRST_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIRST_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..8 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_twenty_second_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_second_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SECOND_APPEARANCE_RAM_WORDS
        {
            self.write_word(address, word)?;
        }
        for (screen_address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SECOND_VIDEO_WORDS {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        let first_size = self.read_appearance_word(&layout, first_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, first_slot, first_size)?;

        let partial_slot = table.base.wrapping_add(table.entry_size);
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 8u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_twenty_third_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for screen_address in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_THIRD_ZERO_VIDEO_BYTES {
            self.write_byte(screen_address, 0)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(table.entry_size);
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 2u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_twenty_fourth_defender_appearance_video_boundary()?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FOURTH_APPEARANCE_RAM_WORDS
        {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FOURTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..7 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_twenty_fifth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIFTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        let first_size = self.read_appearance_word(&layout, first_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, first_slot, first_size)?;

        let partial_slot = table.base.wrapping_add(table.entry_size);
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 8u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_twenty_sixth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_sixth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SIXTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(table.entry_size);
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 2u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_twenty_seventh_defender_appearance_video_boundary()?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SEVENTH_APPEARANCE_RAM_WORDS
        {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SEVENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_twenty_eighth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_eighth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_EIGHTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_twenty_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..8 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(8u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_twenty_ninth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_twenty_ninth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_NINTH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_NINTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirtieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        let first_size = self.read_appearance_word(&layout, first_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, first_slot, first_size)?;

        let partial_slot = table.base.wrapping_add(table.entry_size);
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table.base.wrapping_add(8u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 9u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_thirtieth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirtieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTIETH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTIETH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(table.entry_size);
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 2u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..14 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_thirty_third_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_THIRD_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_THIRD_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..8 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_thirty_fourth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FOURTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..2 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(2u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        for entry_index in 8u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_thirty_fifth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FIFTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(2u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 3u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_thirty_sixth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_sixth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SIXTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SIXTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_thirty_seventh_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SEVENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_thirty_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..14 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_thirty_ninth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_thirty_ninth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_NINTH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_NINTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fortieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..9 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fortieth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fortieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTIETH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTIETH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let partial_slot = table.base;
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        for entry_index in 9u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_forty_first_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_first_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIRST_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIRST_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        self.write_expanded_slot(&layout, table.base, true)?;
        for entry_index in 1u16..9 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..3 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        for entry_index in 9u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_forty_third_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_THIRD_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_THIRD_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 3u16..10 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(10u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_forty_fourth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, word) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FOURTH_APPEARANCE_RAM_WORDS {
            self.write_word(address, word)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FOURTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..3 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(3u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        let resumed_slot = table
            .base
            .wrapping_add(10u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 11u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_forty_fifth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIFTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(3u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 4u16..11 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(11u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 12u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_forty_seventh_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SEVENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.apply_trace_power_on_forty_eighth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_eighth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_EIGHTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_forty_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..14 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.apply_trace_power_on_forty_ninth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_forty_ninth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_NINTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fiftieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fiftieth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fiftieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTIETH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTIETH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..13 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_fifty_first_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_first_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIRST_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIRST_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        let final_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let final_size = self.read_appearance_word(&layout, final_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, final_slot, final_size)?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fifty_second_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_second_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SECOND_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..13 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_fifty_third_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_THIRD_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_THIRD_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        let final_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let final_size = self.read_appearance_word(&layout, final_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, final_slot, final_size)?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fifty_fourth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FOURTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FOURTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..13 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.apply_trace_power_on_fifty_fifth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIFTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        let final_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let final_size = self.read_appearance_word(&layout, final_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, final_slot, final_size)?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fifty_sixth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_sixth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SIXTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SIXTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..13 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_fifty_seventh_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SEVENTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SEVENTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(13u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        let final_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let final_size = self.read_appearance_word(&layout, final_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, final_slot, final_size)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_fifty_eighth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_eighth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_EIGHTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifty_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_fifty_ninth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_fifty_ninth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_NINTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_NINTH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixtieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..14 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_sixtieth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixtieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTIETH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTIETH_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..7 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_sixty_first_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_first_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIRST_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(7u16.wrapping_mul(table.entry_size));
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 8u16..14 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        self.apply_trace_power_on_sixty_second_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_second_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SECOND_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (screen_address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SECOND_VIDEO_BYTES {
            self.write_byte(screen_address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_sixty_third_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_THIRD_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_THIRD_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_sixty_fourth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FOURTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FOURTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..6 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_sixty_fifth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIFTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIFTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_sixth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(6u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 7u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_sixty_sixth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_sixth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SIXTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SIXTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_seventh_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 0u16..7 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        self.apply_trace_power_on_sixty_seventh_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SEVENTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SEVENTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_eighth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 7u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.apply_trace_power_on_sixty_eighth_defender_appearance_video_boundary()?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_sixty_eighth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_EIGHTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_EIGHTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_sixty_ninth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table
            .base
            .wrapping_add(14u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        self.write_byte(table.base, 0x89)?;

        self.advance_red_label_rand(&layout)?;
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_NINTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_seventieth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        self.advance_appearance_slot(&layout, first_slot, 0x8A00)?;

        for entry_index in 1u16..8 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let ninth_slot = table.base.wrapping_add(8u16.wrapping_mul(table.entry_size));
        self.write_byte(ninth_slot, 0x89)?;

        Ok(())
    }

    pub(super) fn run_trace_power_on_seventy_first_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let ninth_slot = table.base.wrapping_add(8u16.wrapping_mul(table.entry_size));
        self.advance_appearance_slot(&layout, ninth_slot, 0x8A00)?;

        for entry_index in 9u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        self.advance_red_label_rand(&layout)?;
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIRST_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_seventy_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..8 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SECOND_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SECOND_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_THIRD_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_THIRD_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FOURTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FOURTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIFTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIFTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIFTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_sixth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.write_red_label_rand_state(
            &layout,
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SIXTH_RAND_STATE,
        )?;
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SIXTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SIXTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SIXTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_seventh_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SEVENTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SEVENTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_eighth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_EIGHTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_EIGHTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_EIGHTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_seventy_ninth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_NINTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_NINTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_NINTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eightieth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.write_red_label_rand_state(
            &layout,
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTIETH_RAND_STATE,
        )?;
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTIETH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTIETH_VISIBLE_NIBBLES {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_first_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIRST_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIRST_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_second_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SECOND_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SECOND_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SECOND_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_third_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_THIRD_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_THIRD_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_fourth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FOURTH_APPEARANCE_RAM_BYTES
        {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FOURTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_fifth_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIFTH_APPEARANCE_RAM_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIFTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_sixth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SIXTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SIXTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_seventh_defender_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SEVENTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_eighth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_EIGHTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_EIGHTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_eighty_ninth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_NINTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_NINTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninetieth_defender_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETIETH_VISIBLE_NIBBLES {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_first_defender_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FIRST_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_second_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SECOND_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SECOND_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_third_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_THIRD_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_THIRD_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_fourth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FOURTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FOURTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_fifth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FIFTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FIFTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_input_video_boundary(
        &mut self,
        frame: u64,
        input_bits: u16,
        recent_input: Option<(u64, u16)>,
    ) -> Result<(), String> {
        let effective_input_bits = red_label_trace_power_on_effective_special_input_bits(
            input_bits,
            recent_input.map(|(_, bits)| bits),
        );
        let visible_nibbles: &[(u32, u8)] = match (frame, effective_input_bits) {
            (RED_LABEL_TRACE_POWER_ON_INPUT_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_INPUT_VIDEO_RECOVERY_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_RECOVERY_VISIBLE_NIBBLES,
            (RED_LABEL_TRACE_POWER_ON_FIRE_SECOND_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_SECOND_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_SECOND_RECOVERY_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_SECOND_RECOVERY_VISIBLE_NIBBLES,
            (RED_LABEL_TRACE_POWER_ON_FIRE_THIRD_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_THIRD_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_FIRE_FOURTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_FOURTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_FIRE_FIFTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_FIFTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_FIRE_SIXTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_SIXTH_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_SEVENTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_SEVENTH_VISIBLE_NIBBLES,
            (RED_LABEL_TRACE_POWER_ON_FIRE_EIGHTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_EIGHTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_FIRE_NINTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_NINTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_FIRE_TENTH_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_FIRE_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_FIRE_TENTH_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_ELEVENTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_ELEVENTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_TWELFTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_TWELFTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_THIRTEENTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_THIRTEENTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_FIRE_FOURTEENTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_FIRE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_FIRE_FOURTEENTH_VISIBLE_NIBBLES,
            (RED_LABEL_TRACE_POWER_ON_INPUT_VIDEO_FRAME, RED_LABEL_TRACE_INPUT_THRUST_BITS) => {
                &RED_LABEL_TRACE_POWER_ON_THRUST_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_INPUT_VIDEO_RECOVERY_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_RECOVERY_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_SECOND_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_SECOND_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_THIRD_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_THIRD_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_FOURTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_FOURTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_FIFTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_FIFTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_SIXTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_SIXTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_SEVENTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_SEVENTH_VISIBLE_NIBBLES,
            (
                RED_LABEL_TRACE_POWER_ON_THRUST_EIGHTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_THRUST_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_THRUST_EIGHTH_VISIBLE_NIBBLES,
            _ => return Ok(()),
        };

        for (visible_index, nibble) in visible_nibbles {
            self.write_visible_pixel_nibble(*visible_index, *nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_delayed_input_video_boundary(
        &mut self,
        frame: u64,
        recent_special_input: Option<(u64, u16)>,
    ) -> Result<(), String> {
        let Some((source_frame, input_bits)) = recent_special_input else {
            return Ok(());
        };
        if source_frame != RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SOURCE_FRAME
            || !matches!(
                input_bits,
                RED_LABEL_TRACE_INPUT_SMART_BOMB_BITS | RED_LABEL_TRACE_INPUT_HYPERSPACE_BITS
            )
        {
            return Ok(());
        }

        let visible_nibbles: &[(u32, u8)] = match (frame, input_bits) {
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_RECOVERY_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_RECOVERY_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SECOND_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_SECOND_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_THIRD_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_THIRD_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_FOURTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_FOURTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_FIFTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_FIFTH_VISIBLE_NIBBLES
            }
            (
                RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SIXTH_VIDEO_FRAME,
                RED_LABEL_TRACE_INPUT_HYPERSPACE_BITS,
            ) => &RED_LABEL_TRACE_POWER_ON_HYPERSPACE_SIXTH_VISIBLE_NIBBLES,
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SIXTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_SIXTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SEVENTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_SEVENTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_SEVENTH_RECOVERY_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_SEVENTH_RECOVERY_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_EIGHTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_EIGHTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_NINTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_NINTH_VISIBLE_NIBBLES
            }
            (RED_LABEL_TRACE_POWER_ON_DELAYED_INPUT_TENTH_VIDEO_FRAME, _) => {
                &RED_LABEL_TRACE_POWER_ON_SMART_HYPERSPACE_TENTH_VISIBLE_NIBBLES
            }
            _ => return Ok(()),
        };

        for (visible_index, nibble) in visible_nibbles {
            self.write_visible_pixel_nibble(*visible_index, *nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_sixth_defender_process_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SIXTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SIXTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_seventh_defender_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SEVENTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_ninety_eighth_defender_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (visible_index, nibble) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_EIGHTH_VISIBLE_NIBBLES
        {
            self.write_visible_pixel_nibble(visible_index, nibble)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_one_hundredth_defender_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDREDTH_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_one_hundred_first_defender_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_FIRST_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_one_hundred_second_defender_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_SECOND_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        Ok(())
    }

    pub(super) fn apply_trace_power_on_one_hundred_third_defender_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (address, byte) in RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_THIRD_PROCESS_BYTES {
            self.write_byte(address, byte)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_second_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0..RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_DRAWN_SLOTS {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_PARTIAL_SLOT
                .wrapping_mul(table.entry_size),
        );
        let size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, size)?;

        let resumed_slot = table.base.wrapping_add(
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_PARTIAL_SLOT
                .wrapping_mul(table.entry_size),
        );
        self.write_expanded_slot(&layout, resumed_slot, false)?;

        for entry_index in 13u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot_geometry(&layout, slot_address, size)?;
            self.erase_expanded_slot(&layout, slot_address)?;
            self.write_expanded_slot(&layout, slot_address, false)?;
        }
        self.apply_trace_power_on_second_defender_appearance_video_boundary()?;
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn apply_trace_power_on_second_defender_appearance_video_boundary(
        &mut self,
    ) -> Result<(), String> {
        for screen_address in RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_ERASED_WORDS {
            self.write_word(screen_address, 0)?;
        }
        for (screen_address, byte) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_MID_WRITE_BYTES
        {
            self.write_byte(screen_address, byte)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_third_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let resumed_slot = table.base.wrapping_add(
            RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_PARTIAL_SLOT
                .wrapping_mul(table.entry_size),
        );
        self.erase_expanded_slot(&layout, resumed_slot)?;
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 10u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        for (screen_address, byte) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRD_APPEARANCE_MID_WRITE_BYTES
        {
            self.write_byte(screen_address, byte)?;
        }
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn run_trace_power_on_fourth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        for entry_index in 0u16..5 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }

        let partial_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        let size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        for (erase_address, screen_address) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTH_APPEARANCE_INACTIVE_ERASE_WORDS
        {
            self.write_word(erase_address, screen_address)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        Ok(())
    }

    pub(super) fn run_trace_power_on_fifth_defender_appearance_video_slice(
        &mut self,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let table = table_descriptor(&layout, "appearance_ram")?;
        self.write_field_byte(&layout, "base_page", "TIMER", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        self.check_player_collision()?;

        let first_slot = table.base;
        let first_size = self.read_appearance_word(&layout, first_slot, "RSIZE")?;
        self.advance_appearance_slot(&layout, first_slot, first_size)?;

        let partial_slot = table.base.wrapping_add(table.entry_size);
        let partial_size = self.read_appearance_word(&layout, partial_slot, "RSIZE")?;
        self.advance_appearance_slot_geometry(&layout, partial_slot, partial_size)?;
        self.erase_expanded_slot(&layout, partial_slot)?;
        for (erase_address, screen_address) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTH_APPEARANCE_INACTIVE_ERASE_WORDS
        {
            self.write_word(erase_address, screen_address)?;
        }
        let resumed_slot = table.base.wrapping_add(5u16.wrapping_mul(table.entry_size));
        self.write_expanded_slot(&layout, resumed_slot, true)?;

        for entry_index in 6u16..15 {
            let slot_address = table
                .base
                .wrapping_add(entry_index.wrapping_mul(table.entry_size));
            let size = self.read_appearance_word(&layout, slot_address, "RSIZE")?;
            self.advance_appearance_slot(&layout, slot_address, size)?;
        }
        for screen_address in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTH_APPEARANCE_ERASED_WORDS {
            self.write_word(screen_address, 0)?;
        }
        for (screen_address, word) in
            RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTH_APPEARANCE_MID_WRITE_WORDS
        {
            self.write_word(screen_address, word)?;
        }
        self.advance_red_label_rand(&layout)?;
        Ok(())
    }

    pub(super) fn reset_exec_current_process_to_active(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let lists = red_label_linked_lists()?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let crproc = ram_field(layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        self.write_word(crproc, active_head)?;
        Ok(active_head)
    }

    pub(super) fn move_first_exec_overload_object_to_inactive(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<RedLabelExecOverloadedObject>, String> {
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(layout, "object")?;
        let active_head = linked_list(&lists, "active_object")?.head_address;
        let inactive_head = linked_list(&lists, "inactive_object")?.head_address;
        let mut previous_link_address = active_head;

        for _ in 0..object.entries {
            let object_address = self.read_word(previous_link_address)?;
            if object_address == 0 {
                return Ok(None);
            }
            object_table_for_address(layout, object_address)?;
            let next_active_object = self.read_object_word(layout, object_address, "OLINK")?;
            if self.read_object_byte(layout, object_address, "OTYP")? != 0 {
                previous_link_address =
                    object_field_range_for_address(layout, object_address, "OLINK")?.start;
                continue;
            }

            self.write_word(previous_link_address, next_active_object)?;
            let seed = self.read_field_byte(layout, "base_page", "SEED")?;
            let hseed = self.read_field_byte(layout, "base_page", "HSEED")?;
            let previous_x16 = self.read_object_word(layout, object_address, "OX16")?;
            let x_delta = u16::from_be_bytes([(seed & 0x3F).wrapping_add(0x60), hseed]);
            let x16 = previous_x16.wrapping_add(x_delta);
            self.write_object_word(layout, object_address, "OX16", x16)?;

            let screen_address = self.read_object_screen_address(layout, object_address)?;
            let picture_address = self.read_object_word(layout, object_address, "OPICT")?;
            self.write_field_byte(layout, "base_page", "MAPCR", 2)?;
            self.erase_object_picture_by_descriptor(screen_address, picture_address)?;
            self.write_object_screen_address(layout, object_address, 0)?;

            let previous_inactive_head = self.read_word(inactive_head)?;
            self.write_word(inactive_head, object_address)?;
            self.write_object_word(layout, object_address, "OLINK", previous_inactive_head)?;

            return Ok(Some(RedLabelExecOverloadedObject {
                object_address,
                previous_link_address,
                next_active_object,
                previous_x16,
                x16,
                screen_address,
                picture_address,
                previous_inactive_head,
            }));
        }

        Err(String::from(
            "red-label EXEC overload active-object list did not terminate",
        ))
    }

    /// Executes the first source-shaped slice of red-label `DISP`: walk ACTIVE,
    /// decrement `PTIME`, update `CRPROC`, and report the due `PADDR`.
    pub fn step_process_scheduler(&mut self) -> Result<Option<RedLabelScheduledProcess>, String> {
        let lists = red_label_linked_lists()?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        self.step_process_scheduler_from_link(active_head)
    }

    pub(super) fn step_process_scheduler_from_link(
        &mut self,
        link_address: u16,
    ) -> Result<Option<RedLabelScheduledProcess>, String> {
        let layout = red_label_ram_layout()?;
        let process = table_descriptor(&layout, "process")?;
        let super_process = table_descriptor(&layout, "super_process")?;
        let mut process_address = self.read_word(link_address)?;

        for _ in 0..(process.entries + super_process.entries) {
            if process_address == 0 {
                return Ok(None);
            }
            let table = process_table_for_address(&layout, process_address)?;
            let ptime_range =
                process_field_range_for_address(&layout, table, process_address, "PTIME")?;
            let ptime_address = ptime_range.start;
            let ptime = self.read_byte(ptime_address)?.wrapping_sub(1);
            self.write_byte(ptime_address, ptime)?;
            if ptime == 0 {
                let crproc = ram_field(&layout, "runtime_pointers", "CRPROC")?
                    .field_range_for_entry(0)
                    .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
                    .start;
                self.write_word(crproc, process_address)?;
                let paddr =
                    process_field_range_for_address(&layout, table, process_address, "PADDR")?
                        .start;
                return Ok(Some(RedLabelScheduledProcess::from_source_routine(
                    process_address,
                    self.read_word(paddr)?,
                )?));
            }

            let plink =
                process_field_range_for_address(&layout, table, process_address, "PLINK")?.start;
            process_address = self.read_word(plink)?;
        }

        Err(String::from(
            "red-label active process list did not terminate within process table size",
        ))
    }

    pub(super) fn step_single_process_scheduler_from_link(
        &mut self,
        link_address: u16,
    ) -> Result<Option<RedLabelScheduledProcess>, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.read_word(link_address)?;
        if process_address == 0 {
            return Ok(None);
        }

        let table = process_table_for_address(&layout, process_address)?;
        let ptime_range =
            process_field_range_for_address(&layout, table, process_address, "PTIME")?;
        let ptime_address = ptime_range.start;
        let ptime = self.read_byte(ptime_address)?.wrapping_sub(1);
        self.write_byte(ptime_address, ptime)?;
        if ptime != 0 {
            return Ok(None);
        }

        let crproc = ram_field(&layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        self.write_word(crproc, process_address)?;
        let paddr =
            process_field_range_for_address(&layout, table, process_address, "PADDR")?.start;
        Ok(Some(RedLabelScheduledProcess::from_source_routine(
            process_address,
            self.read_word(paddr)?,
        )?))
    }

    pub(super) fn active_process_link_before_routine(
        &self,
        routine_addresses: &[u16],
    ) -> Result<Option<u16>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process = table_descriptor(&layout, "process")?;
        let super_process = table_descriptor(&layout, "super_process")?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let mut link_address = active_head;
        let mut process_address = self.read_word(link_address)?;

        for _ in 0..(process.entries + super_process.entries) {
            if process_address == 0 {
                return Ok(None);
            }
            let table = process_table_for_address(&layout, process_address)?;
            let routine_address = self.read_process_word(&layout, process_address, "PADDR")?;
            if routine_addresses.contains(&routine_address) {
                return Ok(Some(link_address));
            }

            link_address =
                process_field_range_for_address(&layout, table, process_address, "PLINK")?.start;
            process_address = self.read_word(link_address)?;
        }

        Err(String::from(
            "red-label active process list did not terminate while searching routine addresses",
        ))
    }

    pub(super) fn active_process_has_routine(
        &self,
        routine_addresses: &[u16],
    ) -> Result<bool, String> {
        self.active_process_link_before_routine(routine_addresses)
            .map(|link_address| link_address.is_some())
    }

    /// Dispatches the translated routine body currently named by `PADDR`.
    /// Untranslated routine addresses remain explicit fidelity gaps.
    pub fn dispatch_translated_process_routine(
        &mut self,
        routine_address: u16,
    ) -> Result<RedLabelProcessDispatch, String> {
        let entry_registers = self.source_entry_registers_for_current_routine(routine_address)?;
        self.dispatch_translated_process_routine_with_entry_registers(
            routine_address,
            entry_registers,
        )
    }

    pub(super) fn dispatch_translated_process_routine_with_entry_registers(
        &mut self,
        routine_address: u16,
        entry_registers: RedLabelCpuRegisters,
    ) -> Result<RedLabelProcessDispatch, String> {
        if [
            red_label_routine_address("SUCIDE")?,
            red_label_routine_address("HYPX")?,
        ]
        .contains(&routine_address)
        {
            let layout = red_label_ram_layout()?;
            return self
                .kill_current_process(&layout)
                .map(RedLabelProcessDispatch::Suicide);
        }

        if routine_address == red_label_routine_address("CREDS")? {
            return self
                .display_attract_credits_current_process()
                .map(RedLabelProcessDispatch::AttractCredits);
        }

        if routine_address == red_label_routine_address("AMODES")? {
            return self
                .start_attract_williams_page_current_process()
                .map(RedLabelProcessDispatch::AttractWilliamsPage);
        }

        if routine_address == red_label_routine_address("LOGO")? {
            return self
                .start_attract_logo_current_process()
                .map(RedLabelProcessDispatch::AttractLogo);
        }

        if routine_address == red_label_routine_address("LOGO0")? {
            return self
                .step_attract_logo_table_current_process()
                .map(RedLabelProcessDispatch::AttractLogo);
        }

        if routine_address == red_label_routine_address("PRES")? {
            return self
                .start_attract_presents_current_process()
                .map(RedLabelProcessDispatch::AttractPresents);
        }

        if routine_address == red_label_routine_address("PRES1")? {
            return self
                .refresh_attract_presents_current_process()
                .map(RedLabelProcessDispatch::AttractPresents);
        }

        if routine_address == red_label_routine_address("DEFEND")? {
            return self
                .delay_attract_defender_current_process()
                .map(RedLabelProcessDispatch::AttractDefenderDelay);
        }

        if routine_address == red_label_routine_address("DEFENS")? {
            return self
                .start_attract_defender_appearances_current_process()
                .map(RedLabelProcessDispatch::AttractDefenderAppearances);
        }

        if routine_address == red_label_routine_address("DEF33")? {
            return self
                .start_attract_defender_restore_current_process()
                .map(RedLabelProcessDispatch::AttractDefenderRestoreStart);
        }

        if routine_address == red_label_routine_address("DEF44")? {
            return self
                .start_attract_copyright_current_process()
                .map(RedLabelProcessDispatch::AttractCopyright);
        }

        if routine_address == red_label_routine_address("COPYRT")? {
            return self
                .write_attract_copyright_current_process()
                .map(RedLabelProcessDispatch::AttractCopyright);
        }

        if routine_address == red_label_routine_address("CPR55")? {
            return self
                .continue_attract_copyright_wait_current_process()
                .map(RedLabelProcessDispatch::AttractCopyrightWait);
        }

        if routine_address == red_label_routine_address("ATTR")? {
            return self
                .start_attract_vector_current_process()
                .map(RedLabelProcessDispatch::AttractVector);
        }

        if routine_address == red_label_routine_address("HALD4")?
            || routine_address == red_label_routine_address("LEDRET")?
        {
            return self
                .start_attract_instruction_page_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionStart);
        }

        if routine_address == red_label_routine_address("CPR56")?
            || routine_address == red_label_routine_address("HALDIS")?
        {
            return self
                .display_hall_of_fame_from_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("HALD3")? {
            return self
                .continue_hall_of_fame_display_wait_current_process()
                .map(RedLabelProcessDispatch::HallOfFameDisplayWait);
        }

        if routine_address == red_label_routine_address("DEF50")? {
            return self
                .refresh_attract_defender_current_process()
                .map(RedLabelProcessDispatch::AttractDefenderRefresh);
        }

        if routine_address == red_label_routine_address("DEF51")? {
            return self
                .delay_attract_defender_refresh_current_process()
                .map(RedLabelProcessDispatch::AttractDefenderRefresh);
        }

        if routine_address == red_label_routine_address("WILLIR")? {
            return self
                .start_attract_williams_restore_current_process()
                .map(RedLabelProcessDispatch::AttractWilliamsRestore);
        }

        if routine_address == red_label_routine_address("WILR1")? {
            return self
                .finish_attract_williams_restore_current_process()
                .map(RedLabelProcessDispatch::AttractWilliamsRestore);
        }

        if routine_address == red_label_routine_address("AMODE1")? {
            return self
                .raise_attract_instruction_objects_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionAscent);
        }

        if routine_address == red_label_routine_address("AMODE2")? {
            return self
                .start_attract_instruction_laser_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionLaserStart);
        }

        if routine_address == red_label_routine_address("LASRS")? {
            return self
                .start_attract_instruction_laser_step_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionLaserStep);
        }

        if routine_address == red_label_routine_address("LAS0")? {
            return self
                .continue_attract_instruction_laser_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionLaserStep);
        }

        if routine_address == red_label_routine_address("AMODE3")? {
            return self
                .start_attract_instruction_rescue_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionRescueStart);
        }

        if routine_address == red_label_routine_address("AMODE4")? {
            return self
                .continue_attract_instruction_free_fall_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionFreeFall);
        }

        if routine_address == red_label_routine_address("AMODE5")? {
            return self
                .start_attract_instruction_intersection_current_process()
                .map(RedLabelAttractInstructionFreeFall::Intersection)
                .map(RedLabelProcessDispatch::AttractInstructionFreeFall);
        }

        if routine_address == red_label_routine_address("AMODE7")? {
            return self
                .start_attract_instruction_ship_return_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionShipReturn);
        }

        if routine_address == red_label_routine_address("AMODE8")? {
            return self
                .start_attract_instruction_enemy_table_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionEnemyTableStart);
        }

        if routine_address == red_label_routine_address("AMOD12")? {
            return self
                .spawn_attract_instruction_enemy_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionEnemySpawn);
        }

        if routine_address == red_label_routine_address("AMOD10")? {
            return self
                .start_attract_instruction_table_laser_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionLaserStart);
        }

        if routine_address == red_label_routine_address("AMOD11")? {
            return self
                .resolve_attract_instruction_enemy_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionEnemyResolve);
        }

        if routine_address == red_label_routine_address("BMODE2")? {
            return self
                .advance_attract_instruction_text_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionTextAdvance);
        }

        if routine_address == red_label_routine_address("BMODE3")? {
            return self
                .decide_attract_instruction_table_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionTableDecision);
        }

        if routine_address == red_label_routine_address("AMOD13")? {
            return self
                .restart_attract_instruction_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionRestart);
        }

        if routine_address == red_label_routine_address("TEXTP")? {
            return self
                .start_attract_instruction_text_process_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionTextProcess);
        }

        if routine_address == red_label_routine_address("TEXTP2")? {
            return self
                .continue_attract_instruction_text_process_current_process()
                .map(RedLabelProcessDispatch::AttractInstructionTextProcess);
        }

        if routine_address == red_label_routine_address("LFIRE")? {
            return self
                .dispatch_laser_fire_current_process()
                .map(RedLabelProcessDispatch::LaserFire);
        }

        if routine_address == red_label_routine_address("LASR")? {
            return self
                .start_right_laser_current_process()
                .map(RedLabelProcessDispatch::LaserStep);
        }

        if routine_address == red_label_routine_address("LASL")? {
            return self
                .start_left_laser_current_process()
                .map(RedLabelProcessDispatch::LaserStep);
        }

        if routine_address == red_label_routine_address("LASR0")? {
            return self
                .step_right_laser_current_process()
                .map(RedLabelProcessDispatch::LaserStep);
        }

        if routine_address == red_label_routine_address("LASL0")? {
            return self
                .step_left_laser_current_process()
                .map(RedLabelProcessDispatch::LaserStep);
        }

        if routine_address == red_label_routine_address("LASD")? {
            return self
                .finish_laser_fire_current_process()
                .map(RedLabelProcessDispatch::LaserFinished);
        }

        if routine_address == red_label_routine_address("THPROC")? {
            return self
                .step_thrust_process_current_process()
                .map(RedLabelProcessDispatch::ThrustProcess);
        }

        if routine_address == red_label_routine_address("COLR")? {
            return self
                .start_laser_color_current_process(true)
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("COLRLP")? {
            return self
                .start_laser_color_current_process(false)
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("FLPUP")? {
            return self
                .start_player_up_flash_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("FLP2")? {
            return self
                .continue_player_up_flash_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("CBOMB")? {
            return self
                .start_bomb_color_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("CBMB1")? {
            return self
                .continue_bomb_color_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("HOFST")? {
            return self
                .step_hall_of_fame_stall_timer_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("HOFBL")? {
            return self
                .step_hall_of_fame_blink_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("HOFUD")? {
            return self
                .start_hall_of_fame_initials_input_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("HOFUD1")? {
            return self
                .continue_hall_of_fame_initials_input_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("HALL1")? {
            return self
                .start_high_score_qualification_current_process()
                .map(RedLabelProcessDispatch::HighScoreQualification);
        }

        if routine_address == red_label_routine_address("HALL3A")? {
            return self
                .start_high_score_fire_switch_current_process()
                .map(RedLabelProcessDispatch::HighScoreFireSwitch);
        }

        if routine_address == red_label_routine_address("HALL4")? {
            return self
                .continue_high_score_fire_switch_current_process()
                .map(RedLabelProcessDispatch::HighScoreFireSwitch);
        }

        if routine_address == red_label_routine_address("HALL5")? {
            return self
                .advance_high_score_fire_switch_current_process()
                .map(RedLabelProcessDispatch::HighScoreFireSwitch);
        }

        if routine_address == red_label_routine_address("HALL6")? {
            return self
                .submit_high_score_initials_current_process()
                .map(RedLabelProcessDispatch::HighScoreSubmission);
        }

        if routine_address == red_label_routine_address("HALL12")? {
            return self
                .advance_high_score_handoff_current_process()
                .map(RedLabelProcessDispatch::HighScoreHandoff);
        }

        if routine_address == red_label_routine_address("TIECOL")? {
            return self
                .start_tie_color_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("TIECL")? {
            return self
                .continue_tie_color_current_process()
                .map(RedLabelProcessDispatch::SupportProcess);
        }

        if routine_address == red_label_routine_address("SCPROC")? {
            return self
                .start_scanner_process_current_process()
                .map(RedLabelProcessDispatch::ScannerProcess);
        }

        if routine_address == red_label_routine_address("SCP1")? {
            return self
                .continue_scanner_process_object_current_process()
                .map(RedLabelProcessDispatch::ScannerProcess);
        }

        if routine_address == red_label_routine_address("SCP2")? {
            return self
                .continue_scanner_process_display_current_process()
                .map(RedLabelProcessDispatch::ScannerProcess);
        }

        if routine_address == red_label_routine_address("TDISP")? {
            return self.top_display().map(RedLabelProcessDispatch::TopDisplay);
        }

        if routine_address == red_label_routine_address("PLSTRT")? {
            return self
                .start_player_start_current_process()
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("PLST1A")? {
            return self
                .continue_player_start_after_coin_counters_current_process()
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("PLSTR3")? {
            return self
                .start_player_runtime_current_process()
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("PLS01")? {
            return self
                .continue_player_start_screen_current_process()
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("PLS1")? {
            return self
                .finish_player_start_current_process_with_entry_registers(entry_registers)
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("GEXEC")? {
            return self
                .start_game_exec_current_process()
                .map(RedLabelProcessDispatch::GameExec);
        }

        if routine_address == red_label_routine_address("GEX0")? {
            return self
                .step_game_exec_current_process()
                .map(RedLabelProcessDispatch::GameExec);
        }

        if routine_address == red_label_routine_address("SBOMB")? {
            return self.dispatch_smart_bomb_process_entry();
        }

        if routine_address == red_label_routine_address("SBMBX1")? {
            return self
                .continue_smart_bomb_flash_tail()
                .map(RedLabelProcessDispatch::SmartBombTail);
        }

        if routine_address == red_label_routine_address("SBX1A")? {
            return self
                .continue_smart_bomb_debounce_tail()
                .map(RedLabelProcessDispatch::SmartBombTail);
        }

        if routine_address == red_label_routine_address("SBX2A")? {
            return self
                .finish_smart_bomb_tail()
                .map(RedLabelProcessDispatch::SmartBombTail);
        }

        if routine_address == red_label_routine_address("SBMBX2")? {
            let layout = red_label_ram_layout()?;
            return self
                .suicide_current_process(&layout)
                .map(RedLabelProcessDispatch::SmartBombTail);
        }

        if routine_address == red_label_routine_address("ST1")? {
            return self
                .dispatch_start_one_current_process()
                .map(RedLabelProcessDispatch::StartSwitch);
        }

        if routine_address == red_label_routine_address("ST2")? {
            return self
                .dispatch_start_two_current_process()
                .map(RedLabelProcessDispatch::StartSwitch);
        }

        if routine_address == red_label_routine_address("HSRES")? {
            return self
                .dispatch_high_score_reset_current_process()
                .map(RedLabelProcessDispatch::AdminSwitch);
        }

        if routine_address == red_label_routine_address("ADVSW")? {
            return self
                .dispatch_advance_switch_current_process()
                .map(RedLabelProcessDispatch::AdminSwitch);
        }

        if routine_address == red_label_routine_address("LCOIN")? {
            return self
                .start_coin_process_current_process(RedLabelCoinSlot::Left)
                .map(RedLabelProcessDispatch::CoinProcess);
        }

        if routine_address == red_label_routine_address("RCOIN")? {
            return self
                .start_coin_process_current_process(RedLabelCoinSlot::Right)
                .map(RedLabelProcessDispatch::CoinProcess);
        }

        if routine_address == red_label_routine_address("CCOIN")? {
            return self
                .start_coin_process_current_process(RedLabelCoinSlot::Center)
                .map(RedLabelProcessDispatch::CoinProcess);
        }

        if routine_address == red_label_routine_address("CN1")? {
            return self
                .continue_coin_process_current_process()
                .map(RedLabelProcessDispatch::CoinProcess);
        }

        if routine_address == red_label_routine_address("HYPER")? {
            return self
                .start_hyperspace_current_process()
                .map(RedLabelProcessDispatch::Hyperspace);
        }

        if routine_address == red_label_routine_address("HYP02")? {
            return self
                .continue_hyperspace_current_process()
                .map(RedLabelProcessDispatch::Hyperspace);
        }

        if routine_address == red_label_routine_address("HYP2")? {
            return self
                .finish_hyperspace_current_process()
                .map(RedLabelProcessDispatch::Hyperspace);
        }

        if routine_address == red_label_routine_address("PLEND")? {
            return self
                .start_player_death_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PDTHL")? {
            return self
                .blank_player_death_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PDTH2")? {
            return self
                .continue_player_death_glow_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PDTH4")? {
            return self
                .finish_player_death_glow_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PDTH5")? {
            return self
                .start_player_death_tail_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PX1A")? {
            return self
                .continue_player_explosion_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PDTH5R")? {
            return self
                .continue_player_death_after_explosion_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("BONUS")? {
            return self
                .start_player_death_bonus_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("BC1")? {
            return self
                .continue_player_death_bonus_astronaut_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("BC2")? {
            return self
                .advance_player_death_bonus_wave_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("BC3")? {
            return self
                .finish_player_death_bonus_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PLE02")? {
            return self
                .continue_player_death_player_switch_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("PLE3")? {
            return self
                .jump_player_death_game_over_to_attract_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("HALL13")? {
            return self
                .display_hall_of_fame_from_current_process()
                .map(RedLabelProcessDispatch::PlayerDeath);
        }

        if routine_address == red_label_routine_address("REV")? {
            return self
                .start_reverse_current_process()
                .map(RedLabelProcessDispatch::Reverse);
        }

        if routine_address == red_label_routine_address("REV2")? {
            return self
                .continue_reverse_current_process()
                .map(RedLabelProcessDispatch::Reverse);
        }

        if routine_address == red_label_routine_address("REVX1")? {
            return self
                .finish_reverse_current_process()
                .map(RedLabelProcessDispatch::Reverse);
        }

        if routine_address == red_label_routine_address("REVX")? {
            let layout = red_label_ram_layout()?;
            return self
                .kill_current_process(&layout)
                .map(RedLabelReverse::Completed)
                .map(RedLabelProcessDispatch::Reverse);
        }

        if routine_address == red_label_routine_address("ASTRO")? {
            return self
                .step_astronaut_current_process()
                .map(RedLabelProcessDispatch::Astronaut);
        }

        if routine_address == red_label_routine_address("AFALL")? {
            return self
                .step_falling_astronaut_current_process(false)
                .map(RedLabelProcessDispatch::FallingAstronaut);
        }

        if routine_address == red_label_routine_address("AFALL2")? {
            return self
                .step_falling_astronaut_current_process(true)
                .map(RedLabelProcessDispatch::FallingAstronaut);
        }

        if routine_address == red_label_routine_address("P250")? {
            return self
                .start_score_sprite_current_process(RedLabelScoreSpriteKind::Points250)
                .map(RedLabelProcessDispatch::ScoreSprite);
        }

        if routine_address == red_label_routine_address("P500")? {
            return self
                .start_score_sprite_current_process(RedLabelScoreSpriteKind::Points500)
                .map(RedLabelProcessDispatch::ScoreSprite);
        }

        if routine_address == red_label_routine_address("P503")? {
            return self
                .finish_score_sprite_current_process()
                .map(RedLabelProcessDispatch::ScoreSprite);
        }

        if routine_address == red_label_routine_address("TERBLO")? {
            return self
                .start_terrain_blow_current_process()
                .map(RedLabelProcessDispatch::TerrainBlow);
        }

        if routine_address == red_label_routine_address("TBL3")? {
            return self
                .continue_terrain_blow_flash_current_process()
                .map(RedLabelProcessDispatch::TerrainBlow);
        }

        if routine_address == red_label_routine_address("TBL4")? {
            return self
                .advance_terrain_blow_iteration_current_process()
                .map(RedLabelProcessDispatch::TerrainBlow);
        }

        if routine_address == red_label_routine_address("MSWM")? {
            return self
                .step_mini_swarmer_current_process(true)
                .map(RedLabelProcessDispatch::MiniSwarmer);
        }

        if routine_address == red_label_routine_address("MSWLP")? {
            return self
                .step_mini_swarmer_current_process(false)
                .map(RedLabelProcessDispatch::MiniSwarmer);
        }

        if routine_address == red_label_routine_address("SCZ0")? {
            return self
                .step_schizoid_current_process()
                .map(RedLabelProcessDispatch::Schizoid);
        }

        if routine_address == red_label_routine_address("UFOST")? {
            return self
                .start_ufo_process()
                .map(RedLabelProcessDispatch::UfoStarted);
        }

        if routine_address == red_label_routine_address("UFOLP")? {
            return self
                .step_ufo_current_process()
                .map(RedLabelProcessDispatch::Ufo);
        }

        if routine_address == red_label_routine_address("TIE")? {
            return self
                .step_tie_current_process()
                .map(RedLabelProcessDispatch::Tie);
        }

        if routine_address == red_label_routine_address("LANDS0")? {
            return self
                .step_lander_orbit_current_process()
                .map(RedLabelProcessDispatch::Lander);
        }

        if routine_address == red_label_routine_address("LANDG")? {
            return self
                .step_lander_grab_current_process()
                .map(RedLabelProcessDispatch::Lander);
        }

        if routine_address == red_label_routine_address("LANDF")? {
            return self
                .step_lander_flee_current_process()
                .map(RedLabelProcessDispatch::Lander);
        }

        if routine_address == red_label_routine_address("LNDFXA")? {
            return self
                .continue_lander_pull_current_process()
                .map(RedLabelProcessDispatch::Lander);
        }

        Err(format!(
            "red-label process routine 0x{routine_address:04X} is not translated"
        ))
    }

    pub fn step_translated_process(&mut self) -> Result<Option<RedLabelProcessDispatch>, String> {
        let Some(scheduled) = self.step_process_scheduler()? else {
            return Ok(None);
        };
        self.dispatch_translated_scheduled_process(scheduled)
            .map(Some)
    }

    pub(super) fn dispatch_translated_scheduled_process(
        &mut self,
        scheduled: RedLabelScheduledProcess,
    ) -> Result<RedLabelProcessDispatch, String> {
        let layout = red_label_ram_layout()?;
        let current_process = self.read_field_word(&layout, "runtime_pointers", "CRPROC")?;
        scheduled.validate_source_disp_context(current_process)?;

        self.dispatch_translated_process_routine_with_entry_registers(
            scheduled.routine_address,
            scheduled.entry_registers,
        )
    }

    pub(super) fn source_entry_registers_for_current_routine(
        &self,
        routine_address: u16,
    ) -> Result<RedLabelCpuRegisters, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.read_field_word(&layout, "runtime_pointers", "CRPROC")?;
        red_label_source_entry_registers_for_routine(process_address, routine_address)
    }

    pub(super) fn make_process_from_free_list(
        &mut self,
        table_name: &str,
        free_list_name: &str,
        class: RedLabelProcessClass,
        routine_address: u16,
        process_type: u8,
        pcod: u8,
    ) -> Result<RedLabelCreatedProcess, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let process_table = table_descriptor(&layout, table_name)?;
        let free_head = linked_list(&lists, free_list_name)?.head_address;
        let process_address = self.read_word(free_head)?;
        if process_address == 0 {
            return Err(format!("red-label `{free_list_name}` is empty"));
        }

        entry_index_for_address(process_table, process_address)?;
        let next_free = self.read_process_word(&layout, process_address, "PLINK")?;
        self.write_word(free_head, next_free)?;
        self.write_process_byte(&layout, process_address, "PCOD", pcod)?;
        self.write_process_word(&layout, process_address, "PADDR", routine_address)?;
        self.write_process_byte(&layout, process_address, "PTYPE", process_type)?;
        self.write_process_byte(&layout, process_address, "PTIME", 1)?;

        let crproc = ram_field(&layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        let insertion_link_address = self.read_word(crproc)?;
        if insertion_link_address == 0 {
            return Err(String::from("red-label CRPROC insertion link is zero"));
        }
        let old_next = self.read_word(insertion_link_address)?;
        self.write_word(insertion_link_address, process_address)?;
        self.write_process_word(&layout, process_address, "PLINK", old_next)?;

        Ok(RedLabelCreatedProcess {
            process_address,
            routine_address,
            process_type,
            class,
        })
    }

    pub(super) fn queue_switch_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process: RedLabelSwitchProcess,
    ) -> Result<(), String> {
        let queue_range = field_range(layout, "base_page", "SWPROC")?;
        let primary_routine = self.read_word(queue_range.start)?;
        let slot_start = if primary_routine == 0 {
            queue_range.start
        } else {
            queue_range.start + 4
        };

        self.write_word(slot_start, process.routine_address)?;
        self.write_byte(slot_start + 2, process.process_type)?;
        self.write_byte(slot_start + 3, process.status_mask)
    }

    pub(super) fn take_next_switch_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<RedLabelSwitchProcess>, String> {
        let queue_range = field_range(layout, "base_page", "SWPROC")?;
        for slot_start in [queue_range.start, queue_range.start + 4] {
            let routine_address = self.read_word(slot_start)?;
            if routine_address == 0 {
                continue;
            }
            let process = RedLabelSwitchProcess {
                switch_bit: 0,
                routine_address,
                process_type: self.read_byte(slot_start + 2)?,
                status_mask: self.read_byte(slot_start + 3)?,
            };
            self.write_word(slot_start, 0)?;
            return Ok(Some(process));
        }

        Ok(None)
    }

    pub(super) fn initialize_process_lists(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
    ) -> Result<(), String> {
        let process = table_descriptor(layout, "process")?;
        let super_process = table_descriptor(layout, "super_process")?;
        self.link_free_table(layout, process, "PLINK")?;
        self.link_free_table(layout, super_process, "PLINK")?;
        self.write_word(
            linked_list(lists, "free_process")?.head_address,
            process.base,
        )?;
        self.write_word(
            linked_list(lists, "free_super_process")?.head_address,
            super_process.base,
        )?;
        self.write_word(linked_list(lists, "active_process")?.head_address, 0)?;
        let crproc = ram_field(layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        self.write_word(crproc, linked_list(lists, "active_process")?.head_address)
    }

    pub(super) fn initialize_object_lists(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
    ) -> Result<(), String> {
        let object = table_descriptor(layout, "object")?;
        self.link_free_table(layout, object, "OLINK")?;
        self.write_word(linked_list(lists, "free_object")?.head_address, object.base)?;
        self.write_word(linked_list(lists, "active_object")?.head_address, 0)?;
        self.write_word(linked_list(lists, "inactive_object")?.head_address, 0)?;
        self.write_word(linked_list(lists, "shell_object")?.head_address, 0)?;
        self.clear_appearance_ram(layout)
    }

    pub(super) fn clear_appearance_ram(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        let appearance = table_descriptor(layout, "appearance_ram")?;
        let range = appearance
            .table_range()
            .ok_or_else(|| String::from("red-label appearance RAM table range is invalid"))?;
        self.clear_range(range)
    }

    pub(super) fn explosion_start_center(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        picture_address: u16,
        top_left: u16,
    ) -> Result<u16, String> {
        let collision_center = self.read_field_word(layout, "base_page", "CENTMP")?;
        let picture = red_label_object_picture(picture_address)?;
        let [top_left_x, top_left_y] = top_left.to_be_bytes();
        let [center_x, center_y] = collision_center.to_be_bytes();
        let (_, x_carry) = top_left_x
            .wrapping_sub(center_x)
            .overflowing_add(picture.width);
        let (_, y_carry) = top_left_y
            .wrapping_sub(center_y)
            .overflowing_add(picture.height);
        if x_carry && y_carry {
            return Ok(collision_center);
        }

        Ok(top_left.wrapping_add(u16::from_be_bytes([
            picture.width >> 1,
            picture.height >> 1,
        ])))
    }

    pub(super) fn advance_appearance_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        size: u16,
    ) -> Result<RedLabelExpandedUpdate, String> {
        let new_size = size.wrapping_sub(0x0100);
        if new_size & 0x8000 == 0 {
            self.write_appearance_word(layout, slot_address, "RSIZE", new_size)?;
            return self.kill_appearance_slot(layout, slot_address);
        }

        let advanced = self.advance_appearance_slot_geometry(layout, slot_address, size)?;
        let erased_previous_image = self.erase_expanded_slot(layout, slot_address)?;
        self.write_expanded_slot(layout, slot_address, true)?;
        Ok(RedLabelExpandedUpdate::AppearanceAdvanced {
            slot_address,
            object_address: advanced.object_address,
            size: advanced.new_size,
            top_left: advanced.top_left,
            center: advanced.center,
            erased_previous_image,
        })
    }

    pub(super) fn advance_appearance_slot_geometry(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        size: u16,
    ) -> Result<RedLabelAppearanceAdvanceGeometry, String> {
        let new_size = size.wrapping_sub(0x0100);
        self.write_appearance_word(layout, slot_address, "RSIZE", new_size)?;
        if new_size & 0x8000 == 0 {
            return self.kill_appearance_slot_geometry(layout, slot_address, new_size);
        }

        let object_address = self.read_appearance_word(layout, slot_address, "OBJPTR")?;
        let mut relative_x = self
            .read_appearance_object_word(layout, object_address, "OX16")?
            .wrapping_sub(self.read_field_word(layout, "base_page", "BGL")?);
        let [mut relative_x_hi, relative_x_lo] = relative_x.to_be_bytes();
        relative_x_hi = relative_x_hi.wrapping_add(0x0C);
        if relative_x_hi & 0xC0 != 0 {
            return self.kill_appearance_slot_geometry(layout, slot_address, new_size);
        }

        relative_x_hi = relative_x_hi.wrapping_sub(0x0C);
        relative_x = u16::from_be_bytes([relative_x_hi, relative_x_lo]);
        let scaled_x = relative_x.wrapping_shl(2).to_be_bytes()[0];
        let object_y = self
            .read_appearance_object_word(layout, object_address, "OY16")?
            .to_be_bytes()[0];
        let top_left = u16::from_be_bytes([scaled_x, object_y]);
        self.write_appearance_word(layout, slot_address, "TOPLFT", top_left)?;

        let picture_address = self.read_appearance_word(layout, slot_address, "OBDESC")?;
        let (picture_width, picture_height) =
            self.appearance_picture_dimensions(picture_address)?;
        let first_product_high = ((u16::from(scaled_x) * 0x00DA).to_be_bytes()[0]).wrapping_shl(1);
        let second_product_high =
            (u16::from(first_product_high) * u16::from(picture_width)).to_be_bytes()[0];
        let center_offset = u16::from_be_bytes([second_product_high, picture_height >> 1]);
        let center = center_offset.wrapping_add(top_left);
        self.write_appearance_word(layout, slot_address, "CENTER", center)?;

        Ok(RedLabelAppearanceAdvanceGeometry {
            object_address,
            new_size,
            top_left,
            center,
        })
    }

    pub(super) fn kill_appearance_slot_geometry(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        new_size: u16,
    ) -> Result<RedLabelAppearanceAdvanceGeometry, String> {
        let killed = self.kill_appearance_slot(layout, slot_address)?;
        let RedLabelExpandedUpdate::AppearanceKilled { object_address, .. } = killed else {
            return Err(String::from(
                "red-label appearance geometry update killed a non-appearance slot",
            ));
        };
        Ok(RedLabelAppearanceAdvanceGeometry {
            object_address,
            new_size,
            top_left: 0,
            center: 0,
        })
    }

    pub(super) fn advance_explosion_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        size: u16,
    ) -> Result<RedLabelExpandedUpdate, String> {
        let new_size = size.wrapping_add(0x00AA);
        self.write_appearance_word(layout, slot_address, "RSIZE", new_size)?;
        if new_size.to_be_bytes()[0] > 0x30 {
            let erased_previous_image = self.erase_expanded_slot(layout, slot_address)?;
            self.write_appearance_word(layout, slot_address, "RSIZE", 0)?;
            return Ok(RedLabelExpandedUpdate::ExplosionKilled {
                slot_address,
                erased_previous_image,
            });
        }

        let background_left = self.read_field_word(layout, "base_page", "BGL")? & 0xFFC0;
        let previous_background_left = self.read_field_word(layout, "base_page", "BGLX")? & 0xFFC0;
        let scroll_delta = previous_background_left
            .wrapping_sub(background_left)
            .wrapping_shl(2)
            .to_be_bytes()[0];
        let top_left = self
            .read_appearance_word(layout, slot_address, "TOPLFT")?
            .wrapping_add(u16::from_be_bytes([scroll_delta, 0]));
        let center = self
            .read_appearance_word(layout, slot_address, "CENTER")?
            .wrapping_add(u16::from_be_bytes([scroll_delta, 0]));
        self.write_appearance_word(layout, slot_address, "TOPLFT", top_left)?;
        self.write_appearance_word(layout, slot_address, "CENTER", center)?;

        let erased_previous_image = self.erase_expanded_slot(layout, slot_address)?;
        self.write_expanded_slot(layout, slot_address, true)?;
        Ok(RedLabelExpandedUpdate::ExplosionAdvanced {
            slot_address,
            size: new_size,
            top_left,
            center,
            erased_previous_image,
        })
    }

    pub(super) fn kill_appearance_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
    ) -> Result<RedLabelExpandedUpdate, String> {
        let object_address = self.read_appearance_word(layout, slot_address, "OBJPTR")?;
        let restored_picture_address = self.read_appearance_word(layout, slot_address, "OBDESC")?;
        self.write_appearance_word(layout, slot_address, "RSIZE", 0)?;
        self.write_appearance_object_word(
            layout,
            object_address,
            "OPICT",
            restored_picture_address,
        )?;
        let object_type = self.read_appearance_object_byte(layout, object_address, "OTYP")?;
        self.write_appearance_object_byte(layout, object_address, "OTYP", object_type & 0xFD)?;
        let erased_previous_image = self.erase_expanded_slot(layout, slot_address)?;
        Ok(RedLabelExpandedUpdate::AppearanceKilled {
            slot_address,
            object_address,
            restored_picture_address,
            erased_previous_image,
        })
    }

    pub(super) fn read_appearance_object_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
    ) -> Result<u16, String> {
        if raw_attract_defender_object_contains(object_address) {
            return self.read_word(raw_attract_defender_object_field_address(
                object_address,
                field,
            )?);
        }

        object_table_for_address(layout, object_address)?;
        self.read_object_word(layout, object_address, field)
    }

    pub(super) fn write_appearance_object_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        if raw_attract_defender_object_contains(object_address) {
            return self.write_word(
                raw_attract_defender_object_field_address(object_address, field)?,
                value,
            );
        }

        object_table_for_address(layout, object_address)?;
        self.write_object_word(layout, object_address, field, value)
    }

    pub(super) fn read_appearance_object_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
    ) -> Result<u8, String> {
        if raw_attract_defender_object_contains(object_address) {
            return self.read_byte(raw_attract_defender_object_field_address(
                object_address,
                field,
            )?);
        }

        object_table_for_address(layout, object_address)?;
        self.read_object_byte(layout, object_address, field)
    }

    pub(super) fn write_appearance_object_byte(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
        value: u8,
    ) -> Result<(), String> {
        if raw_attract_defender_object_contains(object_address) {
            return self.write_byte(
                raw_attract_defender_object_field_address(object_address, field)?,
                value,
            );
        }

        object_table_for_address(layout, object_address)?;
        self.write_object_bytes(layout, object_address, field, &[value])
    }

    pub(super) fn appearance_picture_dimensions(
        &self,
        picture_address: u16,
    ) -> Result<(u8, u8), String> {
        if raw_attract_defender_picture_contains(picture_address) {
            return Ok((
                self.read_byte(picture_address)?,
                self.read_byte(picture_address + 1)?,
            ));
        }

        let picture = red_label_object_picture(picture_address)?;
        Ok((picture.width, picture.height))
    }

    pub(super) fn allocate_appearance_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<(u16, bool)>, String> {
        let table = table_descriptor(layout, "appearance_ram")?;
        let table_range = table
            .table_range()
            .ok_or_else(|| String::from("red-label appearance RAM table range is invalid"))?;
        let last_slot = self.read_field_word(layout, "base_page", "LSEXPL")?;
        if last_slot != 0 {
            appearance_table_for_address(layout, last_slot)?;
        }

        let mut slot = if last_slot == 0 {
            table.base
        } else {
            last_slot.wrapping_add(table.entry_size)
        };
        if slot == table_range.end {
            slot = table.base;
        }

        for _ in 0..table.entries {
            if last_slot != 0 && slot == last_slot {
                return Ok(None);
            }

            let size = self.read_appearance_word(layout, slot, "RSIZE")?;
            if size.to_be_bytes()[0] & 0x80 == 0 {
                let erased_previous_slot = if size == 0 {
                    false
                } else {
                    self.erase_expanded_slot(layout, slot)?
                };
                return Ok(Some((slot, erased_previous_slot)));
            }

            slot = slot.wrapping_add(table.entry_size);
            if slot == table_range.end {
                slot = table.base;
            }
        }

        Ok(None)
    }

    pub(super) fn erase_expanded_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
    ) -> Result<bool, String> {
        let table = appearance_table_for_address(layout, slot_address)?;
        let slot_end = slot_address.wrapping_add(table.entry_size);
        self.write_field_word(layout, "base_page", "DATPTR", slot_end)?;
        let mut erase_pointer = self.read_appearance_word(layout, slot_address, "ERASES")?;
        if erase_pointer == slot_end {
            return Ok(false);
        }
        if erase_pointer < slot_address
            || erase_pointer > slot_end
            || erase_pointer.wrapping_sub(slot_address) % 2 != 0
        {
            return Err(format!(
                "red-label appearance erase pointer 0x{erase_pointer:04X} is outside slot 0x{slot_address:04X}"
            ));
        }

        while erase_pointer != slot_end {
            let target_address = self.read_word(erase_pointer)?;
            self.write_word(target_address, 0)?;
            erase_pointer = erase_pointer.wrapping_add(2);
        }
        self.write_appearance_word(layout, slot_address, "ERASES", slot_end)?;
        Ok(true)
    }

    pub(super) fn write_expanded_slot(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        clear_center: bool,
    ) -> Result<(), String> {
        let table = appearance_table_for_address(layout, slot_address)?;
        let slot_end = slot_address.wrapping_add(table.entry_size);
        let mut erase_pointer = slot_end;
        let size = self
            .read_appearance_word(layout, slot_address, "RSIZE")?
            .to_be_bytes()[0]
            & 0x7F;
        let double_size = size.wrapping_shl(1);
        let picture_address = self.read_appearance_word(layout, slot_address, "OBDESC")?;
        let (picture, mut data_pointer, mut width_counter, mut length_counter) =
            if raw_attract_defender_picture_contains(picture_address) {
                (
                    None,
                    self.read_word(picture_address + 2)?,
                    self.read_byte(picture_address)?,
                    self.read_byte(picture_address + 1)?,
                )
            } else {
                let picture = red_label_object_picture(picture_address)?;
                (
                    Some(picture),
                    picture.primary_image,
                    picture.width,
                    picture.height,
                )
            };
        let [center_x, center_y] = self
            .read_appearance_word(layout, slot_address, "CENTER")?
            .to_be_bytes();
        let [top_left_x, top_left_y] = self
            .read_appearance_word(layout, slot_address, "TOPLFT")?
            .to_be_bytes();
        let x_offset = center_x.wrapping_sub(top_left_x);
        let y_offset_raw = center_y.wrapping_sub(top_left_y);
        let flavor = y_offset_raw & 1;
        let y_offset = y_offset_raw >> 1;
        let mut x_start = u16::from(center_x).wrapping_sub(u16::from(size) * u16::from(x_offset));

        while x_start.to_be_bytes()[0] != 0 {
            data_pointer = data_pointer.wrapping_add(u16::from(length_counter));
            width_counter = width_counter.wrapping_sub(1);
            if width_counter == 0 {
                self.write_appearance_word(layout, slot_address, "ERASES", erase_pointer)?;
                return Ok(());
            }
            x_start = x_start.wrapping_add(u16::from(size));
        }

        let [_, mut screen_x] = x_start.to_be_bytes();
        if screen_x > 0x98 {
            self.write_appearance_word(layout, slot_address, "ERASES", erase_pointer)?;
            return Ok(());
        }

        let y_scaled = u16::from(double_size) * u16::from(y_offset);
        let mut y_start = u16::from(center_y).wrapping_sub(y_scaled);
        let [mut y_start_high, mut y_start_low] = y_start.to_be_bytes();
        let (adjusted_y_low, flavor_borrow) = y_start_low.overflowing_sub(flavor);
        y_start_low = adjusted_y_low;
        y_start_high = y_start_high.wrapping_add(u8::from(flavor_borrow));
        let mut y_skip = 0u8;

        loop {
            if y_start_high == 0 && y_start_low >= 0x2A {
                break;
            }

            y_skip = y_skip.wrapping_add(1);
            length_counter = length_counter.wrapping_sub(1);
            length_counter = length_counter.wrapping_sub(1);
            if (length_counter as i8) <= 0 {
                self.write_appearance_word(layout, slot_address, "ERASES", erase_pointer)?;
                return Ok(());
            }
            let (next_y_low, carry) = y_start_low.overflowing_add(double_size);
            y_start_low = next_y_low;
            y_start_high = y_start_high.wrapping_add(u8::from(carry));
        }
        y_start = u16::from_be_bytes([y_start_high, y_start_low]);
        self.write_field_word(layout, "base_page", "XSTART", x_start)?;

        let y_skip_bytes = y_skip.wrapping_shl(1);
        while width_counter != 0 {
            data_pointer = data_pointer.wrapping_add(u16::from(y_skip_bytes));
            let mut source_pointer = data_pointer;
            let mut screen_y = y_start.to_be_bytes()[1];
            let mut remaining = length_counter;

            while remaining >= 2 {
                erase_pointer = self.store_expanded_erase_address(
                    layout,
                    slot_address,
                    erase_pointer,
                    u16::from_be_bytes([screen_x, screen_y]),
                )?;
                let image_word = match picture {
                    Some(picture) => red_label_object_image_word(source_pointer, picture)?,
                    None => self.read_word(source_pointer)?,
                };
                let target_address = u16::from_be_bytes([screen_x, screen_y]);
                self.write_word(target_address, image_word)?;
                self.record_live_expanded_object_address(target_address, 2);
                source_pointer = source_pointer.wrapping_add(2);
                remaining = remaining.wrapping_sub(2);
                let (next_y, carry) = screen_y.overflowing_add(double_size);
                screen_y = next_y;
                if carry {
                    source_pointer = source_pointer.wrapping_add(u16::from(remaining));
                    remaining = 0;
                    break;
                }
            }

            if remaining == 1 {
                erase_pointer = self.store_expanded_erase_address(
                    layout,
                    slot_address,
                    erase_pointer,
                    u16::from_be_bytes([screen_x, screen_y]),
                )?;
                let image_byte = match picture {
                    Some(picture) => red_label_object_image_byte_required(source_pointer, picture)?,
                    None => self.read_byte(source_pointer)?,
                };
                let target_address = u16::from_be_bytes([screen_x, screen_y]);
                self.write_byte(target_address, image_byte)?;
                self.record_live_expanded_object_address(target_address, 1);
                source_pointer = source_pointer.wrapping_add(1);
            }

            data_pointer = source_pointer;
            width_counter = width_counter.wrapping_sub(1);
            if width_counter == 0 {
                break;
            }
            let (next_screen_x, x_carry) = screen_x.overflowing_add(size);
            screen_x = next_screen_x;
            if x_carry || screen_x > 0x98 {
                break;
            }
        }

        self.write_appearance_word(layout, slot_address, "ERASES", erase_pointer)?;
        if clear_center && center_x <= 0x98 {
            self.write_word(
                u16::from_be_bytes([center_x, center_y.wrapping_sub(flavor)]),
                0,
            )?;
        }
        Ok(())
    }

    pub(super) fn record_live_expanded_object_address(&mut self, address: u16, width: u8) {
        let entry = (address, width);
        if !self.live_expanded_object_addresses.contains(&entry) {
            self.live_expanded_object_addresses.push(entry);
        }
    }

    pub(super) fn store_expanded_erase_address(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        erase_pointer: u16,
        screen_address: u16,
    ) -> Result<u16, String> {
        let table = appearance_table_for_address(layout, slot_address)?;
        let next_pointer = erase_pointer.wrapping_sub(2);
        if next_pointer < slot_address {
            return Err(format!(
                "red-label EWRITE erase table for slot 0x{slot_address:04X} overflowed"
            ));
        }
        let slot_end = slot_address.wrapping_add(table.entry_size);
        if next_pointer >= slot_end {
            return Err(format!(
                "red-label EWRITE erase pointer 0x{next_pointer:04X} is outside slot 0x{slot_address:04X}"
            ));
        }
        self.write_word(next_pointer, screen_address)?;
        Ok(next_pointer)
    }

    pub(super) fn write_status_from_astcnt(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        base_status: u8,
    ) -> Result<u8, String> {
        let mut status = base_status;
        if self.read_field_byte(layout, "base_page", "ASTCNT")? == 0 {
            status |= 0x02;
        }
        self.write_field_byte(layout, "base_page", "STATUS", status)?;
        Ok(status)
    }

    pub(super) fn start_player_explosion_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_center: u16,
        return_address: u16,
    ) -> Result<u16, String> {
        let process_address = self.current_process_address(layout)?;
        self.write_process_data_word(layout, process_address, "PD2", return_address)?;
        self.write_field_word(layout, "player_explosion_state", "PCENT", player_center)?;
        self.write_field_word(layout, "player_explosion_state", "PSED", 0x0808)?;
        self.write_field_word(layout, "player_explosion_state", "PSED2", 0x1732)?;

        let table = table_descriptor(layout, "player_explosion_table")?;
        if table.entries != RED_LABEL_PLAYER_EXPLOSION_PIECES {
            return Err(String::from(
                "red-label player explosion table must contain 128 pieces",
            ));
        }

        for entry_index in 0..table.entries {
            let piece_address = table.base + entry_index * table.entry_size;
            self.initialize_player_explosion_piece(layout, piece_address, player_center)?;
        }

        self.write_field_word(
            layout,
            "player_explosion_state",
            "PCOLP",
            red_label_player_death_table("PXCOL")?.address,
        )?;
        self.write_field_byte(layout, "player_explosion_state", "PCOLC", 56)?;
        let wakeup_address = red_label_routine_address("PX1A")?;
        self.sleep_current_process(1, wakeup_address)?;
        Ok(wakeup_address)
    }

    pub(super) fn initialize_player_explosion_piece(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        piece_address: u16,
        player_center: u16,
    ) -> Result<(), String> {
        loop {
            let [center_x, center_y] = player_center.to_be_bytes();
            self.write_player_explosion_word(
                layout,
                piece_address,
                "PXPOST",
                u16::from_be_bytes([center_x, 0]),
            )?;
            self.write_player_explosion_word(
                layout,
                piece_address,
                "PYPOST",
                u16::from_be_bytes([center_y, 0]),
            )?;

            let x_velocity = self.next_player_explosion_x_velocity(layout)?;
            self.write_player_explosion_word(layout, piece_address, "PXVELT", x_velocity)?;
            let absolute_x_velocity = ones_complement_abs_word(x_velocity);

            let y_velocity = self.next_player_explosion_y_velocity(layout)?;
            self.write_player_explosion_word(layout, piece_address, "PYVELT", y_velocity)?;
            let half_absolute_y_velocity =
                logical_shift_right_word(ones_complement_abs_word(y_velocity));

            if absolute_x_velocity.wrapping_add(half_absolute_y_velocity) < 0x016A {
                self.write_player_explosion_word(layout, piece_address, "PSCR", 0)?;
                return Ok(());
            }
        }
    }

    pub(super) fn next_player_explosion_x_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let seed = self.read_field_word(layout, "player_explosion_state", "PSED")?;
        let (next_seed, carry_after_rotate) = player_explosion_random_seed_step(seed);
        self.write_field_word(layout, "player_explosion_state", "PSED", next_seed)?;
        let velocity_high = (next_seed.to_be_bytes()[0] & 1).wrapping_sub(1);
        let velocity_low = next_seed.to_be_bytes()[1];
        let _ = carry_after_rotate;
        Ok(u16::from_be_bytes([velocity_high, velocity_low]))
    }

    pub(super) fn next_player_explosion_y_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let seed = self.read_field_word(layout, "player_explosion_state", "PSED2")?;
        let (next_seed, carry_after_rotate) = player_explosion_random_seed_step(seed);
        self.write_field_word(layout, "player_explosion_state", "PSED2", next_seed)?;
        let velocity_high = (next_seed.to_be_bytes()[0] & 3).wrapping_sub(2);
        let velocity_low = next_seed.to_be_bytes()[1];
        let _ = carry_after_rotate;
        Ok(u16::from_be_bytes([velocity_high, velocity_low]))
    }

    pub(super) fn player_death_bonus_text_plan(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelBonusTextPlan, String> {
        let wave = self.current_player_wave_from_plrx(layout)?;
        let attack_wave = red_label_message("ATWV")?;
        let completed = red_label_message("COMPV")?;
        let bonus_multiplier = red_label_message("BONSX")?;
        self.write_message_text_block(layout, 0x3850, attack_wave)?;
        self.write_message_number_block(layout, decimal_to_bcd_byte(wave))?;
        self.write_message_text_block(layout, 0x3D60, completed)?;
        self.write_message_text_block(layout, 0x3C90, bonus_multiplier)?;
        self.write_message_number_block(layout, decimal_to_bcd_byte(wave.min(5)))?;

        Ok(RedLabelBonusTextPlan {
            attack_wave: RedLabelBonusTextCall {
                vector_address: attack_wave.vector_address,
                screen_address: 0x3850,
            },
            wave_bcd: decimal_to_bcd_byte(wave),
            completed: RedLabelBonusTextCall {
                vector_address: completed.vector_address,
                screen_address: 0x3D60,
            },
            bonus_multiplier: RedLabelBonusTextCall {
                vector_address: bonus_multiplier.vector_address,
                screen_address: 0x3C90,
            },
            multiplier_bcd: decimal_to_bcd_byte(wave.min(5)),
        })
    }

    /// Source-shaped `HALDIS`: clear the active screen, draw the score
    /// display, hall-of-fame headings, underline bars, two high-score tables,
    /// and the expanded `DEFNNN` Defender logo.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L379-L467>.
    pub fn write_hall_of_fame_display(&mut self) -> Result<RedLabelHallOfFameDisplay, String> {
        let layout = red_label_ram_layout()?;
        let screen_clear = self.clear_screen_ram()?;
        let high_score_reset_flag_address = RED_LABEL_HOF_RESET_FLAG_RAM;
        self.write_byte(high_score_reset_flag_address, 0)?;

        let black_letters_address = field_range(&layout, "base_page", "PCRAM")?.start + 1;
        self.write_byte(black_letters_address, 0)?;
        let score_transfers = self.display_player_scores(&layout)?;
        let headings = self.write_hall_of_fame_headings(&layout)?;
        let underline_words = self.write_hall_of_fame_underlines()?;
        let todays_table = self.write_hall_of_fame_table(
            &layout,
            RuntimeHighScoreTable::TodaysGreatest,
            RED_LABEL_HALL_OF_FAME_TODAYS_TABLE_SCREEN,
        )?;
        let all_time_table = self.write_hall_of_fame_table(
            &layout,
            RuntimeHighScoreTable::AllTime,
            RED_LABEL_HALL_OF_FAME_ALL_TIME_TABLE_SCREEN,
        )?;

        let logo_color_address = field_range(&layout, "base_page", "PCRAM")?.start + 0x0C;
        self.write_byte(logo_color_address, RED_LABEL_HALL_OF_FAME_LOGO_COLOR)?;
        let logo = self.write_hall_of_fame_defender_logo()?;
        self.write_byte(
            RED_LABEL_HOF_STALL_TIMER_RAM,
            RED_LABEL_HALL_OF_FAME_STALL_TICKS,
        )?;

        Ok(RedLabelHallOfFameDisplay {
            screen_clear,
            high_score_reset_flag_address,
            black_letters_address,
            score_transfers,
            headings,
            underline_words,
            todays_table,
            all_time_table,
            logo_color_address,
            logo,
            stall_ticks: RED_LABEL_HALL_OF_FAME_STALL_TICKS,
        })
    }

    /// Source-shaped `SCORES`: redraw player score fields by calling `SCRTR0`
    /// for players `PLRCNT` down to one.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L1003-L1009>.
    pub(super) fn display_player_scores(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Vec<RedLabelScoreTransfer>, String> {
        let player_count = self.read_field_byte(layout, "base_page", "PLRCNT")?;
        let mut score_transfers = Vec::with_capacity(usize::from(player_count));
        let mut player_number = player_count;
        while player_number != 0 {
            score_transfers.push(self.transfer_score_digits(layout, player_number)?);
            player_number = player_number.wrapping_sub(1);
        }
        Ok(score_transfers)
    }

    pub(super) fn write_hall_of_fame_headings(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<[RedLabelBonusTextCall; 5], String> {
        let title = red_label_message("HALLD_TITLE")?;
        let todays = red_label_message("HALLD_TODAYS")?;
        let all_time = red_label_message("HALLD_ALL_TIME")?;
        let left_greatest = red_label_message("HALLD_GREATEST")?;
        let right_greatest = red_label_message("HALLD_GREATEST")?;
        let calls = [
            (title, RED_LABEL_HALL_OF_FAME_HEADINGS_SCREEN),
            (todays, RED_LABEL_HALL_OF_FAME_TODAYS_SCREEN),
            (all_time, RED_LABEL_HALL_OF_FAME_ALL_TIME_SCREEN),
            (left_greatest, RED_LABEL_HALL_OF_FAME_LEFT_GREATEST_SCREEN),
            (right_greatest, RED_LABEL_HALL_OF_FAME_RIGHT_GREATEST_SCREEN),
        ];
        let mut text_calls = [RedLabelBonusTextCall {
            vector_address: 0,
            screen_address: 0,
        }; 5];
        for (index, (message, screen_address)) in calls.iter().enumerate() {
            self.write_message_text_block(layout, *screen_address, message)?;
            text_calls[index] = RedLabelBonusTextCall {
                vector_address: message.vector_address,
                screen_address: *screen_address,
            };
        }
        Ok(text_calls)
    }

    pub(super) fn write_hall_of_fame_underlines(&mut self) -> Result<Vec<u16>, String> {
        let mut underline_words = Vec::new();
        let mut offset_high = 0x5Fu8;
        loop {
            let address = screen_offset(
                RED_LABEL_HALL_OF_FAME_UNDERLINE_LEFT,
                u16::from(offset_high) << 8,
            )?;
            self.write_word(address, RED_LABEL_HOF_UNDERLINE_NORMAL)?;
            underline_words.push(address);
            if offset_high == 0x41 {
                offset_high = 0x1F;
            }
            offset_high = offset_high.wrapping_sub(1);
            if offset_high & 0x80 != 0 {
                break;
            }
        }
        Ok(underline_words)
    }

    pub(super) fn write_hall_of_fame_table(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: RuntimeHighScoreTable,
        start_screen_address: u16,
    ) -> Result<RedLabelHallOfFameTableDisplay, String> {
        self.write_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM + 1, b'/')?;
        self.write_byte(RED_LABEL_HOF_TABLE_INITIALS_RAM + 3, b'/')?;
        self.write_byte(RED_LABEL_HOF_TABLE_SCORE_RAM + 6, b'/')?;
        let mut rows = Vec::with_capacity(RED_LABEL_HIGH_SCORE_ENTRIES);
        let mut screen_address = start_screen_address;
        for index in 0..RED_LABEL_HIGH_SCORE_ENTRIES {
            let rank = u8::try_from(index + 1).expect("red-label HALL table rank fits in u8");
            rows.push(self.write_hall_of_fame_table_row(
                layout,
                table,
                index,
                rank,
                screen_address,
            )?);
            screen_address = text_position_from_top_left(
                screen_address,
                0,
                u8::try_from(RED_LABEL_HALL_OF_FAME_TABLE_ROW_STEP)
                    .expect("red-label HALL row step fits in u8"),
            );
        }
        Ok(RedLabelHallOfFameTableDisplay {
            table: table.kind(),
            start_screen_address,
            rows,
        })
    }

    pub(super) fn write_hall_of_fame_table_row(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: RuntimeHighScoreTable,
        index: usize,
        rank: u8,
        rank_address: u16,
    ) -> Result<RedLabelHallOfFameTableRow, String> {
        let entry = self.high_score_entry(table, index)?;
        let score_chars = hall_of_fame_score_chars(entry.score)?;
        let rank_byte = b'0'.wrapping_add(rank);
        self.write_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM, rank_byte)?;
        for (initial_index, initial) in entry.initials.iter().enumerate() {
            self.write_byte(
                RED_LABEL_HOF_TABLE_INITIALS_RAM
                    + u16::try_from(initial_index).expect("initial index fits in u16"),
                *initial,
            )?;
        }
        for (score_index, score_char) in score_chars.iter().enumerate() {
            self.write_byte(
                RED_LABEL_HOF_TABLE_SCORE_RAM
                    + u16::try_from(score_index).expect("score index fits in u16"),
                *score_char,
            )?;
        }

        let initials_address =
            text_position_from_top_left(rank_address, RED_LABEL_HALL_OF_FAME_INITIALS_OFFSET, 0);
        let score_address =
            text_position_from_top_left(rank_address, RED_LABEL_HALL_OF_FAME_SCORE_OFFSET, 0);
        self.write_text_bytes_with_space(layout, rank_address, &[rank_byte])?;
        self.write_text_bytes_with_space(layout, initials_address, &entry.initials)?;
        self.write_text_bytes_with_space(layout, score_address, &score_chars)?;

        Ok(RedLabelHallOfFameTableRow {
            rank,
            rank_address,
            initials_address,
            score_address,
            score_chars,
            entry: RedLabelHallOfFameEntry {
                score: entry.score,
                initials: entry.initials,
            },
        })
    }

    pub(super) fn write_hall_of_fame_defender_logo(
        &mut self,
    ) -> Result<RedLabelPictureWrite, String> {
        let logo = expanded_defender_logo_image();
        self.write_word(
            RED_LABEL_HALL_OF_FAME_LOGO_DESCRIPTOR,
            u16::from_be_bytes([
                RED_LABEL_HALL_OF_FAME_LOGO_WIDTH,
                RED_LABEL_HALL_OF_FAME_LOGO_HEIGHT,
            ]),
        )?;
        self.write_word(
            RED_LABEL_HALL_OF_FAME_LOGO_DESCRIPTOR + 2,
            RED_LABEL_HALL_OF_FAME_LOGO_DATA_RAM,
        )?;
        let logo_ram_range = checked_defender_logo_ram_range(self.ram.len())?;
        self.ram[logo_ram_range].copy_from_slice(&logo);

        for column in 0..RED_LABEL_HALL_OF_FAME_LOGO_WIDTH {
            let column_address =
                screen_offset(RED_LABEL_HALL_OF_FAME_LOGO_SCREEN, u16::from(column) << 8)?;
            let source_column =
                usize::from(column) * usize::from(RED_LABEL_HALL_OF_FAME_LOGO_HEIGHT);
            for row in 0..RED_LABEL_HALL_OF_FAME_LOGO_HEIGHT {
                self.write_byte(
                    screen_offset(column_address, u16::from(row))?,
                    logo[source_column + usize::from(row)],
                )?;
            }
        }
        Ok(RedLabelPictureWrite {
            screen_address: RED_LABEL_HALL_OF_FAME_LOGO_SCREEN,
            picture_address: RED_LABEL_HALL_OF_FAME_LOGO_DATA_RAM,
            width: RED_LABEL_HALL_OF_FAME_LOGO_WIDTH,
            height: RED_LABEL_HALL_OF_FAME_LOGO_HEIGHT,
        })
    }

    /// Source-shaped `HALLOF` screen setup through the `HOFIN`/`HOFUL`
    /// initials draw. The input process timing is still handled by the live
    /// high-score state machine; these writes keep the verified cabinet frame
    /// backed by red-label video RAM while entry is active.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L141-L187>.
    pub(super) fn write_high_score_entry_display(
        &mut self,
        player: u8,
        state: HighScoreEntryState,
    ) -> Result<RedLabelHighScoreEntryDisplay, String> {
        let layout = red_label_ram_layout()?;
        let screen_clear = self.clear_screen_ram()?;
        self.write_byte(RED_LABEL_HOF_PLAYER_NUMBER_RAM, player)?;
        let letter_color_address = field_range(&layout, "base_page", "PCRAM")?.start + 1;
        self.write_byte(letter_color_address, RED_LABEL_HOF_LETTER_COLOR)?;
        self.copy_color_mapping_to_palette_ram(&layout)?;

        let player_message = if player == 2 {
            red_label_message("PLYR2")?
        } else {
            red_label_message("PLYR1")?
        };
        self.write_message_text_block(&layout, RED_LABEL_HOF_PLAYER_LABEL_SCREEN, player_message)?;
        let player_label = RedLabelBonusTextCall {
            vector_address: player_message.vector_address,
            screen_address: RED_LABEL_HOF_PLAYER_LABEL_SCREEN,
        };

        let hof_messages = [
            red_label_message("HOFV1")?,
            red_label_message("HOFV2")?,
            red_label_message("HOFV3")?,
            red_label_message("HOFV4")?,
        ];
        let mut instruction_lines = [RedLabelBonusTextCall {
            vector_address: 0,
            screen_address: 0,
        }; 4];
        for (index, message) in hof_messages.iter().enumerate() {
            let screen_address = text_position_from_top_left(
                RED_LABEL_HOF_INSTRUCTIONS_TOP_LEFT,
                0,
                RED_LABEL_HOF_LINE_VERTICAL_OFFSETS[index],
            );
            self.write_message_text_block(&layout, screen_address, message)?;
            instruction_lines[index] = RedLabelBonusTextCall {
                vector_address: message.vector_address,
                screen_address,
            };
        }

        Ok(RedLabelHighScoreEntryDisplay {
            player,
            screen_clear,
            letter_color_address,
            letter_color: RED_LABEL_HOF_LETTER_COLOR,
            player_label,
            instruction_lines,
            initials: self.write_high_score_initials_display(state)?,
        })
    }

    /// Source-shaped `HOFIN` plus `HOFUL`: refresh the three displayed
    /// initials, then draw the source underline words for the active initial.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/amode1.src#L246-L269>.
    pub(super) fn write_high_score_initials_display(
        &mut self,
        state: HighScoreEntryState,
    ) -> Result<RedLabelHighScoreInitialsDisplay, String> {
        self.write_high_score_initial_words(state)?;
        self.write_high_score_initials_display_from_current_ram(state.cursor)
    }

    pub(super) fn write_high_score_initials_display_from_current_ram(
        &mut self,
        active_initial: u8,
    ) -> Result<RedLabelHighScoreInitialsDisplay, String> {
        let block_clear = self.block_clear(
            RED_LABEL_HOF_INITIALS_SCREEN,
            RED_LABEL_HOF_INITIALS_BLOCK_WIDTH,
            RED_LABEL_HOF_INITIALS_BLOCK_HEIGHT,
        )?;
        let mut initial_addresses = [0; RED_LABEL_INITIALS_ENTRY_CHARS];
        for (index, initial_screen_address) in initial_addresses.iter_mut().enumerate() {
            let initial_address = RED_LABEL_HOF_INITS_RAM
                + u16::try_from(index * 2).expect("initial index should fit in u16");
            let initial = high_score_initial_display_byte(self.read_byte(initial_address)?);
            let screen_address = text_position_from_top_left(
                RED_LABEL_HOF_INITIALS_SCREEN,
                RED_LABEL_HOF_INITIAL_HORIZONTAL_OFFSETS[index],
                0,
            );
            let glyph = red_label_message_glyph(char::from(initial))?;
            self.write_message_glyph(screen_address, glyph)?;
            *initial_screen_address = screen_address;
        }

        Ok(RedLabelHighScoreInitialsDisplay {
            block_clear,
            initial_addresses,
            active_initial,
            underline_words: self.write_high_score_initial_underlines(active_initial)?,
        })
    }

    pub(super) fn write_high_score_initial_words(
        &mut self,
        state: HighScoreEntryState,
    ) -> Result<(), String> {
        for (index, initial) in state.initials.iter().enumerate() {
            let address = RED_LABEL_HOF_INITS_RAM
                + u16::try_from(index * 2).expect("initial index should fit in u16");
            self.write_byte(address, *initial)?;
            self.write_byte(address + 1, b'/')?;
        }
        self.write_byte(RED_LABEL_HOF_INIT_INDEX_RAM, state.cursor)
    }

    pub(super) fn write_high_score_initial_underlines(
        &mut self,
        active_initial: u8,
    ) -> Result<[[u16; 4]; RED_LABEL_INITIALS_ENTRY_CHARS], String> {
        let mut underline_words = [[0; 4]; RED_LABEL_INITIALS_ENTRY_CHARS];
        for (initial_index, words) in underline_words.iter_mut().enumerate() {
            let initial_base = RED_LABEL_HOF_UNDERLINE_START
                + u16::try_from(initial_index).expect("initial index should fit in u16")
                    * RED_LABEL_HOF_UNDERLINE_INITIAL_STEP;
            let color = if initial_index == usize::from(active_initial) {
                RED_LABEL_HOF_UNDERLINE_ACTIVE
            } else {
                RED_LABEL_HOF_UNDERLINE_NORMAL
            };
            for (word_index, offset) in RED_LABEL_HOF_UNDERLINE_WORD_OFFSETS.iter().enumerate() {
                let address = screen_offset(initial_base, *offset)?;
                self.write_word(address, color)?;
                words[word_index] = address;
            }
        }
        Ok(underline_words)
    }

    pub(super) fn write_message_text_block(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        screen_address: u16,
        message: &RedLabelMessage,
    ) -> Result<u16, String> {
        let mut text_layout = RedLabelMessageTextLayout {
            top_left: screen_address,
            cursor: screen_address,
            line_spacing: 0x0A,
        };
        self.write_field_word(layout, "base_page", "CURSER", text_layout.cursor)?;
        for word in &message.words {
            if let Some(control) = red_label_message_control(word)? {
                text_layout.apply(control);
                self.write_field_word(layout, "base_page", "CURSER", text_layout.cursor)?;
                continue;
            }
            text_layout.cursor =
                self.write_text_bytes_with_space(layout, text_layout.cursor, word.as_bytes())?;
        }
        self.write_field_word(layout, "base_page", "CURSER", text_layout.cursor)?;
        Ok(text_layout.cursor)
    }

    pub(super) fn write_text_bytes_with_space(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        screen_address: u16,
        bytes: &[u8],
    ) -> Result<u16, String> {
        let mut cursor = screen_address;
        self.write_field_word(layout, "base_page", "CURSER", cursor)?;
        for byte in bytes {
            cursor = self.write_text_byte(cursor, *byte)?;
        }
        cursor = self.write_text_byte(cursor, b' ')?;
        self.write_field_word(layout, "base_page", "CURSER", cursor)?;
        Ok(cursor)
    }

    pub(super) fn write_text_byte(&mut self, screen_address: u16, byte: u8) -> Result<u16, String> {
        if byte.is_ascii_digit() {
            let image = red_label_score_digit_image(byte - b'0')?;
            self.write_score_digit_image(screen_address, image)?;
            return Ok(text_cursor_advance(screen_address, image.width));
        }

        let glyph = red_label_message_glyph(char::from(byte))?;
        self.write_message_glyph(screen_address, glyph)?;
        Ok(text_cursor_advance(screen_address, glyph.width))
    }

    pub(super) fn write_message_number_block(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        bcd_number: u8,
    ) -> Result<(), String> {
        let mut cursor = self.read_field_word(layout, "base_page", "CURSER")?;
        let digits = bcd_number_visible_digits(bcd_number);
        for digit in digits {
            let image = red_label_score_digit_image(digit)?;
            self.write_score_digit_image(cursor, image)?;
            cursor = text_cursor_advance(cursor, image.width);
        }
        self.write_field_word(layout, "base_page", "CURSER", cursor)
    }

    pub(super) fn write_message_glyph(
        &mut self,
        screen_address: u16,
        glyph: &RedLabelMessageGlyphImage,
    ) -> Result<(), String> {
        for column in 0..glyph.width {
            let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = usize::from(column) * usize::from(glyph.height);
            for row in 0..glyph.height {
                let source_byte = glyph.bytes[source_column + usize::from(row)];
                self.write_byte(screen_offset(column_address, u16::from(row))?, source_byte)?;
            }
        }
        Ok(())
    }

    pub(super) fn draw_bonus_astronaut_and_sleep(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        return_address: u16,
        intro: RedLabelBonusIntro,
        astronaut_screen_address: u16,
        astronaut_counter: u8,
    ) -> Result<RedLabelPlayerDeath, String> {
        let picture_address = red_label_object_picture_address("ASTP3")?;
        let astronaut =
            self.write_object_picture_primary(astronaut_screen_address, picture_address)?;
        let next_astronaut_screen_address =
            astronaut_screen_address.wrapping_add(RED_LABEL_BONUS_ASTRO_SCREEN_STEP);
        let score_word = self.player_death_bonus_score_word(layout)?;
        let score = self.score_current_player(score_word)?;
        self.write_process_data_word(layout, process_address, "PD", next_astronaut_screen_address)?;
        let wakeup_address = red_label_routine_address("BC1")?;
        self.sleep_current_process(4, wakeup_address)?;
        Ok(RedLabelPlayerDeath::PostExplosionBonusAstronautSleeping {
            process_address,
            return_address,
            screen_clear: intro.screen_clear,
            text: intro.text,
            astronaut_counter,
            astronaut,
            next_astronaut_screen_address,
            score_word,
            score,
            wakeup_address,
        })
    }

    pub(super) fn sleep_player_death_bonus_wave_advance(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        return_address: u16,
        previous_astronaut_counter: u8,
    ) -> Result<RedLabelPlayerDeath, String> {
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let wave = self.get_new_wave_parameters_for_player_address(player_address)?;
        let wakeup_address = red_label_routine_address("BC3")?;
        self.sleep_current_process(0x80, wakeup_address)?;
        Ok(RedLabelPlayerDeath::PostExplosionBonusWaveAdvanceSleeping {
            process_address,
            return_address,
            previous_astronaut_counter,
            wave,
            wakeup_address,
        })
    }

    pub(super) fn player_death_bonus_score_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let multiplier = self.current_player_wave_from_plrx(layout)?.min(5);
        Ok(u16::from_be_bytes([0x01, multiplier << 4]))
    }

    pub(super) fn current_player_wave_from_plrx(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u8, String> {
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let player_table = table_descriptor(layout, "player")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let wave_range = player_field_range_for_entry(layout, player_index, "PWAV")?;
        self.read_byte(wave_range.start)
    }

    pub(super) fn finish_player_death_ship_branch(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        sound_command: SoundCommand,
    ) -> Result<RedLabelPlayerDeath, String> {
        let current_player = self.read_field_byte(layout, "base_page", "CURPLR")?;
        let current_player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let current_lives = self.player_lives_at_address(layout, current_player_address)?;
        if current_lives != 0 {
            return self.jump_to_next_player_with_ships(layout, process_address);
        }

        let player_count_after_decrement = self
            .read_field_byte(layout, "base_page", "PLRCNT")?
            .wrapping_sub(1);
        if player_count_after_decrement == 0 {
            return self.sleep_player_death_game_over(layout, process_address, sound_command);
        }

        let other_player = current_player ^ 0x03;
        if self.player_lives_for_number(layout, other_player)? == 0 {
            return self.sleep_player_death_game_over(layout, process_address, sound_command);
        }

        let player_label_message = if current_player == 2 {
            red_label_message("PLYR2")?
        } else {
            red_label_message("PLYR1")?
        };
        self.write_message_text_block(
            layout,
            RED_LABEL_PLAYER_SWITCH_LABEL_SCREEN,
            player_label_message,
        )?;
        let game_over_message = red_label_message("GO")?;
        self.write_message_text_block(
            layout,
            RED_LABEL_PLAYER_SWITCH_GAME_OVER_SCREEN,
            game_over_message,
        )?;

        let wakeup_address = red_label_routine_address("PLE02")?;
        self.sleep_current_process(0x60, wakeup_address)?;
        Ok(RedLabelPlayerDeath::PostExplosionSwitchPlayerSleeping {
            process_address,
            other_player,
            player_label: RedLabelBonusTextCall {
                vector_address: player_label_message.vector_address,
                screen_address: RED_LABEL_PLAYER_SWITCH_LABEL_SCREEN,
            },
            game_over: RedLabelBonusTextCall {
                vector_address: game_over_message.vector_address,
                screen_address: RED_LABEL_PLAYER_SWITCH_GAME_OVER_SCREEN,
            },
            wakeup_address,
        })
    }

    pub(super) fn jump_to_next_player_with_ships(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
    ) -> Result<RedLabelPlayerDeath, String> {
        let player_count = self.read_field_byte(layout, "base_page", "PLRCNT")?;
        let mut next_player = self.read_field_byte(layout, "base_page", "CURPLR")?;
        for _ in 0..player_count {
            next_player = next_player.wrapping_add(1);
            if next_player > player_count {
                next_player = 1;
            }
            if self.player_lives_for_number(layout, next_player)? != 0 {
                self.write_field_byte(layout, "base_page", "CURPLR", next_player)?;
                let pdf_flag = self.read_field_byte(layout, "base_page", "PDFLG")?;
                self.write_field_byte(layout, "base_page", "PDFLG", pdf_flag.wrapping_add(1))?;
                let player_start_address = red_label_routine_address("PLSTRT")?;
                self.write_process_word(layout, process_address, "PADDR", player_start_address)?;
                return Ok(RedLabelPlayerDeath::PostExplosionRespawnJump {
                    process_address,
                    next_player,
                    player_start_address,
                });
            }
        }

        self.sleep_player_death_game_over(
            layout,
            process_address,
            red_label_sound_output_command(RED_LABEL_PLAYER_END_SOUND_STOP_NUMBER),
        )
    }

    pub(super) fn sleep_player_death_game_over(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        sound_command: SoundCommand,
    ) -> Result<RedLabelPlayerDeath, String> {
        let status = 0xFF;
        self.write_field_byte(layout, "base_page", "STATUS", status)?;
        self.write_message_text_block(layout, 0x3E80, red_label_message("GO")?)?;
        self.write_field_byte(layout, "base_page", "SNDTMR", 0)?;
        let wakeup_address = red_label_routine_address("PLE3")?;
        self.sleep_current_process(40, wakeup_address)?;
        Ok(RedLabelPlayerDeath::GameOverSleeping {
            process_address,
            status,
            sound_command,
            wakeup_address,
        })
    }

    pub(super) fn player_lives_for_number(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        player_number: u8,
    ) -> Result<u8, String> {
        let player_address = self.player_address_for_number(layout, player_number)?;
        self.player_lives_at_address(layout, player_address)
    }

    pub(super) fn player_lives_at_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        player_address: u16,
    ) -> Result<u8, String> {
        let player_table = table_descriptor(layout, "player")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let lives_range = player_field_range_for_entry(layout, player_index, "PLAS")?;
        self.read_byte(lives_range.start)
    }

    pub(super) fn player_address_for_number(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        player_number: u8,
    ) -> Result<u16, String> {
        if player_number == 0 {
            return Err(String::from("red-label player number is one-based"));
        }
        let player_table = table_descriptor(layout, "player")?;
        let player_index = u16::from(player_number - 1);
        table_entry_range(player_table, player_index).map(|range| range.start)
    }

    pub(super) fn erase_player_explosion_piece(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        piece_address: u16,
    ) -> Result<(), String> {
        let screen_address = self.read_player_explosion_word(layout, piece_address, "PSCR")?;
        self.write_word(screen_address, 0)?;
        self.write_word(screen_address.wrapping_add(0x0100), 0)
    }

    pub(super) fn write_player_explosion_piece(
        &mut self,
        screen_address: u16,
        x_fraction: u8,
    ) -> Result<(), String> {
        if x_fraction & 0x80 == 0 {
            self.write_word(screen_address, 0xBBBB)
        } else {
            self.write_word(screen_address, 0x0B0B)?;
            self.write_word(screen_address.wrapping_add(0x0100), 0xB0B0)
        }
    }

    pub(super) fn read_player_explosion_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        piece_address: u16,
        field: &str,
    ) -> Result<u16, String> {
        let table = player_explosion_table_for_address(layout, piece_address)?;
        let range = table_field_range_for_address(layout, table, piece_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!(
                "red-label player_explosion_table.{field} is not two bytes"
            ));
        }
        self.read_word(range.start)
    }

    pub(super) fn write_player_explosion_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        piece_address: u16,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        let table = player_explosion_table_for_address(layout, piece_address)?;
        let range = table_field_range_for_address(layout, table, piece_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!(
                "red-label player_explosion_table.{field} is not two bytes"
            ));
        }
        self.write_word(range.start, value)
    }

    pub(super) fn kill_object_cell_from_lists(
        &mut self,
        object_address: u16,
        candidate_lists: &[&str],
    ) -> Result<u16, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = object_table_for_address(&layout, object_address)?;
        for list_name in candidate_lists {
            let mut previous_link_address = linked_list(&lists, list_name)?.head_address;
            for _ in 0..object.entries {
                let current = self.read_word(previous_link_address)?;
                if current == 0 {
                    break;
                }
                object_table_for_address(&layout, current)?;
                let next = self.read_object_word(&layout, current, "OLINK")?;
                if current == object_address {
                    self.write_word(previous_link_address, next)?;
                    let free_head = linked_list(&lists, "free_object")?.head_address;
                    let old_free = self.read_word(free_head)?;
                    self.write_word(free_head, current)?;
                    self.write_object_word(&layout, current, "OLINK", old_free)?;
                    return Ok(previous_link_address);
                }
                previous_link_address =
                    object_field_range_for_address(&layout, current, "OLINK")?.start;
            }
        }

        Err(format!(
            "red-label object 0x{object_address:04X} was not in searched object lists"
        ))
    }

    pub(super) fn link_free_table(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: &RedLabelRamLayoutEntry,
        link_field: &str,
    ) -> Result<(), String> {
        let link = ram_field(layout, &table.table, link_field)?;
        for entry_index in 0..table.entries {
            let next = if entry_index + 1 == table.entries {
                0
            } else {
                table.base + table.entry_size * (entry_index + 1)
            };
            let range = link
                .field_range_for_entry(entry_index)
                .ok_or_else(|| format!("red-label {link_field} range is invalid"))?;
            self.write_word(range.start, next)?;
        }
        Ok(())
    }

    pub(super) fn initialize_start_game_tables(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelStartGameInit, String> {
        let genocide = self.genocide_other_processes()?;
        let screen_clear = self.clear_screen_ram()?;
        self.clear_appearance_ram(layout)?;
        self.write_field_byte(layout, "base_page", "STATUS", 0x7F)?;
        self.write_field_byte(layout, "base_page", "CURPLR", 1)?;
        self.write_field_byte(layout, "base_page", "PDFLG", 1)?;
        self.write_field_byte(layout, "base_page", "PLRCNT", 0)?;

        let player_table = table_descriptor(layout, "player")?;
        let player_range = player_table.table_range().ok_or_else(|| {
            String::from("red-label player table range overflows main RAM address space")
        })?;
        self.clear_range(player_range)?;

        let ships = self.read_cmos_byte_by_symbol("NSHIP")? & 0x0F;
        let replay_word = self.read_cmos_word_by_symbol("REPLAY")?;
        let replay = replay_word.to_be_bytes();
        self.write_field_word(layout, "base_page", "REPLA", replay_word)?;
        self.write_field_byte(layout, "base_page", "BUNITS", 0)?;

        self.write_player_entry_start_defaults(layout, 0, ships, replay.as_slice())?;
        let first_player = table_entry_range(player_table, 0)?;
        self.get_new_wave_parameters_for_player_address(first_player.start)?;
        let second_player = table_entry_range(player_table, 1)?;
        let source = self
            .ram_range(first_player)
            .ok_or_else(|| String::from("red-label player 1 table range is outside RAM"))?
            .to_vec();
        self.write_range(second_player, &source)?;

        let player_start_process = self.make_process(
            red_label_routine_address("PLSTRT")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;

        Ok(RedLabelStartGameInit {
            killed_processes: genocide.killed_processes.len(),
            screen_clear,
            player_start_process,
            ships,
            replay: replay_word,
        })
    }

    pub(super) fn write_player_entry_start_defaults(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        entry_index: u16,
        nship: u8,
        replay: &[u8],
    ) -> Result<(), String> {
        self.write_field(layout, "player", "PSCOR", entry_index, &[0, 0, 0, 0])?;
        self.write_field(
            layout,
            "player",
            "PRPLA",
            entry_index,
            &[replay[0], replay[1], 0],
        )?;
        self.write_field(layout, "player", "PLAS", entry_index, &[nship & 0x0F])?;
        self.write_field(layout, "player", "PWAV", entry_index, &[0])?;
        self.write_field(layout, "player", "PSBC", entry_index, &[nship & 0x0F])?;
        self.write_field(
            layout,
            "player",
            "PTARG",
            entry_index,
            &[RED_LABEL_START_HUMAN_COUNT],
        )
    }

    pub(super) fn finish_player_start_entry_after_coin_counters(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
        process_address: u16,
        status: u8,
        killed_processes: Vec<RedLabelKilledProcess>,
    ) -> Result<RedLabelPlayerStart, String> {
        let coin_counter_active = self.read_field_byte(layout, "base_page", "LCCNT")?
            | self.read_field_byte(layout, "base_page", "RCCNT")?;
        if coin_counter_active != 0 {
            let wakeup_address = red_label_routine_address("PLST1A")?;
            self.sleep_current_process(15, wakeup_address)?;
            return Ok(RedLabelPlayerStart::CoinCounterSleeping {
                process_address,
                status,
                killed_processes,
                wakeup_address,
            });
        }

        self.initialize_process_lists(layout, lists)?;
        let runtime_process = self.make_process(
            red_label_routine_address("PLSTR3")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        Ok(RedLabelPlayerStart::RuntimeProcessCreated {
            process_address,
            status,
            killed_processes,
            runtime_process,
        })
    }

    pub(super) fn initialize_one_player_runtime_state(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_table: &RedLabelRamLayoutEntry,
    ) -> Result<(), String> {
        self.initialize_player_runtime_bytes(layout, player_table, 0)
            .map(|_| ())
    }

    pub(super) fn initialize_current_player_runtime_state(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelPlayerRuntimeInit, String> {
        let player_table = table_descriptor(layout, "player")?;
        let current_player = self.read_field_byte(layout, "base_page", "CURPLR")?;
        if current_player == 0 || u16::from(current_player) > player_table.entries {
            return Err(format!(
                "red-label PLSTR5 current player {current_player} is outside player table"
            ));
        }
        let player_index = u16::from(current_player - 1);
        let player_address = table_entry_range(player_table, player_index)?.start;
        let (wave, wall_color, remaining_lasers, altitude_table, terrain_tables) =
            self.initialize_player_runtime_bytes(layout, player_table, player_index)?;
        let top_display = self.top_display()?;
        Ok(RedLabelPlayerRuntimeInit {
            current_player,
            player_address,
            screen_switch: None,
            wave,
            wall_color,
            remaining_lasers,
            altitude_table,
            terrain_tables,
            top_display,
        })
    }

    pub(super) fn initialize_player_runtime_bytes(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_table: &RedLabelRamLayoutEntry,
        player_index: u16,
    ) -> Result<
        (
            u8,
            u8,
            u8,
            RedLabelAltitudeTableInit,
            RedLabelTerrainTablesInit,
        ),
        String,
    > {
        self.write_field_word(layout, "base_page", "BGL", 0)?;
        self.write_field_word(layout, "base_page", "BGLX", 0)?;
        self.write_field_byte(layout, "base_page", "MAPCR", 7)?;
        let altitude_table = self.initialize_altitude_table_from_tdata()?;
        let terrain_tables = self.initialize_terrain_tables_from_bgl()?;
        self.write_field_word(layout, "base_page", "PLADIR", 0x0300)?;
        self.write_field_word(layout, "base_page", "NPLAD", 0x0300)?;
        self.write_field_byte(layout, "base_page", "THFLG", 0)?;
        self.write_field_byte(layout, "base_page", "LFLG", 0)?;
        self.write_field_byte(layout, "base_page", "SCRFLG", 0)?;
        self.write_field_byte(layout, "base_page", "REVFLG", 0)?;
        self.write_field_byte(layout, "base_page", "SBFLG", 0)?;
        self.write_field_byte(layout, "base_page", "BMBCNT", 0)?;
        self.write_field_word(
            layout,
            "base_page",
            "TPTR",
            table_descriptor(layout, "target_list")?.base,
        )?;

        let wave_range = ram_field(layout, "player", "PWAV")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PWAV range is invalid"))?;
        let wave = self.read_byte(wave_range.start)? & 0x07;
        if wave == 0 {
            return Err(String::from(
                "red-label PLSTR5 wave color lookup requires a 1-based wave index",
            ));
        }
        let pcram_range = field_range(layout, "base_page", "PCRAM")?;
        let wall_color = RED_LABEL_WALL_COLOR_TABLE[usize::from(wave - 1)];
        self.write_byte(pcram_range.start + 5, wall_color)?;

        let laser_range = ram_field(layout, "player", "PLAS")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PLAS range is invalid"))?;
        let remaining_lasers = self.read_byte(laser_range.start)?.wrapping_sub(1);
        self.write_byte(laser_range.start, remaining_lasers)?;
        self.write_field_word(
            layout,
            "base_page",
            "PLRX",
            table_entry_range(player_table, player_index)?.start,
        )?;
        self.write_field_word(layout, "base_page", "NPLAXC", 0x2080)?;
        self.write_field_word(layout, "base_page", "PLAXC", 0x2080)?;
        self.write_field_word(layout, "base_page", "PLAX16", 0x2000)?;
        self.write_field_word(layout, "base_page", "PLAY16", 0x8000)?;
        let background_left = self.read_field_word(layout, "base_page", "BGL")?;
        self.write_field_word(
            layout,
            "base_page",
            "PLABX",
            (0x2000u16 >> 2).wrapping_add(background_left),
        )?;
        self.write_field(layout, "base_page", "PLAXV", 0, &[0, 0, 0])?;
        self.write_field_word(layout, "base_page", "PLAYV", 0)?;
        Ok((
            wave,
            wall_color,
            remaining_lasers,
            altitude_table,
            terrain_tables,
        ))
    }

    pub(super) fn sleep_player_start_screen_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
    ) -> Result<RedLabelPlayerStart, String> {
        let screen_clear = self.clear_active_screen_ram()?;
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let player_table = table_descriptor(layout, "player")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let target_range = player_field_range_for_entry(layout, player_index, "PTARG")?;
        let target_count = self.read_byte(target_range.start)?;
        let status = self.write_status_from_count(layout, 0x05, target_count)?;
        let wakeup_address = red_label_routine_address("PLS1")?;
        self.sleep_current_process(0x60, wakeup_address)?;
        Ok(RedLabelPlayerStart::ScreenClearedSleeping {
            process_address,
            screen_clear,
            target_count,
            status,
            wakeup_address,
        })
    }

    pub(super) fn restore_player_world_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        entry_registers: RedLabelCpuRegisters,
    ) -> Result<RedLabelPlayerRestore, String> {
        let plres_address = red_label_routine_address("PLRES")?;
        let player_table = table_descriptor(layout, "player")?;
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let target_count =
            self.read_byte(player_field_range_for_entry(layout, player_index, "PTARG")?.start)?;
        let mut plres_b_register = if target_count == 0 {
            entry_registers.b.ok_or_else(|| {
                String::from(
                    "red-label PLRES targetless mini-swarmer restore requires PLS1 entry B register",
                )
            })?
        } else {
            0
        };

        let astro_process = self.make_process(
            red_label_routine_address("ASTRO")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let target_list = field_range(layout, "target_list", "TLIST")?;
        self.write_process_data_word(
            layout,
            astro_process.process_address,
            "PD",
            target_list.start,
        )?;
        self.clear_range(target_list.clone())?;
        self.write_field_byte(layout, "base_page", "ASTCNT", target_count)?;

        let mut target_writer = RedLabelTargetRestoreWriter {
            target_list: target_list.clone(),
            cursor: target_list.start,
            objects: Vec::with_capacity(usize::from(target_count)),
        };
        if target_count != 0 {
            let mut remainder = target_count;
            if target_count > 7 {
                let quadrant_count = target_count >> 2;
                let mut x_bank = 0;
                loop {
                    self.start_astronaut_target_group(
                        layout,
                        astro_process.process_address,
                        quadrant_count,
                        x_bank,
                        &mut target_writer,
                    )?;
                    x_bank = x_bank.wrapping_add(0x40);
                    plres_b_register = x_bank;
                    if x_bank == 0 {
                        break;
                    }
                }
                remainder = target_count.wrapping_sub(quadrant_count << 2);
            }

            if remainder != 0 {
                let xtemp = field_range(layout, "base_page", "XTEMP")?.start;
                self.write_byte(xtemp, remainder)?;
                while self.read_byte(xtemp)? != 0 {
                    let x_bank = self.read_field_byte(layout, "base_page", "HSEED")?;
                    plres_b_register = x_bank;
                    self.start_astronaut_target_group(
                        layout,
                        astro_process.process_address,
                        1,
                        x_bank,
                        &mut target_writer,
                    )?;
                    let remaining = self.read_byte(xtemp)?.wrapping_sub(1);
                    self.write_byte(xtemp, remaining)?;
                }
            }
        }

        let enemy_source = player_field_range_for_entry(layout, player_index, "PENEMY")?;
        let enemy_target = field_range(layout, "enemy_runtime", "ELIST")?;
        let enemy_len = enemy_target.end - enemy_target.start;
        if enemy_source.end - enemy_source.start < enemy_len {
            return Err(String::from(
                "red-label player.PENEMY range is shorter than ELIST",
            ));
        }
        for offset in 0..enemy_len {
            let value = self.read_byte(enemy_source.start + offset)?;
            self.write_byte(enemy_target.start + offset, value)?;
        }

        let active_counts = field_range(layout, "enemy_runtime", "ECNTS")?;
        self.clear_range(active_counts.clone())?;
        let mini_swarmer_restore =
            self.restore_mini_swarmer_reserve_from_plres(layout, plres_b_register)?;
        let schizoid_restore = self.restore_schizoid_reserve_from_plres(layout)?;
        let probe_restore = self.restore_probe_reserve_from_plres(layout)?;
        let tie_restore = self.restore_tie_reserve_from_plres(layout)?;

        Ok(RedLabelPlayerRestore {
            plres_address,
            astro_process,
            target_list_start: target_list.start,
            target_list_bytes_cleared: target_list.end - target_list.start,
            target_count,
            target_objects: target_writer.objects,
            enemy_source_address: enemy_source.start,
            enemy_target_address: enemy_target.start,
            enemy_bytes_copied: enemy_len,
            active_count_start: active_counts.start,
            active_count_bytes_cleared: active_counts.end - active_counts.start,
            mini_swarmer_restore,
            schizoid_restore,
            probe_restore,
            tie_restore,
        })
    }

    pub(super) fn restore_mini_swarmer_reserve_from_plres(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        x_low_register: u8,
    ) -> Result<RedLabelMiniSwarmerRestore, String> {
        let reserve_count = self.read_field_byte(layout, "enemy_runtime", "SWMRES")?;
        let mut remaining_reserve = reserve_count;
        let mut batches = Vec::new();

        loop {
            let phony_object_address = self.get_object_cell()?;
            let seed_before = self.read_field_byte(layout, "base_page", "SEED")?;
            let y16_range = object_field_range_for_address(layout, phony_object_address, "OY16")?;
            self.write_byte(
                y16_range.start,
                seed_before.wrapping_shr(1).wrapping_add(RED_LABEL_Y_MIN),
            )?;
            let phony_y16 = self.read_object_word(layout, phony_object_address, "OY16")?;

            let placement_rand = self.advance_red_label_rand(layout)?;
            let phony_x16 = u16::from_be_bytes([
                (placement_rand.seed & 0x3F).wrapping_add(0x80),
                x_low_register,
            ])
            .wrapping_add(self.read_field_word(layout, "base_page", "BGL")?);
            self.write_object_word(layout, phony_object_address, "OX16", phony_x16)?;

            if remaining_reserve == 0 {
                batches.push(RedLabelMiniSwarmerRestoreBatch {
                    phony_object_address,
                    phony_x16,
                    phony_y16,
                    placement_rand,
                    requested_count: 0,
                    spawned_swarmers: Vec::new(),
                    remaining_reserve,
                    returned_phony_to_free_list: false,
                });
                break;
            }

            let requested_count = remaining_reserve.min(6);
            let spawned_swarmers =
                self.make_mini_swarmers_from_center(layout, phony_object_address, requested_count)?;
            self.return_unlinked_object_to_free_list(layout, phony_object_address)?;
            remaining_reserve = remaining_reserve.wrapping_sub(requested_count);
            self.write_field_byte(layout, "enemy_runtime", "SWMRES", remaining_reserve)?;
            batches.push(RedLabelMiniSwarmerRestoreBatch {
                phony_object_address,
                phony_x16,
                phony_y16,
                placement_rand,
                requested_count,
                spawned_swarmers,
                remaining_reserve,
                returned_phony_to_free_list: true,
            });

            if remaining_reserve == 0 {
                break;
            }
        }

        Ok(RedLabelMiniSwarmerRestore {
            reserve_count,
            x_low_register,
            active_count: self.read_field_byte(layout, "enemy_runtime", "SWCNT")?,
            batches,
        })
    }

    pub(super) fn restore_schizoid_reserve_from_plres(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<RedLabelSchizoidRestore>, String> {
        let reserve_count = self.read_field_byte(layout, "enemy_runtime", "SCZRES")?;
        if reserve_count == 0 {
            return Ok(None);
        }

        let created_objects = self.start_schizoid_restore_group(layout, reserve_count)?;
        self.write_field_byte(layout, "enemy_runtime", "SCZRES", 0)?;
        Ok(Some(RedLabelSchizoidRestore {
            reserve_count,
            active_count: self.read_field_byte(layout, "enemy_runtime", "SCZCNT")?,
            created_objects,
        }))
    }

    pub(super) fn start_schizoid_restore_group(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        count: u8,
    ) -> Result<Vec<RedLabelSchizoidRestoreObject>, String> {
        let xtemp = field_range(layout, "base_page", "XTEMP")?.start;
        self.write_byte(xtemp, count)?;
        let mut created_objects = Vec::with_capacity(usize::from(count));

        while self.read_byte(xtemp)? != 0 {
            let process = self.make_process(
                red_label_routine_address("SCZ0")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("SCZP1")?,
                collision_vector_address: red_label_routine_address("SCZKIL")?,
                scanner_color: 0xCC33,
            };
            let object = self.init_object_cell(process.process_address, descriptor)?;

            let placement_state = self.advance_red_label_rand(layout)?;
            let avoid_left = self
                .read_field_word(layout, "base_page", "BGL")?
                .wrapping_sub(300 * 32);
            let mut relative = u16::from_be_bytes([placement_state.hseed, placement_state.lseed])
                .wrapping_sub(avoid_left);
            if relative < 600 * 32 {
                relative = relative.wrapping_add(0x8000);
            }
            let x16 = relative.wrapping_add(avoid_left);
            self.write_object_word(layout, object.object_address, "OX16", x16)?;

            let y16 = u16::from_be_bytes([
                placement_state
                    .seed
                    .wrapping_shr(1)
                    .wrapping_add(RED_LABEL_Y_MIN),
                0,
            ]);
            let y16_range = object_field_range_for_address(layout, object.object_address, "OY16")?;
            self.write_byte(y16_range.start, y16.to_be_bytes()[0])?;

            self.write_object_word(layout, object.object_address, "OYV", 0)?;
            self.write_object_word(layout, object.object_address, "OXV", 0)?;

            let shot_timer_max = self.read_field_byte(layout, "enemy_runtime", "SZSTIM")?;
            let shot_timer_state = self.advance_red_label_rand(layout)?;
            let discarded_pd_shot_timer = rmax(shot_timer_max, shot_timer_state.seed);
            self.write_process_byte(
                layout,
                process.process_address,
                "PD",
                discarded_pd_shot_timer,
            )?;

            let appearance = self.start_appearance_for_object(object.object_address)?;
            self.write_object_word(
                layout,
                object.object_address,
                "OBJID",
                process.process_address,
            )?;
            self.write_process_data_word(
                layout,
                process.process_address,
                "PD",
                object.object_address,
            )?;
            let active_count = self
                .read_field_byte(layout, "enemy_runtime", "SCZCNT")?
                .wrapping_add(1);
            self.write_field_byte(layout, "enemy_runtime", "SCZCNT", active_count)?;

            created_objects.push(RedLabelSchizoidRestoreObject {
                object,
                appearance,
                x16,
                y16,
                xv: 0,
                yv: 0,
                discarded_pd_shot_timer,
            });

            let remaining = self.read_byte(xtemp)?.wrapping_sub(1);
            self.write_byte(xtemp, remaining)?;
        }

        Ok(created_objects)
    }

    /// Source-shaped `ASTRO`: advance the target-list cursor, skip empty,
    /// offscreen, or captured targets, otherwise walk the astronaut one source
    /// step across `ALTTBL` terrain and sleep back to `ASTRO`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L294-L359>.
    pub fn step_astronaut_current_process(
        &mut self,
    ) -> Result<RedLabelAstronautProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let target_list = field_range(&layout, "target_list", "TLIST")?;
        let mut target_cursor = self
            .read_process_data_word(&layout, process_address, "PD")?
            .wrapping_add(2);
        if target_cursor >= target_list.start + 32 {
            target_cursor = target_list.start;
        }
        self.write_process_data_word(&layout, process_address, "PD", target_cursor)?;

        let object_address = self.read_word(target_cursor)?;
        let mut walk = None;
        let target_object_address = if object_address == 0 {
            None
        } else {
            object_table_for_address(&layout, object_address)?;
            Some(object_address)
        };

        if object_address != 0
            && self.read_object_screen_address(&layout, object_address)? != 0
            && self.read_object_word(&layout, object_address, "OCVECT")?
                == red_label_routine_address("ASTKIL")?
        {
            walk = Some(self.walk_astronaut_target(&layout, object_address)?);
        }

        let wakeup_address = red_label_routine_address("ASTRO")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(RedLabelAstronautProcessStep {
            process_address,
            target_cursor,
            target_object_address,
            walk,
            wakeup_address,
        })
    }

    pub(super) fn walk_astronaut_target(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<RedLabelAstronautWalk, String> {
        let astp2 = red_label_object_picture_address("ASTP2")?;
        let astp4 = red_label_object_picture_address("ASTP4")?;
        let picture = self.read_object_word(layout, object_address, "OPICT")?;
        let walking_left = picture <= astp2;
        let seed = self.read_field_byte(layout, "base_page", "SEED")?;

        let (direction, altitude, target_y, y_position, picture_address, x_velocity) =
            if walking_left {
                if seed <= 8 {
                    (
                        RedLabelAstronautDirection::Right,
                        None,
                        None,
                        self.read_object_byte(layout, object_address, "OY16")?,
                        red_label_object_picture_address("ASTP3")?,
                        0x0020,
                    )
                } else {
                    let altitude = self.object_altitude_from_alttbl(layout, object_address)?;
                    let target_y = altitude.wrapping_add(4).min(0xE8);
                    let y_position = self.step_astronaut_y(layout, object_address, target_y)?;
                    let next_picture = picture.wrapping_add(10);
                    let picture_address = if next_picture <= astp2 {
                        next_picture
                    } else {
                        red_label_object_picture_address("ASTP1")?
                    };
                    (
                        RedLabelAstronautDirection::Left,
                        Some(altitude),
                        Some(target_y),
                        y_position,
                        picture_address,
                        0xFFE0,
                    )
                }
            } else if seed <= 8 {
                (
                    RedLabelAstronautDirection::Left,
                    None,
                    None,
                    self.read_object_byte(layout, object_address, "OY16")?,
                    red_label_object_picture_address("ASTP1")?,
                    0xFFE0,
                )
            } else {
                let altitude = self.object_altitude_from_alttbl(layout, object_address)?;
                let target_y = altitude.wrapping_add(15).min(0xE8);
                let y_position = self.step_astronaut_y(layout, object_address, target_y)?;
                let next_picture = picture.wrapping_add(10);
                let picture_address = if next_picture <= astp4 {
                    next_picture
                } else {
                    red_label_object_picture_address("ASTP3")?
                };
                (
                    RedLabelAstronautDirection::Right,
                    Some(altitude),
                    Some(target_y),
                    y_position,
                    picture_address,
                    0x0020,
                )
            };

        self.write_object_word(layout, object_address, "OPICT", picture_address)?;
        let x_position = self
            .read_object_word(layout, object_address, "OX16")?
            .wrapping_add(x_velocity);
        self.write_object_word(layout, object_address, "OX16", x_position)?;

        Ok(RedLabelAstronautWalk {
            object_address,
            direction,
            altitude,
            target_y,
            y_position,
            picture_address,
            x_velocity,
            x_position,
        })
    }

    pub(super) fn object_altitude_from_alttbl(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<u8, String> {
        let altitude_table = field_range(layout, "terrain_altitude", "ALTTBL")?;
        let offset = self.read_object_word(layout, object_address, "OX16")? >> 6;
        let address = altitude_table.start.wrapping_add(offset);
        if address >= altitude_table.end {
            return Err(format!(
                "red-label GETALT offset 0x{offset:04X} exceeds ALTTBL"
            ));
        }
        self.read_byte(address)
    }

    pub(super) fn step_astronaut_y(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        target_y: u8,
    ) -> Result<u8, String> {
        let object_y = self.read_object_byte(layout, object_address, "OY16")?;
        let y_position = if target_y == object_y {
            object_y
        } else if target_y > object_y {
            object_y.wrapping_add(1)
        } else {
            object_y.wrapping_sub(1)
        };
        let y16_range = object_field_range_for_address(layout, object_address, "OY16")?;
        self.write_byte(y16_range.start, y_position)?;
        Ok(y_position)
    }

    /// Source-shaped `AFALL` / `AFALL2`: advance a falling astronaut, hand off
    /// to the score-popup process on a safe landing, or run `ASTK1` and
    /// suicide the falling process on a fatal impact.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L927-L977>.
    pub fn step_falling_astronaut_current_process(
        &mut self,
        carried_by_player: bool,
    ) -> Result<RedLabelFallingAstronautStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;

        if carried_by_player {
            self.write_object_word(&layout, object_address, "OYV", 0)?;
            let [play_y, _] = self
                .read_field_word(&layout, "base_page", "PLAY16")?
                .to_be_bytes();
            let carried_y = play_y.wrapping_add(10);
            let y_fraction = self
                .read_object_word(&layout, object_address, "OY16")?
                .to_be_bytes()[1];
            let y16 = u16::from_be_bytes([carried_y, y_fraction]);
            let y16_range = object_field_range_for_address(&layout, object_address, "OY16")?;
            self.write_byte(y16_range.start, carried_y)?;
            let x16 = self
                .read_field_word(&layout, "base_page", "PLABX")?
                .wrapping_add(0x0080);
            self.write_object_word(&layout, object_address, "OX16", x16)?;
            let altitude = self.object_altitude_from_alttbl(&layout, object_address)?;
            if altitude < carried_y {
                return self.land_falling_astronaut_current_process(
                    &layout,
                    process_address,
                    object_address,
                    "P500",
                );
            }

            let wakeup_address = red_label_routine_address("AFALL2")?;
            self.sleep_current_process(1, wakeup_address)?;
            return Ok(RedLabelFallingAstronautStep::CarriedSleeping {
                process_address,
                object_address,
                x16,
                y16,
                altitude,
                wakeup_address,
            });
        }

        let previous_y_velocity = self.read_object_word(&layout, object_address, "OYV")?;
        let accelerated_y_velocity = previous_y_velocity.wrapping_add(8);
        let y_velocity = if accelerated_y_velocity >= 0x0300 {
            previous_y_velocity
        } else {
            self.write_object_word(&layout, object_address, "OYV", accelerated_y_velocity)?;
            accelerated_y_velocity
        };
        let altitude = self.object_altitude_from_alttbl(&layout, object_address)?;
        let object_y = self.read_object_byte(&layout, object_address, "OY16")?;
        if altitude > object_y {
            let wakeup_address = red_label_routine_address("AFALL")?;
            self.sleep_current_process(4, wakeup_address)?;
            return Ok(RedLabelFallingAstronautStep::FallingSleeping {
                process_address,
                object_address,
                y_velocity,
                altitude,
                wakeup_address,
            });
        }

        if self.read_object_word(&layout, object_address, "OYV")? <= 0x00E0 {
            return self.land_falling_astronaut_current_process(
                &layout,
                process_address,
                object_address,
                "P250",
            );
        }

        let center = self
            .read_object_screen_address(&layout, object_address)?
            .wrapping_add(0x0107);
        self.write_field_word(&layout, "base_page", "CENTMP", center)?;
        let astronaut_kill = self.kill_astronaut_unchecked(&layout, object_address)?;
        let previous_link_address = self.kill_process(process_address)?;
        Ok(RedLabelFallingAstronautStep::FatalKilled {
            process_address,
            object_address,
            center,
            astronaut_kill,
            killed_process: RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address,
            },
        })
    }

    pub(super) fn land_falling_astronaut_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
        score_routine_label: &str,
    ) -> Result<RedLabelFallingAstronautStep, String> {
        let score_routine_address = red_label_routine_address(score_routine_label)?;
        let score_process =
            self.make_process(score_routine_address, RED_LABEL_SYSTEM_PROCESS_TYPE)?;
        self.write_process_data_word(layout, score_process.process_address, "PD", object_address)?;
        self.write_object_word(layout, object_address, "OBJID", 0)?;
        self.write_object_word(layout, object_address, "OYV", 0)?;
        self.write_object_word(
            layout,
            object_address,
            "OCVECT",
            red_label_routine_address("ASTKIL")?,
        )?;
        let sound_loaded = self.load_sound_table_by_label("ALSND")?;
        let previous_link_address = self.kill_process(process_address)?;
        Ok(RedLabelFallingAstronautStep::Landed {
            process_address,
            object_address,
            score_routine_address,
            score_process,
            sound_loaded,
            killed_process: RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address,
            },
        })
    }

    /// Source-shaped `P250` / `P500`: create the short-lived score object,
    /// score the rescue value, park the object at the astronaut position, and
    /// sleep to `P503`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L499-L526>.
    pub fn start_score_sprite_current_process(
        &mut self,
        kind: RedLabelScoreSpriteKind,
    ) -> Result<RedLabelScoreSpriteStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let (picture_label, addend) = match kind {
            RedLabelScoreSpriteKind::Points250 => ("C25P1", 0x0125),
            RedLabelScoreSpriteKind::Points500 => ("C5P1", 0x0150),
        };
        let descriptor = RedLabelObjectDescriptor {
            picture_address: red_label_object_picture_address(picture_label)?,
            collision_vector_address: red_label_routine_address("NOKILL")?,
            scanner_color: 0,
        };
        let object = self.init_object_cell(process_address, descriptor)?;
        let score = self.score_current_player(addend)?;
        let astronaut_object_address =
            self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, astronaut_object_address)?;
        let x_velocity = self.read_word(field_range(&layout, "base_page", "PLAXV")?.start)?;
        self.write_object_word(&layout, object.object_address, "OXV", x_velocity)?;
        self.write_object_word(&layout, object.object_address, "OYV", 0)?;
        self.write_object_bytes(&layout, object.object_address, "OTYP", &[0x11])?;
        let x16 = self.read_object_word(&layout, astronaut_object_address, "OX16")?;
        self.write_object_word(&layout, object.object_address, "OX16", x16)?;
        let astronaut_y16 = self.read_object_word(&layout, astronaut_object_address, "OY16")?;
        let y16 = if astronaut_y16 & 0x8000 == 0 {
            astronaut_y16.wrapping_add(0x1800)
        } else {
            astronaut_y16.wrapping_sub(0x2000)
        };
        self.write_object_word(&layout, object.object_address, "OY16", y16)?;
        self.activate_object_cell(object.object_address)?;
        self.write_process_data_word(&layout, process_address, "PD", object.object_address)?;
        let wakeup_address = red_label_routine_address("P503")?;
        self.sleep_current_process(50, wakeup_address)?;
        Ok(RedLabelScoreSpriteStep::StartedSleeping(
            RedLabelScoreSpriteStart {
                process_address,
                kind,
                object,
                astronaut_object_address,
                score,
                x_velocity,
                x16,
                y16,
                wakeup_address,
            },
        ))
    }

    pub fn finish_score_sprite_current_process(
        &mut self,
    ) -> Result<RedLabelScoreSpriteStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        let previous_object_link_address = self.kill_object_cell_offscreen(object_address)?;
        let previous_link_address = self.kill_process(process_address)?;
        Ok(RedLabelScoreSpriteStep::Completed {
            process_address,
            object_address,
            previous_object_link_address,
            killed_process: RedLabelKilledProcess {
                killed_process_address: process_address,
                previous_link_address,
            },
        })
    }

    /// Source-shaped `TERBLO`: mark the terrain-blown status bit, erase both
    /// terrain screen tables, clear the scanner terrain footprint, run the
    /// first `TBLP0` terrain-explosion pass, and sleep to `TBL3`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L439-L478>.
    pub fn start_terrain_blow_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let status = self.read_field_byte(&layout, "base_page", "STATUS")? | 0x02;
        self.write_field_byte(&layout, "base_page", "STATUS", status)?;
        self.write_process_byte(&layout, process_address, "PD", 0)?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 7)?;
        let terrain_erase = self.erase_terrain_from_screen_table()?;
        let scanner_terrain_erase = self.erase_scanner_terrain_from_erase_table()?;
        let pass = self.run_terrain_blow_explosion_pass(&layout, 2)?;

        Ok(RedLabelTerrainBlowProcessStep::StartedSleeping {
            process_address,
            status,
            terrain_erase,
            scanner_terrain_erase,
            pass,
        })
    }

    /// Source-shaped `TBL3`: blank the first pseudo-color byte, compute the
    /// random sleep from the current iteration, set the overload counter, and
    /// sleep to `TBL4`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L479-L487>.
    pub fn continue_terrain_blow_flash_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let pcram = field_range(&layout, "base_page", "PCRAM")?.start;
        self.write_byte(pcram, 0)?;
        let iteration = self.read_process_byte(&layout, process_address, "PD")?;
        let sleep_max = (iteration >> 3).wrapping_add(1);
        let rand_state = self.advance_red_label_rand(&layout)?;
        let sleep_time = rmax(sleep_max, rand_state.seed);
        let wakeup_address = red_label_routine_address("TBL4")?;
        let overload_counter =
            self.sleep_terrain_blow_with_overload(&layout, sleep_time, wakeup_address)?;

        Ok(RedLabelTerrainBlowProcessStep::FlashClearedSleeping {
            process_address,
            iteration,
            sleep_max,
            sleep_time,
            rand_state,
            overload_counter,
            wakeup_address,
        })
    }

    /// Source-shaped `TBL4`: advance the terrain-blow iteration, either branch
    /// back through `TBLP0` for another pair of explosions or load `TBSND` and
    /// suicide after iteration 16.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L488-L494>.
    pub fn advance_terrain_blow_iteration_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let iteration = self
            .read_process_byte(&layout, process_address, "PD")?
            .wrapping_add(1);
        self.write_process_byte(&layout, process_address, "PD", iteration)?;

        if iteration != 16 {
            let pass = self.run_terrain_blow_explosion_pass(&layout, 2)?;
            return Ok(RedLabelTerrainBlowProcessStep::IterationAdvancedSleeping {
                process_address,
                iteration,
                pass,
            });
        }

        let sound_loaded = self.load_sound_table_by_label("TBSND")?;
        let killed_process = self.kill_current_process(&layout)?;
        Ok(RedLabelTerrainBlowProcessStep::Completed {
            process_address,
            iteration,
            sound_loaded,
            killed_process,
        })
    }

    pub(super) fn run_terrain_blow_explosion_pass(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        sleep_time: u8,
    ) -> Result<RedLabelTerrainExplosionPass, String> {
        let lists = red_label_linked_lists()?;
        let scratch_object_address =
            self.read_word(linked_list(&lists, "free_object")?.head_address)?;
        if scratch_object_address == 0 {
            return Err(String::from(
                "red-label TERBLO `OFREE` object list is empty",
            ));
        }
        object_table_for_address(layout, scratch_object_address)?;
        self.write_object_word(
            layout,
            scratch_object_address,
            "OPICT",
            red_label_object_picture_address("TEREX")?,
        )?;

        let mut explosions = Vec::with_capacity(2);
        let mut b_register = 2u8;
        let background_left = self.read_field_word(layout, "base_page", "BGL")?;
        for _ in 0..2 {
            let rand_state = self.advance_red_label_rand(layout)?;
            let x16 = u16::from_be_bytes([rand_state.seed & 0x3F, b_register])
                .wrapping_add(background_left);
            self.write_object_word(layout, scratch_object_address, "OX16", x16)?;
            let altitude = self.object_altitude_from_alttbl(layout, scratch_object_address)?;
            let y16_range = object_field_range_for_address(layout, scratch_object_address, "OY16")?;
            self.write_byte(y16_range.start, altitude)?;
            let center = u16::from_be_bytes([altitude.wrapping_sub(10), x16.to_be_bytes()[1]]);
            self.write_field_word(layout, "base_page", "CENTMP", center)?;
            let explosion = self.start_explosion_for_object(scratch_object_address)?;
            explosions.push(RedLabelTerrainExplosion {
                object_address: scratch_object_address,
                rand_state,
                x16,
                altitude,
                center,
                explosion,
            });
            b_register = x16.to_be_bytes()[1];
        }

        let color_table = red_label_color_cycle_table("COLTAB")?;
        let color_index = usize::from(self.read_field_byte(layout, "base_page", "SEED")? & 0x1F);
        let pseudo_color = *color_table.bytes.get(color_index).ok_or_else(|| {
            format!(
                "red-label TERBLO color index {color_index} is outside COLTAB at 0x{:04X}",
                color_table.address
            )
        })?;
        self.write_byte(
            field_range(layout, "base_page", "PCRAM")?.start,
            pseudo_color,
        )?;
        let sound_loaded = self.load_sound_table_by_label("AHSND")?;
        let wakeup_address = red_label_routine_address("TBL3")?;
        let overload_counter =
            self.sleep_terrain_blow_with_overload(layout, sleep_time, wakeup_address)?;

        Ok(RedLabelTerrainExplosionPass {
            scratch_object_address,
            explosions,
            pseudo_color,
            overload_counter,
            sound_loaded,
            wakeup_address,
        })
    }

    pub(super) fn sleep_terrain_blow_with_overload(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        sleep_time: u8,
        wakeup_address: u16,
    ) -> Result<u8, String> {
        let overload_counter = 8;
        self.write_field_byte(layout, "base_page", "OVCNT", overload_counter)?;
        self.sleep_current_process(sleep_time, wakeup_address)?;
        Ok(overload_counter)
    }

    /// Source-shaped `MSWM`/`MSWLP`: set the mini-swarmer horizontal seek
    /// velocity on entry, otherwise run the vertical acceleration/damping loop,
    /// branch back to `MSWM` when it has passed the player, fire through
    /// `SWBMB`, and sleep back to `MSWLP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L198-L287>.
    pub fn step_mini_swarmer_current_process(
        &mut self,
        start_with_horizontal_seek: bool,
    ) -> Result<RedLabelMiniSwarmerProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;

        let mut x_velocity = None;
        let mut y_velocity = None;
        let mut shot = None;
        let mut restarted_horizontal_seek = false;
        let mut shot_timer = self.read_process_byte(&layout, process_address, "PD4")?;

        if start_with_horizontal_seek {
            x_velocity = Some(self.seek_mini_swarmer_horizontally(&layout, object_address)?);
        } else {
            y_velocity = Some(self.update_mini_swarmer_y_velocity(
                &layout,
                process_address,
                object_address,
            )?);

            let player_absolute_x = self.read_field_word(&layout, "base_page", "PLABX")?;
            let object_absolute_x = self.read_object_word(&layout, object_address, "OX16")?;
            let past_window = player_absolute_x
                .wrapping_sub(object_absolute_x)
                .wrapping_add(150 * 32);
            if past_window > 300 * 32 {
                restarted_horizontal_seek = true;
                x_velocity = Some(self.seek_mini_swarmer_horizontally(&layout, object_address)?);
            } else {
                shot_timer = self
                    .read_process_byte(&layout, process_address, "PD4")?
                    .wrapping_sub(1);
                self.write_process_byte(&layout, process_address, "PD4", shot_timer)?;
                if shot_timer == 0 {
                    shot =
                        self.shoot_mini_swarmer_bomb(&layout, process_address, object_address)?;
                    shot_timer = self.read_process_byte(&layout, process_address, "PD4")?;
                }
            }
        }

        let wakeup_address = red_label_routine_address("MSWLP")?;
        self.sleep_current_process(3, wakeup_address)?;
        Ok(RedLabelMiniSwarmerProcessStep {
            process_address,
            object_address,
            x_velocity,
            y_velocity,
            shot_timer,
            shot,
            restarted_horizontal_seek,
            wakeup_address,
        })
    }

    pub(super) fn seek_mini_swarmer_horizontally(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<u16, String> {
        let player_absolute_x = self.read_field_word(layout, "base_page", "PLABX")?;
        let object_absolute_x = self.read_object_word(layout, object_address, "OX16")?;
        let velocity_low = if player_absolute_x >= object_absolute_x {
            self.read_field_byte(layout, "enemy_runtime", "SWXV")?
        } else {
            0u8.wrapping_sub(self.read_field_byte(layout, "enemy_runtime", "SWXV")?)
        };
        let velocity = sign_extend_u8_to_u16(velocity_low);
        self.write_object_word(layout, object_address, "OXV", velocity)?;
        Ok(velocity)
    }

    pub(super) fn update_mini_swarmer_y_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<u16, String> {
        let acceleration = self.read_process_byte(layout, process_address, "PD2")?;
        let player_y = self.read_field_byte(layout, "base_page", "PLAYC")?;
        let object_y = self.read_object_byte(layout, object_address, "OY16")?;
        let acceleration_low = if player_y > object_y {
            acceleration
        } else {
            0u8.wrapping_sub(acceleration)
        };
        let mut y_velocity = sign_extend_u8_to_u16(acceleration_low)
            .wrapping_add(self.read_object_word(layout, object_address, "OYV")?);

        if signed_word_greater_or_equal(y_velocity, 0x0200) {
            y_velocity = 0x0200;
        }
        if signed_word_less_or_equal(y_velocity, 0xFE00) {
            y_velocity = 0xFE00;
        }
        self.write_object_word(layout, object_address, "OYV", y_velocity)?;

        y_velocity = y_velocity.wrapping_add(mini_swarmer_damping_adjustment(y_velocity));
        self.write_object_word(layout, object_address, "OYV", y_velocity)?;

        let random_acceleration = sign_extend_u8_to_u16(
            (self.read_field_byte(layout, "base_page", "SEED")? & 0x1F).wrapping_sub(0x10),
        );
        y_velocity = y_velocity.wrapping_add(random_acceleration);
        self.write_object_word(layout, object_address, "OYV", y_velocity)?;
        Ok(y_velocity)
    }

    pub(super) fn shoot_mini_swarmer_bomb(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<Option<RedLabelEnemyShot>, String> {
        let player_delta = self
            .read_field_word(layout, "base_page", "PLABX")?
            .wrapping_sub(self.read_object_word(layout, object_address, "OX16")?);
        let x_velocity = self.read_object_word(layout, object_address, "OXV")?;
        let mut shot = None;
        if (player_delta.to_be_bytes()[0] ^ x_velocity.to_be_bytes()[0]) & 0x80 == 0 {
            let descriptor = RedLabelShellDescriptor {
                output_routine_address: red_label_routine_address("FBOUT")?,
                picture_address: red_label_object_picture_address("BMBP1")?,
                kill_routine_address: red_label_routine_address("BKIL")?,
            };
            if let Some(shell) = self.get_shell_cell(object_address, process_address, descriptor)? {
                let shell_x_velocity = x_velocity.wrapping_shl(3);
                self.write_object_word(layout, shell.shell_address, "OXV", shell_x_velocity)?;
                let sound_loaded = self.load_sound_table_by_label("SWSSND")?;
                let player_y = self.read_field_byte(layout, "base_page", "PLAYC")?;
                let shell_y = self.read_object_byte(layout, shell.shell_address, "OY16")?;
                let shell_y_velocity = arithmetic_shift_right_word(
                    u16::from_be_bytes([player_y.wrapping_sub(shell_y), 0]),
                    5,
                );
                self.write_object_word(layout, shell.shell_address, "OYV", shell_y_velocity)?;
                shot = Some(RedLabelEnemyShot {
                    shell,
                    x_velocity: shell_x_velocity,
                    y_velocity: shell_y_velocity,
                    sound_loaded,
                });
            }
        }

        let shot_timer_max = self.read_field_byte(layout, "enemy_runtime", "SWSTIM")?;
        let shot_timer = self.advance_red_label_rmax(layout, shot_timer_max)?;
        self.write_process_byte(layout, process_address, "PD4", shot_timer)?;
        Ok(shot)
    }

    /// Source-shaped `UFOST`: create one UFO process/object pair, seed its
    /// position from `SEED`/`HSEED` plus `BGL`, initialize the shot timer, run
    /// the initial `UFONV0` velocity update, then enter `APVCT`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L3-L24>.
    pub fn start_ufo_process(&mut self) -> Result<RedLabelUfoStart, String> {
        let layout = red_label_ram_layout()?;
        let process = self.make_process(
            red_label_routine_address("UFOLP")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let descriptor = RedLabelObjectDescriptor {
            picture_address: red_label_object_picture_address("UFOP1")?,
            collision_vector_address: red_label_routine_address("UFOKIL")?,
            scanner_color: 0x3333,
        };
        let object = self.init_object_cell(process.process_address, descriptor)?;
        self.write_process_data_word(
            &layout,
            process.process_address,
            "PD",
            object.object_address,
        )?;

        let seed = self.read_field_byte(&layout, "base_page", "SEED")?;
        let hseed = self.read_field_byte(&layout, "base_page", "HSEED")?;
        let x16 = u16::from_be_bytes([seed & 0x1F, hseed]).wrapping_add(self.read_field_word(
            &layout,
            "base_page",
            "BGL",
        )?);
        self.write_object_word(&layout, object.object_address, "OX16", x16)?;

        let y16 = u16::from_be_bytes([x16.to_be_bytes()[1] >> 1, 0])
            .wrapping_add(u16::from(RED_LABEL_Y_MIN) << 8);
        self.write_object_word(&layout, object.object_address, "OY16", y16)?;
        self.write_object_word(&layout, object.object_address, "OYV", 0)?;
        self.write_object_word(&layout, object.object_address, "OXV", 0)?;

        let shot_timer = 8;
        self.write_process_byte(&layout, process.process_address, "PD2", shot_timer)?;
        let velocity = self
            .update_ufo_velocity(&layout, object.object_address, false)?
            .ok_or_else(|| String::from("red-label UFONV0 unexpectedly skipped velocity update"))?;
        let appearance = self.start_appearance_for_object(object.object_address)?;

        Ok(RedLabelUfoStart {
            process,
            object,
            x16,
            y16,
            shot_timer,
            velocity,
            appearance,
        })
    }

    /// Source-shaped `UFOLP`: run the UFO shot timer, cycle `UFOP1`-`UFOP3`,
    /// optionally run `UFONV`, and sleep back to `UFOLP`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L26-L46>.
    pub fn step_ufo_current_process(&mut self) -> Result<RedLabelUfoProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;

        let null_picture = red_label_object_picture_address("NULOB")?;
        let mut picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
        let mut shot_timer = self.read_process_byte(&layout, process_address, "PD2")?;
        let mut shot = None;
        let mut velocity = None;

        if picture_address != null_picture {
            shot_timer = shot_timer.wrapping_sub(1);
            self.write_process_byte(&layout, process_address, "PD2", shot_timer)?;
            if shot_timer == 0 {
                let shot_timer_max = self.read_field_byte(&layout, "enemy_runtime", "UFSTIM")?;
                shot_timer = self.advance_red_label_rmax(&layout, shot_timer_max)?;
                self.write_process_byte(&layout, process_address, "PD2", shot_timer)?;
                shot = self.shoot_at_player_from_object_current_process(
                    &layout,
                    object_address,
                    "USHSND",
                )?;
            }

            picture_address = picture_address.wrapping_add(10);
            if picture_address > red_label_object_picture_address("UFOP3")? {
                picture_address = red_label_object_picture_address("UFOP1")?;
                velocity = self.update_ufo_velocity(&layout, object_address, true)?;
            }
            self.write_object_word(&layout, object_address, "OPICT", picture_address)?;
        }

        let wakeup_address = red_label_routine_address("UFOLP")?;
        self.sleep_current_process(6, wakeup_address)?;
        Ok(RedLabelUfoProcessStep {
            process_address,
            object_address,
            shot_timer,
            shot,
            picture_address,
            velocity,
            wakeup_address,
        })
    }

    /// Source-shaped `LANDST`: create the requested number of landers while
    /// astronauts remain targetable, otherwise fall through to the source
    /// `SCZS0` fallback for the remaining count.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L649-L685>.
    pub fn start_lander_processes(&mut self, count: u8) -> Result<RedLabelLanderStart, String> {
        let layout = red_label_ram_layout()?;
        let xtemp = field_range(&layout, "base_page", "XTEMP")?.start;
        self.write_byte(xtemp, count)?;

        let mut landers = Vec::with_capacity(usize::from(count));
        let mut schizoid_fallback = Vec::new();
        while self.read_byte(xtemp)? != 0 {
            if self.read_field_byte(&layout, "base_page", "ASTCNT")? == 0 {
                let remaining = self.read_byte(xtemp)?;
                schizoid_fallback = self.start_schizoid_restore_group(&layout, remaining)?;
                break;
            }

            let process = self.make_process(
                red_label_routine_address("LANDS0")?,
                RED_LABEL_SYSTEM_PROCESS_TYPE,
            )?;
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("LNDP1")?,
                collision_vector_address: red_label_routine_address("LKILL")?,
                scanner_color: 0x4433,
            };
            let object = self.init_object_cell(process.process_address, descriptor)?;

            let placement_state = self.advance_red_label_rand(&layout)?;
            let x16 = u16::from_be_bytes([placement_state.hseed, placement_state.lseed]);
            self.write_object_word(&layout, object.object_address, "OX16", x16)?;
            let y16_range = object_field_range_for_address(&layout, object.object_address, "OY16")?;
            self.write_byte(y16_range.start, RED_LABEL_Y_MIN.wrapping_add(2))?;
            let y16 = self.read_object_word(&layout, object.object_address, "OY16")?;

            let y_velocity = self.read_field_word(&layout, "enemy_runtime", "LNDYV")?;
            self.write_object_word(&layout, object.object_address, "OYV", y_velocity)?;

            let shot_timer_max = self.read_field_byte(&layout, "enemy_runtime", "LDSTIM")?;
            let shot_timer = self.advance_red_label_rmax(&layout, shot_timer_max)?;
            self.write_process_byte(&layout, process.process_address, "PD6", shot_timer)?;

            let x_velocity_max = self.read_field_byte(&layout, "enemy_runtime", "LNDXV")?;
            let x_velocity_byte = self.advance_red_label_rmax(&layout, x_velocity_max)?;
            let x_velocity = if x_velocity_byte & 1 == 0 {
                u16::from(x_velocity_byte)
            } else {
                !u16::from(x_velocity_byte)
            };
            self.write_object_word(&layout, object.object_address, "OXV", x_velocity)?;
            self.write_object_word(
                &layout,
                object.object_address,
                "OBJID",
                process.process_address,
            )?;
            let appearance = self.start_appearance_for_object(object.object_address)?;
            self.write_process_data_word(
                &layout,
                process.process_address,
                "PD",
                object.object_address,
            )?;
            let target = self.select_lander_target(&layout, process.process_address)?;
            let lander_count = self
                .read_field_byte(&layout, "enemy_runtime", "LNDCNT")?
                .wrapping_add(1);
            self.write_field_byte(&layout, "enemy_runtime", "LNDCNT", lander_count)?;

            landers.push(RedLabelLanderStartObject {
                process,
                object,
                x16,
                y16,
                y_velocity,
                shot_timer,
                x_velocity,
                appearance,
                target,
            });

            let remaining = self.read_byte(xtemp)?.wrapping_sub(1);
            self.write_byte(xtemp, remaining)?;
        }

        Ok(RedLabelLanderStart {
            requested_count: count,
            landers,
            schizoid_fallback,
        })
    }

    /// Source-shaped `LANDS0`: validate or retarget the passenger, orbit at
    /// terrain-derived altitude, fire through `LSHOT`, animate `LNDP1`..`3`,
    /// or enter the grab path when the source close-X test succeeds.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L690-L740>.
    pub fn step_lander_orbit_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;

        let target = match self.live_lander_target(&layout, process_address)? {
            Some(target) => target,
            None => {
                return self.retarget_or_convert_lander(&layout, process_address, object_address);
            }
        };

        let lander_x = self.read_object_byte(&layout, object_address, "OX16")? & 0xFC;
        let target_x = self.read_object_byte(&layout, target.target_object_address, "OX16")? & 0xFC;
        if lander_x == target_x {
            return self.step_lander_grab_with_layout(
                &layout,
                process_address,
                object_address,
                true,
            );
        }

        self.orbit_lander_current_process(&layout, process_address, object_address, Some(target))
    }

    /// Source-shaped `LANDG`: move toward the selected astronaut, keep firing,
    /// and when aligned switch the lander/passenger vectors before falling
    /// through the source `LANDF` flee body.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L746-L798>.
    pub fn step_lander_grab_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;
        self.step_lander_grab_with_layout(&layout, process_address, object_address, false)
    }

    /// Source-shaped `LANDF`: continue fleeing upward with a passenger, or
    /// fall through to the `LANDFX` / `LNDFXA` pull-inside path.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L798-L827>.
    pub fn step_lander_flee_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;
        let outcome =
            self.step_lander_flee_with_layout(&layout, process_address, object_address)?;
        Ok(self.lander_flee_outcome_to_step(process_address, object_address, outcome))
    }

    /// Source-shaped `LNDFXA`: pull the passenger into the lander, give up if
    /// the target slot was cleared, or consume the astronaut and fall through
    /// the source `SCZ00` conversion.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L804-L841>.
    pub fn continue_lander_pull_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;
        let outcome =
            self.continue_lander_pull_with_layout(&layout, process_address, object_address, None)?;
        Ok(self.lander_flee_outcome_to_step(process_address, object_address, outcome))
    }

    pub(super) fn step_lander_grab_with_layout(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
        start_grab: bool,
    ) -> Result<RedLabelLanderProcessStep, String> {
        if start_grab {
            self.write_object_bytes(
                layout,
                object_address,
                "OTYP",
                &[self
                    .read_object_byte(layout, object_address, "OTYP")?
                    .wrapping_add(1)],
            )?;
            self.write_object_word(layout, object_address, "OXV", 0)?;
            self.write_object_word(layout, object_address, "OYV", 0)?;
            self.write_object_word(
                layout,
                object_address,
                "OPICT",
                red_label_object_picture_address("LNDP1")?,
            )?;
        }

        let target = match self.live_lander_target(layout, process_address)? {
            Some(target) => target,
            None => {
                return self.retarget_or_convert_lander(layout, process_address, object_address);
            }
        };

        let target_x_masked =
            self.read_object_word(layout, target.target_object_address, "OX16")? & 0xFFE0;
        let lander_x_masked = self.read_object_word(layout, object_address, "OX16")? & 0xFFE0;
        if lander_x_masked != target_x_masked {
            let x_step = if (lander_x_masked as i16) < (target_x_masked as i16) {
                sign_extend_u8_to_u16(0x20)
            } else {
                sign_extend_u8_to_u16(0xE0)
            };
            let x16 = self
                .read_object_word(layout, object_address, "OX16")?
                .wrapping_add(x_step);
            self.write_object_word(layout, object_address, "OX16", x16)?;
        }

        let target_y = self
            .read_object_byte(layout, target.target_object_address, "OY16")?
            .wrapping_sub(12);
        let lander_y = self.read_object_byte(layout, object_address, "OY16")?;
        if target_y != lander_y {
            let mut y_velocity = self.read_field_word(layout, "enemy_runtime", "LNDYV")?;
            if target_y < lander_y {
                y_velocity = !y_velocity;
            }
            let y16 = self
                .read_object_word(layout, object_address, "OY16")?
                .wrapping_add(y_velocity);
            self.write_object_word(layout, object_address, "OY16", y16)?;
            return self.sleep_lander_grab(layout, process_address, object_address, target);
        }

        let capture_delta = self
            .read_object_word(layout, object_address, "OX16")?
            .wrapping_add(0x0040)
            .wrapping_sub(self.read_object_word(layout, target.target_object_address, "OX16")?);
        if capture_delta > 0x0080 {
            return self.sleep_lander_grab(layout, process_address, object_address, target);
        }

        self.write_object_word(
            layout,
            object_address,
            "OCVECT",
            red_label_routine_address("LKIL1")?,
        )?;
        let split_velocity = !self.read_field_word(layout, "enemy_runtime", "LNDYV")?;
        self.write_object_word(layout, object_address, "OYV", split_velocity)?;
        self.write_object_word(layout, target.target_object_address, "OYV", split_velocity)?;
        let sound_loaded = self.load_sound_table_by_label("LPKSND")?;
        self.write_object_word(
            layout,
            target.target_object_address,
            "OCVECT",
            red_label_routine_address("AKIL1")?,
        )?;
        let flee = self.step_lander_flee_with_layout(layout, process_address, object_address)?;
        Ok(RedLabelLanderProcessStep::PassengerCapturedFleeing {
            process_address,
            object_address,
            target,
            lander_y_velocity: split_velocity,
            target_y_velocity: split_velocity,
            sound_loaded,
            flee,
        })
    }

    pub(super) fn sleep_lander_grab(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
        target: RedLabelLanderTarget,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let (shot_timer, shot) =
            self.run_lander_shot_timer(layout, process_address, object_address)?;
        let wakeup_address = red_label_routine_address("LANDG")?;
        self.sleep_current_process(1, wakeup_address)?;
        Ok(RedLabelLanderProcessStep::GrabSleeping {
            process_address,
            object_address,
            target,
            x16: self.read_object_word(layout, object_address, "OX16")?,
            y16: self.read_object_word(layout, object_address, "OY16")?,
            shot_timer,
            shot,
            wakeup_address,
        })
    }

    pub(super) fn step_lander_flee_with_layout(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<RedLabelLanderFleeOutcome, String> {
        if self.read_object_byte(layout, object_address, "OY16")? <= RED_LABEL_Y_MIN + 8 {
            let suck_sound_loaded = self.load_sound_table_by_label("LSKSND")?;
            return self.continue_lander_pull_with_layout(
                layout,
                process_address,
                object_address,
                suck_sound_loaded,
            );
        }

        let (shot_timer, shot) =
            self.run_lander_shot_timer(layout, process_address, object_address)?;
        let wakeup_address = red_label_routine_address("LANDF")?;
        self.sleep_current_process(4, wakeup_address)?;
        Ok(RedLabelLanderFleeOutcome::FleeSleeping {
            shot_timer,
            shot,
            wakeup_address,
        })
    }

    pub(super) fn continue_lander_pull_with_layout(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
        suck_sound_loaded: Option<RedLabelLoadedSoundTable>,
    ) -> Result<RedLabelLanderFleeOutcome, String> {
        let target_slot_address = self.read_process_data_word(layout, process_address, "PD4")?;
        if self.read_word(target_slot_address)? == 0 {
            let previous_object_link_address = self.kill_object_cell_offscreen(object_address)?;
            let active_lander_count = self
                .read_field_byte(layout, "enemy_runtime", "LNDCNT")?
                .wrapping_sub(1);
            self.write_field_byte(layout, "enemy_runtime", "LNDCNT", active_lander_count)?;
            let lander_reserve_count = self
                .read_field_byte(layout, "enemy_runtime", "LNDRES")?
                .wrapping_add(1);
            self.write_field_byte(layout, "enemy_runtime", "LNDRES", lander_reserve_count)?;
            let killed_process = self.kill_current_process(layout)?;
            return Ok(RedLabelLanderFleeOutcome::GaveUpCompleted {
                suck_sound_loaded,
                previous_object_link_address,
                active_lander_count,
                lander_reserve_count,
                killed_process,
            });
        }

        let target_object_address = self.read_process_data_word(layout, process_address, "PD2")?;
        object_table_for_address(layout, target_object_address)?;
        let target = RedLabelLanderTarget {
            target_slot_address,
            target_object_address,
        };
        self.write_object_word(layout, object_address, "OYV", 0)?;
        self.write_object_word(layout, target_object_address, "OYV", 0)?;

        let target_y = self.read_object_byte(layout, target_object_address, "OY16")?;
        let lander_y = self.read_object_byte(layout, object_address, "OY16")?;
        if target_y > lander_y {
            let y16_range = object_field_range_for_address(layout, target_object_address, "OY16")?;
            self.write_byte(y16_range.start, target_y.wrapping_sub(1))?;
            let target_y16 = self.read_object_word(layout, target_object_address, "OY16")?;
            let sound_command = red_label_sound_output_command(RED_LABEL_LANDER_PULL_SOUND_NUMBER);
            let wakeup_address = red_label_routine_address("LNDFXA")?;
            self.sleep_current_process(1, wakeup_address)?;
            return Ok(RedLabelLanderFleeOutcome::PullingPassengerSleeping {
                target,
                target_y16,
                suck_sound_loaded,
                sound_command,
                wakeup_address,
            });
        }

        let [x, y] = self
            .read_object_screen_address(layout, target_object_address)?
            .to_be_bytes();
        let center = u16::from_be_bytes([x.wrapping_add(1), y]);
        self.write_field_word(layout, "base_page", "CENTMP", center)?;
        let astronaut_kill = self.kill_astronaut_unchecked(layout, target_object_address)?;
        let (_, _, schizoid) =
            self.convert_current_lander_to_schizoid_state(layout, process_address, object_address)?;
        Ok(RedLabelLanderFleeOutcome::PassengerConsumedConverted {
            target,
            center,
            suck_sound_loaded,
            astronaut_kill,
            schizoid,
        })
    }

    pub(super) fn lander_flee_outcome_to_step(
        &self,
        process_address: u16,
        object_address: u16,
        outcome: RedLabelLanderFleeOutcome,
    ) -> RedLabelLanderProcessStep {
        match outcome {
            RedLabelLanderFleeOutcome::FleeSleeping {
                shot_timer,
                shot,
                wakeup_address,
            } => RedLabelLanderProcessStep::FleeSleeping {
                process_address,
                object_address,
                shot_timer,
                shot,
                wakeup_address,
            },
            RedLabelLanderFleeOutcome::PullingPassengerSleeping {
                target,
                target_y16,
                suck_sound_loaded,
                sound_command,
                wakeup_address,
            } => RedLabelLanderProcessStep::PullingPassengerSleeping {
                process_address,
                object_address,
                target,
                target_y16,
                suck_sound_loaded,
                sound_command,
                wakeup_address,
            },
            RedLabelLanderFleeOutcome::PassengerConsumedConverted {
                target,
                center,
                suck_sound_loaded,
                astronaut_kill,
                schizoid,
            } => RedLabelLanderProcessStep::PassengerConsumedConverted {
                process_address,
                object_address,
                target,
                center,
                suck_sound_loaded,
                astronaut_kill,
                schizoid,
            },
            RedLabelLanderFleeOutcome::GaveUpCompleted {
                suck_sound_loaded,
                previous_object_link_address,
                active_lander_count,
                lander_reserve_count,
                killed_process,
            } => RedLabelLanderProcessStep::GaveUpCompleted {
                process_address,
                object_address,
                suck_sound_loaded,
                previous_object_link_address,
                active_lander_count,
                lander_reserve_count,
                killed_process,
            },
        }
    }

    pub(super) fn retarget_or_convert_lander(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let object_type = self.read_object_byte(layout, object_address, "OTYP")? & 0xFE;
        self.write_object_bytes(layout, object_address, "OTYP", &[object_type])?;
        let Some(target) = self.select_lander_target(layout, process_address)? else {
            let (active_lander_count, active_schizoid_count, schizoid) = self
                .convert_current_lander_to_schizoid_state(
                    layout,
                    process_address,
                    object_address,
                )?;
            return Ok(RedLabelLanderProcessStep::ConvertedToSchizoid {
                process_address,
                object_address,
                active_lander_count,
                active_schizoid_count,
                schizoid,
            });
        };

        self.orbit_lander_current_process(layout, process_address, object_address, Some(target))
    }

    pub(super) fn orbit_lander_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
        target: Option<RedLabelLanderTarget>,
    ) -> Result<RedLabelLanderProcessStep, String> {
        let altitude = self.object_altitude_from_alttbl(layout, object_address)?;
        let above_ground_delta = altitude.wrapping_sub(50);
        let object_y = self.read_object_byte(layout, object_address, "OY16")?;
        let y_delta = above_ground_delta.wrapping_sub(object_y);
        let y_velocity = if above_ground_delta > object_y {
            self.read_field_word(layout, "enemy_runtime", "LNDYV")?
        } else if (y_delta as i8) < -20 {
            !self.read_field_word(layout, "enemy_runtime", "LNDYV")?
        } else {
            0
        };
        self.write_object_word(layout, object_address, "OYV", y_velocity)?;

        let null_picture = red_label_object_picture_address("NULOB")?;
        let mut picture_address = self.read_object_word(layout, object_address, "OPICT")?;
        let mut shot_timer = self.read_process_byte(layout, process_address, "PD6")?;
        let mut shot = None;
        if picture_address != null_picture {
            (shot_timer, shot) =
                self.run_lander_shot_timer(layout, process_address, object_address)?;
            picture_address = picture_address.wrapping_add(10);
            if picture_address > red_label_object_picture_address("LNDP3")? {
                picture_address = red_label_object_picture_address("LNDP1")?;
            }
            self.write_object_word(layout, object_address, "OPICT", picture_address)?;
        }

        let wakeup_address = red_label_routine_address("LANDS0")?;
        self.sleep_current_process(6, wakeup_address)?;
        Ok(RedLabelLanderProcessStep::OrbitSleeping {
            process_address,
            object_address,
            target,
            altitude,
            y_velocity,
            shot_timer,
            shot,
            picture_address,
            wakeup_address,
        })
    }

    pub(super) fn run_lander_shot_timer(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<(u8, Option<RedLabelEnemyShot>), String> {
        let shot_timer = self
            .read_process_byte(layout, process_address, "PD6")?
            .wrapping_sub(1);
        self.write_process_byte(layout, process_address, "PD6", shot_timer)?;
        if shot_timer != 0 {
            return Ok((shot_timer, None));
        }

        let shot_timer_max = self.read_field_byte(layout, "enemy_runtime", "LDSTIM")?;
        let shot_timer = self.advance_red_label_rmax(layout, shot_timer_max)?;
        self.write_process_byte(layout, process_address, "PD6", shot_timer)?;
        let shot =
            self.shoot_at_player_from_object_current_process(layout, object_address, "LSHSND")?;
        Ok((shot_timer, shot))
    }

    pub(super) fn live_lander_target(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
    ) -> Result<Option<RedLabelLanderTarget>, String> {
        let target_slot_address = self.read_process_data_word(layout, process_address, "PD4")?;
        if self.read_word(target_slot_address)? == 0 {
            return Ok(None);
        }
        let target_object_address = self.read_process_data_word(layout, process_address, "PD2")?;
        if object_table_for_address(layout, target_object_address).is_err() {
            return Ok(None);
        }
        let target_collision = self.read_object_word(layout, target_object_address, "OCVECT")?;
        if target_collision.to_be_bytes()[1]
            != red_label_routine_address("ASTKIL")?.to_be_bytes()[1]
        {
            return Ok(None);
        }
        Ok(Some(RedLabelLanderTarget {
            target_slot_address,
            target_object_address,
        }))
    }

    pub(super) fn select_lander_target(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
    ) -> Result<Option<RedLabelLanderTarget>, String> {
        if self.read_field_byte(layout, "base_page", "ASTCNT")? == 0 {
            return Ok(None);
        }

        let target_list = field_range(layout, "target_list", "TLIST")?;
        let scan_start = target_list.start;
        let scan_end = scan_start.wrapping_add(64);
        let original_cursor = self.read_field_word(layout, "base_page", "TPTR")?;
        let mut cursor = original_cursor;
        for _ in 0..32 {
            cursor = cursor.wrapping_add(2);
            if cursor >= scan_end {
                cursor = scan_start;
            }
            let target_object_address = self.read_word(cursor)?;
            if target_object_address != 0 {
                if object_table_for_address(layout, target_object_address).is_err() {
                    continue;
                }
                self.write_field_word(layout, "base_page", "TPTR", cursor)?;
                self.write_process_data_word(
                    layout,
                    process_address,
                    "PD2",
                    target_object_address,
                )?;
                self.write_process_data_word(layout, process_address, "PD4", cursor)?;
                return Ok(Some(RedLabelLanderTarget {
                    target_slot_address: cursor,
                    target_object_address,
                }));
            }
            if cursor == original_cursor {
                break;
            }
        }
        Ok(None)
    }

    pub(super) fn convert_current_lander_to_schizoid_state(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<(u8, u8, RedLabelSchizoidProcessStep), String> {
        let active_lander_count = self
            .read_field_byte(layout, "enemy_runtime", "LNDCNT")?
            .wrapping_sub(1);
        self.write_field_byte(layout, "enemy_runtime", "LNDCNT", active_lander_count)?;
        let active_schizoid_count = self
            .read_field_byte(layout, "enemy_runtime", "SCZCNT")?
            .wrapping_add(1);
        self.write_field_byte(layout, "enemy_runtime", "SCZCNT", active_schizoid_count)?;
        self.write_object_bytes(layout, object_address, "OTYP", &[0])?;
        self.write_object_word(
            layout,
            object_address,
            "OPICT",
            red_label_object_picture_address("SCZP1")?,
        )?;
        self.write_object_word(layout, object_address, "OBJCOL", 0xCC33)?;
        self.write_object_word(
            layout,
            object_address,
            "OCVECT",
            red_label_routine_address("SCZKIL")?,
        )?;
        let shot_timer = self.read_field_byte(layout, "enemy_runtime", "SZSTIM")?;
        self.write_process_byte(layout, process_address, "PD2", shot_timer)?;
        let schizoid = self.step_schizoid_current_process()?;
        Ok((active_lander_count, active_schizoid_count, schizoid))
    }

    pub(super) fn update_ufo_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        honor_seek_probability: bool,
    ) -> Result<Option<RedLabelUfoVelocityUpdate>, String> {
        if honor_seek_probability
            && self.read_field_byte(layout, "base_page", "SEED")?
                <= self.read_field_byte(layout, "enemy_runtime", "UFOSK")?
        {
            return Ok(None);
        }

        let mut x_seek_byte = 0x40;
        let mut y_seek_byte = 0x01;
        let object_x = self.read_object_word(layout, object_address, "OX16")?;
        let player_x = self.read_field_word(layout, "base_page", "PLABX")?;
        let x_delta = object_x.wrapping_sub(player_x);
        if x_delta & 0x8000 == 0 {
            x_seek_byte = 0u8.wrapping_sub(x_seek_byte);
        }

        let x_velocity = if x_delta.wrapping_add(20 * 32) > 40 * 32 {
            let velocity = sign_extend_u8_to_u16(x_seek_byte)
                .wrapping_add(self.read_word(field_range(layout, "base_page", "PLAXV")?.start)?);
            self.write_object_word(layout, object_address, "OXV", velocity)?;
            Some(velocity)
        } else {
            None
        };

        let object_y = self.read_object_byte(layout, object_address, "OY16")?;
        let player_y = self.read_field_byte(layout, "base_page", "PLAYC")?;
        let y_delta = object_y.wrapping_sub(player_y);
        if y_delta & 0x80 == 0 {
            y_seek_byte = 0u8.wrapping_sub(y_seek_byte);
        }

        let y_velocity = if y_delta.wrapping_add(10) > 20 {
            let velocity = arithmetic_shift_right_word(
                u16::from_be_bytes([y_seek_byte, 0]).wrapping_add(self.read_field_word(
                    layout,
                    "base_page",
                    "PLAYV",
                )?),
                1,
            );
            self.write_object_word(layout, object_address, "OYV", velocity)?;
            Some(velocity)
        } else {
            None
        };

        Ok(Some(RedLabelUfoVelocityUpdate {
            x_velocity,
            y_velocity,
        }))
    }

    /// Source-shaped `SCZ0`: seek horizontally, choose avoid/seek vertical
    /// velocity, apply the random Y hop, run the shot timer, and sleep back to
    /// `SCZ0`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L844-L901>.
    pub fn step_schizoid_current_process(&mut self) -> Result<RedLabelSchizoidProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let object_address = self.read_process_data_word(&layout, process_address, "PD")?;
        object_table_for_address(&layout, object_address)?;

        let player_absolute_x = self.read_field_word(&layout, "base_page", "PLABX")?;
        let object_absolute_x = self.read_object_word(&layout, object_address, "OX16")?;
        let x_velocity_low = if signed_word_greater_or_equal(player_absolute_x, object_absolute_x) {
            self.read_field_byte(&layout, "enemy_runtime", "SZXV")?
        } else {
            0u8.wrapping_sub(self.read_field_byte(&layout, "enemy_runtime", "SZXV")?)
        };
        let x_velocity = sign_extend_u8_to_u16(x_velocity_low);
        self.write_object_word(&layout, object_address, "OXV", x_velocity)?;

        let mut should_run_random_hop_and_shot = true;
        let x_distance = player_absolute_x
            .wrapping_sub(object_absolute_x)
            .wrapping_add(380);
        let y_velocity = if x_distance <= 0x0700 {
            let player_y = self.read_field_byte(&layout, "base_page", "PLAYC")?;
            let object_y = self.read_object_byte(&layout, object_address, "OY16")?;
            let base_y_velocity = self.read_field_word(&layout, "enemy_runtime", "SZYV")?;
            let velocity = if player_y >= object_y {
                base_y_velocity
            } else {
                !base_y_velocity
            };
            self.write_object_word(&layout, object_address, "OYV", velocity)?;
            if self.read_object_screen_address(&layout, object_address)? == 0 {
                should_run_random_hop_and_shot = false;
            }
            velocity
        } else {
            let player_y = self.read_field_byte(&layout, "base_page", "PLAYC")?;
            let object_y = self.read_object_byte(&layout, object_address, "OY16")?;
            let base_y_velocity = self.read_field_word(&layout, "enemy_runtime", "SZYV")?;
            let delta = player_y.wrapping_sub(object_y);
            let velocity = if player_y > object_y {
                if delta > 8 { 0 } else { !base_y_velocity }
            } else if (delta as i8) > -8 {
                base_y_velocity
            } else {
                0
            };
            self.write_object_word(&layout, object_address, "OYV", velocity)?;
            velocity
        };

        let mut y_position = self.read_object_byte(&layout, object_address, "OY16")?;
        let mut shot_timer = self.read_process_byte(&layout, process_address, "PD2")?;
        let mut shot = None;
        if should_run_random_hop_and_shot {
            let y_step = if self.read_field_byte(&layout, "base_page", "SEED")? & 0x80 == 0 {
                0u8.wrapping_sub(self.read_field_byte(&layout, "enemy_runtime", "SZRY")?)
            } else {
                self.read_field_byte(&layout, "enemy_runtime", "SZRY")?
            };
            y_position = y_position.wrapping_add(y_step);
            if y_position < RED_LABEL_Y_MIN {
                y_position = RED_LABEL_Y_MAX;
            }
            let y16_range = object_field_range_for_address(&layout, object_address, "OY16")?;
            self.write_byte(y16_range.start, y_position)?;

            shot_timer = self
                .read_process_byte(&layout, process_address, "PD2")?
                .wrapping_sub(1);
            self.write_process_byte(&layout, process_address, "PD2", shot_timer)?;
            if shot_timer == 0 {
                let shot_timer_max = self.read_field_byte(&layout, "enemy_runtime", "SZSTIM")?;
                let timer_state = self.advance_red_label_rand(&layout)?;
                shot_timer = rmax(shot_timer_max, timer_state.seed);
                self.write_process_byte(&layout, process_address, "PD2", shot_timer)?;
                shot = self.shoot_at_player_from_object_current_process(
                    &layout,
                    object_address,
                    "SSHSND",
                )?;
            }
        }

        let wakeup_address = red_label_routine_address("SCZ0")?;
        self.sleep_current_process(3, wakeup_address)?;
        Ok(RedLabelSchizoidProcessStep {
            process_address,
            object_address,
            x_velocity,
            y_velocity,
            y_position,
            shot_timer,
            shot,
            wakeup_address,
        })
    }

    /// Source-shaped `SHOOT`: allocate a fireball shell and aim it at the
    /// current player using `SEED`, `LSEED`, `PLAXC`, `PLAYC`, and optional
    /// horizontal player-velocity lead.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L531-L570>.
    pub(super) fn shoot_at_player_from_object_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        firing_object_address: u16,
        sound_label: &str,
    ) -> Result<Option<RedLabelEnemyShot>, String> {
        let process_address = self.current_process_address(layout)?;
        let descriptor = RedLabelShellDescriptor {
            output_routine_address: red_label_routine_address("FBOUT")?,
            picture_address: red_label_object_picture_address("BMBP1")?,
            kill_routine_address: red_label_routine_address("BKIL")?,
        };
        let Some(shell) =
            self.get_shell_cell(firing_object_address, process_address, descriptor)?
        else {
            return Ok(None);
        };

        let seed = self.read_field_byte(layout, "base_page", "SEED")?;
        let player_x = self.read_byte(field_range(layout, "base_page", "PLAXC")?.start)?;
        let shell_x = self.read_object_byte(layout, shell.shell_address, "OBJX")?;
        let x_delta = (seed & 0x1F)
            .wrapping_sub(0x10)
            .wrapping_add(player_x)
            .wrapping_sub(shell_x);
        let mut x_velocity = sign_extend_u8_to_u16(x_delta).wrapping_shl(2);
        if seed > 120 {
            let player_velocity =
                self.read_word(field_range(layout, "base_page", "PLAXV")?.start)?;
            x_velocity = x_velocity.wrapping_add(player_velocity.wrapping_shl(2));
        }
        self.write_object_word(layout, shell.shell_address, "OXV", x_velocity)?;

        let player_y = self.read_field_byte(layout, "base_page", "PLAYC")?;
        let shell_y = self.read_object_byte(layout, shell.shell_address, "OBJY")?;
        let y_delta = (self.read_field_byte(layout, "base_page", "LSEED")? & 0x1F)
            .wrapping_sub(0x10)
            .wrapping_add(player_y)
            .wrapping_sub(shell_y);
        let y_velocity = sign_extend_u8_to_u16(y_delta).wrapping_shl(2);
        self.write_object_word(layout, shell.shell_address, "OYV", y_velocity)?;

        Ok(Some(RedLabelEnemyShot {
            shell,
            x_velocity,
            y_velocity,
            sound_loaded: self.load_sound_table_by_label(sound_label)?,
        }))
    }

    pub(super) fn restore_probe_reserve_from_plres(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<RedLabelProbeRestore>, String> {
        let reserve_count = self.read_field_byte(layout, "enemy_runtime", "PRBRES")?;
        self.write_field_byte(layout, "enemy_runtime", "PRBCNT", reserve_count)?;
        if reserve_count == 0 {
            return Ok(None);
        }

        self.write_field_byte(layout, "enemy_runtime", "PRBRES", 0)?;
        let restore = self.start_probe_restore_group(layout, reserve_count)?;
        Ok(Some(restore))
    }

    pub(super) fn start_probe_restore_group(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        count: u8,
    ) -> Result<RedLabelProbeRestore, String> {
        let xtemp = field_range(layout, "base_page", "XTEMP")?.start;
        self.write_byte(xtemp, count)?;
        let process_address = self.current_process_address(layout)?;
        let mut created_objects = Vec::with_capacity(usize::from(count));

        while self.read_byte(xtemp)? != 0 {
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("PRBP1")?,
                collision_vector_address: red_label_routine_address("PRBKIL")?,
                scanner_color: 0xCCCC,
            };
            let object = self.init_object_cell(process_address, descriptor)?;

            let state = self.advance_red_label_rand(layout)?;
            let x16 = u16::from_be_bytes([(state.hseed & 0x3F).wrapping_add(0x10), state.lseed]);
            self.write_object_word(layout, object.object_address, "OX16", x16)?;

            let y16 =
                u16::from_be_bytes([state.lseed.wrapping_shr(1).wrapping_add(RED_LABEL_Y_MIN), 0]);
            let y16_range = object_field_range_for_address(layout, object.object_address, "OY16")?;
            self.write_byte(y16_range.start, y16.to_be_bytes()[0])?;

            let xv_low = (state.seed & 0x3F).wrapping_sub(0x20);
            let xv = sign_extend_u8_to_u16(xv_low);
            self.write_object_word(layout, object.object_address, "OXV", xv)?;

            let mut yv_low = (state.lseed & 0x7F).wrapping_sub(0x40);
            if yv_low & 0x80 == 0 {
                yv_low |= 0x20;
            } else {
                yv_low &= 0xDF;
            }
            let yv = sign_extend_u8_to_u16(yv_low);
            self.write_object_word(layout, object.object_address, "OYV", yv)?;

            let appearance = self.start_appearance_for_object(object.object_address)?;
            created_objects.push(RedLabelProbeRestoreObject {
                object,
                appearance,
                x16,
                y16,
                xv,
                yv,
            });

            let remaining = self.read_byte(xtemp)?.wrapping_sub(1);
            self.write_byte(xtemp, remaining)?;
        }

        Ok(RedLabelProbeRestore {
            reserve_count: count,
            active_count: self.read_field_byte(layout, "enemy_runtime", "PRBCNT")?,
            created_objects,
        })
    }

    pub(super) fn restore_tie_reserve_from_plres(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<RedLabelTieRestore>, String> {
        let reserve_count = self.read_field_byte(layout, "enemy_runtime", "TIERES")?;
        self.write_field_byte(layout, "enemy_runtime", "TIECNT", reserve_count)?;
        if reserve_count == 0 {
            return Ok(None);
        }

        let mut squads = Vec::new();
        while self.read_field_byte(layout, "enemy_runtime", "TIERES")? != 0 {
            let reserve_before = self.read_field_byte(layout, "enemy_runtime", "TIERES")?;
            let squad_count = if reserve_before > 3 {
                3
            } else {
                reserve_before
            };
            squads.push(self.start_tie_restore_squad(layout, squad_count)?);
            self.write_field_byte(
                layout,
                "enemy_runtime",
                "TIERES",
                reserve_before.wrapping_sub(squad_count),
            )?;
        }

        Ok(Some(RedLabelTieRestore {
            reserve_count,
            active_count: self.read_field_byte(layout, "enemy_runtime", "TIECNT")?,
            squads,
        }))
    }

    pub(super) fn start_tie_restore_squad(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        count: u8,
    ) -> Result<RedLabelTieRestoreSquad, String> {
        let xtemp = field_range(layout, "base_page", "XTEMP")?.start;
        self.write_byte(xtemp, count)?;

        let tie_x_velocity = self.read_field_byte(layout, "enemy_runtime", "TIEXV")?;
        let tflg = !self.read_field_byte(layout, "base_page", "TFLG")?;
        self.write_field_byte(layout, "base_page", "TFLG", tflg)?;
        let velocity_low = if tflg & 0x80 != 0 {
            tie_x_velocity
        } else {
            0u8.wrapping_sub(tie_x_velocity)
        };
        let x_velocity = sign_extend_u8_to_u16(velocity_low);
        self.write_byte(xtemp + 1, velocity_low)?;

        let super_process = self.make_super_process(
            red_label_routine_address("TIE")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        )?;
        let process_data = process_field_range_for_address(
            layout,
            process_table_for_address(layout, super_process.process_address)?,
            super_process.process_address,
            "PDATA",
        )?;
        self.clear_range(process_data.clone())?;
        self.write_byte(process_data.start + 8, count)?;

        let mut objects = Vec::with_capacity(usize::from(count));
        while self.read_byte(xtemp)? != 0 {
            let remaining = self.read_byte(xtemp)?;
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("TIEP1")?,
                collision_vector_address: red_label_routine_address("TIEKIL")?,
                scanner_color: 0x8888,
            };
            let object = self.init_object_cell(super_process.process_address, descriptor)?;
            self.write_object_word(layout, object.object_address, "OXV", x_velocity)?;
            self.write_object_word(layout, object.object_address, "OYV", 0)?;

            let spacing = u16::from(remaining).wrapping_mul(0x0180);
            let x16 = self
                .read_field_word(layout, "base_page", "PLABX")?
                .wrapping_add(spacing)
                .wrapping_add(0x8000);
            self.write_object_word(layout, object.object_address, "OX16", x16)?;

            let y16 = u16::from_be_bytes([0x50, 0]);
            let y16_range = object_field_range_for_address(layout, object.object_address, "OY16")?;
            self.write_byte(y16_range.start, y16.to_be_bytes()[0])?;
            self.write_byte(process_data.start + 9, y16.to_be_bytes()[0])?;
            self.write_object_word(
                layout,
                object.object_address,
                "OBJID",
                super_process.process_address,
            )?;
            self.activate_object_cell(object.object_address)?;

            let object_slot = process_data
                .start
                .wrapping_add(u16::from(remaining).wrapping_mul(2))
                .wrapping_sub(2);
            self.write_word(object_slot, object.object_address)?;
            objects.push(RedLabelTieRestoreObject {
                object,
                x16,
                y16,
                xv: x_velocity,
            });

            self.write_byte(xtemp, remaining.wrapping_sub(1))?;
        }

        Ok(RedLabelTieRestoreSquad {
            squad_count: count,
            super_process,
            x_velocity,
            cruise_altitude: 0x50,
            objects,
        })
    }

    /// Source-shaped `TIE`: choose one squad object from `PD`/`PD2`/`PD4`/`PD6`
    /// using `SEED`, advance its picture and Y velocity, optionally adjust to
    /// cruise altitude or player Y, optionally run `BOMBST`, then sleep one
    /// tick back to `TIE`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1024-L1116>.
    pub fn step_tie_current_process(&mut self) -> Result<RedLabelTieProcessStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        let process_table = process_table_for_address(&layout, process_address)?;
        let process_data =
            process_field_range_for_address(&layout, process_table, process_address, "PDATA")?;
        let seed = self.read_field_byte(&layout, "base_page", "SEED")?;
        let selected_slot_address = process_data.start + u16::from(seed & 0x06);
        let object_address = self.read_word(selected_slot_address)?;
        let wakeup_address = red_label_routine_address("TIE")?;

        if object_address == 0 {
            self.sleep_current_process(1, wakeup_address)?;
            return Ok(RedLabelTieProcessStep {
                process_address,
                selected_slot_address,
                object_address: None,
                previous_picture_address: None,
                picture_address: None,
                y_velocity_after_random: None,
                y_velocity: None,
                cruise_altitude: None,
                bomb_attempted: false,
                bomb: None,
                wakeup_address,
            });
        }

        object_table_for_address(&layout, object_address)?;
        let previous_picture_address = self.read_object_word(&layout, object_address, "OPICT")?;
        let picture_address = self.next_tie_picture(seed, previous_picture_address)?;
        self.write_object_word(&layout, object_address, "OPICT", picture_address)?;

        let random_delta_low = (seed & 0x3F).wrapping_sub(0x20);
        let mut y_velocity = self
            .read_object_word(&layout, object_address, "OYV")?
            .wrapping_add(sign_extend_u8_to_u16(random_delta_low));
        self.write_object_word(&layout, object_address, "OYV", y_velocity)?;
        let y_velocity_after_random = y_velocity;

        let damping_high = y_velocity.wrapping_shl(3).to_be_bytes()[0];
        let damping_delta = sign_extend_u8_to_u16(0u8.wrapping_sub(damping_high));
        y_velocity = y_velocity.wrapping_add(damping_delta);
        self.write_object_word(&layout, object_address, "OYV", y_velocity)?;

        let mut cruise_altitude = None;
        let mut bomb_attempted = false;
        let mut bomb = None;
        let object_screen_y = self.read_object_byte(&layout, object_address, "OBJY")?;
        if object_screen_y == 0 {
            y_velocity = self.update_offscreen_tie_y_velocity(
                &layout,
                process_data.start,
                object_address,
                seed,
                y_velocity,
                &mut cruise_altitude,
            )?;
        } else {
            let player_y = self.read_field_byte(&layout, "base_page", "PLAYC")?;
            if let Some(delta) = tie_onscreen_y_velocity_delta(object_screen_y, player_y) {
                y_velocity = y_velocity.wrapping_add(delta);
                self.write_object_word(&layout, object_address, "OYV", y_velocity)?;
            }

            if self.read_field_byte(&layout, "base_page", "LSEED")? & 0x07 == 0 {
                bomb_attempted = true;
                bomb = self.start_tie_bomb_shell(&layout, process_address, object_address)?;
            }
        }

        self.sleep_current_process(1, wakeup_address)?;
        Ok(RedLabelTieProcessStep {
            process_address,
            selected_slot_address,
            object_address: Some(object_address),
            previous_picture_address: Some(previous_picture_address),
            picture_address: Some(picture_address),
            y_velocity_after_random: Some(y_velocity_after_random),
            y_velocity: Some(y_velocity),
            cruise_altitude,
            bomb_attempted,
            bomb,
            wakeup_address,
        })
    }

    pub(super) fn next_tie_picture(
        &self,
        seed: u8,
        previous_picture_address: u16,
    ) -> Result<u16, String> {
        let low = red_label_object_picture_address("TIEP1")?;
        let high = red_label_object_picture_address("TIEP4")?;
        let random_delta = (seed & 0x3F).wrapping_sub(0x20);
        let candidate = if random_delta & 0x80 != 0 {
            previous_picture_address.wrapping_add(RED_LABEL_TIE_PICTURE_STEP)
        } else {
            previous_picture_address.wrapping_sub(RED_LABEL_TIE_PICTURE_STEP)
        };
        Ok(candidate.clamp(low, high))
    }

    pub(super) fn update_offscreen_tie_y_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_data_start: u16,
        object_address: u16,
        seed: u8,
        mut y_velocity: u16,
        cruise_altitude: &mut Option<u8>,
    ) -> Result<u16, String> {
        let cruise_address = process_data_start + 9;
        let mut cruise = self.read_byte(cruise_address)?;
        if seed <= 0x40 {
            let candidate = cruise.wrapping_add((seed & 0x03).wrapping_sub(2));
            cruise = candidate.clamp(0x40, 0x68);
            self.write_byte(cruise_address, cruise)?;
        }
        *cruise_altitude = Some(cruise);

        let object_y = self.read_object_byte(layout, object_address, "OY16")?;
        let delta = cruise.wrapping_sub(object_y).wrapping_add(0x10);
        if delta > 0x20 {
            let delta_after_center = delta.wrapping_sub(0x10);
            let velocity_delta = if delta_after_center & 0x80 == 0 {
                0xFFF0
            } else {
                0x0010
            };
            y_velocity = y_velocity.wrapping_add(velocity_delta);
            self.write_object_word(layout, object_address, "OYV", y_velocity)?;
        }

        Ok(y_velocity)
    }

    /// Source-shaped `BOMBST`: cap bomb shells at ten, allocate a `BMBOUT`
    /// shell through `GETSHL`, and replace `ODATA` with the source lifetime
    /// `(SEED & $1F) + 1`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1134-L1149>.
    pub(super) fn start_tie_bomb_shell(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        object_address: u16,
    ) -> Result<Option<RedLabelTieBombStart>, String> {
        if self.read_field_byte(layout, "base_page", "BMBCNT")? >= RED_LABEL_TIE_BOMB_SHELL_LIMIT {
            return Ok(None);
        }

        let descriptor = RedLabelShellDescriptor {
            output_routine_address: red_label_routine_address("BMBOUT")?,
            picture_address: red_label_object_picture_address("BMBP1")?,
            kill_routine_address: red_label_routine_address("BKIL")?,
        };
        let Some(shell) = self.get_shell_cell(object_address, process_address, descriptor)? else {
            return Ok(None);
        };

        let discarded_hseed_velocity =
            sign_extend_u8_to_u16(self.read_field_byte(layout, "base_page", "HSEED")?)
                .wrapping_shl(1);
        let lifetime = (self.read_field_byte(layout, "base_page", "SEED")? & 0x1F).wrapping_add(1);
        let data_range = object_field_range_for_address(layout, shell.shell_address, "ODATA")?;
        self.write_byte(data_range.start, lifetime)?;

        Ok(Some(RedLabelTieBombStart {
            shell,
            lifetime,
            discarded_hseed_velocity,
        }))
    }

    pub(super) fn start_astronaut_target_group(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        astro_process_address: u16,
        count: u8,
        x_bank: u8,
        target_writer: &mut RedLabelTargetRestoreWriter,
    ) -> Result<(), String> {
        let xtemp1 = field_range(layout, "base_page", "XTEMP")?.start + 1;
        self.write_byte(xtemp1, count)?;
        while self.read_byte(xtemp1)? != 0 {
            let descriptor = RedLabelObjectDescriptor {
                picture_address: red_label_object_picture_address("ASTP1")?,
                collision_vector_address: red_label_routine_address("ASTKIL")?,
                scanner_color: 0x6666,
            };
            let mut created = self.init_object_cell(astro_process_address, descriptor)?;
            self.advance_red_label_rand(layout)?;
            let hseed = self.read_field_byte(layout, "base_page", "HSEED")?;
            let lseed = self.read_field_byte(layout, "base_page", "LSEED")?;
            let object_x = u16::from_be_bytes([(hseed & 0x1F).wrapping_add(x_bank), lseed]);
            self.write_object_word(layout, created.object_address, "OX16", object_x)?;

            if lseed & 0x01 != 0 {
                created.descriptor.picture_address = red_label_object_picture_address("ASTP3")?;
                self.write_object_word(
                    layout,
                    created.object_address,
                    "OPICT",
                    created.descriptor.picture_address,
                )?;
            }

            let object_y_range =
                object_field_range_for_address(layout, created.object_address, "OY16")?;
            self.write_byte(object_y_range.start, RED_LABEL_ASTRO_RESTORE_Y)?;
            self.write_object_bytes(layout, created.object_address, "OTYP", &[0x10])?;
            self.write_object_word(layout, created.object_address, "OYV", 0)?;
            self.write_object_word(layout, created.object_address, "OXV", 0)?;
            self.write_object_word(layout, created.object_address, "OBJID", 0)?;
            self.activate_object_cell(created.object_address)?;

            let target_entry_end = target_writer
                .cursor
                .checked_add(2)
                .ok_or_else(|| String::from("red-label PLRES target-list cursor overflows"))?;
            if target_entry_end > target_writer.target_list.end {
                return Err(format!(
                    "red-label PLRES target-list cursor 0x{:04X} exceeds TLIST",
                    target_writer.cursor
                ));
            }
            self.write_word(target_writer.cursor, created.object_address)?;
            target_writer.cursor = target_entry_end;
            target_writer.objects.push(created);

            let remaining = self.read_byte(xtemp1)?.wrapping_sub(1);
            self.write_byte(xtemp1, remaining)?;
        }
        Ok(())
    }

    pub(super) fn advance_red_label_rand(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RandState, String> {
        let mut state = RandState {
            seed: self.read_field_byte(layout, "base_page", "SEED")?,
            hseed: self.read_field_byte(layout, "base_page", "HSEED")?,
            lseed: self.read_field_byte(layout, "base_page", "LSEED")?,
        };
        state.advance();
        self.write_field_byte(layout, "base_page", "SEED", state.seed)?;
        self.write_field_byte(layout, "base_page", "HSEED", state.hseed)?;
        self.write_field_byte(layout, "base_page", "LSEED", state.lseed)?;
        Ok(state)
    }

    pub(super) fn advance_red_label_rmax(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        max: u8,
    ) -> Result<u8, String> {
        let state = self.advance_red_label_rand(layout)?;
        Ok(rmax(max, state.seed))
    }

    pub(super) fn write_terrain_status(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        base_status: u8,
    ) -> Result<u8, String> {
        let astronaut_count = self.read_field_byte(layout, "base_page", "ASTCNT")?;
        self.write_status_from_count(layout, base_status, astronaut_count)
    }

    pub(super) fn write_status_from_count(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        base_status: u8,
        count: u8,
    ) -> Result<u8, String> {
        let status = if count == 0 {
            base_status | 0x02
        } else {
            base_status
        };
        self.write_field_byte(layout, "base_page", "STATUS", status)?;
        Ok(status)
    }

    pub(super) fn apply_player_horizontal_damping(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        let velocity_range = field_range(layout, "base_page", "PLAXV")?;
        let [high, middle, low] = self.read_fixed_bytes(velocity_range.start)?;
        let negated_high_word = (!u16::from_be_bytes([high, middle])).wrapping_add(1);
        let sign_extension: u8 = if negated_high_word & 0x8000 == 0 {
            0x00
        } else {
            0xFF
        };
        let shifted = negated_high_word.wrapping_shl(2);
        let (next_middle_low, carry) = u16::from_be_bytes([middle, low]).overflowing_add(shifted);
        let next_high = sign_extension
            .wrapping_add(high)
            .wrapping_add(u8::from(carry));
        let [next_middle, next_low] = next_middle_low.to_be_bytes();
        self.write_range(
            velocity_range.start..velocity_range.start + 3,
            &[next_high, next_middle, next_low],
        )
    }

    pub(super) fn add_signed_word_to_player_x_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        delta: u16,
    ) -> Result<(), String> {
        let velocity_range = field_range(layout, "base_page", "PLAXV")?;
        let [high, middle, low] = self.read_fixed_bytes(velocity_range.start)?;
        let sign_extension: u8 = if delta & 0x8000 == 0 { 0x00 } else { 0xFF };
        let (next_middle_low, carry) = u16::from_be_bytes([middle, low]).overflowing_add(delta);
        let next_high = sign_extension
            .wrapping_add(high)
            .wrapping_add(u8::from(carry));
        let [next_middle, next_low] = next_middle_low.to_be_bytes();
        self.write_range(
            velocity_range.start..velocity_range.start + 3,
            &[next_high, next_middle, next_low],
        )
    }

    pub(super) fn update_player_calculated_x_from_velocity(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        direction: u16,
    ) -> Result<u16, String> {
        let velocity_range = field_range(layout, "base_page", "PLAXV")?;
        let [mut accumulator, mut b, _low] = self.read_fixed_bytes::<3>(velocity_range.start)?;

        for _ in 0..2 {
            let carry = accumulator & 1;
            accumulator = (accumulator >> 1) | (accumulator & 0x80);
            b = (b >> 1) | (carry << 7);
        }

        accumulator = 0;
        let carry = b & 1;
        b = (b >> 1) | (b & 0x80);
        accumulator = (accumulator >> 1) | (carry << 7);

        let mut pcx_high = b;
        let mut pcx_low = accumulator;
        let mut position_high: u8 = if direction & 0x8000 == 0 { 0x20 } else { 0x70 };
        let moving_with_direction = if direction & 0x8000 == 0 {
            pcx_high & 0x80 == 0
        } else {
            pcx_high & 0x80 != 0
        };
        if !moving_with_direction {
            pcx_high = 0;
            pcx_low = 0;
        }

        position_high = position_high.wrapping_add(pcx_high);
        let pcx_range = field_range(layout, "base_page", "PCX")?;
        self.write_word(
            pcx_range.start,
            u16::from_be_bytes([position_high, pcx_low]),
        )?;
        Ok(u16::from_be_bytes([position_high, pcx_low]))
    }

    pub(super) fn write_player_on86_picture(
        &mut self,
        screen_address: u16,
        picture_address: u16,
        alternate_flavor: bool,
    ) -> Result<(), String> {
        let picture = red_label_object_picture(picture_address)?;
        if picture.width != 8 || picture.height != 6 {
            return Err(format!(
                "red-label ON86 expects an 8x6 picture, got `{}` {}x{}",
                picture.label, picture.width, picture.height
            ));
        }

        let image_address = if alternate_flavor {
            picture.alternate_image.unwrap_or(picture.primary_image)
        } else {
            picture.primary_image
        };
        for column in 0..picture.width {
            let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = image_address + u16::from(column) * u16::from(picture.height);
            for row in 0..picture.height {
                let source_byte =
                    red_label_object_image_byte_required(source_column + u16::from(row), picture)?;
                self.write_byte(screen_offset(column_address, u16::from(row))?, source_byte)?;
            }
        }
        Ok(())
    }

    pub(super) fn clear_player_on86_footprint(
        &mut self,
        screen_address: u16,
    ) -> Result<(), String> {
        self.clear_screen_block(screen_address, 8, 6)
    }

    pub(super) fn write_player_thrust_forward(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        screen_address: u16,
    ) -> Result<(), String> {
        let thrust_pointer = self.read_field_word(layout, "base_page", "THX")?;
        let origin = screen_offset_relative(screen_address, -0x100 + 1)?;
        self.write_byte(
            origin,
            self.read_thrust_table_byte(layout, thrust_pointer, 0)?,
        )?;
        self.write_byte(
            screen_offset(origin, 1)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 1)?,
        )?;
        self.write_byte(
            screen_offset(origin, 2)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 5)?,
        )?;
        self.write_byte(
            screen_offset(origin, 3)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 9)?,
        )?;
        self.write_byte(
            screen_offset(origin, 4)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 12)?,
        )?;

        if self.read_field_byte(layout, "base_page", "PIA21")? & 0x02 == 0 {
            return Ok(());
        }

        self.write_byte(
            screen_offset_relative(origin, -0x100 + 1)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 3)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x100 + 2)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 6)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x100 + 3)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 10)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x200 + 1)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 4)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x200 + 2)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 7)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x200 + 3)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 11)?,
        )?;
        self.write_byte(
            screen_offset_relative(origin, -0x300 + 2)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 8)?,
        )
    }

    pub(super) fn write_player_thrust_backward(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        screen_address: u16,
    ) -> Result<(), String> {
        let thrust_pointer = self.read_field_word(layout, "base_page", "THX")?;
        let origin = screen_offset(screen_address, 0x801)?;
        for offset in 0..5 {
            self.write_byte(
                screen_offset(origin, offset)?,
                self.read_thrust_table_byte(layout, thrust_pointer, offset)?,
            )?;
        }

        if self.read_field_byte(layout, "base_page", "PIA21")? & 0x02 == 0 {
            return Ok(());
        }

        self.write_byte(
            screen_offset(origin, 0x101)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 5)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x102)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 6)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x103)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 7)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x201)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 10)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x202)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 11)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x203)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 8)?,
        )?;
        self.write_byte(
            screen_offset(origin, 0x302)?,
            self.read_thrust_table_byte(layout, thrust_pointer, 9)?,
        )
    }

    pub(super) fn clear_player_thrust_forward(
        &mut self,
        screen_address: u16,
    ) -> Result<(), String> {
        let origin = screen_offset_relative(screen_address, -0x100 + 1)?;
        for offset in 0..5 {
            self.write_byte(screen_offset(origin, offset)?, 0)?;
        }
        for offset in [
            -0x100 + 1,
            -0x100 + 2,
            -0x100 + 3,
            -0x200 + 1,
            -0x200 + 2,
            -0x200 + 3,
            -0x300 + 2,
        ] {
            self.write_byte(screen_offset_relative(origin, offset)?, 0)?;
        }
        Ok(())
    }

    pub(super) fn clear_player_thrust_backward(
        &mut self,
        screen_address: u16,
    ) -> Result<(), String> {
        let origin = screen_offset(screen_address, 0x801)?;
        for offset in 0..5 {
            self.write_byte(screen_offset(origin, offset)?, 0)?;
        }
        for offset in [0x101, 0x102, 0x103, 0x201, 0x202, 0x203, 0x302] {
            self.write_byte(screen_offset(origin, offset)?, 0)?;
        }
        Ok(())
    }

    pub(super) fn read_thrust_table_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        thrust_pointer: u16,
        offset: u16,
    ) -> Result<u8, String> {
        let table_range = field_range(layout, "thrust_table", "THTAB")?;
        let address = thrust_pointer.checked_add(offset).ok_or_else(|| {
            format!("red-label THTAB pointer 0x{thrust_pointer:04X}+0x{offset:04X} overflows")
        })?;
        if !table_range.contains(&address) {
            return Err(format!(
                "red-label THTAB byte 0x{address:04X} is outside extracted thrust table"
            ));
        }
        self.read_byte(address)
    }

    pub(super) fn next_player_y_velocity(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        player_y_screen: u8,
        action: PlayerVerticalAction,
    ) -> Result<Option<u16>, String> {
        let current_velocity = self.read_field_word(layout, "base_page", "PLAYV")?;
        let next_velocity = match action {
            PlayerVerticalAction::Idle => 0,
            PlayerVerticalAction::Up => {
                if player_y_screen <= RED_LABEL_Y_MIN + 1 {
                    return Ok(None);
                }
                if current_velocity & 0x8000 == 0 {
                    0xFF00
                } else {
                    let candidate = current_velocity.wrapping_sub(8);
                    if signed_word_greater_or_equal(candidate, 0xFE00) {
                        candidate
                    } else {
                        0xFE00
                    }
                }
            }
            PlayerVerticalAction::Down => {
                if player_y_screen >= 238 {
                    return Ok(None);
                }
                if signed_word_less_or_equal(current_velocity, 0) {
                    0x0100
                } else {
                    let candidate = current_velocity.wrapping_add(8);
                    if candidate <= 0x0200 {
                        candidate
                    } else {
                        0x0200
                    }
                }
            }
        };
        Ok(Some(next_velocity))
    }

    pub(super) fn write_field(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: &str,
        field: &str,
        entry_index: u16,
        bytes: &[u8],
    ) -> Result<(), String> {
        let range = ram_field(layout, table, field)?
            .field_range_for_entry(entry_index)
            .ok_or_else(|| format!("red-label {table}.{field} range is invalid"))?;
        self.write_range(range, bytes)
    }

    pub(super) fn read_field_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        table: &str,
        field: &str,
    ) -> Result<u8, String> {
        let range = field_range(layout, table, field)?;
        if range.end - range.start != 1 {
            return Err(format!("red-label {table}.{field} is not one byte"));
        }
        self.read_byte(range.start)
    }

    pub(super) fn read_player_field_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        entry_index: u16,
        field: &str,
    ) -> Result<u8, String> {
        let range = player_field_range_for_entry(layout, entry_index, field)?;
        if range.end - range.start != 1 {
            return Err(format!("red-label player.{field} is not one byte"));
        }
        self.read_byte(range.start)
    }

    pub(super) fn write_field_byte(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: &str,
        field: &str,
        value: u8,
    ) -> Result<(), String> {
        let range = field_range(layout, table, field)?;
        if range.end - range.start != 1 {
            return Err(format!("red-label {table}.{field} is not one byte"));
        }
        self.write_byte(range.start, value)
    }

    pub(super) fn read_field_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        table: &str,
        field: &str,
    ) -> Result<u16, String> {
        let range = field_range(layout, table, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label {table}.{field} is not two bytes"));
        }
        self.read_word(range.start)
    }

    pub(super) fn write_field_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        table: &str,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        let range = field_range(layout, table, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label {table}.{field} is not two bytes"));
        }
        self.write_word(range.start, value)
    }

    pub(super) fn current_player_entry_index(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let current_player = self.read_field_byte(layout, "base_page", "CURPLR")?;
        if current_player == 1 { Ok(0) } else { Ok(1) }
    }

    pub(super) fn current_player_pointer_and_smart_bomb_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(u8, u16), String> {
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let player_table = table_descriptor(layout, "player")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let player_number = if player_index == 0 { 1 } else { 2 };
        let smart_bomb_range = ram_field(layout, "player", "PSBC")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSBC range is invalid"))?;
        Ok((player_number, smart_bomb_range.start))
    }

    pub(super) fn dispatch_smart_bomb_active_collisions(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        lists: &[RedLabelLinkedList],
    ) -> Result<Vec<RedLabelObjectCollision>, String> {
        let object = table_descriptor(layout, "object")?;
        let active_head = linked_list(lists, "active_object")?.head_address;
        let mut collisions = Vec::new();

        'restart_after_collision: loop {
            let mut object_address = self.read_word(active_head)?;
            let mut scanned = 0;
            while object_address != 0 {
                if scanned >= object.entries {
                    return Err(String::from(
                        "red-label active object list did not terminate during SBOMB",
                    ));
                }
                scanned += 1;
                object_table_for_address(layout, object_address)?;

                if self.read_object_screen_address(layout, object_address)? != 0
                    && self.read_object_byte(layout, object_address, "OTYP")? < 2
                {
                    let collision = self.dispatch_object_collision_vector(object_address)?;
                    collisions.push(collision);
                    if collisions.len() > usize::from(object.entries) {
                        return Err(String::from(
                            "red-label SBOMB collision dispatch did not make list progress",
                        ));
                    }
                    continue 'restart_after_collision;
                }

                object_address = self.read_object_word(layout, object_address, "OLINK")?;
            }

            return Ok(collisions);
        }
    }

    pub(super) fn collide_picture_with_list(
        &mut self,
        list_name: &str,
        picture_address: u16,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        let layout = red_label_ram_layout()?;
        let lists = red_label_linked_lists()?;
        let object = table_descriptor(&layout, "object")?;
        let candidate_picture = red_label_object_picture(picture_address)?;
        let head = linked_list(&lists, list_name)?.head_address;
        let mut object_address = self.read_word(head)?;
        let mut scanned = 0;

        while object_address != 0 {
            if scanned >= object.entries {
                return Err(format!(
                    "red-label {list_name} list did not terminate during COLIDE"
                ));
            }
            scanned += 1;
            object_table_for_address(&layout, object_address)?;

            let object_upper_left = self.read_object_screen_address(&layout, object_address)?;
            if object_upper_left != 0 {
                let object_picture_address =
                    self.read_object_word(&layout, object_address, "OPICT")?;
                let object_picture = red_label_object_picture(object_picture_address)?;
                if let Some(center) = picture_collision_center(
                    candidate_picture,
                    upper_left,
                    object_picture,
                    object_upper_left,
                )? {
                    self.write_field_word(&layout, "base_page", "CENTMP", center)?;
                    return self
                        .dispatch_object_collision_vector(object_address)
                        .map(Some);
                }
            }

            object_address = self.read_object_word(&layout, object_address, "OLINK")?;
        }

        Ok(None)
    }

    pub(super) fn step_laser_current_process(
        &mut self,
        direction: RedLabelLaserDirection,
    ) -> Result<RedLabelLaserStep, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.current_process_address(&layout)?;
        if self.read_field_byte(&layout, "base_page", "STATUS")? & 0x40 != 0 {
            return self.finish_laser_step(
                &layout,
                process_address,
                direction,
                RedLabelLaserStopReason::StatusDisabled,
                None,
            );
        }

        let beam_start = self.read_process_data_word(&layout, process_address, "PD")?;
        if laser_reached_screen_edge(direction, beam_start) {
            return self.finish_laser_step(
                &layout,
                process_address,
                direction,
                RedLabelLaserStopReason::ScreenEdge,
                None,
            );
        }

        let tip_address = self.draw_laser_body(direction, beam_start)?;
        self.write_process_data_word(&layout, process_address, "PD", tip_address)?;
        let fizzle_target = self.read_process_data_word(&layout, process_address, "PD2")?;
        let (fizzle_source_next, fizzle_target_next) =
            self.draw_laser_fizzle(&layout, direction, fizzle_target)?;
        self.write_process_data_word(&layout, process_address, "PD2", fizzle_target_next)?;
        let old_erase = self.read_process_data_word(&layout, process_address, "PD4")?;
        self.write_byte(old_erase, 0)?;
        let erase_next = step_laser_address(direction, old_erase);
        self.write_process_data_word(&layout, process_address, "PD4", erase_next)?;

        let collision_upper_left = match direction {
            RedLabelLaserDirection::Right => {
                let [x, y] = tip_address.to_be_bytes();
                u16::from_be_bytes([x.wrapping_sub(6), y])
            }
            RedLabelLaserDirection::Left => tip_address,
        };
        let collision = self.collide_laser_with_active_objects(collision_upper_left)?;
        if let Some(collision) = collision {
            return self.finish_laser_step(
                &layout,
                process_address,
                direction,
                RedLabelLaserStopReason::Collision,
                Some(collision),
            );
        }

        let wakeup_address = match direction {
            RedLabelLaserDirection::Right => red_label_routine_address("LASR0")?,
            RedLabelLaserDirection::Left => red_label_routine_address("LASL0")?,
        };
        self.sleep_current_process(1, wakeup_address)?;
        Ok(RedLabelLaserStep::Sleeping {
            process_address,
            direction,
            wakeup_address,
            tip_address,
            fizzle_source_next,
            fizzle_target_next,
            erase_next,
        })
    }

    pub(super) fn draw_laser_body(
        &mut self,
        direction: RedLabelLaserDirection,
        start: u16,
    ) -> Result<u16, String> {
        let mut address = start;
        for _ in 0..4 {
            self.write_byte(address, 0x11)?;
            address = step_laser_address(direction, address);
        }
        self.write_byte(address, 0x99)?;
        Ok(address)
    }

    pub(super) fn draw_laser_fizzle(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        direction: RedLabelLaserDirection,
        target_start: u16,
    ) -> Result<(u16, u16), String> {
        let fizzle_range = field_range(layout, "laser_fizzle", "FISTAB")?;
        let wrap_threshold = fizzle_range
            .end
            .checked_sub(3)
            .ok_or_else(|| String::from("red-label FISTAB range is too short"))?;
        let mut source = self.read_field_word(layout, "base_page", "FISX")?;
        if source >= wrap_threshold {
            source = fizzle_range.start;
        }

        let mut target = target_start;
        for _ in 0..3 {
            let byte = self.read_byte(source)?;
            self.write_byte(target, byte)?;
            source = source.wrapping_add(1);
            target = step_laser_address(direction, target);
        }
        self.write_field_word(layout, "base_page", "FISX", source)?;
        Ok((source, target))
    }

    pub(super) fn finish_laser_step(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        direction: RedLabelLaserDirection,
        reason: RedLabelLaserStopReason,
        collision: Option<RedLabelObjectCollision>,
    ) -> Result<RedLabelLaserStep, String> {
        self.erase_laser_trail(layout, process_address, direction)?;
        let killed_process = self.finish_laser_fire_current_process()?;
        Ok(RedLabelLaserStep::Finished {
            process_address,
            direction,
            reason,
            collision,
            killed_process,
        })
    }

    pub(super) fn erase_laser_trail(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        direction: RedLabelLaserDirection,
    ) -> Result<(), String> {
        let tip = self.read_process_data_word(layout, process_address, "PD")?;
        let mut address = self.read_process_data_word(layout, process_address, "PD4")?;
        for _ in 0..=u8::MAX {
            self.write_byte(address, 0)?;
            address = step_laser_address(direction, address);
            let keep_erasing = match direction {
                RedLabelLaserDirection::Right => address <= tip,
                RedLabelLaserDirection::Left => address >= tip,
            };
            if !keep_erasing {
                return Ok(());
            }
        }

        Err(format!(
            "red-label {:?} laser erase did not reach PD 0x{tip:04X}",
            direction
        ))
    }

    pub(super) fn dispatch_smart_bomb_process_entry(
        &mut self,
    ) -> Result<RedLabelProcessDispatch, String> {
        if let Some(smart_bomb) = self.start_smart_bomb_current_player()? {
            return Ok(RedLabelProcessDispatch::SmartBombStarted(Some(smart_bomb)));
        }

        let layout = red_label_ram_layout()?;
        self.suicide_current_process(&layout)
            .map(RedLabelProcessDispatch::SmartBombTail)
    }

    pub(super) fn suicide_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelSmartBombTail, String> {
        let killed_process = self.kill_current_process(layout)?;
        Ok(RedLabelSmartBombTail::Completed {
            killed_process_address: killed_process.killed_process_address,
            previous_link_address: killed_process.previous_link_address,
        })
    }

    pub(super) fn kill_current_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<RedLabelKilledProcess, String> {
        let killed_process_address = self.current_process_address(layout)?;
        let previous_link_address = self.kill_process(killed_process_address)?;
        let crproc = ram_field(layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        self.write_word(crproc, previous_link_address)?;
        Ok(RedLabelKilledProcess {
            killed_process_address,
            previous_link_address,
        })
    }

    pub(super) fn write_current_process_data_byte(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        field: &str,
        value: u8,
    ) -> Result<(), String> {
        let current_process = self.current_process_address(layout)?;
        self.write_process_byte(layout, current_process, field, value)
    }

    pub(super) fn start_smart_bomb_flash_sleep(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let pcram = field_range(layout, "base_page", "PCRAM")?.start;
        let current_pseudo_color = self.read_byte(pcram)?;
        self.write_byte(pcram, !current_pseudo_color)?;
        let wakeup_address = red_label_routine_address("SBMBX1")?;
        self.sleep_current_process(2, wakeup_address)?;
        Ok(wakeup_address)
    }

    pub(super) fn save_current_player_state_for_death(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        let player_table = table_descriptor(layout, "player")?;
        let player_address = self.read_field_word(layout, "base_page", "PLRX")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let target_range = player_field_range_for_entry(layout, player_index, "PTARG")?;
        let enemy_range = player_field_range_for_entry(layout, player_index, "PENEMY")?;
        self.clear_range(target_range.start..enemy_range.end)?;

        let astronaut_count = self.read_field_byte(layout, "base_page", "ASTCNT")?;
        self.write_byte(target_range.start, astronaut_count)?;

        let saved_enemy_range = field_range(layout, "enemy_runtime", "ELIST")?;
        let active_enemy_range = field_range(layout, "enemy_runtime", "ECNTS")?;
        if enemy_range.end - enemy_range.start < saved_enemy_range.end - saved_enemy_range.start {
            return Err(String::from(
                "red-label player.PENEMY range is shorter than ELIST",
            ));
        }

        for offset in 0..saved_enemy_range.end - saved_enemy_range.start {
            let mut value = self.read_byte(saved_enemy_range.start + offset)?;
            if offset < 5 {
                value = value.wrapping_add(self.read_byte(active_enemy_range.start + offset)?);
            }
            self.write_byte(enemy_range.start + offset, value)?;
        }

        Ok(())
    }

    pub(super) fn write_monochrome_player_picture(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        picture_address: u16,
    ) -> Result<u16, String> {
        let mono_range = field_range(layout, "mono_picture", "MONOTB")?;
        let picture = red_label_object_picture(picture_address)?;
        let image_size = u16::from(picture.width) * u16::from(picture.height);
        let first_image_address = mono_range.start.wrapping_add(10);
        let second_image_address = first_image_address.wrapping_add(image_size);
        if second_image_address.wrapping_add(image_size) > mono_range.end {
            return Err(format!(
                "red-label MONOTB is too small for `{}` monochrome data",
                picture.label
            ));
        }

        self.write_byte(mono_range.start, picture.width)?;
        self.write_byte(mono_range.start + 1, picture.height)?;
        self.write_word(mono_range.start + 2, first_image_address)?;
        self.write_word(mono_range.start + 4, second_image_address)?;
        self.write_word(
            mono_range.start + 6,
            picture.output_routine.ok_or_else(|| {
                format!(
                    "red-label picture `{}` has no output routine for MONO",
                    picture.label
                )
            })?,
        )?;
        self.write_word(
            mono_range.start + 8,
            picture.erase_routine.ok_or_else(|| {
                format!(
                    "red-label picture `{}` has no erase routine for MONO",
                    picture.label
                )
            })?,
        )?;

        self.write_monochrome_image(picture.primary_image, first_image_address, image_size)?;
        self.write_monochrome_image(
            picture.alternate_image.unwrap_or(picture.primary_image),
            second_image_address,
            image_size,
        )?;
        self.write_field_byte(layout, "base_page", "MAPCR", 2)?;
        Ok(mono_range.start)
    }

    pub(super) fn write_monochrome_image(
        &mut self,
        source_address: u16,
        target_address: u16,
        image_size: u16,
    ) -> Result<(), String> {
        for offset in 0..image_size {
            let source_byte = red_label_object_image_byte(source_address.wrapping_add(offset))?
                .ok_or_else(|| {
                    format!(
                        "red-label object image asset has no byte at 0x{:04X} for MONO",
                        source_address.wrapping_add(offset)
                    )
                })?;
            self.write_byte(
                target_address.wrapping_add(offset),
                monochrome_player_byte(source_byte),
            )?;
        }
        Ok(())
    }

    pub(super) fn write_ram_picture_descriptor(
        &mut self,
        screen_address: u16,
        descriptor_address: u16,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let previous_map = self.read_field_byte(&layout, "base_page", "MAPCR")?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let result = (|| {
            let width = self.read_byte(descriptor_address)?;
            let height = self.read_byte(descriptor_address + 1)?;
            let image_address = self.read_word(descriptor_address + 2)?;
            for column in 0..width {
                let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
                let source_column = image_address + u16::from(column) * u16::from(height);
                for row in 0..height {
                    let source_byte = self.read_byte(source_column + u16::from(row))?;
                    self.write_byte(screen_offset(column_address, u16::from(row))?, source_byte)?;
                }
            }
            Ok(())
        })();
        self.write_field_byte(&layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    pub(super) fn clear_ram_picture_descriptor_footprint(
        &mut self,
        screen_address: u16,
        descriptor_address: u16,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let previous_map = self.read_field_byte(&layout, "base_page", "MAPCR")?;
        self.write_field_byte(&layout, "base_page", "MAPCR", 2)?;
        let result = self.clear_screen_block(
            screen_address,
            self.read_byte(descriptor_address)?,
            self.read_byte(descriptor_address + 1)?,
        );
        self.write_field_byte(&layout, "base_page", "MAPCR", previous_map)?;
        result
    }

    pub(super) fn add_bcd_word_to_score(
        &mut self,
        score_start: u16,
        score_offset: u8,
        addend: u16,
    ) -> Result<(), String> {
        let [addend_high, addend_low] = addend.to_be_bytes();
        let mut offset = score_offset;
        let (new_value, mut carry) = bcd_add_byte(
            self.read_byte(score_start + u16::from(offset))?,
            addend_low,
            false,
        );
        self.write_byte(score_start + u16::from(offset), new_value)?;

        if offset == 0 {
            return Ok(());
        }
        offset -= 1;
        let mut addend_byte = addend_high;
        loop {
            let (new_value, next_carry) = bcd_add_byte(
                self.read_byte(score_start + u16::from(offset))?,
                addend_byte,
                carry,
            );
            self.write_byte(score_start + u16::from(offset), new_value)?;
            carry = next_carry;
            addend_byte = 0;
            if offset == 0 {
                return Ok(());
            }
            offset -= 1;
        }
    }

    pub(super) fn apply_score_replay_award(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        player_index: u16,
        score_range: std::ops::Range<u16>,
    ) -> Result<bool, String> {
        let replay_increment = self.read_field_word(layout, "base_page", "REPLA")?;
        if replay_increment == 0 {
            return Ok(false);
        }

        let replay_range = ram_field(layout, "player", "PRPLA")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PRPLA range is invalid"))?;
        let score_digits = self.read_fixed_bytes::<3>(score_range.start + 1)?;
        let replay_digits = self.read_fixed_bytes::<3>(replay_range.start)?;
        if score_digits < replay_digits {
            return Ok(false);
        }

        let [replay_high, replay_low] = replay_increment.to_be_bytes();
        let (new_low, carry) =
            bcd_add_byte(self.read_byte(replay_range.start + 1)?, replay_low, false);
        self.write_byte(replay_range.start + 1, new_low)?;
        let (new_high, _) = bcd_add_byte(self.read_byte(replay_range.start)?, replay_high, carry);
        self.write_byte(replay_range.start, new_high)?;

        let laser_range = ram_field(layout, "player", "PLAS")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PLAS range is invalid"))?;
        let smart_bomb_range = ram_field(layout, "player", "PSBC")?
            .field_range_for_entry(player_index)
            .ok_or_else(|| String::from("red-label player.PSBC range is invalid"))?;
        self.write_byte(
            laser_range.start,
            self.read_byte(laser_range.start)?.wrapping_add(1),
        )?;
        self.write_byte(
            smart_bomb_range.start,
            self.read_byte(smart_bomb_range.start)?.wrapping_add(1),
        )?;
        Ok(true)
    }

    pub(super) fn load_sound_table_by_label(
        &mut self,
        label: &str,
    ) -> Result<Option<RedLabelLoadedSoundTable>, String> {
        self.load_sound_table(red_label_sound_table_address(label)?)
    }

    pub(super) fn load_sound_table(
        &mut self,
        sound_table_address: u16,
    ) -> Result<Option<RedLabelLoadedSoundTable>, String> {
        let layout = red_label_ram_layout()?;
        let table = red_label_sound_table(sound_table_address)?;
        let priority = *table
            .bytes
            .first()
            .ok_or_else(|| format!("red-label sound table `{}` is empty", table.label))?;
        self.write_field_byte(&layout, "base_page", "THFLG", 0)?;
        if !red_label_sound_priority_allows_load(
            priority,
            self.read_field_byte(&layout, "base_page", "SNDPRI")?,
        ) {
            return Ok(None);
        }
        self.write_field_byte(&layout, "base_page", "SNDPRI", priority)?;
        self.write_field_word(
            &layout,
            "base_page",
            "SNDX",
            sound_table_address.wrapping_sub(2),
        )?;
        self.write_field_byte(&layout, "base_page", "SNDTMR", 1)?;
        self.write_field_byte(&layout, "base_page", "SNDREP", 1)?;
        Ok(Some(RedLabelLoadedSoundTable {
            address: sound_table_address,
            priority,
        }))
    }

    /// Source-shaped `SNDSEQ`: advance the currently loaded sound table,
    /// output at most one raw sound-board command, and maintain the thrust
    /// sound gate from `PIA21` / `THFLG`.
    /// Source: <https://github.com/mwenge/defender/blob/master/src/defa7.src#L725-L755>.
    pub(super) fn step_sound_sequence(&mut self) -> Result<RedLabelSoundSequenceStep, String> {
        let layout = red_label_ram_layout()?;
        let timer_before = self.read_field_byte(&layout, "base_page", "SNDTMR")?;
        let repeat_before = self.read_field_byte(&layout, "base_page", "SNDREP")?;
        let table_pointer_before = self.read_field_word(&layout, "base_page", "SNDX")?;
        let thrust_flag_before = self.read_field_byte(&layout, "base_page", "THFLG")?;

        let mut source = RedLabelSoundSequenceSource::Idle;
        let mut timer_after = timer_before;
        let mut sound_number = None;
        let mut sound_output = None;
        let mut command = None;

        if timer_before != 0 {
            timer_after = timer_before.wrapping_sub(1);
            self.write_field_byte(&layout, "base_page", "SNDTMR", timer_after)?;
            if timer_after != 0 {
                source = RedLabelSoundSequenceSource::Timer;
            } else {
                let mut table_pointer = table_pointer_before;
                let mut repeat = repeat_before.wrapping_sub(1);
                self.write_field_byte(&layout, "base_page", "SNDREP", repeat)?;

                if repeat == 0 {
                    table_pointer = table_pointer_before.wrapping_add(3);
                    self.write_field_word(&layout, "base_page", "SNDX", table_pointer)?;
                    repeat = red_label_sound_table_byte_required(table_pointer)?;
                    if repeat == 0 {
                        self.write_field_byte(&layout, "base_page", "SNDPRI", 0)?;
                        source = RedLabelSoundSequenceSource::SequenceEnded;
                    } else {
                        self.write_field_byte(&layout, "base_page", "SNDREP", repeat)?;
                    }
                }

                if repeat != 0 {
                    timer_after =
                        red_label_sound_table_byte_required(table_pointer.wrapping_add(1))?;
                    let table_sound =
                        red_label_sound_table_byte_required(table_pointer.wrapping_add(2))?;
                    self.write_field_byte(&layout, "base_page", "SNDTMR", timer_after)?;
                    sound_number = Some(table_sound);
                    let output = red_label_sound_output(table_sound);
                    sound_output = Some(output);
                    command = Some(output.command);
                    source = RedLabelSoundSequenceSource::Table;
                }
            }
        }

        if command.is_none()
            && timer_after == 0
            && let Some((thrust_source, thrust_output)) = self.step_thrust_sound_gate(&layout)?
        {
            source = thrust_source;
            sound_number = Some(thrust_output.sound_number);
            sound_output = Some(thrust_output);
            command = Some(thrust_output.command);
        }

        Ok(RedLabelSoundSequenceStep {
            source,
            timer_before,
            timer_after: self.read_field_byte(&layout, "base_page", "SNDTMR")?,
            repeat_before,
            repeat_after: self.read_field_byte(&layout, "base_page", "SNDREP")?,
            table_pointer_before,
            table_pointer_after: self.read_field_word(&layout, "base_page", "SNDX")?,
            priority_after: self.read_field_byte(&layout, "base_page", "SNDPRI")?,
            thrust_flag_before,
            thrust_flag_after: self.read_field_byte(&layout, "base_page", "THFLG")?,
            sound_number,
            sound_output,
            command,
        })
    }

    pub(super) fn step_thrust_sound_gate(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<Option<(RedLabelSoundSequenceSource, RedLabelSoundOutput)>, String> {
        let thrust_active =
            self.read_field_byte(layout, "base_page", "PIA21")? & RED_LABEL_THRUST_SWITCH_BIT != 0;
        let thrust_flag = self.read_field_byte(layout, "base_page", "THFLG")?;

        if !thrust_active {
            if thrust_flag == 0 {
                return Ok(None);
            }
            self.write_field_byte(layout, "base_page", "THFLG", 0)?;
            let sound_number = RED_LABEL_THRUST_SOUND_STOP_NUMBER;
            return Ok(Some((
                RedLabelSoundSequenceSource::ThrustStopped,
                red_label_sound_output(sound_number),
            )));
        }

        if thrust_flag != 0 {
            return Ok(None);
        }

        if self.read_field_byte(layout, "base_page", "STATUS")?
            & RED_LABEL_SOUND_PLAYER_ALIVE_BLOCK_MASK
            != 0
        {
            return Ok(None);
        }

        let sound_number = RED_LABEL_THRUST_SOUND_START_NUMBER;
        self.write_field_byte(layout, "base_page", "THFLG", sound_number)?;
        Ok(Some((
            RedLabelSoundSequenceSource::ThrustStarted,
            red_label_sound_output(sound_number),
        )))
    }

    pub(super) fn current_process_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<u16, String> {
        let crproc = ram_field(layout, "runtime_pointers", "CRPROC")?
            .field_range_for_entry(0)
            .ok_or_else(|| String::from("red-label CRPROC range is invalid"))?
            .start;
        let current_process = self.read_word(crproc)?;
        if current_process == 0 {
            return Err(String::from("red-label CRPROC does not name a process"));
        }
        process_table_for_address(layout, current_process)?;
        Ok(current_process)
    }

    pub(super) fn read_process_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        field: &str,
    ) -> Result<u8, String> {
        let table = process_table_for_address(layout, process_address)?;
        let range = process_field_range_for_address(layout, table, process_address, field)?;
        if range.end - range.start != 1 {
            return Err(format!("red-label {}.{field} is not one byte", table.table));
        }
        self.read_byte(range.start)
    }

    pub(super) fn write_process_byte(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        field: &str,
        value: u8,
    ) -> Result<(), String> {
        let table = process_table_for_address(layout, process_address)?;
        let range = process_field_range_for_address(layout, table, process_address, field)?;
        if range.end - range.start != 1 {
            return Err(format!("red-label {}.{field} is not one byte", table.table));
        }
        self.write_byte(range.start, value)
    }

    pub(super) fn read_process_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        field: &str,
    ) -> Result<u16, String> {
        let table = process_table_for_address(layout, process_address)?;
        let range = process_field_range_for_address(layout, table, process_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!(
                "red-label {}.{field} is not two bytes",
                table.table
            ));
        }
        self.read_word(range.start)
    }

    pub(super) fn write_process_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        let table = process_table_for_address(layout, process_address)?;
        let range = process_field_range_for_address(layout, table, process_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!(
                "red-label {}.{field} is not two bytes",
                table.table
            ));
        }
        self.write_word(range.start, value)
    }

    pub(super) fn write_process_data_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        first_byte_field: &str,
        value: u16,
    ) -> Result<(), String> {
        let table = process_table_for_address(layout, process_address)?;
        let range =
            process_field_range_for_address(layout, table, process_address, first_byte_field)?;
        self.write_word(range.start, value)
    }

    pub(super) fn read_process_data_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        first_byte_field: &str,
    ) -> Result<u16, String> {
        let table = process_table_for_address(layout, process_address)?;
        let range =
            process_field_range_for_address(layout, table, process_address, first_byte_field)?;
        self.read_word(range.start)
    }

    pub(super) fn read_object_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
    ) -> Result<u16, String> {
        let range = object_field_range_for_address(layout, object_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label object.{field} is not two bytes"));
        }
        self.read_word(range.start)
    }

    pub(super) fn read_object_byte(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
    ) -> Result<u8, String> {
        let range = object_field_range_for_address(layout, object_address, field)?;
        self.read_byte(range.start)
    }

    pub(super) fn read_object_screen_address(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<u16, String> {
        let range = object_field_range_for_address(layout, object_address, "OBJX")?;
        self.read_word(range.start)
    }

    pub(super) fn write_object_screen_address(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        value: u16,
    ) -> Result<(), String> {
        let range = object_field_range_for_address(layout, object_address, "OBJX")?;
        self.write_word(range.start, value)
    }

    pub(super) fn mark_shell_dead(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        shell_address: u16,
    ) -> Result<(), String> {
        let range = object_field_range_for_address(layout, shell_address, "ODATA")?;
        self.write_byte(range.start + 1, 0)
    }

    pub(super) fn clear_shell_video_footprint(
        &mut self,
        screen_address: u16,
    ) -> Result<(), String> {
        self.write_word(screen_address, 0)?;
        self.write_byte(screen_offset(screen_address, 2)?, 0)?;
        self.write_word(screen_offset(screen_address, 0x100)?, 0)?;
        self.write_byte(screen_offset(screen_address, 0x102)?, 0)
    }

    pub(super) fn write_object_picture_primary(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<RedLabelPictureWrite, String> {
        self.write_object_picture_cwrit(screen_address, picture_address)
    }

    pub(super) fn write_object_picture_image(
        &mut self,
        screen_address: u16,
        picture: &RedLabelObjectPicture,
        image_address: u16,
    ) -> Result<RedLabelPictureWrite, String> {
        for column in 0..picture.width {
            let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = image_address + u16::from(column) * u16::from(picture.height);
            for row in 0..picture.height {
                let source_byte =
                    red_label_object_image_byte_required(source_column + u16::from(row), picture)?;
                self.write_byte(screen_offset(column_address, u16::from(row))?, source_byte)?;
            }
        }
        Ok(RedLabelPictureWrite {
            screen_address,
            picture_address: picture.address,
            width: picture.width,
            height: picture.height,
        })
    }

    pub(super) fn write_score_digit_image(
        &mut self,
        screen_address: u16,
        image: &RedLabelScoreDigitImage,
    ) -> Result<(), String> {
        for column in 0..image.width {
            let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
            let source_column = usize::from(column) * usize::from(image.height);
            for row in 0..image.height {
                let source_byte = image.bytes[source_column + usize::from(row)];
                self.write_byte(screen_offset(column_address, u16::from(row))?, source_byte)?;
            }
        }
        Ok(())
    }

    pub(super) fn clear_score_digit_picture(&mut self, screen_address: u16) -> Result<(), String> {
        let image = red_label_score_digit_image(0)?;
        self.clear_screen_block(screen_address, image.width, image.height)
    }

    pub(super) fn clear_screen_block(
        &mut self,
        screen_address: u16,
        width: u8,
        height: u8,
    ) -> Result<(), String> {
        for column in 0..width {
            let column_address = screen_offset(screen_address, u16::from(column) << 8)?;
            let mut row = 0u8;
            while row.saturating_add(1) < height {
                self.write_word(screen_offset(column_address, u16::from(row))?, 0)?;
                row = row.wrapping_add(2);
            }
            if row < height {
                self.write_byte(screen_offset(column_address, u16::from(row))?, 0)?;
            }
        }
        Ok(())
    }

    pub(super) fn bomb_image_pointer_for_shell(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        shell_address: u16,
    ) -> Result<u16, String> {
        let base_pointer = self.read_field_word(layout, "base_page", "BAX")?;
        let flavor = self
            .read_object_word(layout, shell_address, "OX16")?
            .to_be_bytes()[1];
        if flavor & 0x80 == 0 {
            Ok(base_pointer)
        } else {
            screen_offset(base_pointer, 6)
        }
    }

    pub(super) fn inactive_object_next_y(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
    ) -> Result<u16, String> {
        let computed_y = self
            .read_object_word(layout, object_address, "OYV")?
            .wrapping_shl(3)
            .wrapping_add(self.read_object_word(layout, object_address, "OY16")?);
        let [mut y, fraction] = computed_y.to_be_bytes();
        if y < RED_LABEL_Y_MIN {
            y = RED_LABEL_Y_MAX;
        } else if y > RED_LABEL_Y_MAX {
            y = RED_LABEL_Y_MIN;
        }
        Ok(u16::from_be_bytes([y, fraction]))
    }

    pub(super) fn write_object_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        let range = object_field_range_for_address(layout, object_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label object.{field} is not two bytes"));
        }
        self.write_word(range.start, value)
    }

    pub(super) fn write_object_bytes(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        object_address: u16,
        field: &str,
        bytes: &[u8],
    ) -> Result<(), String> {
        let range = object_field_range_for_address(layout, object_address, field)?;
        self.write_range(range, bytes)
    }

    pub(super) fn read_appearance_word(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        field: &str,
    ) -> Result<u16, String> {
        let range = appearance_field_range_for_address(layout, slot_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label appearance_ram.{field} is not two bytes"));
        }
        self.read_word(range.start)
    }

    pub(super) fn write_appearance_word(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        slot_address: u16,
        field: &str,
        value: u16,
    ) -> Result<(), String> {
        let range = appearance_field_range_for_address(layout, slot_address, field)?;
        if range.end - range.start != 2 {
            return Err(format!("red-label appearance_ram.{field} is not two bytes"));
        }
        self.write_word(range.start, value)
    }

    pub(super) fn clear_range(&mut self, range: std::ops::Range<u16>) -> Result<(), String> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.ram.len() {
            return Err(format!(
                "red-label RAM clear range 0x{:04X}..0x{:04X} is outside main RAM",
                range.start, range.end
            ));
        }
        self.ram[start..end].fill(0);
        Ok(())
    }

    pub(super) fn clear_screen_ram(&mut self) -> Result<RedLabelScreenClear, String> {
        self.clear_range(0..RED_LABEL_SCREEN_CLEAR_END)?;
        Ok(RedLabelScreenClear {
            start: 0,
            end: RED_LABEL_SCREEN_CLEAR_END,
            bytes_cleared: RED_LABEL_SCREEN_CLEAR_END,
        })
    }

    pub(super) fn clear_screen_ram_from_high_address_down(
        &mut self,
        bytes_cleared: u16,
    ) -> Result<(), String> {
        if bytes_cleared > RED_LABEL_SCREEN_CLEAR_END {
            return Err(format!(
                "red-label reverse screen clear byte count 0x{bytes_cleared:04X} exceeds screen RAM"
            ));
        }
        let start = RED_LABEL_SCREEN_CLEAR_END - bytes_cleared;
        self.clear_range(start..RED_LABEL_SCREEN_CLEAR_END)
    }

    pub(super) fn write_trace_text_bytes(
        &mut self,
        screen_address: u16,
        bytes: &[u8],
    ) -> Result<u16, String> {
        let mut cursor = screen_address;
        for byte in bytes {
            cursor = self.write_text_byte(cursor, *byte)?;
        }
        Ok(cursor)
    }

    pub(super) fn write_trace_initial_tests_ok_screen(
        &mut self,
        unit_ok_text: &[u8],
    ) -> Result<(), String> {
        self.clear_screen_ram()?;
        self.write_trace_text_bytes(
            RED_LABEL_TRACE_INITIAL_TESTS_LINE_ADDRESS,
            RED_LABEL_TRACE_INITIAL_TESTS_LINE_TEXT,
        )?;
        self.write_trace_text_bytes(RED_LABEL_TRACE_INITIAL_TESTS_UNIT_OK_ADDRESS, unit_ok_text)?;
        Ok(())
    }

    pub(super) fn clear_active_screen_ram(&mut self) -> Result<RedLabelScreenClear, String> {
        for page in 0..(RED_LABEL_SCREEN_CLEAR_END >> 8) {
            let page_start = page << 8;
            self.clear_range(page_start + RED_LABEL_ACTIVE_SCREEN_CLEAR_START..page_start + 0x100)?;
        }

        Ok(RedLabelScreenClear {
            start: RED_LABEL_ACTIVE_SCREEN_CLEAR_START,
            end: RED_LABEL_SCREEN_CLEAR_END,
            bytes_cleared: RED_LABEL_ACTIVE_SCREEN_CLEAR_BYTES,
        })
    }

    pub(super) fn write_range(
        &mut self,
        range: std::ops::Range<u16>,
        bytes: &[u8],
    ) -> Result<(), String> {
        let start = usize::from(range.start);
        let end = usize::from(range.end);
        if start > end || end > self.ram.len() || end - start != bytes.len() {
            return Err(format!(
                "red-label RAM write range 0x{:04X}..0x{:04X} does not match {} byte(s)",
                range.start,
                range.end,
                bytes.len()
            ));
        }
        self.ram[start..end].copy_from_slice(bytes);
        Ok(())
    }

    pub(super) fn read_byte(&self, address: u16) -> Result<u8, String> {
        self.ram
            .get(usize::from(address))
            .copied()
            .ok_or_else(|| format!("red-label RAM read 0x{address:04X} is outside main RAM"))
    }

    pub(super) fn read_fixed_bytes<const N: usize>(&self, address: u16) -> Result<[u8; N], String> {
        let mut bytes = [0; N];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = self.read_byte(address + offset as u16)?;
        }
        Ok(bytes)
    }

    pub(super) fn read_byte_at_offset(&self, address: u16, offset: u16) -> Result<u8, String> {
        let target = screen_offset(address, offset)?;
        if usize::from(target) < MAIN_CPU_RAM_SIZE {
            return self.read_byte(target);
        }

        if let Some(byte) = red_label_shell_image_byte(target)? {
            return Ok(byte);
        }

        Err(format!(
            "red-label shell image read 0x{target:04X} is outside RAM and embedded shell images"
        ))
    }

    pub(super) fn write_byte(&mut self, address: u16, value: u8) -> Result<(), String> {
        let cell = self
            .ram
            .get_mut(usize::from(address))
            .ok_or_else(|| format!("red-label RAM write 0x{address:04X} is outside main RAM"))?;
        *cell = value;
        Ok(())
    }

    pub(super) fn write_visible_pixel_nibble(
        &mut self,
        visible_index: u32,
        nibble: u8,
    ) -> Result<(), String> {
        let visible_width = u32::from(VISIBLE_WIDTH);
        let visible_height = u32::from(VISIBLE_HEIGHT);
        if visible_index >= visible_width * visible_height {
            return Err(format!(
                "red-label visible pixel write 0x{visible_index:04X} is outside native frame"
            ));
        }

        let visible_x = visible_index % visible_width;
        let visible_y = visible_index / visible_width;
        let screen_x = DEFENDER_VISIBLE_X_START + visible_x as u16;
        let screen_y = DEFENDER_VISIBLE_Y_START + visible_y as u16;
        let byte_offset = williams_screen_byte_offset(screen_x, screen_y);
        let cell = self.ram.get_mut(byte_offset).ok_or_else(|| {
            format!(
                "red-label visible pixel write maps to RAM offset 0x{byte_offset:04X}, outside main RAM"
            )
        })?;
        let nibble = nibble & 0x0F;
        if screen_x & 1 == 0 {
            *cell = (*cell & 0x0F) | (nibble << 4);
        } else {
            *cell = (*cell & 0xF0) | nibble;
        }
        Ok(())
    }

    pub(super) fn read_word(&self, address: u16) -> Result<u16, String> {
        let high = self.read_byte(address)?;
        let low_address = address
            .checked_add(1)
            .ok_or_else(|| format!("red-label RAM word read 0x{address:04X} overflows"))?;
        let low = self.read_byte(low_address)?;
        Ok(u16::from_be_bytes([high, low]))
    }

    pub(super) fn write_word(&mut self, address: u16, value: u16) -> Result<(), String> {
        let [high, low] = value.to_be_bytes();
        self.write_byte(address, high)?;
        let low_address = address
            .checked_add(1)
            .ok_or_else(|| format!("red-label RAM word write 0x{address:04X} overflows"))?;
        self.write_byte(low_address, low)
    }
}

#[derive(Debug, Clone, Copy)]
struct ScorePopupMetadata {
    lifetime_ticks: u8,
    value: u16,
}

fn expanded_object_kind_for_detail(
    size: u16,
    picture_label: Option<&str>,
) -> ExpandedObjectKindState {
    if score_popup_metadata_for_picture_label(picture_label).is_some() {
        return ExpandedObjectKindState::ScorePopup;
    }
    if size & 0x8000 != 0 {
        ExpandedObjectKindState::Appearance
    } else {
        ExpandedObjectKindState::Explosion
    }
}

fn score_popup_metadata_for_picture_label(
    picture_label: Option<&str>,
) -> Option<ScorePopupMetadata> {
    match picture_label {
        Some("C25P1") => Some(ScorePopupMetadata {
            lifetime_ticks: 50,
            value: 250,
        }),
        Some("C5P1") => Some(ScorePopupMetadata {
            lifetime_ticks: 50,
            value: 500,
        }),
        _ => None,
    }
}

fn source_expanded_explosion_frame(kind: ExpandedObjectKindState, size: u16) -> Option<u8> {
    if kind != ExpandedObjectKindState::Explosion {
        return None;
    }
    source_explosion_frame_index(size)
}

fn source_player_explosion_frame(source_color_index: u8, source_color_counter: u8) -> u16 {
    if source_color_index == 0 {
        return u16::from(56u8.saturating_sub(source_color_counter));
    }

    let completed_initial_color_frames = 56u16;
    let completed_reload_color_frames =
        u16::from(source_color_index.saturating_sub(1)).saturating_mul(4);
    completed_initial_color_frames
        .saturating_add(completed_reload_color_frames)
        .saturating_add(u16::from(4u8.saturating_sub(source_color_counter)))
}

fn table_word_entries(table: &std::ops::Range<u16>) -> Result<u16, String> {
    if !(table.end - table.start).is_multiple_of(2) {
        return Err(format!(
            "red-label table range 0x{:04X}..0x{:04X} must contain word entries",
            table.start, table.end
        ));
    }
    Ok((table.end - table.start) / 2)
}
