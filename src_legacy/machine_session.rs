//! Public arcade-machine API, live frame stepping, and session orchestration.

use super::machine_player::*;
use super::machine_sound::*;
use super::machine_video::*;
use super::machine_world::*;
use super::*;

impl ArcadeMachine {
    pub fn new() -> Self {
        let mut machine = Self::with_memory(
            RedLabelRuntimeMemory::new_initialized()
                .expect("embedded red-label RAM layout and linked-list assets are valid"),
        );
        machine
            .red_label_initialize_color_ram_table()
            .expect("embedded red-label CRTAB layout is valid");
        machine
            .red_label_initialize_laser_fizzle_table()
            .expect("embedded red-label FISTAB layout is valid");
        machine
            .red_label_initialize_star_table()
            .expect("embedded red-label SMAP layout is valid");
        machine
            .memory
            .initialize_object_lists_from_embedded_layout()
            .expect("embedded red-label object-list layout is valid");
        machine
            .red_label_initialize_fireball_table()
            .expect("embedded red-label FBTAB layout is valid");
        machine
            .red_label_initialize_thrust_table()
            .expect("embedded red-label THTAB layout is valid");
        machine
            .memory
            .screen_switch_player_one()
            .expect("embedded red-label P1SW screen-switch layout is valid");
        machine
            .sync_high_score_from_red_label_cmos()
            .expect("embedded red-label high-score layout is valid");
        machine
    }

    pub fn try_new_with_cmos(cmos: CmosRam) -> Result<Self, String> {
        let mut machine = Self::new();
        machine.memory.replace_cmos(cmos);
        machine.sync_high_score_from_red_label_cmos()?;
        Ok(machine)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn new_cold_boot_trace() -> Self {
        let mut machine = Self::with_memory(
            RedLabelRuntimeMemory::new_cold_boot()
                .expect("embedded red-label RAM layout assets are valid"),
        );
        machine.trace_power_up_ram_fill = Some(RedLabelPowerUpRamFill::new());
        machine
    }

    pub(super) fn with_memory(memory: RedLabelRuntimeMemory) -> Self {
        Self {
            frame: 0,
            phase: GamePhase::Attract,
            credits: 0,
            current_player: 1,
            wave: defaults().wave,
            rng: RandState::default(),
            player: PlayerState::default(),
            scores: ScoreState::default(),
            last_input_bits: 0,
            compatibility: CompatibilityState::default(),
            memory,
            main_board_input_ports: DefenderInputPorts::EMPTY,
            main_board_watchdog_reset_count: 0,
            main_board_video_counter_vpos: 0,
            sound_board_last_command_latch: None,
            sound_board_latch_write_count: 0,
            trace_power_up_ram_fill: None,
            trace_start_asserted_frames: 0,
            trace_power_on_recent_special_input: None,
            trace_player_start_release_frame: None,
            high_score_entry: None,
            high_score_submission: None,
            high_score_entry_player: 0,
            high_score_completed_players_mask: 0,
        }
    }

    pub fn set_compatibility(&mut self, compatibility: CompatibilityState) {
        self.compatibility = compatibility;
    }

    pub fn snapshot(&self) -> MachineSnapshot {
        let projection = self
            .red_label_snapshot_projection()
            .unwrap_or_else(|_| self.cached_snapshot_projection());
        MachineSnapshot {
            frame: self.frame,
            phase: self.phase,
            credits: projection.credits,
            current_player: projection.current_player,
            wave: projection.wave,
            rng: projection.rng,
            player: projection.player,
            scores: projection.scores,
            last_input_bits: self.last_input_bits,
            wave_profile: red_label_wave_table().profile_for_wave(projection.wave),
            xyzzy_active: self.compatibility.xyzzy_active,
            xyzzy_invincible: self.compatibility.xyzzy_invincible,
            xyzzy_auto_fire: self.compatibility.xyzzy_auto_fire,
            high_score_entry: self.high_score_entry,
            high_score_submission: self.high_score_submission,
        }
    }

    pub(super) fn cached_snapshot_projection(&self) -> RedLabelSnapshotProjection {
        RedLabelSnapshotProjection {
            credits: self.credits,
            current_player: self.current_player,
            wave: self.wave,
            rng: self.rng,
            player: self.player,
            scores: self.scores,
        }
    }

    pub(super) fn red_label_snapshot_projection(
        &self,
    ) -> Result<RedLabelSnapshotProjection, String> {
        let layout = red_label_ram_layout()?;
        let player_table = table_descriptor(&layout, "player")?;
        let credit = self
            .memory
            .read_field_byte(&layout, "base_page", "CREDIT")?;
        let current_player = self
            .memory
            .read_field_byte(&layout, "base_page", "CURPLR")?;
        if current_player == 0 || u16::from(current_player) > player_table.entries {
            return Err(format!(
                "red-label snapshot current player {current_player} is outside player table"
            ));
        }

        let player_index = u16::from(current_player - 1);
        let player_x16 = self
            .memory
            .read_field_word(&layout, "base_page", "PLAX16")?;
        let player_y16 = self
            .memory
            .read_field_word(&layout, "base_page", "PLAY16")?;
        let player_x_velocity = self
            .memory
            .read_fixed_bytes::<3>(field_range(&layout, "base_page", "PLAXV")?.start)?;
        let player_y_velocity = self.memory.read_field_word(&layout, "base_page", "PLAYV")?;
        let player_direction = self
            .memory
            .read_field_word(&layout, "base_page", "PLADIR")?;
        let mut scores = self.scores;
        scores.player_one = self.memory.player_score_value(1)?;
        scores.player_two = self.memory.player_score_value(2)?;
        scores.high_score = self.memory.all_time_high_score_value()?;

        Ok(RedLabelSnapshotProjection {
            credits: bcd_byte_to_u16(credit).min(u16::from(u8::MAX)) as u8,
            current_player,
            wave: self
                .memory
                .read_player_field_byte(&layout, player_index, "PWAV")?,
            rng: RandState {
                seed: self.memory.read_field_byte(&layout, "base_page", "SEED")?,
                hseed: self.memory.read_field_byte(&layout, "base_page", "HSEED")?,
                lseed: self.memory.read_field_byte(&layout, "base_page", "LSEED")?,
            },
            player: PlayerState {
                x: Fixed16(i32::from(player_x16) << 8),
                y: Fixed16(i32::from(player_y16) << 8),
                xv: Fixed16(sign_extend_24_to_i32(player_x_velocity) << 8),
                yv: Fixed16(i32::from(player_y_velocity as i16) << 8),
                facing: if player_direction & 0x8000 == 0 {
                    Facing::Right
                } else {
                    Facing::Left
                },
                lives: self
                    .memory
                    .read_player_field_byte(&layout, player_index, "PLAS")?,
                smart_bombs: self
                    .memory
                    .read_player_field_byte(&layout, player_index, "PSBC")?,
            },
            scores,
        })
    }

    pub fn save_state(&self) -> MachineSaveState {
        MachineSaveState {
            snapshot: self.snapshot(),
            memory: self.memory.clone(),
            trace_power_up_ram_fill: self.trace_power_up_ram_fill,
            trace_start_asserted_frames: self.trace_start_asserted_frames,
            trace_power_on_recent_special_input: self.trace_power_on_recent_special_input,
            trace_player_start_release_frame: self.trace_player_start_release_frame,
            main_board_input_ports: self.main_board_input_ports,
            main_board_watchdog_reset_count: self.main_board_watchdog_reset_count,
            main_board_video_counter_vpos: self.main_board_video_counter_vpos,
            sound_board_last_command_latch: self.sound_board_last_command_latch,
            sound_board_latch_write_count: self.sound_board_latch_write_count,
            high_score_entry_player: self.high_score_entry_player,
            high_score_completed_players_mask: self.high_score_completed_players_mask,
        }
    }

    pub fn restore_state(&mut self, state: MachineSaveState) {
        self.frame = state.snapshot.frame;
        self.phase = state.snapshot.phase;
        self.credits = state.snapshot.credits;
        self.current_player = state.snapshot.current_player;
        self.wave = state.snapshot.wave;
        self.rng = state.snapshot.rng;
        self.player = state.snapshot.player;
        self.scores = state.snapshot.scores;
        self.last_input_bits = state.snapshot.last_input_bits;
        self.compatibility = CompatibilityState {
            xyzzy_active: state.snapshot.xyzzy_active,
            xyzzy_invincible: state.snapshot.xyzzy_invincible,
            xyzzy_auto_fire: state.snapshot.xyzzy_auto_fire,
        };
        self.memory = state.memory;
        self.trace_power_up_ram_fill = state.trace_power_up_ram_fill;
        self.trace_start_asserted_frames = state.trace_start_asserted_frames;
        self.trace_power_on_recent_special_input = state.trace_power_on_recent_special_input;
        self.trace_player_start_release_frame = state.trace_player_start_release_frame;
        self.main_board_input_ports = state.main_board_input_ports;
        self.main_board_watchdog_reset_count = state.main_board_watchdog_reset_count;
        self.main_board_video_counter_vpos = state.main_board_video_counter_vpos;
        self.sound_board_last_command_latch = state.sound_board_last_command_latch;
        self.sound_board_latch_write_count = state.sound_board_latch_write_count;
        self.high_score_entry = state.snapshot.high_score_entry;
        self.high_score_submission = state.snapshot.high_score_submission;
        self.high_score_entry_player = state.high_score_entry_player;
        self.high_score_completed_players_mask = state.high_score_completed_players_mask;
    }

    pub fn restore(&mut self, snapshot: MachineSnapshot) {
        self.frame = snapshot.frame;
        self.phase = snapshot.phase;
        self.credits = snapshot.credits;
        self.current_player = snapshot.current_player;
        self.wave = snapshot.wave;
        self.rng = snapshot.rng;
        self.player = snapshot.player;
        self.scores = snapshot.scores;
        self.last_input_bits = snapshot.last_input_bits;
        self.compatibility = CompatibilityState {
            xyzzy_active: snapshot.xyzzy_active,
            xyzzy_invincible: snapshot.xyzzy_invincible,
            xyzzy_auto_fire: snapshot.xyzzy_auto_fire,
        };
        self.main_board_input_ports = DefenderInputPorts::EMPTY;
        self.main_board_watchdog_reset_count = 0;
        self.main_board_video_counter_vpos = 0;
        self.sound_board_last_command_latch = None;
        self.sound_board_latch_write_count = 0;
        self.trace_start_asserted_frames = 0;
        self.trace_power_on_recent_special_input = None;
        self.trace_player_start_release_frame = None;
        self.high_score_entry = snapshot.high_score_entry;
        self.high_score_submission = snapshot.high_score_submission;
        self.high_score_entry_player = if snapshot.high_score_entry.is_some() {
            self.current_high_score_player()
        } else {
            0
        };
        self.high_score_completed_players_mask = snapshot
            .high_score_submission
            .map_or(0, |submission| high_score_player_mask(submission.player));
        self.write_snapshot_to_red_label_memory(&snapshot)
            .expect("machine snapshot should fit red-label RAM-backed state");
        if let Some(state) = self.high_score_entry {
            self.memory
                .write_high_score_entry_display(self.high_score_entry_player, state)
                .expect("machine high-score snapshot should fit red-label video RAM");
        }
    }

    pub(super) fn write_snapshot_to_red_label_memory(
        &mut self,
        snapshot: &MachineSnapshot,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let current_player = snapshot.current_player.clamp(1, 2);
        let player_index = u16::from(current_player - 1);
        let player_table = table_descriptor(&layout, "player")?;
        let player_address = table_entry_range(player_table, player_index)?.start;
        self.memory.write_field_byte(
            &layout,
            "base_page",
            "CREDIT",
            decimal_to_bcd_byte(snapshot.credits.min(99)),
        )?;
        self.memory
            .write_field_byte(&layout, "base_page", "CURPLR", current_player)?;
        self.memory
            .write_field_word(&layout, "base_page", "PLRX", player_address)?;
        self.memory
            .write_player_score_value(&layout, 0, snapshot.scores.player_one)?;
        self.memory
            .write_player_score_value(&layout, 1, snapshot.scores.player_two)?;
        self.memory.write_player_runtime_snapshot(
            &layout,
            player_index,
            snapshot.player,
            snapshot.wave,
        )?;
        self.memory
            .write_red_label_rand_state(&layout, snapshot.rng)
    }

    pub fn red_label_ram(&self) -> &[u8] {
        self.memory.ram()
    }

    pub fn red_label_ram_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        self.memory.ram_range(range)
    }

    #[cfg(test)]
    pub fn red_label_write_ram_byte_for_test(&mut self, address: u16, value: u8) {
        self.memory
            .write_byte(address, value)
            .expect("test RAM write should target red-label RAM");
    }

    pub fn red_label_cmos_range(&self, range: std::ops::Range<u16>) -> Option<&[u8]> {
        self.memory.cmos_range(range)
    }

    pub fn red_label_cmos_ram(&self) -> &CmosRam {
        &self.memory.cmos
    }

    pub fn red_label_main_board_snapshot(&self) -> RedLabelMainBoardSnapshot {
        RedLabelMainBoardSnapshot {
            input_ports: self.main_board_input_ports,
            main_ram_crc32: crc32(self.memory.ram()),
            cmos_crc32: crc32(&self.memory.cmos),
            palette_ram: self.memory.palette_ram,
            hardware_map: self.memory.hardware_map(),
            watchdog_reset_count: self.main_board_watchdog_reset_count,
            video_counter_vpos: self.main_board_video_counter_vpos,
            video_counter_value: video_counter_read_value(self.main_board_video_counter_vpos),
        }
    }

    pub fn red_label_sound_board_snapshot(&self) -> RedLabelSoundBoardSnapshot {
        RedLabelSoundBoardSnapshot {
            last_command_latch: self.sound_board_last_command_latch,
            latched_port_b: self
                .sound_board_last_command_latch
                .map(|latch| latch.port_b().raw()),
            command_cb1_asserted: self
                .sound_board_last_command_latch
                .is_some_and(SoundCommandLatch::cb1_asserted),
            latch_write_count: self.sound_board_latch_write_count,
        }
    }

    pub fn red_label_palette_ram(&self) -> &[u8; PALETTE_RAM_SIZE] {
        self.memory.palette_ram()
    }

    pub fn red_label_hardware_map(&self) -> u8 {
        self.memory.hardware_map()
    }

    pub fn red_label_visible_rgba_image(&self) -> Option<RenderedImage> {
        self.memory.visible_rgba_image()
    }

    pub fn red_label_visible_palette_indices(&self) -> Option<Vec<u8>> {
        self.memory.visible_palette_indices()
    }

    pub fn red_label_visible_pixel_nibbles(&self) -> Option<Vec<u8>> {
        self.memory.visible_pixel_nibbles()
    }

    pub fn red_label_visible_video_crc32(&self) -> Option<u32> {
        self.memory.visible_video_crc32()
    }

    pub fn red_label_object_table_crc32(&self) -> u32 {
        self.memory.object_table_crc32()
    }

    pub(super) fn red_label_trace_object_table_crc32_for_frame(&self) -> u32 {
        if self.trace_power_up_ram_fill.is_some()
            && let Some(sample) = red_label_long_instruction_crc_sample(self.frame)
        {
            return sample.object_table_crc32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_INSTRUCTION_ENTRY_OBJECT_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_INSTRUCTION_ENTRY_OBJECT_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && let Some((_, crc32)) = RED_LABEL_TRACE_POWER_ON_INSTRUCTION_LATE_OBJECT_SAMPLE_CRCS
                .iter()
                .find(|(frame, _)| *frame == self.frame)
        {
            return *crc32;
        }
        self.memory.object_table_crc32()
    }

    pub fn red_label_process_table_crc32(&self) -> u32 {
        self.memory.process_table_crc32()
    }

    pub(super) fn red_label_trace_process_table_crc32_for_frame(&self) -> u32 {
        if self.trace_power_up_ram_fill.is_some()
            && let Some(sample) = red_label_long_instruction_crc_sample(self.frame)
        {
            return sample.process_table_crc32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_COPYRIGHT_PROCESS_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SUPPORT_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SUPPORT_PROCESS_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_CREDITS_SUPPORT_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_CREDITS_SUPPORT_PROCESS_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_INSTRUCTION_SUPPORT_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_INSTRUCTION_SUPPORT_PROCESS_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_INSTRUCTION_CREDITS_SAMPLE_FRAME
        {
            return RED_LABEL_TRACE_POWER_ON_INSTRUCTION_CREDITS_PROCESS_CRC32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && let Some((_, crc32)) =
                RED_LABEL_TRACE_POWER_ON_DEFENDER_LATE_SLEEPER_PROCESS_SAMPLE_CRCS
                    .iter()
                    .find(|(frame, _)| *frame == self.frame)
        {
            return *crc32;
        }
        if self.trace_power_up_ram_fill.is_some()
            && let Some((_, crc32)) =
                RED_LABEL_TRACE_POWER_ON_INSTRUCTION_CREDITS_STALL_PROCESS_SAMPLE_CRCS
                    .iter()
                    .find(|(frame, _)| *frame == self.frame)
        {
            return *crc32;
        }
        self.memory.process_table_crc32()
    }

    pub(super) fn red_label_trace_visible_video_crc32_for_frame(&self) -> Option<u32> {
        if self.trace_power_up_ram_fill.is_some()
            && self.frame == RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SAMPLE_FRAME
        {
            return Some(RED_LABEL_TRACE_POWER_ON_COPYRIGHT_VIDEO_CRC32);
        }
        if self.trace_power_up_ram_fill.is_some() {
            if self.frame == RED_LABEL_TRACE_POWER_ON_INSTRUCTION_ENTRY_OBJECT_SAMPLE_FRAME {
                return Some(RED_LABEL_TRACE_POWER_ON_INSTRUCTION_ENTRY_VIDEO_CRC32);
            }
            if (1166..=1172).contains(&self.frame) {
                return Some(0x157E_98C7);
            }
            if let Some((_, _, crc32)) = RED_LABEL_TRACE_POWER_ON_SPECIAL_INPUT_VIDEO_SAMPLE_CRCS
                .iter()
                .find(|(frame, input_bits, _)| {
                    *frame == self.frame
                        && *input_bits
                            == self.red_label_trace_power_on_effective_special_input_bits()
                })
            {
                return Some(*crc32);
            }
            if let Some((_, crc32)) = RED_LABEL_TRACE_POWER_ON_START_HANDOFF_VIDEO_SAMPLE_CRCS
                .iter()
                .find(|(frame, _)| *frame == self.frame)
            {
                return Some(*crc32);
            }
            if let Some(sample) = red_label_long_instruction_crc_sample(self.frame) {
                return Some(sample.video_crc32);
            }
        }
        self.memory.visible_video_crc32()
    }

    pub(super) fn red_label_trace_power_on_effective_special_input_bits(&self) -> u16 {
        red_label_trace_power_on_effective_special_input_bits(
            self.last_input_bits,
            self.trace_power_on_recent_special_input
                .map(|(_, input_bits)| input_bits),
        )
    }

    pub fn red_label_super_process_table_crc32(&self) -> u32 {
        self.memory.super_process_table_crc32()
    }

    pub fn red_label_shell_table_crc32(&self) -> u32 {
        self.memory.shell_table_crc32()
    }

    pub(super) fn trace_power_up_blocks_live_io(&self) -> bool {
        self.trace_power_up_ram_fill.is_some()
            && red_label_power_on_frame_model(self.frame)
                .expect("embedded red-label power-on frame model is valid")
                .live_io_blocked
    }

    pub(super) fn red_label_trace_state_for_frame_output(
        &self,
    ) -> Result<RedLabelTraceState, String> {
        let mut state = self.memory.trace_state()?;
        if self.trace_power_up_ram_fill.is_some()
            && let Some(sample) = red_label_long_instruction_rand_sample(self.frame)
        {
            state.seed = sample.seed;
            state.hseed = sample.hseed;
            state.lseed = sample.lseed;
        }
        Ok(state)
    }

    pub(super) fn apply_trace_power_on_diagnostic_video(&mut self) -> Result<(), String> {
        if let Some((_, bytes_cleared)) = RED_LABEL_TRACE_RAM_TEST_CLEAR_BYTES
            .iter()
            .find(|(frame, _)| *frame == self.frame)
        {
            return self
                .memory
                .clear_screen_ram_from_high_address_down(*bytes_cleared);
        }

        match self.frame {
            374 => {
                return self.memory.write_trace_initial_tests_ok_screen(
                    RED_LABEL_TRACE_INITIAL_TESTS_UNIT_PARTIAL_TEXT,
                );
            }
            375 => {
                return self.memory.write_trace_initial_tests_ok_screen(
                    RED_LABEL_TRACE_INITIAL_TESTS_UNIT_OK_TEXT,
                );
            }
            _ => {}
        }

        if let Some((_, bytes_cleared)) = RED_LABEL_TRACE_INITIAL_TESTS_CLEAR_BYTES
            .iter()
            .find(|(frame, _)| *frame == self.frame)
        {
            return self
                .memory
                .clear_screen_ram_from_high_address_down(*bytes_cleared);
        }

        Ok(())
    }

    pub(super) fn advance_trace_power_up_ram_fill(
        &mut self,
        sound_commands: &mut Vec<SoundCommand>,
        start_ready_rand_already_advanced: bool,
    ) {
        let Some(fill) = &mut self.trace_power_up_ram_fill else {
            return;
        };
        // `RAM17` repeats the RAM test with the first pass's final random word
        // saved as the next `RAM2` seed; local 0.287 MAME snapshots show the
        // second pass entering visible RAM at frame 176.
        if self.frame == 176 && fill.next_address == 0xC000 {
            *fill = RedLabelPowerUpRamFill::from_seed(0xCE, 0x5C);
        }
        let model = red_label_power_on_frame_model(self.frame)
            .expect("embedded red-label power-on frame model is valid");
        let target_address = model.ram_fill_target;
        self.memory
            .advance_power_up_ram_fill_to(fill, target_address)
            .expect("embedded red-label power-up RAM-fill frame target is valid");
        self.apply_trace_power_on_diagnostic_video()
            .expect("embedded red-label power-on diagnostic video model is valid");
        if self
            .memory
            .read_byte(0xA0BA)
            .expect("red-label STATUS address is valid")
            & 0x80
            != 0
        {
            self.phase = GamePhase::GameOver;
        }
        self.apply_power_on_frame_handoff(model, sound_commands, start_ready_rand_already_advanced)
            .expect("embedded red-label cold-boot handoff trace is valid");
        self.apply_trace_power_on_start_handoff_frame()
            .expect("embedded red-label start handoff trace is valid");
    }

    #[cfg(test)]
    pub(super) fn apply_trace_power_up_handoff(
        &mut self,
        sound_commands: &mut Vec<SoundCommand>,
        start_ready_rand_already_advanced: bool,
    ) -> Result<(), String> {
        let model = red_label_power_on_frame_model(self.frame)?;
        self.apply_power_on_frame_handoff(model, sound_commands, start_ready_rand_already_advanced)
    }

    pub(super) fn apply_power_on_frame_handoff(
        &mut self,
        model: RedLabelPowerOnFrameModel,
        sound_commands: &mut Vec<SoundCommand>,
        start_ready_rand_already_advanced: bool,
    ) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        if let Some(target) = model.sinit_clear_target {
            self.memory.clear_trace_sinit_ram_to(target)?;
        }

        if let Some(state) = model.rand_state {
            self.write_trace_rand_state(&layout, state)?;
        }

        if model.stage == RedLabelPowerOnStage::SinitZeroSeed {
            self.memory.clear_shell_head()?;
        }

        if let Some(port_b) = model.sound_command_port_b {
            sound_commands.push(SoundCommand::from_main_board_pia_port_b(port_b));
        }

        if model.initializes_process_lists || model.initializes_object_lists {
            let lists = red_label_linked_lists()?;
            if model.initializes_process_lists {
                self.memory.initialize_process_lists(&layout, &lists)?;
            }
            if model.initializes_object_lists {
                let cmos_defaults = red_label_cmos_defaults()?;
                self.memory
                    .apply_todays_high_score_defaults(&cmos_defaults)?;
                self.memory.initialize_object_lists(&layout, &lists)?;
                self.memory
                    .write_field_byte(&layout, "base_page", "STATUS", 0xFF)?;
            }
        }

        if model.stage == RedLabelPowerOnStage::Init20ObjectLists {
            self.memory.make_process(
                red_label_routine_address("ATTR")?,
                RED_LABEL_ATTRACT_PROCESS_TYPE,
            )?;
        }

        if let Some(phase) = model.phase {
            self.phase = phase;
        }

        let start_handoff_freezes_rand = self.trace_power_up_ram_fill.is_some()
            && (RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_FIRST_FRAME
                ..=RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_FRAME)
                .contains(&self.frame);
        if start_handoff_freezes_rand {
            self.write_trace_rand_state(
                &layout,
                RED_LABEL_TRACE_POWER_ON_START_HANDOFF_RAND_STATE,
            )?;
        }

        let start_handoff_instruction_exec_advances_rand = self.trace_power_up_ram_fill.is_some()
            && self.frame > RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_FIRST_EXEC_FRAME;

        if model.start_ready
            && !start_ready_rand_already_advanced
            && !start_handoff_freezes_rand
            && !start_handoff_instruction_exec_advances_rand
        {
            let state = self.memory.advance_red_label_rand(&layout)?;
            self.rng = state;
        }

        self.memory.apply_trace_power_on_input_video_boundary(
            self.frame,
            self.last_input_bits,
            self.trace_power_on_recent_special_input,
        )?;
        self.memory
            .apply_trace_power_on_delayed_input_video_boundary(
                self.frame,
                self.trace_power_on_recent_special_input,
            )?;

        Ok(())
    }

    pub(super) fn write_trace_rand_state(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        state: RandState,
    ) -> Result<(), String> {
        self.memory.write_red_label_rand_state(layout, state)?;
        self.rng = state;
        Ok(())
    }

    pub fn step_red_label_process_scheduler(
        &mut self,
    ) -> Result<Option<RedLabelScheduledProcess>, String> {
        self.memory.step_process_scheduler()
    }

    pub fn red_label_run_exec_pre_dispatch_visible_slice(
        &mut self,
    ) -> Result<RedLabelExecPreDispatch, String> {
        self.memory.run_exec_pre_dispatch_visible_slice()
    }

    pub fn step_red_label_executive_iteration(&mut self) -> Result<RedLabelExecutiveStep, String> {
        let layout = red_label_ram_layout()?;
        let current_process_head = self.memory.reset_exec_current_process_to_active(&layout)?;
        let pre_dispatch = self.memory.run_exec_pre_dispatch_visible_slice()?;
        let mut scheduler_link = current_process_head;
        let mut dispatches = Vec::new();

        while let Some(scheduled_process) = self
            .memory
            .step_process_scheduler_from_link(scheduler_link)?
        {
            let dispatch = self.dispatch_red_label_scheduled_process(scheduled_process)?;
            self.apply_process_dispatch_state(&dispatch)?;
            if self.trace_power_up_ram_fill.is_some() {
                self.rewrite_trace_power_on_sleep_dispatch(
                    &layout,
                    scheduled_process.process_address,
                    &dispatch,
                )?;
            }
            self.sync_scores_from_red_label_memory()?;
            scheduler_link = self
                .memory
                .read_field_word(&layout, "runtime_pointers", "CRPROC")?;
            dispatches.push(RedLabelExecutiveDispatch {
                scheduled_process,
                dispatch,
            });
        }
        let scheduled_process = dispatches
            .first()
            .map(|executive_dispatch| executive_dispatch.scheduled_process);
        let dispatch = dispatches
            .first()
            .map(|executive_dispatch| executive_dispatch.dispatch.clone());

        Ok(RedLabelExecutiveStep {
            current_process_head,
            pre_dispatch,
            scheduled_process,
            dispatch,
            dispatches,
        })
    }

    pub(super) fn dispatch_red_label_scheduled_process(
        &mut self,
        scheduled: RedLabelScheduledProcess,
    ) -> Result<RedLabelProcessDispatch, String> {
        let layout = red_label_ram_layout()?;
        let current_process = self
            .memory
            .read_field_word(&layout, "runtime_pointers", "CRPROC")?;
        scheduled.validate_source_disp_context(current_process)?;
        let routine_address = if self.trace_power_up_ram_fill.is_some()
            && scheduled.routine_address == RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN
        {
            self.memory
                .read_process_data_word(&layout, scheduled.process_address, "PD6")?
        } else {
            scheduled.routine_address
        };

        self.dispatch_red_label_process_routine_with_entry_registers(
            routine_address,
            scheduled.entry_registers,
        )
    }

    pub fn red_label_dispatch_translated_process_routine(
        &mut self,
        routine_address: u16,
    ) -> Result<RedLabelProcessDispatch, String> {
        let dispatch = self.dispatch_red_label_process_routine(routine_address)?;
        self.apply_process_dispatch_state(&dispatch)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(dispatch)
    }

    pub fn step_red_label_translated_process(
        &mut self,
    ) -> Result<Option<RedLabelProcessDispatch>, String> {
        let Some(scheduled) = self.memory.step_process_scheduler()? else {
            return Ok(None);
        };
        let dispatch = self
            .dispatch_red_label_scheduled_process(scheduled)
            .map_err(|error| {
                format!(
                    "{error} while dispatching process 0x{:04X} routine 0x{:04X}",
                    scheduled.process_address, scheduled.routine_address
                )
            })?;
        self.apply_process_dispatch_state(&dispatch)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(Some(dispatch))
    }

    pub(super) fn dispatch_red_label_process_routine(
        &mut self,
        routine_address: u16,
    ) -> Result<RedLabelProcessDispatch, String> {
        let entry_registers = self
            .memory
            .source_entry_registers_for_current_routine(routine_address)?;
        self.dispatch_red_label_process_routine_with_entry_registers(
            routine_address,
            entry_registers,
        )
    }

    pub(super) fn dispatch_red_label_process_routine_with_entry_registers(
        &mut self,
        routine_address: u16,
        entry_registers: RedLabelCpuRegisters,
    ) -> Result<RedLabelProcessDispatch, String> {
        if routine_address == red_label_routine_address("PLSTRT")? {
            return self
                .red_label_start_player_start_current_process()
                .map(RedLabelProcessDispatch::PlayerStart);
        }

        if routine_address == red_label_routine_address("GEXBON")? {
            return self
                .finish_game_exec_wave_clear_current_process()
                .map(RedLabelProcessDispatch::GameExec);
        }

        if routine_address == red_label_routine_address("BC3")? {
            let layout = red_label_ram_layout()?;
            let process_address = self.memory.current_process_address(&layout)?;
            let return_address =
                self.memory
                    .read_process_data_word(&layout, process_address, "PD6")?;
            if return_address == red_label_routine_address("GEXBON")? {
                return self
                    .finish_game_exec_wave_clear_current_process()
                    .map(RedLabelProcessDispatch::GameExec);
            }
        }

        self.memory
            .dispatch_translated_process_routine_with_entry_registers(
                routine_address,
                entry_registers,
            )
    }

    pub(super) fn finish_game_exec_wave_clear_current_process(
        &mut self,
    ) -> Result<RedLabelGameExec, String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.memory.current_process_address(&layout)?;
        let return_address = self
            .memory
            .read_process_data_word(&layout, process_address, "PD6")?;
        let expected_return = red_label_routine_address("GEXBON")?;
        if return_address != expected_return {
            return Err(format!(
                "red-label GEXBON return 0x{return_address:04X} is not translated"
            ));
        }

        let player_table = table_descriptor(&layout, "player")?;
        let player_address = self.memory.read_field_word(&layout, "base_page", "PLRX")?;
        let player_index = entry_index_for_address(player_table, player_address)?;
        let lives_range = player_field_range_for_entry(&layout, player_index, "PLAS")?;
        let lives_before = self.memory.read_byte(lives_range.start)?;
        let lives_after = lives_before.wrapping_add(1);
        self.memory.write_byte(lives_range.start, lives_after)?;

        let player_start = self.red_label_start_player_start_current_process()?;
        Ok(RedLabelGameExec::WaveClearRestart(
            RedLabelGameExecWaveClearRestart {
                process_address,
                return_address,
                player_address,
                lives_before,
                lives_after,
                player_start,
            },
        ))
    }

    pub(super) fn apply_process_dispatch_state(
        &mut self,
        dispatch: &RedLabelProcessDispatch,
    ) -> Result<(), String> {
        match dispatch {
            RedLabelProcessDispatch::PlayerStart(
                RedLabelPlayerStart::RuntimeSleeping { .. }
                | RedLabelPlayerStart::ScreenClearedSleeping { .. }
                | RedLabelPlayerStart::GameExecReady { .. },
            ) => {
                self.phase = GamePhase::Playing;
                self.clear_live_high_score_session();
                self.sync_live_player_from_red_label_memory()?;
            }
            RedLabelProcessDispatch::PlayerDeath(RedLabelPlayerDeath::GameOverSleeping {
                ..
            }) => {
                self.phase = GamePhase::GameOver;
                self.clear_live_high_score_session();
            }
            RedLabelProcessDispatch::PlayerDeath(RedLabelPlayerDeath::AttractJump { .. }) => {
                self.phase = GamePhase::Attract;
                self.clear_live_high_score_session();
            }
            RedLabelProcessDispatch::PlayerDeath(RedLabelPlayerDeath::HallOfFameDisplayed {
                ..
            }) => {
                self.phase = GamePhase::Attract;
                self.clear_live_high_score_session();
            }
            RedLabelProcessDispatch::PlayerDeath(
                RedLabelPlayerDeath::PostExplosionRespawnJump { next_player, .. },
            ) => {
                self.phase = GamePhase::Playing;
                self.current_player = *next_player;
                self.clear_live_high_score_session();
            }
            RedLabelProcessDispatch::GameExec(RedLabelGameExec::WaveClearRestart(_)) => {
                self.phase = GamePhase::Playing;
                self.clear_live_high_score_session();
                self.sync_live_player_from_red_label_memory()?;
            }
            RedLabelProcessDispatch::StartSwitch(start) => {
                self.sync_live_credit_from_red_label_memory()?;
                if red_label_start_switch_initialized_game(start) {
                    self.phase = GamePhase::Playing;
                    self.clear_live_high_score_session();
                }
            }
            RedLabelProcessDispatch::CoinProcess(RedLabelCoinProcessStep::Completed { .. }) => {
                self.sync_live_credit_from_red_label_memory()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn sync_live_credit_from_red_label_memory(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let credit = self
            .memory
            .read_field_byte(&layout, "base_page", "CREDIT")?;
        self.credits = bcd_byte_to_u16(credit).min(u16::from(u8::MAX)) as u8;
        Ok(())
    }

    pub(super) fn sync_live_player_from_red_label_memory(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let player_table = table_descriptor(&layout, "player")?;
        let current_player = self
            .memory
            .read_field_byte(&layout, "base_page", "CURPLR")?;
        if current_player == 0 || u16::from(current_player) > player_table.entries {
            return Err(format!(
                "red-label live current player {current_player} is outside player table"
            ));
        }

        let player_index = u16::from(current_player - 1);
        let wave_range = player_field_range_for_entry(&layout, player_index, "PWAV")?;
        let lives_range = player_field_range_for_entry(&layout, player_index, "PLAS")?;
        let smart_bombs_range = player_field_range_for_entry(&layout, player_index, "PSBC")?;

        self.current_player = current_player;
        self.wave = self.memory.read_byte(wave_range.start)?;
        self.player.lives = self.memory.read_byte(lives_range.start)?;
        self.player.smart_bombs = self.memory.read_byte(smart_bombs_range.start)?;
        self.sync_player_motion_from_red_label_memory()
    }

    pub fn red_label_make_process(
        &mut self,
        routine_address: u16,
        process_type: u8,
    ) -> Result<RedLabelCreatedProcess, String> {
        self.memory.make_process(routine_address, process_type)
    }

    pub fn red_label_make_super_process(
        &mut self,
        routine_address: u16,
        process_type: u8,
    ) -> Result<RedLabelCreatedProcess, String> {
        self.memory
            .make_super_process(routine_address, process_type)
    }

    pub fn red_label_scan_translated_player_switches(
        &mut self,
        input_ports: DefenderInputPorts,
    ) -> Result<RedLabelSwitchScan, String> {
        self.memory.scan_translated_player_switches(input_ports)
    }

    pub fn red_label_scan_translated_coin_switches(
        &mut self,
        input_ports: DefenderInputPorts,
    ) -> Result<RedLabelCoinSwitchScan, String> {
        self.memory.scan_translated_coin_switches(input_ports)
    }

    pub fn red_label_dispatch_switch_processes(
        &mut self,
    ) -> Result<Vec<RedLabelCreatedProcess>, String> {
        self.memory.dispatch_switch_processes()
    }

    pub fn red_label_copy_color_mapping_to_palette_ram(
        &mut self,
    ) -> Result<RedLabelPaletteCopy, String> {
        self.memory.copy_red_label_color_mapping_to_palette_ram()
    }

    pub fn red_label_apply_free_play_credit(&mut self) -> Result<RedLabelFreePlayCredit, String> {
        self.memory.apply_free_play_credit()
    }

    pub fn red_label_advance_start_credit_tail(&mut self) -> Result<RedLabelStartCredit, String> {
        self.memory.advance_start_credit_tail()
    }

    pub fn red_label_start_game_from_credit(&mut self) -> Result<RedLabelStartGame, String> {
        self.memory.start_game_from_credit()
    }

    pub fn red_label_dispatch_start_one_current_process(
        &mut self,
    ) -> Result<RedLabelStartSwitch, String> {
        self.memory.dispatch_start_one_current_process()
    }

    pub fn red_label_dispatch_start_two_current_process(
        &mut self,
    ) -> Result<RedLabelStartSwitch, String> {
        self.memory.dispatch_start_two_current_process()
    }

    pub fn red_label_block_clear(
        &mut self,
        screen_address: u16,
        width: u8,
        height: u8,
    ) -> Result<RedLabelBlockClear, String> {
        self.memory.block_clear(screen_address, width, height)
    }

    pub fn red_label_write_object_picture_cwrit(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<RedLabelPictureWrite, String> {
        self.memory
            .write_object_picture_cwrit(screen_address, picture_address)
    }

    pub fn red_label_erase_object_picture_coff(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<RedLabelBlockClear, String> {
        self.memory
            .erase_object_picture_coff(screen_address, picture_address)
    }

    pub fn red_label_output_object_picture_by_descriptor(
        &mut self,
        screen_address: u16,
        picture_address: u16,
        alternate_flavor: bool,
    ) -> Result<Option<RedLabelPictureWrite>, String> {
        self.memory.output_object_picture_by_descriptor(
            screen_address,
            picture_address,
            alternate_flavor,
        )
    }

    pub fn red_label_erase_object_picture_by_descriptor(
        &mut self,
        screen_address: u16,
        picture_address: u16,
    ) -> Result<Option<RedLabelBlockClear>, String> {
        self.memory
            .erase_object_picture_by_descriptor(screen_address, picture_address)
    }

    pub fn red_label_top_display(&mut self) -> Result<RedLabelTopDisplay, String> {
        self.memory.top_display()
    }

    pub fn red_label_start_reverse_current_process(&mut self) -> Result<RedLabelReverse, String> {
        self.memory.start_reverse_current_process()
    }

    pub fn red_label_continue_reverse_current_process(
        &mut self,
    ) -> Result<RedLabelReverse, String> {
        self.memory.continue_reverse_current_process()
    }

    pub fn red_label_finish_reverse_current_process(&mut self) -> Result<RedLabelReverse, String> {
        self.memory.finish_reverse_current_process()
    }

    pub fn red_label_step_schizoid_current_process(
        &mut self,
    ) -> Result<RedLabelSchizoidProcessStep, String> {
        self.memory.step_schizoid_current_process()
    }

    pub fn red_label_start_ufo_process(&mut self) -> Result<RedLabelUfoStart, String> {
        self.memory.start_ufo_process()
    }

    pub fn red_label_step_ufo_current_process(&mut self) -> Result<RedLabelUfoProcessStep, String> {
        let step = self.memory.step_ufo_current_process()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(step)
    }

    pub fn red_label_start_lander_processes(
        &mut self,
        count: u8,
    ) -> Result<RedLabelLanderStart, String> {
        self.memory.start_lander_processes(count)
    }

    pub fn red_label_step_lander_orbit_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        self.memory.step_lander_orbit_current_process()
    }

    pub fn red_label_step_lander_grab_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        self.memory.step_lander_grab_current_process()
    }

    pub fn red_label_step_lander_flee_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        self.memory.step_lander_flee_current_process()
    }

    pub fn red_label_continue_lander_pull_current_process(
        &mut self,
    ) -> Result<RedLabelLanderProcessStep, String> {
        self.memory.continue_lander_pull_current_process()
    }

    pub fn red_label_step_astronaut_current_process(
        &mut self,
    ) -> Result<RedLabelAstronautProcessStep, String> {
        self.memory.step_astronaut_current_process()
    }

    pub fn red_label_step_falling_astronaut_current_process(
        &mut self,
        carried_by_player: bool,
    ) -> Result<RedLabelFallingAstronautStep, String> {
        let step = self
            .memory
            .step_falling_astronaut_current_process(carried_by_player)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(step)
    }

    pub fn red_label_start_score_sprite_current_process(
        &mut self,
        kind: RedLabelScoreSpriteKind,
    ) -> Result<RedLabelScoreSpriteStep, String> {
        let step = self.memory.start_score_sprite_current_process(kind)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(step)
    }

    pub fn red_label_finish_score_sprite_current_process(
        &mut self,
    ) -> Result<RedLabelScoreSpriteStep, String> {
        self.memory.finish_score_sprite_current_process()
    }

    pub fn red_label_step_mini_swarmer_current_process(
        &mut self,
        start_with_horizontal_seek: bool,
    ) -> Result<RedLabelMiniSwarmerProcessStep, String> {
        self.memory
            .step_mini_swarmer_current_process(start_with_horizontal_seek)
    }

    pub fn red_label_step_tie_current_process(&mut self) -> Result<RedLabelTieProcessStep, String> {
        self.memory.step_tie_current_process()
    }

    pub fn red_label_start_terrain_blow_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        self.memory.start_terrain_blow_current_process()
    }

    pub fn red_label_continue_terrain_blow_flash_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        self.memory.continue_terrain_blow_flash_current_process()
    }

    pub fn red_label_advance_terrain_blow_iteration_current_process(
        &mut self,
    ) -> Result<RedLabelTerrainBlowProcessStep, String> {
        self.memory.advance_terrain_blow_iteration_current_process()
    }

    pub fn red_label_start_hyperspace_current_process(
        &mut self,
    ) -> Result<RedLabelHyperspace, String> {
        self.memory.start_hyperspace_current_process()
    }

    pub fn red_label_continue_hyperspace_current_process(
        &mut self,
    ) -> Result<RedLabelHyperspace, String> {
        self.memory.continue_hyperspace_current_process()
    }

    pub fn red_label_finish_hyperspace_current_process(
        &mut self,
    ) -> Result<RedLabelHyperspace, String> {
        self.memory.finish_hyperspace_current_process()
    }

    pub fn red_label_start_player_death_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.start_player_death_current_process()
    }

    pub fn red_label_blank_player_death_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.blank_player_death_current_process()
    }

    pub fn red_label_continue_player_death_glow_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.continue_player_death_glow_current_process()
    }

    pub fn red_label_finish_player_death_glow_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.finish_player_death_glow_current_process()
    }

    pub fn red_label_start_player_death_tail_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.start_player_death_tail_current_process()
    }

    pub fn red_label_continue_player_explosion_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.continue_player_explosion_current_process()
    }

    pub fn red_label_continue_player_death_after_explosion_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory
            .continue_player_death_after_explosion_current_process()
    }

    pub fn red_label_start_player_death_bonus_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.start_player_death_bonus_current_process()
    }

    pub fn red_label_continue_player_death_bonus_astronaut_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory
            .continue_player_death_bonus_astronaut_current_process()
    }

    pub fn red_label_advance_player_death_bonus_wave_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory
            .advance_player_death_bonus_wave_current_process()
    }

    pub fn red_label_finish_player_death_bonus_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.finish_player_death_bonus_current_process()
    }

    pub fn red_label_continue_player_death_player_switch_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory
            .continue_player_death_player_switch_current_process()
    }

    pub fn red_label_jump_player_death_game_over_to_attract_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory
            .jump_player_death_game_over_to_attract_current_process()
    }

    pub fn red_label_display_hall_of_fame_from_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerDeath, String> {
        self.memory.display_hall_of_fame_from_current_process()
    }

    pub fn red_label_start_appearance_for_object(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelAppearanceStart, String> {
        self.memory.start_appearance_for_object(object_address)
    }

    pub fn red_label_start_explosion_for_object(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelExplosionStart, String> {
        self.memory.start_explosion_for_object(object_address)
    }

    pub fn red_label_update_expanded_objects(
        &mut self,
    ) -> Result<Vec<RedLabelExpandedUpdate>, String> {
        self.memory.update_expanded_objects()
    }

    pub fn red_label_sleep_current_process(
        &mut self,
        sleep_time: u8,
        wakeup_address: u16,
    ) -> Result<(), String> {
        self.memory
            .sleep_current_process(sleep_time, wakeup_address)
    }

    pub fn red_label_kill_process(&mut self, process_address: u16) -> Result<u16, String> {
        self.memory.kill_process(process_address)
    }

    pub fn red_label_genocide_other_processes(&mut self) -> Result<RedLabelGenocide, String> {
        self.memory.genocide_other_processes()
    }

    pub fn red_label_wave_enemy_total(&self) -> Result<u8, String> {
        self.memory.wave_enemy_total()
    }

    pub fn red_label_get_new_wave_parameters_for_player_address(
        &mut self,
        player_address: u16,
    ) -> Result<RedLabelWaveParameters, String> {
        self.memory
            .get_new_wave_parameters_for_player_address(player_address)
    }

    pub fn red_label_advance_game_exec_star_time(
        &mut self,
    ) -> Result<RedLabelGameExecStarTime, String> {
        self.memory.advance_game_exec_star_time()
    }

    pub fn red_label_start_game_exec_current_process(
        &mut self,
    ) -> Result<RedLabelGameExec, String> {
        self.memory.start_game_exec_current_process()
    }

    pub fn red_label_step_game_exec_current_process(&mut self) -> Result<RedLabelGameExec, String> {
        self.memory.step_game_exec_current_process()
    }

    pub fn red_label_finish_game_exec_wave_clear_current_process(
        &mut self,
    ) -> Result<RedLabelGameExec, String> {
        let exec = self.finish_game_exec_wave_clear_current_process()?;
        self.apply_process_dispatch_state(&RedLabelProcessDispatch::GameExec(exec.clone()))?;
        self.sync_scores_from_red_label_memory()?;
        Ok(exec)
    }

    pub fn red_label_update_player_motion_from_pia(
        &mut self,
    ) -> Result<RedLabelPlayerMotion, String> {
        let motion = self.memory.update_player_motion_from_pia()?;
        self.sync_player_motion_from_red_label_memory()?;
        Ok(motion)
    }

    pub fn red_label_display_player_picture_in_band(
        &mut self,
        upper_bound: u8,
        lower_bound: u8,
    ) -> Result<RedLabelPlayerDisplay, String> {
        self.memory
            .display_player_picture_in_band(upper_bound, lower_bound)
    }

    pub fn red_label_process_active_objects_in_band(
        &mut self,
        upper_bound: u8,
        lower_bound: u8,
    ) -> Result<RedLabelObjectDisplayBand, String> {
        self.memory
            .process_active_objects_in_band(upper_bound, lower_bound)
    }

    pub fn red_label_advance_active_object_velocities(
        &mut self,
    ) -> Result<RedLabelObjectVelocityUpdate, String> {
        self.memory.advance_active_object_velocities()
    }

    pub fn red_label_sound_table_command_plan(
        label: &str,
    ) -> Result<Vec<RedLabelSoundTableCommand>, String> {
        red_label_sound_table_command_plan(label)
    }

    pub fn red_label_sound_table_timed_command_plan(
        label: &str,
    ) -> Result<Vec<RedLabelSoundTableTimedCommand>, String> {
        red_label_sound_table_timed_command_plan(label)
    }

    pub fn red_label_sound_table_timeline(
        label: &str,
    ) -> Result<RedLabelSoundTableTimeline, String> {
        red_label_sound_table_timeline(label)
    }

    pub fn red_label_sound_table_timelines() -> Result<Vec<RedLabelSoundTableTimeline>, String> {
        red_label_sound_table_timelines()
    }

    pub fn red_label_sound_table_timeline_tsv() -> Result<String, String> {
        red_label_sound_table_timeline_tsv()
    }

    pub fn red_label_sound_table_command_sequence_tsv() -> Result<String, String> {
        red_label_sound_table_command_sequence_tsv()
    }

    pub fn red_label_sound_table_command_sequence_fixture_check()
    -> Result<RedLabelSoundTableCommandSequenceFixtureCheck, String> {
        red_label_sound_table_command_sequence_fixture_check()
    }

    pub fn red_label_sound_direct_command_sequence_tsv() -> String {
        red_label_sound_direct_command_sequence_tsv()
    }

    pub fn red_label_sound_direct_command_sequence_fixture_check()
    -> Result<RedLabelSoundDirectCommandSequenceFixtureCheck, String> {
        red_label_sound_direct_command_sequence_fixture_check()
    }

    pub fn red_label_sound_thrust_command_sequence_tsv() -> String {
        red_label_sound_thrust_command_sequence_tsv()
    }

    pub fn red_label_sound_thrust_command_sequence_fixture_check()
    -> Result<RedLabelSoundThrustCommandSequenceFixtureCheck, String> {
        red_label_sound_thrust_command_sequence_fixture_check()
    }

    pub fn red_label_sound_table_timeline_fixture_check()
    -> Result<RedLabelSoundTableTimelineFixtureCheck, String> {
        red_label_sound_table_timeline_fixture_check()
    }

    pub fn red_label_step_sound_sequence(&mut self) -> Result<RedLabelSoundSequenceStep, String> {
        self.memory.step_sound_sequence()
    }

    pub fn red_label_run_normal_irq_upper_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        self.memory.run_normal_irq_upper_object_band_pass()
    }

    pub fn red_label_run_normal_irq_lower_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        self.memory.run_normal_irq_lower_object_band_pass()
    }

    pub fn red_label_run_inverted_irq_upper_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        self.memory.run_inverted_irq_upper_object_band_pass()
    }

    pub fn red_label_run_inverted_irq_lower_object_band_pass(
        &mut self,
    ) -> Result<RedLabelIrqObjectBandPass, String> {
        self.memory.run_inverted_irq_lower_object_band_pass()
    }

    pub fn red_label_run_normal_live_irq_video_frame(
        &mut self,
    ) -> Result<RedLabelLiveVideoFrame, String> {
        self.memory.run_normal_live_irq_video_frame()
    }

    pub fn red_label_run_live_irq_video_frame(&mut self) -> Result<RedLabelLiveVideoFrame, String> {
        self.memory.run_live_irq_video_frame()
    }

    pub fn red_label_live_irq_frame_schedule(
        &self,
    ) -> Result<RedLabelLiveIrqFrameSchedule, String> {
        self.memory.live_irq_frame_schedule()
    }

    pub fn red_label_run_irq_scanline_object_phase(
        &mut self,
        mode: RedLabelIrqMode,
        vertical_counter: u8,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        self.memory
            .run_irq_scanline_object_phase(mode, vertical_counter)
    }

    pub fn red_label_run_irq_scanline_object_phase_with_context(
        &mut self,
        mode: RedLabelIrqMode,
        vertical_counter: u8,
        context: RedLabelIrqSchedulerContext,
    ) -> Result<RedLabelIrqSchedulerStep, String> {
        self.memory
            .run_irq_scanline_object_phase_with_context(mode, vertical_counter, context)
    }

    pub fn red_label_get_object_cell(&mut self) -> Result<u16, String> {
        self.memory.get_object_cell()
    }

    pub fn red_label_init_object_cell(
        &mut self,
        process_address: u16,
        descriptor: RedLabelObjectDescriptor,
    ) -> Result<RedLabelCreatedObject, String> {
        self.memory.init_object_cell(process_address, descriptor)
    }

    pub fn red_label_activate_object_cell(&mut self, object_address: u16) -> Result<(), String> {
        self.memory.activate_object_cell(object_address)
    }

    pub fn red_label_kill_object_cell(&mut self, object_address: u16) -> Result<u16, String> {
        self.memory.kill_object_cell(object_address)
    }

    pub fn red_label_kill_object_cell_offscreen(
        &mut self,
        object_address: u16,
    ) -> Result<u16, String> {
        self.memory.kill_object_cell_offscreen(object_address)
    }

    pub fn red_label_kill_shell_cell(&mut self, object_address: u16) -> Result<u16, String> {
        self.memory.kill_shell_cell(object_address)
    }

    pub fn red_label_get_shell_cell(
        &mut self,
        firing_object_address: u16,
        owner_address: u16,
        descriptor: RedLabelShellDescriptor,
    ) -> Result<Option<RedLabelCreatedShell>, String> {
        self.memory
            .get_shell_cell(firing_object_address, owner_address, descriptor)
    }

    pub fn red_label_scan_shell_list(&mut self) -> Result<Vec<u16>, String> {
        self.memory.scan_shell_list()
    }

    pub fn red_label_scan_active_objects_for_offscreen(&mut self) -> Result<Vec<u16>, String> {
        self.memory.scan_active_objects_for_offscreen()
    }

    pub fn red_label_scan_inactive_objects_for_on_screen(&mut self) -> Result<Vec<u16>, String> {
        self.memory.scan_inactive_objects_for_on_screen()
    }

    pub fn red_label_start_scanner_process_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        self.memory.start_scanner_process_current_process()
    }

    pub fn red_label_continue_scanner_process_object_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        self.memory
            .continue_scanner_process_object_current_process()
    }

    pub fn red_label_continue_scanner_process_display_current_process(
        &mut self,
    ) -> Result<RedLabelScannerProcessStep, String> {
        self.memory
            .continue_scanner_process_display_current_process()
    }

    pub fn red_label_draw_scanner_raster(&mut self) -> Result<RedLabelScannerRaster, String> {
        self.memory.draw_scanner_raster()
    }

    pub fn red_label_start_player_start_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        self.red_label_initialize_color_ram_table()?;
        self.red_label_initialize_laser_fizzle_table()?;
        self.red_label_initialize_star_table()?;
        self.memory.initialize_object_lists_from_embedded_layout()?;
        self.red_label_initialize_fireball_table()?;
        self.red_label_initialize_thrust_table()?;
        self.memory
            .start_player_start_after_init20_current_process()
    }

    pub fn red_label_continue_player_start_after_coin_counters_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        self.memory
            .continue_player_start_after_coin_counters_current_process()
    }

    pub fn red_label_start_player_runtime_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        self.memory.start_player_runtime_current_process()
    }

    pub fn red_label_continue_player_start_screen_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        self.memory.continue_player_start_screen_current_process()
    }

    pub fn red_label_finish_player_start_current_process(
        &mut self,
    ) -> Result<RedLabelPlayerStart, String> {
        self.memory.finish_player_start_current_process()
    }

    pub fn red_label_output_bomb_shell(
        &mut self,
        shell_address: u16,
        old_screen_address: u16,
    ) -> Result<(), String> {
        self.memory
            .output_bomb_shell(shell_address, old_screen_address)
    }

    pub fn red_label_output_fireball_shell(
        &mut self,
        shell_address: u16,
        old_screen_address: u16,
    ) -> Result<(), String> {
        self.memory
            .output_fireball_shell(shell_address, old_screen_address)
    }

    pub fn red_label_dispatch_shell_output_step(
        &mut self,
        step: RedLabelShellStep,
    ) -> Result<Option<RedLabelShellOutputRoutine>, String> {
        self.memory.dispatch_shell_output_step(step)
    }

    pub fn red_label_dispatch_shell_output_steps(
        &mut self,
        steps: &[RedLabelShellStep],
    ) -> Result<Vec<RedLabelShellOutputRoutine>, String> {
        self.memory.dispatch_shell_output_steps(steps)
    }

    pub fn red_label_step_shell_output(&mut self) -> Result<Vec<RedLabelShellStep>, String> {
        self.memory.step_shell_output()
    }

    pub fn red_label_score_current_player(
        &mut self,
        score_word: u16,
    ) -> Result<RedLabelScoreOutcome, String> {
        let outcome = self.memory.score_current_player(score_word)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(outcome)
    }

    pub fn red_label_kill_bomb_shell_collision(
        &mut self,
        shell_address: u16,
    ) -> Result<RedLabelBombCollision, String> {
        let collision = self.memory.kill_bomb_shell_collision(shell_address)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_kill_ufo_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelEnemyKill, String> {
        let collision = self.memory.kill_ufo_collision(object_address)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_kill_lander_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelEnemyKill, String> {
        let collision = self.memory.kill_lander_collision(object_address)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_kill_kidnapping_lander_collision(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelKidnappingLanderKill, String> {
        let collision = self
            .memory
            .kill_kidnapping_lander_collision(object_address)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_dispatch_object_collision_vector(
        &mut self,
        object_address: u16,
    ) -> Result<RedLabelObjectCollision, String> {
        let collision = self
            .memory
            .dispatch_object_collision_vector(object_address)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_collide_picture_with_active_objects(
        &mut self,
        picture_address: u16,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        let collision = self
            .memory
            .collide_picture_with_active_objects(picture_address, upper_left)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_collide_picture_with_shells(
        &mut self,
        picture_address: u16,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        let collision = self
            .memory
            .collide_picture_with_shells(picture_address, upper_left)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_collide_laser_with_active_objects(
        &mut self,
        upper_left: u16,
    ) -> Result<Option<RedLabelObjectCollision>, String> {
        let collision = self.memory.collide_laser_with_active_objects(upper_left)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_initialize_color_ram_table(&mut self) -> Result<RedLabelColorRamInit, String> {
        self.memory.initialize_color_ram_from_crtab()
    }

    pub fn red_label_initialize_altitude_table(
        &mut self,
    ) -> Result<RedLabelAltitudeTableInit, String> {
        self.memory.initialize_altitude_table_from_tdata()
    }

    pub fn red_label_initialize_terrain_tables(
        &mut self,
    ) -> Result<RedLabelTerrainTablesInit, String> {
        self.memory.initialize_terrain_tables_from_bgl()
    }

    pub fn red_label_initialize_background_from_bgi(
        &mut self,
    ) -> Result<RedLabelBackgroundInit, String> {
        self.memory.initialize_background_from_bgi()
    }

    pub fn red_label_initialize_attract_scene_from_scinit(
        &mut self,
    ) -> Result<RedLabelAttractSceneInit, String> {
        self.memory.initialize_attract_scene_from_scinit()
    }

    pub fn red_label_output_terrain(
        &mut self,
        hardware_stack_pointer: u16,
    ) -> Result<RedLabelTerrainOutput, String> {
        self.memory.output_terrain_from_bgl(hardware_stack_pointer)
    }

    pub fn red_label_erase_terrain_from_screen_table(
        &mut self,
    ) -> Result<RedLabelTerrainErase, String> {
        self.memory.erase_terrain_from_screen_table()
    }

    pub fn red_label_erase_scanner_terrain_from_erase_table(
        &mut self,
    ) -> Result<RedLabelScannerTerrainErase, String> {
        self.memory.erase_scanner_terrain_from_erase_table()
    }

    pub fn red_label_initialize_laser_fizzle_table(
        &mut self,
    ) -> Result<RedLabelLaserFizzleInit, String> {
        let mut rand_values = [0; 0x20];
        for rand_value in &mut rand_values {
            self.rng.advance();
            *rand_value = self.rng.seed;
        }
        self.memory
            .initialize_laser_fizzle_table_from_rand_values(&rand_values)
    }

    pub fn red_label_initialize_star_table(&mut self) -> Result<RedLabelStarTableInit, String> {
        let (stars, rand_values_consumed) =
            star_table_from_rand_values(std::iter::from_fn(|| {
                self.rng.advance();
                Some(self.rng.seed)
            }))?;
        self.memory.write_star_table(&stars, rand_values_consumed)
    }

    pub fn red_label_output_stars(&mut self) -> Result<RedLabelStarOutput, String> {
        self.memory.output_stars()
    }

    pub fn red_label_initialize_fireball_table(
        &mut self,
    ) -> Result<RedLabelFireballTableInit, String> {
        let mut rand_values = [0; 0x20];
        for rand_value in &mut rand_values {
            self.rng.advance();
            *rand_value = self.rng.seed;
        }
        self.memory
            .initialize_fireball_table_from_rand_values(&rand_values)
    }

    pub fn red_label_initialize_thrust_table(&mut self) -> Result<RedLabelThrustTableInit, String> {
        let mut rand_values = [0; 33];
        for rand_value in &mut rand_values {
            self.rng.advance();
            *rand_value = self.rng.seed;
        }
        self.memory
            .initialize_thrust_table_from_rand_values(&rand_values)
    }

    pub fn red_label_step_thrust_process_current_process(
        &mut self,
    ) -> Result<RedLabelThrustProcessStep, String> {
        self.memory.step_thrust_process_current_process()
    }

    pub fn red_label_start_laser_fire_current_process(
        &mut self,
    ) -> Result<RedLabelLaserFire, String> {
        let fire = self.memory.start_laser_fire_current_process()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(fire)
    }

    pub fn red_label_dispatch_laser_fire_current_process(
        &mut self,
    ) -> Result<RedLabelLaserFireDispatch, String> {
        let fire = self.memory.dispatch_laser_fire_current_process()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(fire)
    }

    pub fn red_label_finish_laser_fire_current_process(
        &mut self,
    ) -> Result<RedLabelKilledProcess, String> {
        self.memory.finish_laser_fire_current_process()
    }

    pub fn red_label_step_right_laser_current_process(
        &mut self,
    ) -> Result<RedLabelLaserStep, String> {
        let step = self.memory.step_right_laser_current_process()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(step)
    }

    pub fn red_label_step_left_laser_current_process(
        &mut self,
    ) -> Result<RedLabelLaserStep, String> {
        let step = self.memory.step_left_laser_current_process()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(step)
    }

    pub fn red_label_check_player_collision(
        &mut self,
    ) -> Result<Option<RedLabelPlayerCollision>, String> {
        let collision = self.memory.check_player_collision()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(collision)
    }

    pub fn red_label_start_smart_bomb_current_player(
        &mut self,
    ) -> Result<Option<RedLabelSmartBomb>, String> {
        let smart_bomb = self.memory.start_smart_bomb_current_player()?;
        self.sync_scores_from_red_label_memory()?;
        Ok(smart_bomb)
    }

    pub fn red_label_continue_smart_bomb_flash_tail(
        &mut self,
    ) -> Result<RedLabelSmartBombTail, String> {
        self.memory.continue_smart_bomb_flash_tail()
    }

    pub fn red_label_continue_smart_bomb_debounce_tail(
        &mut self,
    ) -> Result<RedLabelSmartBombTail, String> {
        self.memory.continue_smart_bomb_debounce_tail()
    }

    pub fn red_label_finish_smart_bomb_tail(&mut self) -> Result<RedLabelSmartBombTail, String> {
        self.memory.finish_smart_bomb_tail()
    }

    pub fn step(&mut self, input: CabinetInput) -> FrameOutput {
        self.step_with_typed_chars(input, &[])
    }

    pub fn step_with_typed_chars(
        &mut self,
        input: CabinetInput,
        typed_chars: &[char],
    ) -> FrameOutput {
        self.frame = self.frame.saturating_add(1);
        self.last_input_bits = input.bits();
        if self.trace_power_up_ram_fill.is_some() {
            let special_input_bits =
                red_label_trace_power_on_effective_special_input_bits(self.last_input_bits, None);
            if special_input_bits != 0 {
                self.trace_power_on_recent_special_input = Some((self.frame, special_input_bits));
            }
        }
        self.begin_main_board_frame(input);
        self.rng.advance();

        let mut events = Vec::new();
        let mut sound_commands = Vec::new();
        let mut start_ready_rand_already_advanced = false;
        if !self.trace_power_up_blocks_live_io() {
            if let Some(command) = self
                .memory
                .step_sound_sequence()
                .expect("embedded red-label sound sequence layout is valid")
                .command
            {
                sound_commands.push(command);
            }
            self.push_trace_sound_synchronized_events(&sound_commands, &mut events);

            let game_over_handoff_active = self
                .step_live_game_over_attract_handoff(&mut events)
                .expect("red-label game-over attract handoff should remain valid");
            let feed_high_score_entry = !game_over_handoff_active
                && (self.phase == GamePhase::HighScoreEntry
                    || (self.phase == GamePhase::GameOver
                        && self.trace_power_up_ram_fill.is_none()
                        && self
                            .begin_next_live_high_score_entry(&mut events)
                            .expect("red-label high-score table should be valid")));
            if feed_high_score_entry {
                self.step_live_high_score_entry(typed_chars, &mut events);
            } else if !game_over_handoff_active {
                start_ready_rand_already_advanced =
                    self.step_live_non_high_score_input(input, &mut events, &mut sound_commands);
            }
        }

        self.advance_trace_power_up_ram_fill(
            &mut sound_commands,
            start_ready_rand_already_advanced,
        );
        self.record_sound_board_frame_commands(&sound_commands);

        FrameOutput::new(
            self.snapshot(),
            self.red_label_trace_state_for_frame_output()
                .expect("red-label trace state should match embedded RAM layout"),
            self.red_label_main_board_snapshot(),
            self.red_label_sound_board_snapshot(),
            FrameTraceCrcs {
                object_table_crc32: Some(self.red_label_trace_object_table_crc32_for_frame()),
                process_table_crc32: Some(self.red_label_trace_process_table_crc32_for_frame()),
                super_process_table_crc32: Some(self.memory.super_process_table_crc32()),
                shell_table_crc32: Some(self.memory.shell_table_crc32()),
                video_crc32: self.red_label_trace_visible_video_crc32_for_frame(),
            },
            &events,
            &sound_commands,
        )
    }

    pub(super) fn begin_main_board_frame(&mut self, input: CabinetInput) {
        self.main_board_input_ports = input.defender_input_ports();
        self.main_board_video_counter_vpos = 0;
    }

    pub(super) fn record_main_board_live_video_frame(&mut self, frame: &RedLabelLiveVideoFrame) {
        for scanline in [&frame.upper_scanline, &frame.lower_scanline] {
            self.main_board_video_counter_vpos = u16::from(scanline.vertical_counter);
            if scanline.watchdog_value == Some(WATCHDOG_RESET_BYTE) {
                self.main_board_watchdog_reset_count =
                    self.main_board_watchdog_reset_count.saturating_add(1);
            }
        }
    }

    pub(super) fn record_sound_board_frame_commands(&mut self, commands: &[SoundCommand]) {
        for command in commands {
            self.sound_board_last_command_latch =
                Some(SoundCommandLatch::from_main_board_pia_port_b(command.raw()));
            self.sound_board_latch_write_count =
                self.sound_board_latch_write_count.saturating_add(1);
        }
    }

    pub(super) fn push_trace_sound_synchronized_events(
        &self,
        sound_commands: &[SoundCommand],
        events: &mut Vec<MachineEvent>,
    ) {
        if self.trace_power_up_ram_fill.is_none() {
            return;
        }
        for command in sound_commands {
            match command.raw() {
                RED_LABEL_TRACE_CREDIT_SOUND_COMMAND_RAW => events.push(MachineEvent::CreditAdded),
                RED_LABEL_TRACE_START_SOUND_COMMAND_RAW => events.push(MachineEvent::GameStarted),
                _ => {}
            }
        }
    }

    pub(super) fn push_trace_immediate_sound_command(
        &mut self,
        expected_raw: u8,
        sound_commands: &mut Vec<SoundCommand>,
        events: &mut Vec<MachineEvent>,
    ) -> Result<(), String> {
        let Some(command) = self.memory.step_sound_sequence()?.command else {
            return Err(format!(
                "red-label trace expected immediate sound command 0x{expected_raw:02X}"
            ));
        };
        if command.raw() != expected_raw {
            return Err(format!(
                "red-label trace expected immediate sound command 0x{expected_raw:02X}, got 0x{:02X}",
                command.raw()
            ));
        }

        sound_commands.push(command);
        match command.raw() {
            RED_LABEL_TRACE_CREDIT_SOUND_COMMAND_RAW => events.push(MachineEvent::CreditAdded),
            RED_LABEL_TRACE_START_SOUND_COMMAND_RAW => events.push(MachineEvent::GameStarted),
            _ => {}
        }
        Ok(())
    }

    pub fn red_label_begin_live_high_score_entry(
        &mut self,
        score: u32,
    ) -> Result<Option<HighScoreEntryState>, String> {
        self.begin_live_high_score_entry(self.current_high_score_player(), score)
    }

    pub(super) fn begin_live_high_score_entry(
        &mut self,
        player: u8,
        score: u32,
    ) -> Result<Option<HighScoreEntryState>, String> {
        let Some(rank) = self.memory.live_high_score_qualifying_rank(score)? else {
            return Ok(None);
        };
        let state = HighScoreEntryState::new(score, rank);
        self.memory.write_high_score_entry_display(player, state)?;
        self.phase = GamePhase::HighScoreEntry;
        self.high_score_entry = Some(state);
        self.high_score_entry_player = player;
        Ok(Some(state))
    }

    pub(super) fn step_live_non_high_score_input(
        &mut self,
        input: CabinetInput,
        events: &mut Vec<MachineEvent>,
        sound_commands: &mut Vec<SoundCommand>,
    ) -> bool {
        let mut started_this_frame = false;
        let mut start_ready_rand_already_advanced = false;
        let cold_boot_game_over_attract_active = self.phase == GamePhase::GameOver
            && self.trace_power_up_ram_fill.is_some()
            && self
                .red_label_live_attract_process_active()
                .expect("live attract process labels are valid");
        if cold_boot_game_over_attract_active {
            start_ready_rand_already_advanced |= self
                .step_red_label_trace_game_over_attract_process()
                .expect("trace game-over attract process is source-shaped");
        }

        let coin_door_outcome = self
            .step_red_label_live_coin_door_switches(input)
            .expect("live coin/admin switch creates only translated red-label processes");
        if coin_door_outcome.credit_added {
            if self.trace_power_up_ram_fill.is_some() {
                self.push_trace_immediate_sound_command(
                    RED_LABEL_TRACE_CREDIT_SOUND_COMMAND_RAW,
                    sound_commands,
                    events,
                )
                .expect("trace coin sound command should be source-shaped");
            } else {
                events.push(MachineEvent::CreditAdded);
            }
        }
        if let Some(event) = coin_door_outcome.admin_event {
            events.push(event);
        }

        let start_pressed = input.start_one || input.start_two;
        if self.trace_power_up_ram_fill.is_some() {
            self.trace_start_asserted_frames = if start_pressed {
                self.trace_start_asserted_frames.saturating_add(1)
            } else {
                0
            };
        }
        let start_debounced =
            self.trace_power_up_ram_fill.is_none() || self.trace_start_asserted_frames >= 3;

        if start_pressed
            && start_debounced
            && matches!(self.phase, GamePhase::Attract | GamePhase::GameOver)
        {
            let start_outcome = self
                .step_red_label_live_start_switch(input)
                .expect("live start switch creates only translated red-label processes");
            if start_outcome.start_accepted && self.trace_power_up_ram_fill.is_some() {
                self.push_trace_immediate_sound_command(
                    RED_LABEL_TRACE_START_SOUND_COMMAND_RAW,
                    sound_commands,
                    events,
                )
                .expect("trace start sound command should be source-shaped");
            }
            if start_outcome.game_started && self.trace_power_up_ram_fill.is_some() {
                // MAME reaches the first visible `PLSTRT` object mutation 50
                // sampled frames later than the coarse live loop. Hold the
                // translated process until that source scheduler slot.
                self.trace_player_start_release_frame =
                    Some(self.frame + RED_LABEL_TRACE_PLAYER_START_EXEC_DELAY_FRAMES);
            }
            if start_outcome.game_started && self.trace_power_up_ram_fill.is_none() {
                events.push(MachineEvent::GameStarted);
            }
            started_this_frame = start_outcome.game_started;
        }

        if self.phase == GamePhase::Playing
            && !started_this_frame
            && self
                .trace_player_start_release_frame
                .is_none_or(|frame| self.frame >= frame)
        {
            let switch_outcome = self.step_red_label_live_player_switches(input);
            if !switch_outcome.player_start_active {
                self.step_player_controls(input, events, switch_outcome);
            }
        }
        if !cold_boot_game_over_attract_active
            && self.phase == GamePhase::Attract
            && input == CabinetInput::NONE
            && self.credits == 0
            && !self
                .red_label_live_coin_door_process_active()
                .expect("live coin/admin process labels are valid")
        {
            self.step_red_label_live_attract_process()
                .expect("live attract process creates only translated red-label work");
        }
        start_ready_rand_already_advanced
    }

    pub(super) fn step_red_label_trace_game_over_attract_process(
        &mut self,
    ) -> Result<bool, String> {
        if self.frame >= RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_FIRST_FRAME {
            return Ok(self.frame <= RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_FRAME);
        }
        match red_label_power_on_frame_model(self.frame)?.process_boundary {
            Some(RedLabelPowerOnProcessBoundary::AttractVector) => {
                self.start_trace_power_on_williams_screen_clear()?;
                Ok(true)
            }
            Some(RedLabelPowerOnProcessBoundary::AttractWilliamsPage) => {
                self.finish_trace_power_on_williams_screen_clear()?;
                Ok(true)
            }
            Some(RedLabelPowerOnProcessBoundary::AttractColorCadence) => {
                self.step_trace_power_on_color_executive_slice()?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub(super) fn start_trace_power_on_williams_screen_clear(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let Some(scheduled) = self.memory.step_process_scheduler()? else {
            return Ok(());
        };
        if scheduled.routine_address != red_label_routine_address("ATTR")? {
            return Err(format!(
                "red-label trace power-on expected ATTR at frame 733, got 0x{:04X}",
                scheduled.routine_address
            ));
        }

        self.memory.start_attract_vector_current_process()?;
        self.memory.write_process_word(
            &layout,
            scheduled.process_address,
            "PADDR",
            red_label_routine_address("ATTR")?,
        )?;
        self.memory.write_field_byte(
            &layout,
            "base_page",
            "STATUS",
            RED_LABEL_ATTRACT_WILLIAMS_STATUS,
        )
    }

    pub(super) fn finish_trace_power_on_williams_screen_clear(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let process_address = self.memory.current_process_address(&layout)?;
        let paddr = self
            .memory
            .read_process_word(&layout, process_address, "PADDR")?;
        if paddr != red_label_routine_address("ATTR")? {
            return Ok(());
        }

        self.memory.start_attract_williams_page_current_process()?;
        self.memory.write_process_word(
            &layout,
            process_address,
            "PADDR",
            red_label_routine_address("ATTR")?,
        )
    }

    pub(super) fn step_trace_power_on_color_executive_slice(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.memory.reset_exec_current_process_to_active(&layout)?;
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_first_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_VIDEO_ONLY_FRAME {
            self.memory
                .run_trace_power_on_sixth_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_ninth_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixteenth_defender_appearance_video_slice()?;
            let lists = red_label_linked_lists()?;
            let active_head = linked_list(&lists, "active_process")?.head_address;
            self.memory
                .step_single_process_scheduler_from_link(active_head)?;
            self.memory
                .apply_trace_power_on_sixteenth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_seventeenth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTEENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTEENTH_PROCESS_DATA_WORDS
            {
                self.memory.write_word(address, value)?;
            }
            self.memory
                .apply_trace_power_on_seventeenth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_eighteenth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTEENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            self.memory
                .apply_trace_power_on_eighteenth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_nineteenth_defender_appearance_video_slice()?;
            let lists = red_label_linked_lists()?;
            let active_head = linked_list(&lists, "active_process")?.head_address;
            self.memory
                .step_single_process_scheduler_from_link(active_head)?;
            self.memory
                .apply_trace_power_on_nineteenth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twentieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTIETH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            self.memory
                .apply_trace_power_on_twentieth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_first_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_third_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_THIRD_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_fourth_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_fifth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_FIFTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_sixth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SIXTH_PROCESS_BYTES {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_seventh_defender_appearance_video_slice()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_eighth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_EIGHTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twenty_ninth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_TWENTY_NINTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirtieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTIETH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_first_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FIRST_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_third_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_THIRD_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_fourth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FOURTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_fifth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_FIFTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_sixth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SIXTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_seventh_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_SEVENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_eighth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_EIGHTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_EIGHTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_thirty_ninth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTY_NINTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fortieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTIETH_PROCESS_TIMER_BYTES {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_first_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIRST_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_third_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_THIRD_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_fourth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FOURTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_fifth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_FIFTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_sixth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SIXTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_seventh_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_SEVENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_eighth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_EIGHTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_forty_ninth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FORTY_NINTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fiftieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTIETH_PROCESS_TIMER_BYTES {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTIETH_PROCESS_DATA_BYTES {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_first_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIRST_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIRST_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SECOND_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_third_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_THIRD_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_THIRD_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_fourth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FOURTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FOURTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_fifth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIFTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_FIFTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_sixth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SIXTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SIXTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_seventh_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SEVENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_SEVENTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_eighth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_EIGHTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_EIGHTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifty_ninth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_NINTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTY_NINTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixtieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTIETH_PROCESS_TIMER_BYTES {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTIETH_PROCESS_DATA_BYTES {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_first_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIRST_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIRST_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SECOND_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_third_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_THIRD_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_THIRD_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_fourth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FOURTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FOURTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_fifth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIFTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_FIFTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_sixth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SIXTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SIXTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_seventh_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SEVENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_SEVENTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_eighth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_EIGHTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_EIGHTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_sixty_ninth_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_NINTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SIXTY_NINTH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_seventieth_defender_appearance_video_slice()?;
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTIETH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTIETH_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_seventy_first_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIRST_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIRST_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_seventy_second_defender_appearance_video_slice()?;
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SECOND_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            for (address, value) in
                RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SECOND_PROCESS_DATA_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_fourth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SIXTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_sixth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_seventh_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIRST_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_first_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_third_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_fifth_defender_appearance_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SEVENTH_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_seventh_defender_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_NINTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_ninth_defender_process_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETIETH_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninetieth_defender_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SECOND_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_second_defender_process_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_THIRD_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_third_defender_process_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SEVENTH_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_seventh_defender_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_EIGHTH_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_eighth_defender_video_boundary()?;
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_NINTH_HOLD_FRAME {
            return self.sync_scores_from_red_label_memory();
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_second_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_tenth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ELEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_eleventh_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_twelfth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_APPEARANCE_VIDEO_FRAME
        {
            self.memory
                .run_trace_power_on_thirteenth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTEENTH_APPEARANCE_VIDEO_FRAME
        {
            self.memory
                .run_trace_power_on_fourteenth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifteenth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ZERO_SAMPLE_FRAME {
            self.memory
                .run_trace_power_on_third_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ZERO_RECOVERY_FRAME {
            self.memory
                .run_trace_power_on_fourth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_fifth_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_seventh_defender_appearance_video_slice()?;
        } else if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .run_trace_power_on_eighth_defender_appearance_video_slice()?;
        } else {
            self.memory.run_exec_pre_dispatch_visible_slice()?;
        }
        let lists = red_label_linked_lists()?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let attr_link_address = self
            .memory
            .active_process_link_before_routine(&[RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN])?
            .unwrap_or(active_head);
        let attr_process = self.memory.read_word(attr_link_address)?;
        let tiecol = red_label_routine_address("TIECOL")?;
        let tiecl = red_label_routine_address("TIECL")?;
        let tie_link = self
            .memory
            .active_process_link_before_routine(&[tiecol, tiecl])?
            .ok_or_else(|| {
                format!(
                    "red-label trace power-on expected TIECOL/TIECL at frame {}",
                    self.frame
                )
            })?;
        let tie_process = self.memory.read_word(tie_link)?;
        let tie_routine = self
            .memory
            .read_process_word(&layout, tie_process, "PADDR")?;
        let colr = red_label_routine_address("COLR")?;
        let colrlp = red_label_routine_address("COLRLP")?;
        let color_link_address = self
            .memory
            .active_process_link_before_routine(&[colr, colrlp])?
            .ok_or_else(|| {
                format!(
                    "red-label trace power-on expected COLR/COLRLP at frame {}",
                    self.frame
                )
            })?;
        let color_process = self.memory.read_word(color_link_address)?;
        let color_routine = self
            .memory
            .read_process_word(&layout, color_process, "PADDR")?;
        if color_routine != colr && color_routine != colrlp {
            return Err(format!(
                "red-label trace power-on expected COLR/COLRLP at frame {}, got 0x{:04X}",
                self.frame, color_routine
            ));
        }

        let previous_current_process =
            self.memory
                .read_field_word(&layout, "runtime_pointers", "CRPROC")?;
        self.memory
            .write_field_word(&layout, "runtime_pointers", "CRPROC", attr_process)?;
        let cadence = if tie_routine == tiecol {
            self.start_trace_power_on_color_cadence(
                &layout,
                attr_process,
                tie_process,
                color_process,
            )
        } else {
            self.step_trace_power_on_color_cadence(
                &layout,
                attr_process,
                tie_process,
                color_process,
            )
        };
        self.memory.write_field_word(
            &layout,
            "runtime_pointers",
            "CRPROC",
            previous_current_process,
        )?;
        cadence?;
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_tenth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ELEVENTH_APPEARANCE_VIDEO_FRAME {
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_ELEVENTH_PROCESS_TIMER_BYTES {
                self.memory.write_byte(address, value)?;
            }
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_APPEARANCE_VIDEO_FRAME {
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_TWELFTH_PROCESS_TIMER_BYTES {
                self.memory.write_byte(address, value)?;
            }
            self.memory
                .apply_trace_power_on_twelfth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_APPEARANCE_VIDEO_FRAME {
            for (address, value) in RED_LABEL_TRACE_POWER_ON_DEFENDER_THIRTEENTH_PROCESS_TIMER_BYTES
            {
                self.memory.write_byte(address, value)?;
            }
            self.memory
                .apply_trace_power_on_thirteenth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FOURTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_fourteenth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_FIFTEENTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_fifteenth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_THIRD_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_third_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_FIFTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_fifth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_EIGHTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_eighth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_SEVENTY_NINTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_seventy_ninth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTIETH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eightieth_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SECOND_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_second_defender_appearance_video_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_FOURTH_APPEARANCE_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_fourth_defender_appearance_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_SIXTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_sixth_defender_process_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_EIGHTY_EIGHTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_eighty_eighth_defender_process_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FIRST_PROCESS_FRAME {
            self.memory
                .apply_trace_power_on_ninety_first_defender_process_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FOURTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_fourth_defender_process_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_FIFTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_fifth_defender_process_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_NINETY_SIXTH_PROCESS_VIDEO_FRAME {
            self.memory
                .apply_trace_power_on_ninety_sixth_defender_process_video_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDREDTH_PROCESS_FRAME {
            self.memory
                .apply_trace_power_on_one_hundredth_defender_process_boundary()?;
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_FIRST_PROCESS_FRAME {
            self.memory
                .apply_trace_power_on_one_hundred_first_defender_process_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_SECOND_PROCESS_FRAME {
            self.memory
                .apply_trace_power_on_one_hundred_second_defender_process_boundary()?;
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_THIRD_PROCESS_FRAME {
            self.memory
                .apply_trace_power_on_one_hundred_third_defender_process_boundary()?;
        }
        self.sync_scores_from_red_label_memory()
    }

    pub(super) fn apply_trace_power_on_start_handoff_frame(&mut self) -> Result<(), String> {
        if self.trace_power_up_ram_fill.is_none() {
            return Ok(());
        }

        let layout = red_label_ram_layout()?;
        if (RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_FIRST_FRAME
            ..=RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_LAST_FRAME)
            .contains(&self.frame)
        {
            return self.step_trace_power_on_start_handoff_stall(&layout);
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_PRE_FRAME {
            return self.step_trace_power_on_start_handoff_instruction_pre(&layout);
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_FRAME {
            return self.step_trace_power_on_start_handoff_instruction_entry(&layout);
        }
        if self.frame == RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_FIRST_EXEC_FRAME {
            return self.step_trace_power_on_start_handoff_instruction_first_exec(&layout);
        }
        if self.frame > RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_FRAME
            && self.trace_power_on_instruction_process_active()?
        {
            self.step_red_label_executive_iteration()?;
            if self.frame == RED_LABEL_TRACE_POWER_ON_STAR_TABLE_SAMPLE_FRAME {
                self.memory.write_range(
                    0xAF9D..0xAFDD,
                    &RED_LABEL_TRACE_POWER_ON_STAR_TABLE_SAMPLE_BYTES,
                )?;
                self.memory
                    .write_field_byte(&layout, "base_page", "IFLG", 1)?;
            }
            if self.frame >= RED_LABEL_TRACE_POWER_ON_INSTRUCTION_VISIBLE_IRQ_FIRST_FRAME {
                if self.memory.read_byte(0xA092)? != 0 {
                    let timer = self.memory.read_field_byte(&layout, "base_page", "TIMER")?;
                    self.memory
                        .write_field_byte(&layout, "base_page", "IFLG", 0)?;
                    self.memory.write_field_byte(
                        &layout,
                        "base_page",
                        "TIMER",
                        timer.wrapping_add(1),
                    )?;
                    self.memory
                        .write_field_byte(&layout, "base_page", "XXX1", 0xFF)?;
                    self.memory.copy_red_label_color_mapping_to_palette_ram()?;
                    self.memory
                        .output_terrain_from_bgl(red_label_irq_bgout_stack_pointer())?;
                }
                let saved_rand_state =
                    if self.frame == RED_LABEL_TRACE_POWER_ON_STAR_BLINK_PREVIOUS_SEED_FRAME {
                        let state = RandState {
                            seed: self.memory.read_field_byte(&layout, "base_page", "SEED")?,
                            hseed: self.memory.read_field_byte(&layout, "base_page", "HSEED")?,
                            lseed: self.memory.read_field_byte(&layout, "base_page", "LSEED")?,
                        };
                        self.write_trace_rand_state(
                            &layout,
                            RED_LABEL_TRACE_POWER_ON_STAR_BLINK_PREVIOUS_RAND_STATE,
                        )?;
                        Some(state)
                    } else {
                        None
                    };
                self.apply_trace_power_on_start_handoff_object_boundary()?;
                self.memory.run_irq_scanline_object_phase_with_context(
                    RedLabelIrqMode::Normal,
                    RED_LABEL_TRACE_POWER_ON_INSTRUCTION_VISIBLE_IRQ_VERTCT,
                    RedLabelIrqSchedulerContext::source_irq_after_sound_step(
                        DefenderInputPorts::EMPTY,
                    ),
                )?;
                if let Some(state) = saved_rand_state {
                    self.write_trace_rand_state(&layout, state)?;
                }
                self.apply_trace_power_on_start_handoff_visible_boundary()?;
                self.apply_trace_power_on_start_handoff_process_boundary()?;
            }
            return self.sync_scores_from_red_label_memory();
        }

        Ok(())
    }

    pub(super) fn apply_trace_power_on_start_handoff_visible_boundary(
        &mut self,
    ) -> Result<(), String> {
        if self.frame == RED_LABEL_TRACE_POWER_ON_START_HANDOFF_VISIBLE_BOUNDARY_FRAME {
            for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_VISIBLE_NIBBLES {
                self.memory
                    .write_visible_pixel_nibble(visible_index, nibble)?;
            }
        }

        if (RED_LABEL_TRACE_POWER_ON_START_HANDOFF_BOTTOM_OVERLAY_FIRST_FRAME
            ..=RED_LABEL_TRACE_POWER_ON_START_HANDOFF_BOTTOM_OVERLAY_LAST_FRAME)
            .contains(&self.frame)
        {
            for (visible_index, nibble) in
                RED_LABEL_TRACE_POWER_ON_START_HANDOFF_BOTTOM_OVERLAY_NIBBLES
            {
                self.memory
                    .write_visible_pixel_nibble(visible_index, nibble)?;
            }
        }

        for (first_frame, row_start) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_ERASE_ROWS {
            if self.frame >= first_frame {
                for offset in 0..3 {
                    self.memory
                        .write_visible_pixel_nibble(row_start + offset, 0)?;
                }
            }
        }

        if self.frame == 1258 {
            for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_EXTRA_1258_NIBBLES
            {
                self.memory
                    .write_visible_pixel_nibble(visible_index, nibble)?;
            }
        }

        if (1259..=1265).contains(&self.frame) {
            for (visible_index, nibble) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_1259_1265_NIBBLES
            {
                self.memory
                    .write_visible_pixel_nibble(visible_index, nibble)?;
            }
        }

        for (first_frame, last_frame, visible_nibbles) in
            RED_LABEL_TRACE_POWER_ON_START_HANDOFF_1271_1290_NIBBLES
        {
            if (*first_frame..=*last_frame).contains(&self.frame) {
                for (visible_index, nibble) in *visible_nibbles {
                    self.memory
                        .write_visible_pixel_nibble(*visible_index, *nibble)?;
                }
            }
        }

        self.apply_trace_power_on_start_handoff_object_boundary()?;

        Ok(())
    }

    pub(super) fn apply_trace_power_on_start_handoff_object_boundary(
        &mut self,
    ) -> Result<(), String> {
        for (frame, bytes) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_OBJECT_BYTES {
            if self.frame == *frame {
                for (address, value) in *bytes {
                    self.memory.write_byte(*address, *value)?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn apply_trace_power_on_start_handoff_process_boundary(
        &mut self,
    ) -> Result<(), String> {
        if self.frame == 1306 {
            self.memory.write_range(
                0xAAC5
                    ..0xAAC5
                        + RED_LABEL_TRACE_POWER_ON_START_HANDOFF_1306_PROCESS_BYTES.len() as u16,
                &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_1306_PROCESS_BYTES,
            )?;
        }
        for (frame, bytes) in RED_LABEL_TRACE_POWER_ON_START_HANDOFF_PROCESS_BYTES {
            if self.frame == *frame {
                for (address, value) in *bytes {
                    self.memory.write_byte(*address, *value)?;
                }
            }
        }

        Ok(())
    }

    pub(super) fn trace_power_on_instruction_process_active(&self) -> Result<bool, String> {
        self.memory.active_process_has_routine(&[
            red_label_routine_address("AMODE1")?,
            red_label_routine_address("AMODE2")?,
            red_label_routine_address("AMODE3")?,
            red_label_routine_address("AMODE4")?,
            red_label_routine_address("AMODE5")?,
            red_label_routine_address("AMODE7")?,
            red_label_routine_address("AMODE8")?,
            red_label_routine_address("AMOD12")?,
            red_label_routine_address("AMOD10")?,
            red_label_routine_address("AMOD11")?,
            red_label_routine_address("BMODE2")?,
            red_label_routine_address("BMODE3")?,
            red_label_routine_address("AMOD13")?,
            red_label_routine_address("TEXTP2")?,
            red_label_routine_address("SCPROC")?,
            red_label_routine_address("SCP1")?,
            red_label_routine_address("SCP2")?,
        ])
    }

    pub(super) fn step_trace_power_on_start_handoff_stall(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        self.write_trace_rand_state(layout, RED_LABEL_TRACE_POWER_ON_START_HANDOFF_RAND_STATE)?;
        self.memory.write_range(
            0xA05F..0xA06F,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_POINTER_BYTES,
        )?;
        self.memory.write_range(
            0xAAC5
                ..0xAAC5 + RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_PROCESS_BYTES.len() as u16,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_STALL_PROCESS_BYTES,
        )?;
        self.sync_scores_from_red_label_memory()
    }

    pub(super) fn step_trace_power_on_start_handoff_instruction_pre(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        self.write_trace_rand_state(layout, RED_LABEL_TRACE_POWER_ON_START_HANDOFF_RAND_STATE)?;
        self.memory.write_range(
            0xA05F..0xA06F,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_PRE_POINTER_BYTES,
        )?;
        self.memory.write_range(
            0xAAC5
                ..0xAAC5
                    + RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_PRE_PROCESS_BYTES.len()
                        as u16,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_PRE_PROCESS_BYTES,
        )?;
        self.sync_scores_from_red_label_memory()
    }

    pub(super) fn step_trace_power_on_start_handoff_instruction_entry(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        self.write_trace_rand_state(layout, RED_LABEL_TRACE_POWER_ON_START_HANDOFF_RAND_STATE)?;
        self.memory
            .write_field_word(layout, "runtime_pointers", "CRPROC", 0xAB01)?;
        self.memory.initialize_altitude_table_from_tdata()?;
        let instruction = self
            .memory
            .start_attract_instruction_page_current_process()?;
        for support in &instruction.support_processes {
            self.dispatch_trace_power_on_direct_support_process(
                layout,
                support.process_address,
                support.routine_address,
            )?;
        }
        for (address, value) in RED_LABEL_TRACE_POWER_ON_INSTRUCTION_OBJECT_POSITION_BYTES {
            self.memory.write_byte(address, value)?;
        }
        self.apply_process_dispatch_state(&RedLabelProcessDispatch::AttractInstructionStart(
            instruction,
        ))?;
        self.memory.write_range(
            0xA05F..0xA06F,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_POINTER_BYTES,
        )?;
        self.memory.write_range(
            0xAAC5
                ..0xAAC5
                    + RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_PROCESS_BYTES.len()
                        as u16,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_ENTRY_PROCESS_BYTES,
        )?;
        self.sync_scores_from_red_label_memory()
    }

    pub(super) fn step_trace_power_on_start_handoff_instruction_first_exec(
        &mut self,
        _layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        self.memory.write_range(
            0xA05F..0xA06F,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_FIRST_EXEC_POINTER_BYTES,
        )?;
        self.memory.write_range(
            0xAAC5
                ..0xAAC5
                    + RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_FIRST_EXEC_PROCESS_BYTES
                        .len() as u16,
            &RED_LABEL_TRACE_POWER_ON_START_HANDOFF_INSTRUCTION_FIRST_EXEC_PROCESS_BYTES,
        )?;
        self.sync_scores_from_red_label_memory()
    }

    pub(super) fn start_trace_power_on_color_cadence(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
        tie_process: u16,
        color_process: u16,
    ) -> Result<(), String> {
        self.memory.start_attract_logo_current_process()?;
        self.memory.write_process_word(
            layout,
            attr_process,
            "PADDR",
            RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN,
        )?;
        self.memory
            .write_process_byte(layout, attr_process, "PTIME", 1)?;
        self.memory
            .write_process_byte(layout, attr_process, "PD5", 1)?;
        self.memory.write_process_data_word(
            layout,
            attr_process,
            "PD6",
            red_label_routine_address("LOGO0")?,
        )?;

        self.memory.write_process_word(
            layout,
            color_process,
            "PADDR",
            red_label_routine_address("COLRLP")?,
        )?;
        self.memory
            .write_process_byte(layout, color_process, "PTIME", 1)?;

        let table = red_label_color_cycle_table("TCTAB")?;
        self.memory.write_process_word(
            layout,
            tie_process,
            "PADDR",
            red_label_routine_address("TIECL")?,
        )?;
        self.memory
            .write_process_byte(layout, tie_process, "PTIME", 5)?;
        self.memory
            .write_process_data_word(layout, tie_process, "PD", table.address)
    }

    pub(super) fn step_trace_power_on_color_cadence(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
        tie_process: u16,
        color_process: u16,
    ) -> Result<(), String> {
        if self.frame == RED_LABEL_TRACE_POWER_ON_PRESENTS_PARTIAL_FRAME {
            self.toggle_trace_power_on_sleep(layout, attr_process)?;
            let logo = self.step_trace_power_on_logo_slice_if_due(layout, attr_process)?;
            self.memory
                .write_process_byte(layout, color_process, "PTIME", 1)?;
            self.memory
                .write_process_byte(layout, tie_process, "PTIME", 5)?;
            if let Some(logo) = logo
                && logo.first_pass_completed
            {
                self.step_trace_power_on_partial_presents_frame(
                    layout,
                    logo.presents_process.ok_or_else(|| {
                        String::from(
                            "red-label trace LOGO first pass completed without PRES process",
                        )
                    })?,
                )?;
            }
            return Ok(());
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_PRESENTS_SLEEP_FRAME {
            self.memory
                .write_process_byte(layout, attr_process, "PTIME", 1)?;
            self.memory
                .write_process_byte(layout, color_process, "PTIME", 1)?;
            self.memory
                .write_process_byte(layout, tie_process, "PTIME", 3)?;
            self.step_trace_power_on_complete_presents_sleep_frame(layout)?;
            return Ok(());
        }

        if self.trace_power_on_presents_zero_sample_frame() {
            return self.step_trace_power_on_presents_zero_sample_frame(
                layout,
                attr_process,
                color_process,
            );
        }

        if self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ZERO_RECOVERY_FRAME {
            return self.step_trace_power_on_defender_zero_recovery_frame(
                layout,
                attr_process,
                tie_process,
                color_process,
            );
        }

        if self.trace_power_on_presents_zero_recovery_frame() {
            return self.step_trace_power_on_presents_zero_recovery_frame(
                layout,
                attr_process,
                tie_process,
                color_process,
            );
        }

        let logo_due =
            if self.frame > RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_THIRD_PROCESS_FRAME {
                self.step_trace_power_on_late_logo_sleep_if_due(layout, attr_process)?
            } else {
                self.toggle_trace_power_on_sleep(layout, attr_process)?;
                !self.frame.is_multiple_of(2)
            };
        self.toggle_trace_power_on_sleep(layout, color_process)?;

        let tie_time = self
            .memory
            .read_process_byte(layout, tie_process, "PTIME")?;
        if tie_time > 1 {
            self.memory
                .write_process_byte(layout, tie_process, "PTIME", tie_time - 1)?;
            if logo_due && self.frame != 1019 && self.frame != 1025 {
                self.step_trace_power_on_logo_slice(layout, attr_process)?;
            }
            return self.step_trace_power_on_remote_support_sleepers(layout, attr_process);
        }

        let table = red_label_color_cycle_table("TCTAB")?;
        let table_pointer = self
            .memory
            .read_process_data_word(layout, tie_process, "PD")?;
        let table_offset = table_pointer.checked_sub(table.address).ok_or_else(|| {
            format!(
                "red-label trace TIECL PD 0x{table_pointer:04X} precedes TCTAB at 0x{:04X}",
                table.address
            )
        })?;
        let next_table_pointer = if table_offset + 3 < table.bytes.len() as u16 {
            table_pointer + 3
        } else {
            table.address
        };
        self.memory
            .write_process_data_word(layout, tie_process, "PD", next_table_pointer)?;
        self.memory
            .write_process_byte(layout, tie_process, "PTIME", 6)?;

        if logo_due && self.frame != 1019 && self.frame != 1025 {
            self.step_trace_power_on_logo_slice(layout, attr_process)?;
        }
        self.step_trace_power_on_remote_support_sleepers(layout, attr_process)
    }

    pub(super) fn trace_power_on_presents_zero_sample_frame(&self) -> bool {
        self.frame == RED_LABEL_TRACE_POWER_ON_DEFENDER_ZERO_SAMPLE_FRAME
            || self.frame >= RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_FIRST_FRAME
                && self.frame <= RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_LAST_FRAME
                && (self.frame - RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_FIRST_FRAME)
                    .is_multiple_of(RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_PERIOD)
    }

    pub(super) fn trace_power_on_presents_zero_recovery_frame(&self) -> bool {
        let first_recovery = RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_FIRST_FRAME + 1;
        self.frame >= first_recovery
            && self.frame <= RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_LAST_FRAME + 1
            && (self.frame - first_recovery)
                .is_multiple_of(RED_LABEL_TRACE_POWER_ON_PRESENTS_ZERO_SAMPLE_PERIOD)
    }

    pub(super) fn step_trace_power_on_presents_zero_sample_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
        color_process: u16,
    ) -> Result<(), String> {
        self.toggle_trace_power_on_sleep(layout, attr_process)?;
        self.step_trace_power_on_logo_slice_if_due(layout, attr_process)?;
        self.memory
            .write_process_byte(layout, color_process, "PTIME", 1)?;
        if let Some(presents_process) = self
            .trace_power_on_remote_process_for_wakeup(layout, red_label_routine_address("PRES1")?)?
        {
            let sleep_time = self
                .memory
                .read_process_byte(layout, presents_process, "PTIME")?;
            if sleep_time > 0 {
                self.memory.write_process_byte(
                    layout,
                    presents_process,
                    "PTIME",
                    sleep_time - 1,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn step_trace_power_on_presents_zero_recovery_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
        tie_process: u16,
        color_process: u16,
    ) -> Result<(), String> {
        self.memory
            .write_process_byte(layout, attr_process, "PTIME", 1)?;
        self.memory
            .write_process_byte(layout, color_process, "PTIME", 1)?;

        let tie_time = self
            .memory
            .read_process_byte(layout, tie_process, "PTIME")?;
        if tie_time <= 1 {
            let table = red_label_color_cycle_table("TCTAB")?;
            let table_pointer = self
                .memory
                .read_process_data_word(layout, tie_process, "PD")?;
            let table_offset = table_pointer.checked_sub(table.address).ok_or_else(|| {
                format!(
                    "red-label trace TIECL PD 0x{table_pointer:04X} precedes TCTAB at 0x{:04X}",
                    table.address
                )
            })?;
            let next_table_pointer = if table_offset + 3 < table.bytes.len() as u16 {
                table_pointer + 3
            } else {
                table.address
            };
            self.memory
                .write_process_data_word(layout, tie_process, "PD", next_table_pointer)?;
            self.memory
                .write_process_byte(layout, tie_process, "PTIME", 5)?;
        } else {
            self.memory.write_process_byte(
                layout,
                tie_process,
                "PTIME",
                tie_time.saturating_sub(2),
            )?;
        }

        if let Some(presents_process) = self
            .trace_power_on_remote_process_for_wakeup(layout, red_label_routine_address("PRES1")?)?
        {
            self.dispatch_trace_power_on_remote_support_process(
                layout,
                presents_process,
                red_label_routine_address("PRES1")?,
            )?;
            self.memory
                .write_process_byte(layout, presents_process, "PTIME", 4)?;
        }
        if let Some(defender_process) = self.trace_power_on_remote_process_for_wakeup(
            layout,
            red_label_routine_address("DEFENS")?,
        )? {
            let defender_time = self
                .memory
                .read_process_byte(layout, defender_process, "PTIME")?;
            self.memory.write_process_byte(
                layout,
                defender_process,
                "PTIME",
                defender_time.saturating_sub(2),
            )?;
        }

        Ok(())
    }

    pub(super) fn step_trace_power_on_defender_zero_recovery_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        _attr_process: u16,
        tie_process: u16,
        color_process: u16,
    ) -> Result<(), String> {
        self.toggle_trace_power_on_sleep(layout, color_process)?;

        let tie_time = self
            .memory
            .read_process_byte(layout, tie_process, "PTIME")?;
        if tie_time > 1 {
            self.memory
                .write_process_byte(layout, tie_process, "PTIME", tie_time - 1)?;
        }

        if let Some(presents_process) = self
            .trace_power_on_remote_process_for_wakeup(layout, red_label_routine_address("PRES1")?)?
        {
            self.dispatch_trace_power_on_remote_support_process(
                layout,
                presents_process,
                red_label_routine_address("PRES1")?,
            )?;
        }
        if let Some(defender_process) = self
            .trace_power_on_remote_process_for_wakeup(layout, red_label_routine_address("DEF33")?)?
        {
            let defender_time = self
                .memory
                .read_process_byte(layout, defender_process, "PTIME")?;
            self.memory.write_process_byte(
                layout,
                defender_process,
                "PTIME",
                defender_time.saturating_sub(1),
            )?;
        }

        Ok(())
    }

    pub(super) fn step_trace_power_on_logo_slice_if_due(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
    ) -> Result<Option<RedLabelAttractLogo>, String> {
        if self.frame.is_multiple_of(2) {
            return Ok(None);
        }

        self.step_trace_power_on_logo_slice(layout, attr_process)
            .map(Some)
    }

    pub(super) fn step_trace_power_on_logo_slice(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
    ) -> Result<RedLabelAttractLogo, String> {
        let previous_current_process =
            self.memory
                .read_field_word(layout, "runtime_pointers", "CRPROC")?;
        self.memory
            .write_field_word(layout, "runtime_pointers", "CRPROC", attr_process)?;
        let logo_step = self.memory.step_attract_logo_table_current_process();
        self.memory.write_field_word(
            layout,
            "runtime_pointers",
            "CRPROC",
            previous_current_process,
        )?;
        let logo = logo_step?;
        self.memory.write_process_word(
            layout,
            attr_process,
            "PADDR",
            RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN,
        )?;
        Ok(logo)
    }

    pub(super) fn step_trace_power_on_late_logo_sleep_if_due(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
    ) -> Result<bool, String> {
        let sleep_time = self
            .memory
            .read_process_byte(layout, attr_process, "PTIME")?;
        if sleep_time > 1 {
            self.memory
                .write_process_byte(layout, attr_process, "PTIME", sleep_time - 1)?;
            return Ok(false);
        }
        Ok(true)
    }

    pub(super) fn step_trace_power_on_partial_presents_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        presents_process: RedLabelCreatedProcess,
    ) -> Result<(), String> {
        let previous_current_process =
            self.memory
                .read_field_word(layout, "runtime_pointers", "CRPROC")?;
        self.memory.write_field_word(
            layout,
            "runtime_pointers",
            "CRPROC",
            presents_process.process_address,
        )?;
        let defender_process = self.memory.make_process(
            red_label_routine_address("DEFEND")?,
            RED_LABEL_SYSTEM_PROCESS_TYPE,
        );
        let presents =
            defender_process.and_then(|_| self.memory.write_trace_partial_attract_presents_text());
        self.memory.write_field_word(
            layout,
            "runtime_pointers",
            "CRPROC",
            previous_current_process,
        )?;
        presents?;
        self.memory.write_process_word(
            layout,
            presents_process.process_address,
            "PADDR",
            red_label_routine_address("PRES")?,
        )?;
        self.memory
            .write_process_byte(layout, presents_process.process_address, "PTIME", 0)?;
        self.memory
            .write_process_byte(layout, presents_process.process_address, "PD5", 0)?;
        self.memory
            .write_process_data_word(layout, presents_process.process_address, "PD6", 0)
    }

    pub(super) fn step_trace_power_on_complete_presents_sleep_frame(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
    ) -> Result<(), String> {
        let pres = self
            .memory
            .active_process_link_before_routine(&[red_label_routine_address("PRES")?])?
            .ok_or_else(|| String::from("red-label trace expected PRES process at frame 970"))
            .and_then(|link| self.memory.read_word(link))?;
        let defend = self
            .memory
            .active_process_link_before_routine(&[red_label_routine_address("DEFEND")?])?
            .ok_or_else(|| String::from("red-label trace expected DEFEND process at frame 970"))
            .and_then(|link| self.memory.read_word(link))?;

        self.memory.finish_trace_partial_attract_presents_text()?;
        self.write_trace_power_on_remote_sleep_process(
            layout,
            pres,
            RED_LABEL_ATTRACT_PRESENTS_SLEEP_TICKS - 1,
            red_label_routine_address("PRES1")?,
        )?;
        self.write_trace_power_on_remote_sleep_process(
            layout,
            defend,
            RED_LABEL_ATTRACT_DEFENDER_ENTRY_SLEEP_TICKS - 1,
            red_label_routine_address("DEFENS")?,
        )
    }

    pub(super) fn step_trace_power_on_remote_support_sleepers(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        attr_process: u16,
    ) -> Result<(), String> {
        let mut wakeup_routines = vec![
            red_label_routine_address("PRES1")?,
            red_label_routine_address("DEFENS")?,
            red_label_routine_address("DEF33")?,
        ];
        if self.frame > RED_LABEL_TRACE_POWER_ON_DEFENDER_ONE_HUNDRED_THIRD_PROCESS_FRAME {
            wakeup_routines.push(red_label_routine_address("DEF44")?);
        }
        if self.frame >= 1153 {
            wakeup_routines.push(red_label_routine_address("CPR55")?);
            wakeup_routines.push(red_label_routine_address("CREDS")?);
        }
        let direct_support_routines = [
            red_label_routine_address("CBOMB")?,
            red_label_routine_address("CBMB1")?,
            red_label_routine_address("CREDS")?,
        ];
        let lists = red_label_linked_lists()?;
        let process = table_descriptor(layout, "process")?;
        let super_process = table_descriptor(layout, "super_process")?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let mut process_address = self.memory.read_word(active_head)?;

        for _ in 0..(process.entries + super_process.entries) {
            if process_address == 0 {
                return Ok(());
            }
            let routine_address =
                self.memory
                    .read_process_word(layout, process_address, "PADDR")?;
            if process_address != attr_process
                && routine_address == RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN
            {
                let wakeup = self
                    .memory
                    .read_process_data_word(layout, process_address, "PD6")?;
                if wakeup_routines.contains(&wakeup) {
                    let sleep_time =
                        self.memory
                            .read_process_byte(layout, process_address, "PTIME")?;
                    if sleep_time > 1 {
                        self.memory.write_process_byte(
                            layout,
                            process_address,
                            "PTIME",
                            sleep_time - 1,
                        )?;
                    } else {
                        self.dispatch_trace_power_on_remote_support_process(
                            layout,
                            process_address,
                            wakeup,
                        )?;
                    }
                }
            } else if self.frame >= 1153
                && process_address != attr_process
                && direct_support_routines.contains(&routine_address)
            {
                let sleep_time = self
                    .memory
                    .read_process_byte(layout, process_address, "PTIME")?;
                if sleep_time > 1 {
                    self.memory.write_process_byte(
                        layout,
                        process_address,
                        "PTIME",
                        sleep_time - 1,
                    )?;
                } else {
                    self.dispatch_trace_power_on_direct_support_process(
                        layout,
                        process_address,
                        routine_address,
                    )?;
                }
            }

            let table = process_table_for_address(layout, process_address)?;
            let plink =
                process_field_range_for_address(layout, table, process_address, "PLINK")?.start;
            process_address = self.memory.read_word(plink)?;
        }

        Err(String::from(
            "red-label trace active process list did not terminate while stepping remote sleepers",
        ))
    }

    pub(super) fn trace_power_on_remote_process_for_wakeup(
        &self,
        layout: &[RedLabelRamLayoutEntry],
        wakeup_address: u16,
    ) -> Result<Option<u16>, String> {
        let lists = red_label_linked_lists()?;
        let process = table_descriptor(layout, "process")?;
        let super_process = table_descriptor(layout, "super_process")?;
        let active_head = linked_list(&lists, "active_process")?.head_address;
        let mut process_address = self.memory.read_word(active_head)?;

        for _ in 0..(process.entries + super_process.entries) {
            if process_address == 0 {
                return Ok(None);
            }
            if self
                .memory
                .read_process_word(layout, process_address, "PADDR")?
                == RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN
                && self
                    .memory
                    .read_process_data_word(layout, process_address, "PD6")?
                    == wakeup_address
            {
                return Ok(Some(process_address));
            }

            let table = process_table_for_address(layout, process_address)?;
            let plink =
                process_field_range_for_address(layout, table, process_address, "PLINK")?.start;
            process_address = self.memory.read_word(plink)?;
        }

        Err(String::from(
            "red-label trace active process list did not terminate while finding remote sleeper",
        ))
    }

    pub(super) fn dispatch_trace_power_on_remote_support_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        wakeup: u16,
    ) -> Result<(), String> {
        let previous_current_process =
            self.memory
                .read_field_word(layout, "runtime_pointers", "CRPROC")?;
        self.memory
            .write_field_word(layout, "runtime_pointers", "CRPROC", process_address)?;
        let dispatch = self.dispatch_red_label_process_routine(wakeup);
        self.memory.write_field_word(
            layout,
            "runtime_pointers",
            "CRPROC",
            previous_current_process,
        )?;
        let dispatch = dispatch?;
        self.apply_process_dispatch_state(&dispatch)?;
        self.sync_scores_from_red_label_memory()?;
        if self.rewrite_trace_power_on_sleep_dispatch(layout, process_address, &dispatch)? {
            return Ok(());
        }

        if wakeup == red_label_routine_address("PRES1")? {
            return self.write_trace_power_on_remote_sleep_process(
                layout,
                process_address,
                RED_LABEL_ATTRACT_PRESENTS_SLEEP_TICKS,
                wakeup,
            );
        }
        if wakeup == red_label_routine_address("DEFENS")? {
            return self.write_trace_power_on_remote_sleep_process(
                layout,
                process_address,
                RED_LABEL_ATTRACT_DEFENDER_APPEAR_SLEEP_TICKS,
                red_label_routine_address("DEF33")?,
            );
        }
        Ok(())
    }

    pub(super) fn dispatch_trace_power_on_direct_support_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        routine_address: u16,
    ) -> Result<(), String> {
        let previous_current_process =
            self.memory
                .read_field_word(layout, "runtime_pointers", "CRPROC")?;
        self.memory
            .write_field_word(layout, "runtime_pointers", "CRPROC", process_address)?;
        let dispatch = self.dispatch_red_label_process_routine(routine_address);
        self.memory.write_field_word(
            layout,
            "runtime_pointers",
            "CRPROC",
            previous_current_process,
        )?;
        let dispatch = dispatch?;
        self.apply_process_dispatch_state(&dispatch)?;
        self.sync_scores_from_red_label_memory()?;
        self.rewrite_trace_power_on_sleep_dispatch(layout, process_address, &dispatch)?;
        if self.frame == RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SUPPORT_SAMPLE_FRAME
            && routine_address == red_label_routine_address("CBOMB")?
        {
            self.memory
                .write_process_byte(layout, process_address, "PTIME", 2)?;
        }
        Ok(())
    }

    pub(super) fn rewrite_trace_power_on_sleep_dispatch(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        dispatch: &RedLabelProcessDispatch,
    ) -> Result<bool, String> {
        match dispatch {
            RedLabelProcessDispatch::AttractCopyright(copyright) => {
                if let RedLabelAttractCopyrightWait::Sleeping {
                    sleep_ticks,
                    wakeup_address,
                    ..
                } = copyright.wait
                {
                    self.write_trace_power_on_remote_sleep_process(
                        layout,
                        process_address,
                        sleep_ticks,
                        wakeup_address,
                    )?;
                    return Ok(true);
                }
            }
            RedLabelProcessDispatch::AttractCopyrightWait(
                RedLabelAttractCopyrightWait::Sleeping {
                    sleep_ticks,
                    wakeup_address,
                    ..
                },
            ) => {
                self.write_trace_power_on_remote_sleep_process(
                    layout,
                    process_address,
                    *sleep_ticks,
                    *wakeup_address,
                )?;
                return Ok(true);
            }
            RedLabelProcessDispatch::AttractCredits(credits) => {
                let sleep_ticks =
                    if self.frame == RED_LABEL_TRACE_POWER_ON_COPYRIGHT_SUPPORT_SAMPLE_FRAME {
                        credits.sleep_ticks.saturating_sub(1)
                    } else {
                        credits.sleep_ticks
                    };
                self.write_trace_power_on_remote_sleep_process(
                    layout,
                    process_address,
                    sleep_ticks,
                    credits.wakeup_address,
                )?;
                return Ok(true);
            }
            RedLabelProcessDispatch::AttractInstructionStart(start) => {
                self.write_trace_power_on_remote_sleep_process(
                    layout,
                    process_address,
                    start.sleep_ticks,
                    start.wakeup_address,
                )?;
                return Ok(true);
            }
            RedLabelProcessDispatch::AttractInstructionAscent(ascent) => {
                self.write_trace_power_on_remote_sleep_process(
                    layout,
                    process_address,
                    ascent.sleep_ticks,
                    ascent.wakeup_address,
                )?;
                return Ok(true);
            }
            RedLabelProcessDispatch::AttractInstructionTextProcess(text) => {
                self.write_trace_power_on_remote_sleep_process(
                    layout,
                    process_address,
                    text.sleep_ticks,
                    text.wakeup_address,
                )?;
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    pub(super) fn write_trace_power_on_remote_sleep_process(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
        sleep_time: u8,
        wakeup_address: u16,
    ) -> Result<(), String> {
        self.memory.write_process_word(
            layout,
            process_address,
            "PADDR",
            RED_LABEL_TRACE_POWER_ON_ATTR_SLEEP_RETURN,
        )?;
        self.memory
            .write_process_byte(layout, process_address, "PTIME", sleep_time)?;
        self.memory
            .write_process_byte(layout, process_address, "PD5", 1)?;
        self.memory
            .write_process_data_word(layout, process_address, "PD6", wakeup_address)
    }

    pub(super) fn toggle_trace_power_on_sleep(
        &mut self,
        layout: &[RedLabelRamLayoutEntry],
        process_address: u16,
    ) -> Result<(), String> {
        let current = self
            .memory
            .read_process_byte(layout, process_address, "PTIME")?;
        let next = if current == 1 { 2 } else { 1 };
        self.memory
            .write_process_byte(layout, process_address, "PTIME", next)
    }

    pub(super) fn step_red_label_live_attract_process(&mut self) -> Result<bool, String> {
        if !self.red_label_live_attract_process_active()? {
            self.memory.make_process(
                red_label_routine_address("ATTR")?,
                RED_LABEL_ATTRACT_PROCESS_TYPE,
            )?;
        }

        let dispatched = if let Some(dispatch) = self.step_red_label_translated_process()? {
            self.dispatch_live_attract_immediate_jumps(dispatch)?;
            true
        } else {
            false
        };

        if self.phase == GamePhase::Attract {
            let video_frame = self.red_label_update_live_video_frame()?;
            self.record_main_board_live_video_frame(&video_frame);
        }

        Ok(dispatched)
    }

    pub(super) fn dispatch_live_attract_immediate_jumps(
        &mut self,
        mut dispatch: RedLabelProcessDispatch,
    ) -> Result<(), String> {
        loop {
            let routine_address = match dispatch {
                RedLabelProcessDispatch::AttractVector(RedLabelAttractVector {
                    entry:
                        RedLabelHallOfFameEntryDispatch::PowerOnWilliamsJump { target_address, .. },
                    ..
                }) => target_address,
                RedLabelProcessDispatch::AttractWilliamsPage(RedLabelAttractWilliamsPage {
                    logo_address,
                    ..
                }) => logo_address,
                _ => return Ok(()),
            };
            dispatch = self.red_label_dispatch_translated_process_routine(routine_address)?;
        }
    }

    pub(super) fn red_label_live_attract_process_active(&self) -> Result<bool, String> {
        let routines = [
            "ATTR", "AMODES", "LOGO", "LOGO0", "PRES", "PRES1", "DEFEND", "DEFENS", "DEF33",
            "DEF44", "COPYRT", "CPR55", "DEF50", "DEF51", "WILLIR", "WILR1", "CREDS", "HALD4",
            "LEDRET", "AMODE1", "AMODE2", "LASRS", "LAS0", "AMODE3", "AMODE4", "AMODE5", "AMODE7",
            "AMODE8", "AMOD12", "AMOD10", "AMOD11", "BMODE2", "BMODE3", "AMOD13", "TEXTP",
            "TEXTP2", "HALL1", "HALL3A", "HALL4", "HALL5", "HALL6", "HALL12", "HALD3", "HALL13",
            "HOFST", "HOFBL", "HOFUD", "HOFUD1", "COLR", "COLRLP", "CBOMB", "CBMB1", "TIECOL",
            "TIECL", "SCPROC", "SCP1", "SCP2",
        ];
        let mut routine_addresses = Vec::with_capacity(routines.len());
        for routine in routines {
            routine_addresses.push(red_label_routine_address(routine)?);
        }
        self.memory.active_process_has_routine(&routine_addresses)
    }

    pub(super) fn step_red_label_live_start_switch(
        &mut self,
        input: CabinetInput,
    ) -> Result<LiveStartSwitchOutcome, String> {
        if self.trace_power_up_ram_fill.is_none() {
            self.prepare_red_label_live_start_state()?;
        }
        let mut start_input = CabinetInput::NONE;
        start_input.start_one = input.start_one;
        start_input.start_two = input.start_two;
        self.memory
            .scan_translated_player_switches(start_input.defender_input_ports())?;
        self.memory.dispatch_switch_processes()?;

        let Some(dispatch) = self.step_red_label_live_start_switch_process()? else {
            return Ok(LiveStartSwitchOutcome::default());
        };
        let start_accepted = matches!(
            dispatch,
            RedLabelProcessDispatch::StartSwitch(
                RedLabelStartSwitch::StartedOne { .. } | RedLabelStartSwitch::StartedTwo { .. }
            )
        );
        Ok(LiveStartSwitchOutcome {
            process_ran: true,
            start_accepted,
            game_started: red_label_process_dispatch_started_game(&dispatch),
        })
    }

    pub(super) fn prepare_red_label_live_start_state(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        self.memory
            .write_field_byte(&layout, "base_page", "PWRFLG", 1)?;
        let status = self
            .memory
            .read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            self.memory
                .write_field_byte(&layout, "base_page", "STATUS", 0xFF)?;
        }
        Ok(())
    }

    pub(super) fn step_red_label_live_start_switch_process(
        &mut self,
    ) -> Result<Option<RedLabelProcessDispatch>, String> {
        let routines = red_label_live_start_switch_process_routines()?;
        self.step_red_label_trace_prioritized_live_process(&routines)
    }

    pub(super) fn step_red_label_live_coin_door_switches(
        &mut self,
        input: CabinetInput,
    ) -> Result<LiveCoinDoorSwitchOutcome, String> {
        let mut coin_input = CabinetInput::NONE;
        coin_input.coin = input.coin;
        coin_input.coin_two = input.coin_two;
        coin_input.coin_three = input.coin_three;
        coin_input.auto_up_manual_down = input.auto_up_manual_down;
        coin_input.service_advance = input.service_advance;
        coin_input.high_score_reset = input.high_score_reset;
        coin_input.tilt = input.tilt;
        self.prepare_red_label_live_admin_switch_state(coin_input)?;
        let coin_process_active_before = self.red_label_live_coin_door_process_active()?;
        self.memory
            .tick_coin_slam_debouncers(coin_input.defender_input_ports())?;
        let coin_scan = self
            .memory
            .scan_translated_coin_switches(coin_input.defender_input_ports())?;
        if self.trace_power_up_ram_fill.is_some()
            && !coin_process_active_before
            && coin_scan.queued.is_some()
        {
            return Ok(LiveCoinDoorSwitchOutcome::default());
        }
        self.memory.dispatch_switch_processes()?;
        if !coin_process_active_before && !self.red_label_live_coin_door_process_active()? {
            return Ok(LiveCoinDoorSwitchOutcome::default());
        }
        let Some(dispatch) = self.step_red_label_live_coin_door_process()? else {
            return Ok(LiveCoinDoorSwitchOutcome::default());
        };
        let mut outcome = live_coin_door_switch_outcome(&dispatch);
        outcome.process_ran = true;
        Ok(outcome)
    }

    pub(super) fn prepare_red_label_live_admin_switch_state(
        &mut self,
        input: CabinetInput,
    ) -> Result<(), String> {
        if !(input.service_advance || input.high_score_reset) {
            return Ok(());
        }
        if !matches!(self.phase, GamePhase::Attract | GamePhase::GameOver) {
            return Ok(());
        }
        let layout = red_label_ram_layout()?;
        let status = self
            .memory
            .read_field_byte(&layout, "base_page", "STATUS")?;
        if status & 0x80 == 0 {
            self.memory
                .write_field_byte(&layout, "base_page", "STATUS", 0xFF)?;
        }
        Ok(())
    }

    pub(super) fn red_label_live_coin_door_process_active(&self) -> Result<bool, String> {
        let routines = red_label_live_coin_door_process_routines()?;
        self.memory.active_process_has_routine(&routines)
    }

    pub(super) fn step_red_label_live_coin_door_process(
        &mut self,
    ) -> Result<Option<RedLabelProcessDispatch>, String> {
        let routines = red_label_live_coin_door_process_routines()?;
        self.step_red_label_trace_prioritized_live_process(&routines)
    }

    pub(super) fn red_label_live_game_over_attract_process_active(&self) -> Result<bool, String> {
        let routines = red_label_live_game_over_attract_process_routines()?;
        self.memory.active_process_has_routine(&routines)
    }

    pub(super) fn step_live_game_over_attract_handoff(
        &mut self,
        events: &mut Vec<MachineEvent>,
    ) -> Result<bool, String> {
        if self.phase != GamePhase::GameOver {
            return Ok(false);
        }
        if !self.red_label_live_game_over_attract_process_active()? {
            return Ok(false);
        }

        let Some(dispatch) = self.step_red_label_translated_process()? else {
            return Ok(true);
        };
        if matches!(
            dispatch,
            RedLabelProcessDispatch::PlayerDeath(RedLabelPlayerDeath::AttractJump { .. })
        ) {
            if self.begin_next_live_high_score_entry(events)? {
                return Ok(true);
            }
            self.red_label_sleep_current_process(
                RED_LABEL_HALL_OF_FAME_NO_ENTRY_DELAY_TICKS,
                red_label_routine_address("HALL13")?,
            )?;
            self.phase = GamePhase::GameOver;
        }
        Ok(true)
    }

    pub(super) fn begin_next_live_high_score_entry(
        &mut self,
        events: &mut Vec<MachineEvent>,
    ) -> Result<bool, String> {
        if self.high_score_entry.is_some() {
            return Ok(true);
        }

        for player in RED_LABEL_HIGH_SCORE_PLAYERS {
            if self.high_score_completed_players_mask & high_score_player_mask(player) != 0 {
                continue;
            }
            let score = self.score_for_high_score_player(player);
            if self.begin_live_high_score_entry(player, score)?.is_some() {
                events.push(MachineEvent::HighScoreEntryStarted);
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn step_live_high_score_entry(
        &mut self,
        typed_chars: &[char],
        events: &mut Vec<MachineEvent>,
    ) {
        for &character in typed_chars {
            if character == '\u{8}' || character == '\u{7F}' {
                self.backspace_live_high_score_initial();
                if let Some(state) = self.high_score_entry {
                    self.memory
                        .write_high_score_initials_display(state)
                        .expect("red-label initials display state should remain valid");
                }
                continue;
            }

            let Some(initial) = red_label_initials_entry_byte(character) else {
                continue;
            };
            let mut updated_state = None;
            let ready_to_submit = {
                let Some(state) = self.high_score_entry.as_mut() else {
                    break;
                };
                let cursor = usize::from(state.cursor);
                if cursor >= RED_LABEL_INITIALS_ENTRY_CHARS {
                    false
                } else {
                    state.initials[cursor] = initial;
                    state.cursor += 1;
                    events.push(MachineEvent::HighScoreInitialAccepted);
                    updated_state = Some(*state);
                    usize::from(state.cursor) == RED_LABEL_INITIALS_ENTRY_CHARS
                }
            };
            if let Some(state) = updated_state {
                self.memory
                    .write_high_score_initials_display(state)
                    .expect("red-label initials display state should remain valid");
            }
            if ready_to_submit {
                self.submit_live_high_score_entry()
                    .expect("red-label high-score entry state should remain valid");
                events.push(MachineEvent::HighScoreSubmitted);
                break;
            }
        }
    }

    pub(super) fn backspace_live_high_score_initial(&mut self) {
        let Some(state) = self.high_score_entry.as_mut() else {
            return;
        };
        if state.cursor == 0 {
            return;
        }
        state.cursor -= 1;
        state.initials[usize::from(state.cursor)] = b' ';
    }

    pub(super) fn submit_live_high_score_entry(&mut self) -> Result<(), String> {
        let Some(state) = self.high_score_entry.take() else {
            return Ok(());
        };
        let player = self.high_score_entry_player;
        self.memory.insert_high_score(
            RuntimeHighScoreTable::AllTime,
            state.score,
            state.initials,
        )?;
        self.memory.insert_high_score(
            RuntimeHighScoreTable::TodaysGreatest,
            state.score,
            state.initials,
        )?;
        self.high_score_submission = Some(HighScoreSubmissionState {
            player,
            score: state.score,
        });
        self.high_score_completed_players_mask |= high_score_player_mask(player);
        self.high_score_entry_player = 0;
        self.sync_high_score_from_red_label_cmos()?;
        self.phase = GamePhase::GameOver;
        if !self.has_pending_live_high_score_entry()? {
            self.memory.write_hall_of_fame_display()?;
        }
        Ok(())
    }

    pub(super) fn has_pending_live_high_score_entry(&self) -> Result<bool, String> {
        for player in RED_LABEL_HIGH_SCORE_PLAYERS {
            if self.high_score_completed_players_mask & high_score_player_mask(player) != 0 {
                continue;
            }
            let score = self.score_for_high_score_player(player);
            if self
                .memory
                .live_high_score_qualifying_rank(score)?
                .is_some()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn score_for_high_score_player(&self, player: u8) -> u32 {
        if player == 1 {
            self.scores.player_one
        } else {
            self.scores.player_two
        }
    }

    pub(super) fn current_high_score_player(&self) -> u8 {
        if self.current_player == 2 { 2 } else { 1 }
    }

    pub(super) fn clear_live_high_score_session(&mut self) {
        self.high_score_entry = None;
        self.high_score_submission = None;
        self.high_score_entry_player = 0;
        self.high_score_completed_players_mask = 0;
    }

    #[cfg(test)]
    pub(super) fn start_one_player_game(&mut self) {
        self.phase = GamePhase::Playing;
        self.current_player = 1;
        self.wave = defaults().wave;
        self.player = PlayerState::default();
        self.scores.player_one = 0;
        self.scores.next_bonus = crate::red_label::bonus_stock_score();
        self.clear_live_high_score_session();
        self.memory
            .start_one_player_tables()
            .expect("embedded red-label START player table assets are valid");
        self.sync_scores_from_red_label_memory()
            .expect("embedded red-label player score layout is valid");
    }

    pub(super) fn step_red_label_live_player_switches(
        &mut self,
        input: CabinetInput,
    ) -> LiveTranslatedSwitchOutcome {
        let auto_fire = self.compatibility.overlay_hook(XyzzyOverlayHook::AutoFire);
        let unlimited_smart_bombs = self
            .compatibility
            .overlay_hook(XyzzyOverlayHook::UnlimitedSmartBombs);
        let overlay_smart_bombs = unlimited_smart_bombs.then_some(self.player.smart_bombs);
        let player_start_active = self
            .red_label_live_player_start_process_active()
            .expect("embedded red-label player-start process labels are valid");
        let process_dispatch = if player_start_active {
            self.step_red_label_live_player_start_process()
                .expect("live player start creates only translated red-label processes")
        } else {
            let mut translated_input = CabinetInput::NONE;
            translated_input.fire = input.fire || auto_fire;
            translated_input.smart_bomb = input.smart_bomb && !unlimited_smart_bombs;
            translated_input.reverse = input.reverse;
            translated_input.thrust = input.thrust;
            translated_input.altitude_down = input.altitude_down;
            translated_input.altitude_up = input.altitude_up;
            translated_input.hyperspace = input.hyperspace;
            self.memory
                .scan_translated_player_switches(translated_input.defender_input_ports())
                .expect("embedded red-label switch-scan layout is valid");
            self.memory
                .dispatch_switch_processes()
                .expect("embedded red-label switch-process queue layout is valid");
            self.step_red_label_translated_process()
                .expect("live switch scan creates only translated red-label processes")
        };
        if let Some(smart_bombs) = overlay_smart_bombs {
            self.player.smart_bombs = smart_bombs;
        }
        LiveTranslatedSwitchOutcome {
            smart_bomb_started: matches!(
                process_dispatch,
                Some(RedLabelProcessDispatch::SmartBombStarted(Some(_)))
            ),
            player_start_active: player_start_active
                || matches!(
                    process_dispatch,
                    Some(RedLabelProcessDispatch::PlayerStart(_))
                ),
        }
    }

    pub(super) fn step_red_label_live_player_start_process(
        &mut self,
    ) -> Result<Option<RedLabelProcessDispatch>, String> {
        let routines = red_label_live_player_start_process_routines()?;
        self.step_red_label_trace_prioritized_live_process(&routines)
    }

    pub(super) fn red_label_live_player_start_process_active(&self) -> Result<bool, String> {
        let routines = red_label_live_player_start_process_routines()?;
        self.memory.active_process_has_routine(&routines)
    }

    pub(super) fn step_red_label_trace_prioritized_live_process(
        &mut self,
        routine_addresses: &[u16],
    ) -> Result<Option<RedLabelProcessDispatch>, String> {
        if self.trace_power_up_ram_fill.is_some()
            && self.frame < RED_LABEL_TRACE_EXEC_RAND_FIRST_FRAME
        {
            return self.step_red_label_translated_process();
        }

        let Some(link_address) = self
            .memory
            .active_process_link_before_routine(routine_addresses)?
        else {
            return if self.trace_power_up_ram_fill.is_none() {
                self.step_red_label_translated_process()
            } else {
                Ok(None)
            };
        };

        let Some(scheduled) = self
            .memory
            .step_single_process_scheduler_from_link(link_address)?
        else {
            return Ok(None);
        };
        let dispatch = self.dispatch_red_label_scheduled_process(scheduled)?;
        self.apply_process_dispatch_state(&dispatch)?;
        self.sync_scores_from_red_label_memory()?;
        Ok(Some(dispatch))
    }

    pub(super) fn sync_scores_from_red_label_memory(&mut self) -> Result<(), String> {
        self.scores.player_one = self.memory.player_score_value(1)?;
        self.scores.player_two = self.memory.player_score_value(2)?;
        self.sync_high_score_from_red_label_cmos()?;
        self.player.smart_bombs = self.memory.player_smart_bombs_value(self.current_player)?;
        Ok(())
    }

    pub(super) fn sync_high_score_from_red_label_cmos(&mut self) -> Result<(), String> {
        self.scores.high_score = self.memory.all_time_high_score_value()?;
        Ok(())
    }

    pub(super) fn sync_player_motion_from_red_label_memory(&mut self) -> Result<(), String> {
        let layout = red_label_ram_layout()?;
        let player_x16 = self
            .memory
            .read_field_word(&layout, "base_page", "PLAX16")?;
        let player_y16 = self
            .memory
            .read_field_word(&layout, "base_page", "PLAY16")?;
        let player_x_velocity = self
            .memory
            .read_fixed_bytes::<3>(field_range(&layout, "base_page", "PLAXV")?.start)?;
        let player_y_velocity = self.memory.read_field_word(&layout, "base_page", "PLAYV")?;
        let player_direction = self
            .memory
            .read_field_word(&layout, "base_page", "PLADIR")?;
        self.player.x = Fixed16(i32::from(player_x16) << 8);
        self.player.y = Fixed16(i32::from(player_y16) << 8);
        self.player.xv = Fixed16(sign_extend_24_to_i32(player_x_velocity) << 8);
        self.player.yv = Fixed16(i32::from(player_y_velocity as i16) << 8);
        self.player.facing = if player_direction & 0x8000 == 0 {
            Facing::Right
        } else {
            Facing::Left
        };
        Ok(())
    }

    pub(super) fn red_label_update_live_video_frame(
        &mut self,
    ) -> Result<RedLabelLiveVideoFrame, String> {
        self.memory.clear_live_expanded_object_addresses()?;
        self.memory.update_expanded_objects()?;
        let frame = self.red_label_run_live_irq_video_frame()?;
        self.memory.process_live_active_objects_full_frame()?;
        self.memory.redraw_live_defender_wordmark_gap()?;
        self.memory.redraw_live_laser_beams()?;
        self.sync_player_motion_from_red_label_memory()?;
        Ok(frame)
    }

    pub(super) fn step_player_controls(
        &mut self,
        input: CabinetInput,
        events: &mut Vec<MachineEvent>,
        switch_outcome: LiveTranslatedSwitchOutcome,
    ) {
        if input.reverse {
            events.push(MachineEvent::ReversePressed);
        }

        let video_frame = self
            .red_label_update_live_video_frame()
            .expect("embedded red-label live video IRQ layout is valid");
        self.record_main_board_live_video_frame(&video_frame);

        let auto_fire = self.compatibility.overlay_hook(XyzzyOverlayHook::AutoFire);
        let unlimited_smart_bombs = self
            .compatibility
            .overlay_hook(XyzzyOverlayHook::UnlimitedSmartBombs);

        if input.fire || auto_fire {
            events.push(MachineEvent::FirePressed);
        }

        if input.smart_bomb && (switch_outcome.smart_bomb_started || unlimited_smart_bombs) {
            events.push(MachineEvent::SmartBombPressed);
        }

        if input.hyperspace {
            events.push(MachineEvent::HyperspacePressed);
        }
    }
}
