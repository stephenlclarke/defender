//! Data-only machine state and frame-output contracts.

use crate::{
    board::{PALETTE_RAM_SIZE, RED_LABEL_HIGH_SCORE_ENTRIES},
    input::DefenderInputPorts,
    red_label::{Facing, Fixed16, RandState, bonus_stock_score, default_high_scores, defaults},
    red_label_wave::WaveProfile,
    sound::{FRAME_SOUND_COMMAND_CAPACITY, SoundCommand, SoundCommandLatch},
};

pub const RED_LABEL_INITIALS_ENTRY_CHARS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerState {
    pub x: Fixed16,
    pub y: Fixed16,
    pub xv: Fixed16,
    pub yv: Fixed16,
    pub facing: Facing,
    pub lives: u8,
    pub smart_bombs: u8,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            x: Fixed16(0x1800_0000),
            y: Fixed16(0x7800_0000),
            xv: Fixed16::ZERO,
            yv: Fixed16::ZERO,
            facing: Facing::Right,
            lives: defaults().lives,
            smart_bombs: defaults().smart_bombs,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerStockState {
    pub lives: u8,
    pub smart_bombs: u8,
}

impl From<PlayerState> for PlayerStockState {
    fn from(player: PlayerState) -> Self {
        Self {
            lives: player.lives,
            smart_bombs: player.smart_bombs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreState {
    pub player_one: u32,
    pub player_two: u32,
    pub high_score: u32,
    pub next_bonus: u32,
}

impl Default for ScoreState {
    fn default() -> Self {
        Self {
            player_one: 0,
            player_two: 0,
            high_score: default_high_scores()[0].score,
            next_bonus: bonus_stock_score(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreEntryState {
    pub score: u32,
    pub rank: u8,
    pub initials: [u8; RED_LABEL_INITIALS_ENTRY_CHARS],
    pub cursor: u8,
}

impl HighScoreEntryState {
    pub(crate) fn new(score: u32, rank: u8) -> Self {
        Self {
            score,
            rank,
            initials: [b' '; RED_LABEL_INITIALS_ENTRY_CHARS],
            cursor: 0,
        }
    }

    pub fn initials_text(self) -> String {
        self.initials.iter().map(|byte| char::from(*byte)).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreSubmissionState {
    pub player: u8,
    pub score: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreTableEntryState {
    pub rank: u8,
    pub score: u32,
    pub initials: [u8; RED_LABEL_INITIALS_ENTRY_CHARS],
}

impl HighScoreTableEntryState {
    pub const EMPTY: Self = Self {
        rank: 0,
        score: 0,
        initials: [b' '; RED_LABEL_INITIALS_ENTRY_CHARS],
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreTablesState {
    pub all_time: [HighScoreTableEntryState; RED_LABEL_HIGH_SCORE_ENTRIES],
    pub todays_greatest: [HighScoreTableEntryState; RED_LABEL_HIGH_SCORE_ENTRIES],
}

impl Default for HighScoreTablesState {
    fn default() -> Self {
        let mut entries = [HighScoreTableEntryState::EMPTY; RED_LABEL_HIGH_SCORE_ENTRIES];

        for (index, seed) in default_high_scores()
            .iter()
            .take(RED_LABEL_HIGH_SCORE_ENTRIES)
            .enumerate()
        {
            let initials = seed.initials.as_bytes();
            let mut entry_initials = [b' '; RED_LABEL_INITIALS_ENTRY_CHARS];
            entry_initials.copy_from_slice(initials);
            entries[index] = HighScoreTableEntryState {
                rank: u8::try_from(index + 1).expect("red-label high-score rank should fit in u8"),
                score: seed.score,
                initials: entry_initials,
            };
        }

        Self {
            all_time: entries,
            todays_greatest: entries,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameOverState {
    pub player_death_sleep_remaining: Option<u8>,
    pub player_switch_sleep_remaining: Option<u8>,
    pub player_switch_from: Option<u8>,
    pub player_switch_to: Option<u8>,
    pub no_entry_delay_remaining: Option<u8>,
    pub hall_of_fame_stall_remaining: Option<u8>,
}

pub const OBJECT_LIST_DETAIL_LIMIT: usize = crate::game::OBJECT_EVIDENCE_DETAIL_LIMIT;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ObjectListNameState {
    #[default]
    Active,
    Inactive,
    Projectile,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectListDetailState {
    pub list: ObjectListNameState,
    pub address: u16,
    pub slot: u16,
    pub screen_x: u8,
    pub screen_y: u8,
    pub world_x: u16,
    pub world_y: u16,
    pub velocity_x: u16,
    pub velocity_y: u16,
    pub picture_address: u16,
    pub picture_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub primary_image_address: Option<u16>,
    pub alternate_image_address: Option<u16>,
    pub mapped_sprite: Option<u16>,
    pub object_type: u8,
    pub scanner_color: u16,
}

impl ObjectListDetailState {
    pub const EMPTY: Self = Self {
        list: ObjectListNameState::Active,
        address: 0,
        slot: 0,
        screen_x: 0,
        screen_y: 0,
        world_x: 0,
        world_y: 0,
        velocity_x: 0,
        velocity_y: 0,
        picture_address: 0,
        picture_label: None,
        picture_size: None,
        primary_image_address: None,
        alternate_image_address: None,
        mapped_sprite: None,
        object_type: 0,
        scanner_color: 0,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectListState {
    pub active_count: u16,
    pub inactive_count: u16,
    pub projectile_count: u16,
    pub visible_count: u16,
    pub evidence_crc32: u32,
    pub detail_count: u8,
    pub details: [ObjectListDetailState; OBJECT_LIST_DETAIL_LIMIT],
}

pub const EXPANDED_OBJECT_DETAIL_LIMIT: usize = crate::game::EXPANDED_OBJECT_DETAIL_LIMIT;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpandedObjectKindState {
    #[default]
    Appearance,
    Explosion,
    ScorePopup,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectDetailState {
    pub kind: ExpandedObjectKindState,
    pub slot_address: u16,
    pub size: u16,
    pub descriptor_address: u16,
    pub picture_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub mapped_sprite: Option<u16>,
    pub erase_address: u16,
    pub center_x: u8,
    pub center_y: u8,
    pub top_left_x: u8,
    pub top_left_y: u8,
    pub object_address: Option<u16>,
    pub score_popup_lifetime_ticks: Option<u8>,
    pub score_popup_value: Option<u16>,
    pub explosion_frame: Option<u8>,
    pub explosion_lifetime_frames: Option<u8>,
}

impl ExpandedObjectDetailState {
    pub const EMPTY: Self = Self {
        kind: ExpandedObjectKindState::Appearance,
        slot_address: 0,
        size: 0,
        descriptor_address: 0,
        picture_label: None,
        picture_size: None,
        mapped_sprite: None,
        erase_address: 0,
        center_x: 0,
        center_y: 0,
        top_left_x: 0,
        top_left_y: 0,
        object_address: None,
        score_popup_lifetime_ticks: None,
        score_popup_value: None,
        explosion_frame: None,
        explosion_lifetime_frames: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectState {
    pub active_count: u16,
    pub last_slot_address: Option<u16>,
    pub detail_count: u8,
    pub details: [ExpandedObjectDetailState; EXPANDED_OBJECT_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineSnapshot {
    pub frame: u64,
    pub phase: GamePhase,
    pub credits: u8,
    pub current_player: u8,
    pub player_count: u8,
    pub wave: u8,
    pub rng: RandState,
    pub player: PlayerState,
    pub player_stocks: [PlayerStockState; 2],
    pub scores: ScoreState,
    pub last_input_bits: u16,
    pub wave_profile: WaveProfile,
    pub xyzzy_active: bool,
    pub xyzzy_invincible: bool,
    pub xyzzy_auto_fire: bool,
    pub high_score_entry: Option<HighScoreEntryState>,
    pub high_score_submission: Option<HighScoreSubmissionState>,
    pub high_score_tables: HighScoreTablesState,
    pub game_over: GameOverState,
    pub object_lists: ObjectListState,
    pub expanded_objects: ExpandedObjectState,
    pub player_explosion: Option<crate::game::PlayerExplosionCloudSnapshot>,
    pub terrain_blow: Option<crate::game::TerrainBlowSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelMainBoardSnapshot {
    pub input_ports: DefenderInputPorts,
    pub main_ram_crc32: u32,
    pub cmos_crc32: u32,
    pub palette_ram: [u8; PALETTE_RAM_SIZE],
    pub hardware_map: u8,
    pub watchdog_reset_count: u64,
    pub video_counter_vpos: u16,
    pub video_counter_value: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelSoundBoardSnapshot {
    pub last_command_latch: Option<SoundCommandLatch>,
    pub latched_port_b: Option<u8>,
    pub command_cb1_asserted: bool,
    pub latch_write_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelTraceState {
    pub player_one_score: u32,
    pub player_two_score: u32,
    pub wave: u8,
    pub lives: u8,
    pub smart_bombs: u8,
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompatibilityState {
    pub xyzzy_active: bool,
    pub xyzzy_invincible: bool,
    pub xyzzy_auto_fire: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XyzzyOverlayHook {
    AutoFire,
    UnlimitedSmartBombs,
    Invincibility,
    ShotCapOverride,
    BulletMineClear,
    SafeHyperspace,
    CollisionDeathOverride,
    FallingHumanoidSurvival,
}

impl XyzzyOverlayHook {
    pub const ALL: [Self; 8] = [
        Self::AutoFire,
        Self::UnlimitedSmartBombs,
        Self::Invincibility,
        Self::ShotCapOverride,
        Self::BulletMineClear,
        Self::SafeHyperspace,
        Self::CollisionDeathOverride,
        Self::FallingHumanoidSurvival,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::AutoFire => "auto_fire",
            Self::UnlimitedSmartBombs => "unlimited_smart_bombs",
            Self::Invincibility => "invincibility",
            Self::ShotCapOverride => "shot_cap_override",
            Self::BulletMineClear => "bullet_mine_clear",
            Self::SafeHyperspace => "safe_hyperspace",
            Self::CollisionDeathOverride => "collision_death_override",
            Self::FallingHumanoidSurvival => "falling_humanoid_survival",
        }
    }
}

impl CompatibilityState {
    pub const fn overlay_hook(self, hook: XyzzyOverlayHook) -> bool {
        if !self.xyzzy_active {
            return false;
        }

        match hook {
            XyzzyOverlayHook::AutoFire => self.xyzzy_auto_fire,
            XyzzyOverlayHook::UnlimitedSmartBombs => true,
            XyzzyOverlayHook::Invincibility => self.xyzzy_invincible,
            XyzzyOverlayHook::ShotCapOverride
            | XyzzyOverlayHook::BulletMineClear
            | XyzzyOverlayHook::SafeHyperspace
            | XyzzyOverlayHook::CollisionDeathOverride
            | XyzzyOverlayHook::FallingHumanoidSurvival => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineEvent {
    CreditAdded,
    GameStarted,
    DiagnosticsSelected,
    AuditsSelected,
    HighScoreReset,
    ReversePressed,
    FirePressed,
    SmartBombPressed,
    HyperspacePressed,
    BonusAwarded,
    HighScoreEntryStarted,
    HighScoreInitialAccepted,
    HighScoreSubmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameOutput {
    pub snapshot: MachineSnapshot,
    pub red_label_trace: RedLabelTraceState,
    pub main_board: RedLabelMainBoardSnapshot,
    pub sound_board: RedLabelSoundBoardSnapshot,
    pub object_table_crc32: Option<u32>,
    pub process_table_crc32: Option<u32>,
    pub super_process_table_crc32: Option<u32>,
    pub shell_table_crc32: Option<u32>,
    pub video_crc32: Option<u32>,
    pub events: [Option<MachineEvent>; 8],
    pub sound_commands: [Option<SoundCommand>; FRAME_SOUND_COMMAND_CAPACITY],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrameTraceCrcs {
    pub(crate) object_table_crc32: Option<u32>,
    pub(crate) process_table_crc32: Option<u32>,
    pub(crate) super_process_table_crc32: Option<u32>,
    pub(crate) shell_table_crc32: Option<u32>,
    pub(crate) video_crc32: Option<u32>,
}

impl FrameOutput {
    pub(crate) fn new(
        snapshot: MachineSnapshot,
        red_label_trace: RedLabelTraceState,
        main_board: RedLabelMainBoardSnapshot,
        sound_board: RedLabelSoundBoardSnapshot,
        trace_crcs: FrameTraceCrcs,
        events: &[MachineEvent],
        sound_commands: &[SoundCommand],
    ) -> Self {
        let mut output = [None; 8];
        for (slot, event) in output.iter_mut().zip(events.iter().copied()) {
            *slot = Some(event);
        }
        let mut command_output = [None; FRAME_SOUND_COMMAND_CAPACITY];
        for (slot, command) in command_output
            .iter_mut()
            .zip(sound_commands.iter().copied())
        {
            *slot = Some(command);
        }
        Self {
            snapshot,
            red_label_trace,
            main_board,
            sound_board,
            object_table_crc32: trace_crcs.object_table_crc32,
            process_table_crc32: trace_crcs.process_table_crc32,
            super_process_table_crc32: trace_crcs.super_process_table_crc32,
            shell_table_crc32: trace_crcs.shell_table_crc32,
            video_crc32: trace_crcs.video_crc32,
            events: output,
            sound_commands: command_output,
        }
    }

    pub fn events(&self) -> impl Iterator<Item = MachineEvent> + '_ {
        self.events.iter().filter_map(|event| *event)
    }

    pub fn sound_commands(&self) -> impl Iterator<Item = SoundCommand> + '_ {
        self.sound_commands.iter().filter_map(|command| *command)
    }
}
