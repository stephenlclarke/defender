//! Bridge from the neutral accepted-behavior contracts to the current oracle.

use crate::{
    accepted::{
        AcceptedDirection, AcceptedEvent, AcceptedExpandedObjectDetail,
        AcceptedExpandedObjectEvidence, AcceptedExpandedObjectKind, AcceptedFrame,
        AcceptedGameOverState, AcceptedHighScoreEntry, AcceptedHighScoreInitials,
        AcceptedHighScoreSubmission, AcceptedHighScoreTableEntry, AcceptedHighScoreTables,
        AcceptedObjectEvidence, AcceptedObjectEvidenceDetail, AcceptedObjectEvidenceList,
        AcceptedPhase, AcceptedPlayer, AcceptedPlayerStock, AcceptedScores, AcceptedSnapshot,
        AcceptedWaveProfile,
    },
    machine_state::{self, MachineEvent, MachineSnapshot},
    red_label::Facing,
    red_label_wave::WaveProfile,
    video,
};

#[cfg(test)]
use crate::{game::GameInput, input::CabinetInput, machine::ArcadeMachine};

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct AcceptedMachineAdapter {
    machine: ArcadeMachine,
}

#[cfg(test)]
impl AcceptedMachineAdapter {
    pub(crate) fn new() -> Self {
        Self {
            machine: ArcadeMachine::new(),
        }
    }

    pub(crate) fn snapshot(&self) -> AcceptedSnapshot {
        AcceptedSnapshot::from(self.machine.snapshot())
    }

    pub(crate) fn step(&mut self, input: GameInput) -> AcceptedFrame {
        AcceptedFrame::from(self.machine.step(to_cabinet_input(input)))
    }
}

impl From<machine_state::FrameOutput> for AcceptedFrame {
    fn from(output: machine_state::FrameOutput) -> Self {
        Self {
            snapshot: AcceptedSnapshot::from(output.snapshot),
            events: output.events().map(AcceptedEvent::from).collect(),
            sound_commands: output
                .sound_commands()
                .map(|command| command.raw())
                .collect(),
            visual_signature: output.video_crc32,
        }
    }
}

impl From<MachineSnapshot> for AcceptedSnapshot {
    fn from(snapshot: MachineSnapshot) -> Self {
        Self {
            frame: snapshot.frame,
            phase: AcceptedPhase::from(snapshot.phase),
            credits: snapshot.credits,
            current_player: snapshot.current_player,
            player_count: snapshot.player_count,
            wave: snapshot.wave,
            wave_profile: accepted_wave_profile(snapshot.wave_profile),
            player: AcceptedPlayer {
                x_subpixels: snapshot.player.x.0,
                y_subpixels: snapshot.player.y.0,
                x_velocity_subpixels: snapshot.player.xv.0,
                y_velocity_subpixels: snapshot.player.yv.0,
                direction: AcceptedDirection::from(snapshot.player.facing),
                lives: snapshot.player.lives,
                smart_bombs: snapshot.player.smart_bombs,
            },
            player_stocks: snapshot.player_stocks.map(accepted_player_stock),
            scores: AcceptedScores {
                player_one: snapshot.scores.player_one,
                player_two: snapshot.scores.player_two,
                high_score: snapshot.scores.high_score,
                next_bonus: snapshot.scores.next_bonus,
            },
            high_score_initials: accepted_high_score_initials(snapshot.high_score_entry),
            high_score_entry: snapshot.high_score_entry.map(accepted_high_score_entry),
            high_score_submission: snapshot
                .high_score_submission
                .map(accepted_high_score_submission),
            high_score_tables: accepted_high_score_tables(snapshot.high_score_tables),
            game_over: accepted_game_over(snapshot.game_over),
            object_evidence: accepted_object_evidence(snapshot.object_lists),
            expanded_objects: accepted_expanded_objects(snapshot.expanded_objects),
            player_explosion: snapshot.player_explosion,
            terrain_blow: snapshot.terrain_blow,
        }
    }
}

fn accepted_high_score_initials(
    state: Option<machine_state::HighScoreEntryState>,
) -> AcceptedHighScoreInitials {
    let Some(state) = state else {
        return AcceptedHighScoreInitials::default();
    };

    AcceptedHighScoreInitials {
        initials: state.initials.map(accepted_initial),
        cursor: state.cursor.min(3),
    }
}

fn accepted_wave_profile(state: WaveProfile) -> AcceptedWaveProfile {
    AcceptedWaveProfile {
        landers: state.landers,
        bombers: state.bombers,
        pods: state.pods,
        mutants: state.mutants,
        swarmers: state.swarmers,
        lander_x_velocity: state.lander_x_velocity,
        lander_y_velocity_msb: state.lander_y_velocity_msb,
        lander_y_velocity_lsb: state.lander_y_velocity_lsb,
        mutant_random_y: state.mutant_random_y,
        mutant_y_velocity_msb: state.mutant_y_velocity_msb,
        mutant_y_velocity_lsb: state.mutant_y_velocity_lsb,
        mutant_x_velocity: state.mutant_x_velocity,
        swarmer_x_velocity: state.swarmer_x_velocity,
        wave_time: state.wave_time,
        wave_size: state.wave_size,
        lander_shot_time: state.lander_shot_time,
        bomber_x_velocity: state.bomber_x_velocity,
        mutant_shot_time: state.mutant_shot_time,
        swarmer_shot_time: state.swarmer_shot_time,
        swarmer_acceleration_mask: state.swarmer_acceleration_mask,
        baiter_delay: state.baiter_delay,
        baiter_shot_time: state.baiter_shot_time,
        baiter_seek_probability: state.baiter_seek_probability,
    }
}

fn accepted_player_stock(state: machine_state::PlayerStockState) -> AcceptedPlayerStock {
    AcceptedPlayerStock {
        lives: state.lives,
        smart_bombs: state.smart_bombs,
    }
}

fn accepted_initial(initial: u8) -> Option<char> {
    char::from(initial)
        .is_ascii_alphabetic()
        .then(|| char::from(initial).to_ascii_uppercase())
}

fn accepted_high_score_entry(state: machine_state::HighScoreEntryState) -> AcceptedHighScoreEntry {
    AcceptedHighScoreEntry {
        score: state.score,
        rank: state.rank,
    }
}

fn accepted_high_score_submission(
    state: machine_state::HighScoreSubmissionState,
) -> AcceptedHighScoreSubmission {
    AcceptedHighScoreSubmission {
        player: state.player,
        score: state.score,
    }
}

fn accepted_high_score_tables(
    state: machine_state::HighScoreTablesState,
) -> AcceptedHighScoreTables {
    AcceptedHighScoreTables {
        all_time: state.all_time.map(accepted_high_score_table_entry),
        todays_greatest: state.todays_greatest.map(accepted_high_score_table_entry),
    }
}

fn accepted_game_over(state: machine_state::GameOverState) -> AcceptedGameOverState {
    AcceptedGameOverState {
        player_death_sleep_remaining: state.player_death_sleep_remaining,
        player_switch_sleep_remaining: state.player_switch_sleep_remaining,
        player_switch_from: state.player_switch_from,
        player_switch_to: state.player_switch_to,
        no_entry_delay_remaining: state.no_entry_delay_remaining,
        hall_of_fame_stall_remaining: state.hall_of_fame_stall_remaining,
    }
}

fn accepted_object_evidence(state: machine_state::ObjectListState) -> AcceptedObjectEvidence {
    let mut accepted = AcceptedObjectEvidence {
        active_count: state.active_count,
        inactive_count: state.inactive_count,
        projectile_count: state.projectile_count,
        visible_count: state.visible_count,
        evidence_crc32: state.evidence_crc32,
        detail_count: state.detail_count,
        details: [AcceptedObjectEvidenceDetail::EMPTY; crate::game::OBJECT_EVIDENCE_DETAIL_LIMIT],
    };
    for index in 0..usize::from(state.detail_count).min(crate::game::OBJECT_EVIDENCE_DETAIL_LIMIT) {
        accepted.details[index] = accepted_object_evidence_detail(state.details[index]);
    }
    accepted
}

fn accepted_object_evidence_detail(
    state: machine_state::ObjectListDetailState,
) -> AcceptedObjectEvidenceDetail {
    AcceptedObjectEvidenceDetail {
        list: match state.list {
            machine_state::ObjectListNameState::Active => AcceptedObjectEvidenceList::Active,
            machine_state::ObjectListNameState::Inactive => AcceptedObjectEvidenceList::Inactive,
            machine_state::ObjectListNameState::Projectile => {
                AcceptedObjectEvidenceList::Projectile
            }
        },
        address: state.address,
        slot: state.slot,
        screen_x: state.screen_x,
        screen_y: state.screen_y,
        world_x: state.world_x,
        world_y: state.world_y,
        velocity_x: state.velocity_x,
        velocity_y: state.velocity_y,
        picture_address: state.picture_address,
        picture_label: state.picture_label,
        picture_size: state.picture_size,
        primary_image_address: state.primary_image_address,
        alternate_image_address: state.alternate_image_address,
        mapped_sprite: state.mapped_sprite,
        object_type: state.object_type,
        scanner_color: state.scanner_color,
    }
}

fn accepted_expanded_objects(
    state: machine_state::ExpandedObjectState,
) -> AcceptedExpandedObjectEvidence {
    let mut accepted = AcceptedExpandedObjectEvidence {
        active_count: state.active_count,
        last_slot_address: state.last_slot_address,
        detail_count: state.detail_count,
        details: [AcceptedExpandedObjectDetail::EMPTY; crate::game::EXPANDED_OBJECT_DETAIL_LIMIT],
    };
    for index in 0..usize::from(state.detail_count).min(crate::game::EXPANDED_OBJECT_DETAIL_LIMIT) {
        accepted.details[index] = accepted_expanded_object_detail(state.details[index]);
    }
    accepted
}

fn accepted_expanded_object_detail(
    state: machine_state::ExpandedObjectDetailState,
) -> AcceptedExpandedObjectDetail {
    AcceptedExpandedObjectDetail {
        kind: match state.kind {
            machine_state::ExpandedObjectKindState::Appearance => {
                AcceptedExpandedObjectKind::Appearance
            }
            machine_state::ExpandedObjectKindState::Explosion => {
                AcceptedExpandedObjectKind::Explosion
            }
            machine_state::ExpandedObjectKindState::ScorePopup => {
                AcceptedExpandedObjectKind::ScorePopup
            }
        },
        slot_address: state.slot_address,
        size: state.size,
        descriptor_address: state.descriptor_address,
        picture_label: state.picture_label,
        picture_size: state.picture_size,
        mapped_sprite: state.mapped_sprite,
        erase_address: state.erase_address,
        center_x: state.center_x,
        center_y: state.center_y,
        top_left_x: state.top_left_x,
        top_left_y: state.top_left_y,
        object_address: state.object_address,
        score_popup_lifetime_ticks: state.score_popup_lifetime_ticks,
        score_popup_value: state.score_popup_value,
        explosion_frame: state.explosion_frame,
        explosion_lifetime_frames: state.explosion_lifetime_frames,
    }
}

fn accepted_high_score_table_entry(
    entry: machine_state::HighScoreTableEntryState,
) -> AcceptedHighScoreTableEntry {
    AcceptedHighScoreTableEntry {
        rank: entry.rank,
        score: entry.score,
        initials: entry.initials.map(accepted_initial),
    }
}

impl From<machine_state::GamePhase> for AcceptedPhase {
    fn from(phase: machine_state::GamePhase) -> Self {
        match phase {
            machine_state::GamePhase::Attract => Self::Attract,
            machine_state::GamePhase::Playing => Self::Playing,
            machine_state::GamePhase::GameOver => Self::GameOver,
            machine_state::GamePhase::HighScoreEntry => Self::HighScoreEntry,
        }
    }
}

impl From<Facing> for AcceptedDirection {
    fn from(direction: Facing) -> Self {
        match direction {
            Facing::Left => Self::Left,
            Facing::Right => Self::Right,
        }
    }
}

impl From<MachineEvent> for AcceptedEvent {
    fn from(event: MachineEvent) -> Self {
        match event {
            MachineEvent::CreditAdded => Self::CreditAdded,
            MachineEvent::GameStarted => Self::GameStarted,
            MachineEvent::DiagnosticsSelected => Self::DiagnosticsSelected,
            MachineEvent::AuditsSelected => Self::AuditsSelected,
            MachineEvent::HighScoreReset => Self::HighScoreReset,
            MachineEvent::ReversePressed => Self::ReversePressed,
            MachineEvent::FirePressed => Self::FirePressed,
            MachineEvent::SmartBombPressed => Self::SmartBombPressed,
            MachineEvent::HyperspacePressed => Self::HyperspacePressed,
            MachineEvent::BonusAwarded => Self::BonusAwarded,
            MachineEvent::HighScoreEntryStarted => Self::HighScoreEntryStarted,
            MachineEvent::HighScoreInitialAccepted => Self::HighScoreInitialAccepted,
            MachineEvent::HighScoreSubmitted => Self::HighScoreSubmitted,
        }
    }
}

pub(crate) fn native_visible_size() -> (u16, u16) {
    video::native_visible_size()
}

#[cfg(test)]
fn to_cabinet_input(input: GameInput) -> CabinetInput {
    CabinetInput {
        coin: input.coin,
        coin_two: input.coin_two,
        coin_three: input.coin_three,
        start_one: input.start_one,
        start_two: input.start_two,
        altitude_up: input.altitude_up,
        altitude_down: input.altitude_down,
        reverse: input.reverse,
        thrust: input.thrust,
        fire: input.fire,
        smart_bomb: input.smart_bomb,
        hyperspace: input.hyperspace,
        auto_up_manual_down: input.service_auto_up,
        service_advance: input.service_advance,
        high_score_reset: input.high_score_reset,
        tilt: input.tilt,
    }
}

#[cfg(test)]
pub(crate) fn cabinet_input_for_test(input: GameInput) -> CabinetInput {
    to_cabinet_input(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        accepted::{
            AcceptedDirection, AcceptedEvent, AcceptedFrame, AcceptedGameplayMachine,
            AcceptedPhase, AcceptedSnapshot,
        },
        input::CabinetInput,
        machine_state::{
            self, GameOverState, HighScoreEntryState, HighScoreSubmissionState,
            HighScoreTableEntryState, MachineEvent,
        },
        red_label::Facing,
    };

    use super::to_cabinet_input;
    use crate::game::GameInput;

    #[test]
    fn accepted_input_maps_every_clean_control() {
        let cabinet = to_cabinet_input(GameInput {
            coin: true,
            coin_two: true,
            coin_three: true,
            start_one: true,
            start_two: true,
            altitude_up: true,
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            service_auto_up: true,
            service_advance: true,
            high_score_reset: true,
            high_score_initial: None,
            high_score_backspace: false,
            tilt: true,
        });

        assert_eq!(
            cabinet,
            CabinetInput {
                coin: true,
                coin_two: true,
                coin_three: true,
                start_one: true,
                start_two: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                thrust: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                auto_up_manual_down: true,
                service_advance: true,
                high_score_reset: true,
                tilt: true,
            }
        );
    }

    #[test]
    fn accepted_phase_maps_all_current_phase_variants() {
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::Attract),
            AcceptedPhase::Attract
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::Playing),
            AcceptedPhase::Playing
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::GameOver),
            AcceptedPhase::GameOver
        );
        assert_eq!(
            AcceptedPhase::from(machine_state::GamePhase::HighScoreEntry),
            AcceptedPhase::HighScoreEntry
        );
    }

    #[test]
    fn accepted_direction_maps_both_current_direction_variants() {
        assert_eq!(
            AcceptedDirection::from(Facing::Left),
            AcceptedDirection::Left
        );
        assert_eq!(
            AcceptedDirection::from(Facing::Right),
            AcceptedDirection::Right
        );
    }

    #[test]
    fn accepted_event_maps_all_current_event_variants() {
        let pairs = [
            (MachineEvent::CreditAdded, AcceptedEvent::CreditAdded),
            (MachineEvent::GameStarted, AcceptedEvent::GameStarted),
            (
                MachineEvent::DiagnosticsSelected,
                AcceptedEvent::DiagnosticsSelected,
            ),
            (MachineEvent::AuditsSelected, AcceptedEvent::AuditsSelected),
            (MachineEvent::HighScoreReset, AcceptedEvent::HighScoreReset),
            (MachineEvent::ReversePressed, AcceptedEvent::ReversePressed),
            (MachineEvent::FirePressed, AcceptedEvent::FirePressed),
            (
                MachineEvent::SmartBombPressed,
                AcceptedEvent::SmartBombPressed,
            ),
            (
                MachineEvent::HyperspacePressed,
                AcceptedEvent::HyperspacePressed,
            ),
            (MachineEvent::BonusAwarded, AcceptedEvent::BonusAwarded),
            (
                MachineEvent::HighScoreEntryStarted,
                AcceptedEvent::HighScoreEntryStarted,
            ),
            (
                MachineEvent::HighScoreInitialAccepted,
                AcceptedEvent::HighScoreInitialAccepted,
            ),
            (
                MachineEvent::HighScoreSubmitted,
                AcceptedEvent::HighScoreSubmitted,
            ),
        ];

        for (legacy, accepted) in pairs {
            assert_eq!(AcceptedEvent::from(legacy), accepted);
        }
    }

    #[test]
    fn accepted_frame_owns_snapshot_sounds_and_visual_signature() {
        let mut machine = crate::machine::ArcadeMachine::new();
        let output = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        let frame = AcceptedFrame::from(output);

        assert_eq!(frame.snapshot.frame, 1);
        assert!(frame.sound_commands.is_empty());
        assert!(frame.visual_signature.is_some());
    }

    #[test]
    fn accepted_snapshot_carries_clean_direction_and_score_fields() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.phase = machine_state::GamePhase::Playing;
        snapshot.player.facing = Facing::Left;
        snapshot.player.x.0 = 0x1234;
        snapshot.player.y.0 = 0x5678;
        snapshot.player.xv.0 = 0x0100;
        snapshot.player.yv.0 = -0x0200;
        snapshot.scores.player_one = 100;
        snapshot.player_count = 2;
        snapshot.player_stocks = [
            machine_state::PlayerStockState {
                lives: 5,
                smart_bombs: 3,
            },
            machine_state::PlayerStockState {
                lives: 2,
                smart_bombs: 1,
            },
        ];

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.phase, AcceptedPhase::Playing);
        assert_eq!(accepted.player_count, 2);
        assert_eq!(accepted.player_stocks[0].lives, 5);
        assert_eq!(accepted.player_stocks[0].smart_bombs, 3);
        assert_eq!(accepted.player_stocks[1].lives, 2);
        assert_eq!(accepted.player_stocks[1].smart_bombs, 1);
        assert_eq!(accepted.player.direction, AcceptedDirection::Left);
        assert_eq!(accepted.player.x_subpixels, 0x1234);
        assert_eq!(accepted.player.y_subpixels, 0x5678);
        assert_eq!(accepted.player.x_velocity_subpixels, 0x0100);
        assert_eq!(accepted.player.y_velocity_subpixels, -0x0200);
        assert_eq!(accepted.scores.player_one, 100);
        assert_eq!(accepted.wave_profile.landers, 15);
        assert_eq!(accepted.wave_profile.baiter_delay, 212);
    }

    #[test]
    fn accepted_snapshot_carries_active_high_score_initials() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.phase = machine_state::GamePhase::HighScoreEntry;
        snapshot.high_score_entry = Some(HighScoreEntryState {
            score: 30_000,
            rank: 1,
            initials: [b'A', b' ', b'z'],
            cursor: 2,
        });

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.phase, AcceptedPhase::HighScoreEntry);
        assert_eq!(
            accepted.high_score_initials.initials,
            [Some('A'), None, Some('Z')]
        );
        assert_eq!(accepted.high_score_initials.cursor, 2);
        assert_eq!(
            accepted.high_score_entry,
            Some(crate::accepted::AcceptedHighScoreEntry {
                score: 30_000,
                rank: 1,
            })
        );
        assert_eq!(accepted.high_score_submission, None);
    }

    #[test]
    fn accepted_snapshot_carries_high_score_submission() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.high_score_submission = Some(HighScoreSubmissionState {
            player: 2,
            score: 41_500,
        });

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(
            accepted.high_score_submission,
            Some(crate::accepted::AcceptedHighScoreSubmission {
                player: 2,
                score: 41_500,
            })
        );
        assert_eq!(accepted.high_score_entry, None);
    }

    #[test]
    fn accepted_snapshot_carries_high_score_tables() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.high_score_tables.all_time[0] = HighScoreTableEntryState {
            rank: 1,
            score: 50_000,
            initials: [b'A', b'C', b'E'],
        };
        snapshot.high_score_tables.todays_greatest[1] = HighScoreTableEntryState {
            rank: 2,
            score: 40_000,
            initials: [b'B', b' ', b'z'],
        };

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.high_score_tables.all_time[0].rank, 1);
        assert_eq!(accepted.high_score_tables.all_time[0].score, 50_000);
        assert_eq!(
            accepted.high_score_tables.all_time[0].initials,
            [Some('A'), Some('C'), Some('E')]
        );
        assert_eq!(accepted.high_score_tables.todays_greatest[1].rank, 2);
        assert_eq!(accepted.high_score_tables.todays_greatest[1].score, 40_000);
        assert_eq!(
            accepted.high_score_tables.todays_greatest[1].initials,
            [Some('B'), None, Some('Z')]
        );
    }

    #[test]
    fn accepted_snapshot_carries_game_over_return_timing() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.game_over = GameOverState {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: Some(0xFF),
            hall_of_fame_stall_remaining: None,
        };

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.game_over.player_death_sleep_remaining, None);
        assert_eq!(accepted.game_over.player_switch_sleep_remaining, None);
        assert_eq!(accepted.game_over.player_switch_from, None);
        assert_eq!(accepted.game_over.player_switch_to, None);
        assert_eq!(accepted.game_over.no_entry_delay_remaining, Some(0xFF));
        assert_eq!(accepted.game_over.hall_of_fame_stall_remaining, None);
    }

    #[test]
    fn accepted_snapshot_carries_player_switch_timing() {
        let mut snapshot = crate::machine::ArcadeMachine::new().snapshot();
        snapshot.game_over = GameOverState {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: Some(0x60),
            player_switch_from: Some(1),
            player_switch_to: Some(2),
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: None,
        };

        let accepted = AcceptedSnapshot::from(snapshot);

        assert_eq!(accepted.game_over.player_death_sleep_remaining, None);
        assert_eq!(accepted.game_over.player_switch_sleep_remaining, Some(0x60));
        assert_eq!(accepted.game_over.player_switch_from, Some(1));
        assert_eq!(accepted.game_over.player_switch_to, Some(2));
        assert_eq!(accepted.game_over.no_entry_delay_remaining, None);
        assert_eq!(accepted.game_over.hall_of_fame_stall_remaining, None);
    }

    #[test]
    fn accepted_adapter_reaches_current_machine() {
        let mut machine = AcceptedGameplayMachine::new();
        let frame = machine.step(GameInput::NONE);

        assert_eq!(frame.snapshot.frame, 1);
        assert!(frame.visual_signature.is_some());
    }
}
