//! Data-only machine state and frame-output contracts.

use crate::{
    board::PALETTE_RAM_SIZE,
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
pub struct MachineSnapshot {
    pub frame: u64,
    pub phase: GamePhase,
    pub credits: u8,
    pub current_player: u8,
    pub wave: u8,
    pub rng: RandState,
    pub player: PlayerState,
    pub scores: ScoreState,
    pub last_input_bits: u16,
    pub wave_profile: WaveProfile,
    pub xyzzy_active: bool,
    pub xyzzy_invincible: bool,
    pub xyzzy_auto_fire: bool,
    pub high_score_entry: Option<HighScoreEntryState>,
    pub high_score_submission: Option<HighScoreSubmissionState>,
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
