//! Neutral facade over the temporary accepted-behavior implementation.
//!
//! The clean rewrite should depend on the contracts in this module, not on
//! legacy module names. This keeps the current machine available as an oracle
//! while making the remaining retirement work explicit and localized.

#[cfg(test)]
use crate::game::GameInput;

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct AcceptedGameplayMachine {
    machine: crate::accepted_behavior::AcceptedMachineAdapter,
}

#[cfg(test)]
impl AcceptedGameplayMachine {
    pub(crate) fn new() -> Self {
        Self {
            machine: crate::accepted_behavior::AcceptedMachineAdapter::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> AcceptedSnapshot {
        self.machine.snapshot()
    }

    pub(crate) fn step(&mut self, input: GameInput) -> AcceptedFrame {
        self.machine.step(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcceptedFrame {
    pub(crate) snapshot: AcceptedSnapshot,
    pub(crate) events: Vec<AcceptedEvent>,
    pub(crate) sound_commands: Vec<u8>,
    pub(crate) visual_signature: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedSnapshot {
    pub(crate) frame: u64,
    pub(crate) phase: AcceptedPhase,
    pub(crate) credits: u8,
    pub(crate) current_player: u8,
    pub(crate) player_count: u8,
    pub(crate) wave: u8,
    pub(crate) wave_profile: AcceptedWaveProfile,
    pub(crate) player: AcceptedPlayer,
    pub(crate) player_stocks: [AcceptedPlayerStock; 2],
    pub(crate) scores: AcceptedScores,
    pub(crate) high_score_initials: AcceptedHighScoreInitials,
    pub(crate) high_score_entry: Option<AcceptedHighScoreEntry>,
    pub(crate) high_score_submission: Option<AcceptedHighScoreSubmission>,
    pub(crate) high_score_tables: AcceptedHighScoreTables,
    pub(crate) game_over: AcceptedGameOverState,
    pub(crate) object_evidence: AcceptedObjectEvidence,
    pub(crate) expanded_objects: AcceptedExpandedObjectEvidence,
    pub(crate) player_explosion: Option<crate::game::PlayerExplosionCloudSnapshot>,
    pub(crate) terrain_blow: Option<crate::game::TerrainBlowSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedPlayer {
    pub(crate) x_subpixels: i32,
    pub(crate) y_subpixels: i32,
    pub(crate) x_velocity_subpixels: i32,
    pub(crate) y_velocity_subpixels: i32,
    pub(crate) direction: AcceptedDirection,
    pub(crate) lives: u8,
    pub(crate) smart_bombs: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedPlayerStock {
    pub(crate) lives: u8,
    pub(crate) smart_bombs: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedScores {
    pub(crate) player_one: u32,
    pub(crate) player_two: u32,
    pub(crate) high_score: u32,
    pub(crate) next_bonus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedWaveProfile {
    pub(crate) landers: u8,
    pub(crate) bombers: u8,
    pub(crate) pods: u8,
    pub(crate) mutants: u8,
    pub(crate) swarmers: u8,
    pub(crate) lander_x_velocity: u8,
    pub(crate) lander_y_velocity_msb: u8,
    pub(crate) lander_y_velocity_lsb: u8,
    pub(crate) mutant_random_y: u8,
    pub(crate) mutant_y_velocity_msb: u8,
    pub(crate) mutant_y_velocity_lsb: u8,
    pub(crate) mutant_x_velocity: u8,
    pub(crate) swarmer_x_velocity: u8,
    pub(crate) wave_time: u32,
    pub(crate) wave_size: u8,
    pub(crate) lander_shot_time: u32,
    pub(crate) bomber_x_velocity: u8,
    pub(crate) mutant_shot_time: u32,
    pub(crate) swarmer_shot_time: u32,
    pub(crate) swarmer_acceleration_mask: u8,
    pub(crate) baiter_delay: u32,
    pub(crate) baiter_shot_time: u32,
    pub(crate) baiter_seek_probability: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedHighScoreInitials {
    pub(crate) initials: [Option<char>; 3],
    pub(crate) cursor: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedHighScoreEntry {
    pub(crate) score: u32,
    pub(crate) rank: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedHighScoreSubmission {
    pub(crate) player: u8,
    pub(crate) score: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedHighScoreTableEntry {
    pub(crate) rank: u8,
    pub(crate) score: u32,
    pub(crate) initials: [Option<char>; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AcceptedHighScoreTables {
    pub(crate) all_time: [AcceptedHighScoreTableEntry; crate::game::HIGH_SCORE_TABLE_ENTRIES],
    pub(crate) todays_greatest:
        [AcceptedHighScoreTableEntry; crate::game::HIGH_SCORE_TABLE_ENTRIES],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedGameOverState {
    pub(crate) player_death_sleep_remaining: Option<u8>,
    pub(crate) player_switch_sleep_remaining: Option<u8>,
    pub(crate) player_switch_from: Option<u8>,
    pub(crate) player_switch_to: Option<u8>,
    pub(crate) no_entry_delay_remaining: Option<u8>,
    pub(crate) hall_of_fame_stall_remaining: Option<u8>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedObjectEvidence {
    pub(crate) active_count: u16,
    pub(crate) inactive_count: u16,
    pub(crate) projectile_count: u16,
    pub(crate) visible_count: u16,
    pub(crate) evidence_crc32: u32,
    pub(crate) detail_count: u8,
    pub(crate) details: [AcceptedObjectEvidenceDetail; crate::game::OBJECT_EVIDENCE_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AcceptedObjectEvidenceList {
    #[default]
    Active,
    Inactive,
    Projectile,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedObjectEvidenceDetail {
    pub(crate) list: AcceptedObjectEvidenceList,
    pub(crate) address: u16,
    pub(crate) slot: u16,
    pub(crate) screen_x: u8,
    pub(crate) screen_y: u8,
    pub(crate) world_x: u16,
    pub(crate) world_y: u16,
    pub(crate) velocity_x: u16,
    pub(crate) velocity_y: u16,
    pub(crate) picture_address: u16,
    pub(crate) picture_label: Option<&'static str>,
    pub(crate) picture_size: Option<(u8, u8)>,
    pub(crate) primary_image_address: Option<u16>,
    pub(crate) alternate_image_address: Option<u16>,
    pub(crate) mapped_sprite: Option<u16>,
    pub(crate) object_type: u8,
    pub(crate) scanner_color: u16,
}

impl AcceptedObjectEvidenceDetail {
    pub(crate) const EMPTY: Self = Self {
        list: AcceptedObjectEvidenceList::Active,
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
pub(crate) struct AcceptedExpandedObjectEvidence {
    pub(crate) active_count: u16,
    pub(crate) last_slot_address: Option<u16>,
    pub(crate) detail_count: u8,
    pub(crate) details: [AcceptedExpandedObjectDetail; crate::game::EXPANDED_OBJECT_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum AcceptedExpandedObjectKind {
    #[default]
    Appearance,
    Explosion,
    ScorePopup,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AcceptedExpandedObjectDetail {
    pub(crate) kind: AcceptedExpandedObjectKind,
    pub(crate) slot_address: u16,
    pub(crate) size: u16,
    pub(crate) descriptor_address: u16,
    pub(crate) picture_label: Option<&'static str>,
    pub(crate) picture_size: Option<(u8, u8)>,
    pub(crate) mapped_sprite: Option<u16>,
    pub(crate) erase_address: u16,
    pub(crate) center_x: u8,
    pub(crate) center_y: u8,
    pub(crate) top_left_x: u8,
    pub(crate) top_left_y: u8,
    pub(crate) object_address: Option<u16>,
    pub(crate) score_popup_lifetime_ticks: Option<u8>,
    pub(crate) score_popup_value: Option<u16>,
    pub(crate) explosion_frame: Option<u8>,
    pub(crate) explosion_lifetime_frames: Option<u8>,
}

impl AcceptedExpandedObjectDetail {
    pub(crate) const EMPTY: Self = Self {
        kind: AcceptedExpandedObjectKind::Appearance,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedPhase {
    Attract,
    Playing,
    GameOver,
    HighScoreEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptedEvent {
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

pub(crate) fn native_visible_size() -> (u16, u16) {
    crate::accepted_behavior::native_visible_size()
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{AcceptedGameplayMachine, AcceptedPhase},
        game::GameInput,
    };

    #[test]
    fn accepted_machine_starts_from_attract_snapshot() {
        let machine = AcceptedGameplayMachine::new();
        let snapshot = machine.snapshot();

        assert_eq!(snapshot.frame, 0);
        assert_eq!(snapshot.phase, AcceptedPhase::Attract);
        assert_eq!(snapshot.current_player, 1);
    }

    #[test]
    fn accepted_machine_exposes_source_object_evidence() {
        let machine = AcceptedGameplayMachine::new();
        let snapshot = machine.snapshot();

        assert!(snapshot.object_evidence.active_count <= 95);
        assert!(snapshot.object_evidence.inactive_count <= 95);
        assert!(snapshot.object_evidence.projectile_count <= 95);
        assert!(snapshot.object_evidence.visible_count <= snapshot.object_evidence.active_count);
        assert!(
            usize::from(snapshot.object_evidence.detail_count)
                <= crate::game::OBJECT_EVIDENCE_DETAIL_LIMIT
        );
        let detail_count = usize::from(snapshot.object_evidence.detail_count);
        if let Some(detail) = snapshot.object_evidence.details[..detail_count]
            .iter()
            .find(|detail| detail.picture_label.is_some())
        {
            assert!(detail.picture_size.is_some());
            assert!(detail.primary_image_address.is_some());
        }
        assert_ne!(snapshot.object_evidence.evidence_crc32, 0);
    }

    #[test]
    fn accepted_machine_exposes_expanded_object_evidence() {
        let machine = AcceptedGameplayMachine::new();
        let snapshot = machine.snapshot();

        assert!(
            usize::from(snapshot.expanded_objects.detail_count)
                <= crate::game::EXPANDED_OBJECT_DETAIL_LIMIT
        );
        assert!(
            snapshot.expanded_objects.detail_count as u16 <= snapshot.expanded_objects.active_count
        );
        let detail_count = usize::from(snapshot.expanded_objects.detail_count);
        for detail in &snapshot.expanded_objects.details[..detail_count] {
            assert_ne!(detail.size, 0);
            assert_ne!(detail.slot_address, 0);
            if detail.picture_label.is_some() {
                assert_ne!(detail.descriptor_address, 0);
                assert!(detail.picture_size.is_some());
            }
        }
    }

    #[test]
    fn accepted_machine_steps_clean_input_contract() {
        let mut machine = AcceptedGameplayMachine::new();

        let frame = machine.step(GameInput {
            coin: true,
            start_one: true,
            fire: true,
            service_auto_up: true,
            ..GameInput::NONE
        });

        assert_eq!(frame.snapshot.frame, 1);
        assert_eq!(frame.snapshot.phase, AcceptedPhase::Attract);
        assert!(frame.visual_signature.is_some());
    }

    #[test]
    fn accepted_facade_exposes_native_visible_size() {
        assert_eq!(crate::accepted::native_visible_size(), (292, 240));
    }
}
