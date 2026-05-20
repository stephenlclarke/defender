//! Clean gameplay equivalence harness.
//!
//! This module is test-only because the accepted machine is an oracle, not a
//! production runtime dependency.

use std::fmt;

use anyhow::{anyhow, bail};

use crate::{
    fidelity::GameplayEquivalenceSignature,
    game::{Game, GameInput, GameOverSnapshot, GamePhase, GameSnapshot, SoundEvent, WorldSnapshot},
    oracle::GameplayOracle,
    renderer::RenderSceneSummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanFidelityReport {
    pub(crate) scenario: String,
    pub(crate) input_frames: usize,
    pub(crate) compared_frames: usize,
    pub(crate) first_mismatch: Option<CleanFidelityMismatch>,
}

impl CleanFidelityReport {
    pub(crate) fn to_tsv(&self) -> String {
        let mut text = String::from(
            "scenario\tinput_frames\tcompared_frames\tstatus\tfirst_frame_index\tfirst_frame\tfield\tclean\taccepted\n",
        );

        let Some(mismatch) = &self.first_mismatch else {
            text.push_str(&format!(
                "{}\t{}\t{}\tmatch\t\t\t\t\t\n",
                self.scenario, self.input_frames, self.compared_frames
            ));
            return text;
        };

        for field in &mismatch.fields {
            text.push_str(&format!(
                "{}\t{}\t{}\tmismatch\t{}\t{}\t{}\t{}\t{}\n",
                self.scenario,
                self.input_frames,
                self.compared_frames,
                mismatch.frame_index,
                mismatch.frame,
                field.field,
                tsv_cell(&field.clean),
                tsv_cell(&field.accepted)
            ));
        }

        text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanFidelityMismatch {
    pub(crate) frame_index: usize,
    pub(crate) frame: u64,
    pub(crate) fields: Vec<CleanFidelityFieldMismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanFidelityFieldMismatch {
    pub(crate) field: &'static str,
    pub(crate) clean: String,
    pub(crate) accepted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanFidelityProfile {
    Full,
    CoreCabinetFlow,
    PlayerControlFlow,
    LongPlayfieldFlow,
}

pub(crate) fn compare_scenario(name: &str) -> anyhow::Result<CleanFidelityReport> {
    let scenario = crate::fidelity_manifest::scenarios()?
        .into_iter()
        .find(|scenario| scenario.name == name)
        .ok_or_else(|| anyhow!("unknown fidelity scenario: {name}"))?;
    let inputs = input_program_frames(&scenario.input_program)?;

    Ok(compare_clean_game_to_accepted(&scenario.name, &inputs))
}

pub(crate) fn compare_clean_game_to_accepted(
    scenario: &str,
    inputs: &[GameInput],
) -> CleanFidelityReport {
    let profile = profile_for_scenario(scenario);
    let mut clean = Game::new();
    let mut accepted = GameplayOracle::new();

    for (frame_index, input) in inputs.iter().copied().enumerate() {
        let clean_frame = clean.step(input);
        let accepted_frame = accepted.step(input);
        let clean_signature = GameplayEquivalenceSignature::from_frame(&clean_frame);
        let accepted_signature = GameplayEquivalenceSignature::from_frame(&accepted_frame);
        let fields = compare_signatures(profile, &clean_signature, &accepted_signature);

        if !fields.is_empty() {
            return CleanFidelityReport {
                scenario: scenario.to_owned(),
                input_frames: inputs.len(),
                compared_frames: frame_index + 1,
                first_mismatch: Some(CleanFidelityMismatch {
                    frame_index: frame_index + 1,
                    frame: clean_signature.state.frame,
                    fields,
                }),
            };
        }
    }

    CleanFidelityReport {
        scenario: scenario.to_owned(),
        input_frames: inputs.len(),
        compared_frames: inputs.len(),
        first_mismatch: None,
    }
}

fn profile_for_scenario(scenario: &str) -> CleanFidelityProfile {
    match scenario {
        "attract_boot" | "start_game" => CleanFidelityProfile::CoreCabinetFlow,
        "first_300_frames" | "firing" | "thrust_reverse" | "smart_bomb" | "hyperspace" => {
            CleanFidelityProfile::PlayerControlFlow
        }
        "abduction" | "death" | "wave_advance" | "planet_destruction" | "high_score_entry" => {
            CleanFidelityProfile::LongPlayfieldFlow
        }
        _ => CleanFidelityProfile::Full,
    }
}

pub(crate) fn input_program_frames(input_program: &str) -> anyhow::Result<Vec<GameInput>> {
    let expanded = crate::fidelity_manifest::expanded_input_text(input_program)?;
    parse_expanded_input_text(&expanded)
}

fn parse_expanded_input_text(text: &str) -> anyhow::Result<Vec<GameInput>> {
    text.split(';')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(parse_input_frame)
        .collect()
}

fn parse_input_frame(frame: &str) -> anyhow::Result<GameInput> {
    let mut input = GameInput::NONE;

    for action in frame
        .split(',')
        .map(str::trim)
        .filter(|action| !action.is_empty())
    {
        match action {
            "none" => {}
            "coin" => input.coin = true,
            "coin_two" => input.coin_two = true,
            "coin_three" => input.coin_three = true,
            "start_one" => input.start_one = true,
            "start_two" => input.start_two = true,
            "altitude_up" => input.altitude_up = true,
            "altitude_down" => input.altitude_down = true,
            "reverse" => input.reverse = true,
            "thrust" => input.thrust = true,
            "fire" => input.fire = true,
            "smart_bomb" => input.smart_bomb = true,
            "hyperspace" => input.hyperspace = true,
            "service_auto_up" => input.service_auto_up = true,
            "service_advance" => input.service_advance = true,
            "high_score_reset" => input.high_score_reset = true,
            "high_score_backspace" => input.high_score_backspace = true,
            "tilt" => input.tilt = true,
            action if action.starts_with("initial_") => {
                input.high_score_initial = parse_initial(action)?;
            }
            action => bail!("unknown clean fidelity input action: {action}"),
        }
    }

    Ok(input)
}

fn parse_initial(action: &str) -> anyhow::Result<Option<char>> {
    let mut chars = action["initial_".len()..].chars();
    let Some(initial) = chars.next() else {
        bail!("missing high-score initial in action: {action}");
    };

    if chars.next().is_some() {
        bail!("high-score initial action must contain one character: {action}");
    }

    Ok(Some(initial))
}

fn compare_signatures(
    profile: CleanFidelityProfile,
    clean: &GameplayEquivalenceSignature,
    accepted: &GameplayEquivalenceSignature,
) -> Vec<CleanFidelityFieldMismatch> {
    let mut fields = Vec::new();

    compare_state(profile, &mut fields, &clean.state, &accepted.state);
    compare_render(profile, &mut fields, &clean.render, &accepted.render);
    compare_gameplay_events(
        profile,
        &mut fields,
        &clean.gameplay_events,
        &accepted.gameplay_events,
    );
    compare_sound_events(
        profile,
        &mut fields,
        &clean.sound_events,
        &accepted.sound_events,
    );

    fields
}

fn compare_state(
    profile: CleanFidelityProfile,
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &GameSnapshot,
    accepted: &GameSnapshot,
) {
    push_if_different(fields, "state.frame", &clean.frame, &accepted.frame);
    push_if_different(fields, "state.phase", &clean.phase, &accepted.phase);
    push_if_different(fields, "state.credits", &clean.credits, &accepted.credits);
    push_if_different(
        fields,
        "state.current_player",
        &clean.current_player,
        &accepted.current_player,
    );
    if !matches!(
        profile,
        CleanFidelityProfile::PlayerControlFlow | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        push_if_different(fields, "state.wave", &clean.wave, &accepted.wave);
    }
    push_wave_profile_if_comparable(profile, fields, clean, accepted);
    if profile == CleanFidelityProfile::CoreCabinetFlow {
        push_if_different(fields, "state.scores", &clean.scores, &accepted.scores);
        push_if_different(
            fields,
            "state.high_score_initials",
            &clean.high_score_initials,
            &accepted.high_score_initials,
        );
        push_if_different(
            fields,
            "state.high_score_entry",
            &clean.high_score_entry,
            &accepted.high_score_entry,
        );
        push_if_different(
            fields,
            "state.high_score_submission",
            &clean.high_score_submission,
            &accepted.high_score_submission,
        );
        push_if_different(
            fields,
            "state.high_score_tables",
            &clean.high_score_tables,
            &accepted.high_score_tables,
        );
        push_game_over_if_different(fields, clean, accepted);
        return;
    }

    if matches!(
        profile,
        CleanFidelityProfile::PlayerControlFlow | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        push_if_different(
            fields,
            "state.scores.high_score",
            &clean.scores.high_score,
            &accepted.scores.high_score,
        );
        push_if_different(
            fields,
            "state.scores.next_bonus",
            &clean.scores.next_bonus,
            &accepted.scores.next_bonus,
        );
        push_if_different(
            fields,
            "state.high_score_initials",
            &clean.high_score_initials,
            &accepted.high_score_initials,
        );
        push_if_different(
            fields,
            "state.high_score_entry",
            &clean.high_score_entry,
            &accepted.high_score_entry,
        );
        push_if_different(
            fields,
            "state.high_score_submission",
            &clean.high_score_submission,
            &accepted.high_score_submission,
        );
        push_if_different(
            fields,
            "state.high_score_tables",
            &clean.high_score_tables,
            &accepted.high_score_tables,
        );
        push_game_over_if_different(fields, clean, accepted);
        return;
    }

    push_if_different(
        fields,
        "state.player.lives",
        &clean.player.lives,
        &accepted.player.lives,
    );
    push_if_different(
        fields,
        "state.player.smart_bombs",
        &clean.player.smart_bombs,
        &accepted.player.smart_bombs,
    );
    push_if_different(
        fields,
        "state.player.position",
        &clean.player.position,
        &accepted.player.position,
    );
    push_if_different(
        fields,
        "state.player.velocity",
        &clean.player.velocity,
        &accepted.player.velocity,
    );
    push_if_different(
        fields,
        "state.player.direction",
        &clean.player.direction,
        &accepted.player.direction,
    );
    push_if_different(fields, "state.scores", &clean.scores, &accepted.scores);
    push_if_different(
        fields,
        "state.high_score_initials",
        &clean.high_score_initials,
        &accepted.high_score_initials,
    );
    push_if_different(
        fields,
        "state.high_score_entry",
        &clean.high_score_entry,
        &accepted.high_score_entry,
    );
    push_if_different(
        fields,
        "state.high_score_submission",
        &clean.high_score_submission,
        &accepted.high_score_submission,
    );
    push_if_different(
        fields,
        "state.high_score_tables",
        &clean.high_score_tables,
        &accepted.high_score_tables,
    );
    push_game_over_if_different(fields, clean, accepted);
    compare_world(fields, &clean.world, &accepted.world);
}

fn push_game_over_if_different(
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &GameSnapshot,
    accepted: &GameSnapshot,
) {
    let accepted_game_over = comparable_accepted_game_over(clean, accepted);
    push_if_different(
        fields,
        "state.game_over",
        &clean.game_over,
        &accepted_game_over,
    );
}

fn push_wave_profile_if_comparable(
    profile: CleanFidelityProfile,
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &GameSnapshot,
    accepted: &GameSnapshot,
) {
    if matches!(
        profile,
        CleanFidelityProfile::PlayerControlFlow | CleanFidelityProfile::LongPlayfieldFlow
    ) && clean.wave != accepted.wave
    {
        return;
    }

    push_if_different(
        fields,
        "state.wave_profile",
        &clean.wave_profile,
        &accepted.wave_profile,
    );
}

fn comparable_accepted_game_over(
    clean: &GameSnapshot,
    accepted: &GameSnapshot,
) -> GameOverSnapshot {
    let mut state = accepted.game_over;
    if accepted.phase == GamePhase::Attract
        && clean.game_over.hall_of_fame_stall_remaining.is_none()
        && state.player_death_sleep_remaining.is_none()
        && state.no_entry_delay_remaining.is_none()
    {
        state.hall_of_fame_stall_remaining = None;
    }
    state
}

fn compare_world(
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &WorldSnapshot,
    accepted: &WorldSnapshot,
) {
    push_if_different(
        fields,
        "state.world.terrain",
        &clean.terrain,
        &accepted.terrain,
    );
    push_if_different(
        fields,
        "state.world.terrain_blow",
        &clean.terrain_blow,
        &accepted.terrain_blow,
    );
    push_if_different(fields, "state.world.stars", &clean.stars, &accepted.stars);
    push_if_different(
        fields,
        "state.world.enemies",
        &clean.enemies,
        &accepted.enemies,
    );
    push_if_different(
        fields,
        "state.world.humans",
        &clean.humans,
        &accepted.humans,
    );
    push_if_different(
        fields,
        "state.world.projectiles",
        &clean.projectiles,
        &accepted.projectiles,
    );
    push_if_different(
        fields,
        "state.world.enemy_projectiles",
        &clean.enemy_projectiles,
        &accepted.enemy_projectiles,
    );
    push_if_different(
        fields,
        "state.world.object_evidence",
        &clean.object_evidence,
        &accepted.object_evidence,
    );
    push_if_different(
        fields,
        "state.world.expanded_objects",
        &clean.expanded_objects,
        &accepted.expanded_objects,
    );
    push_if_different(
        fields,
        "state.world.player_explosion",
        &clean.player_explosion,
        &accepted.player_explosion,
    );
    push_if_different(
        fields,
        "state.world.scanner",
        &clean.scanner,
        &accepted.scanner,
    );
}

fn compare_render(
    profile: CleanFidelityProfile,
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &RenderSceneSummary,
    accepted: &RenderSceneSummary,
) {
    push_if_different(fields, "render.frame", &clean.frame, &accepted.frame);
    push_if_different(fields, "render.surface", &clean.surface, &accepted.surface);
    if matches!(
        profile,
        CleanFidelityProfile::PlayerControlFlow | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        push_if_different(
            fields,
            "render.raster_count",
            &clean.raster_count,
            &accepted.raster_count,
        );
        return;
    }

    push_if_different(
        fields,
        "render.visual_signature",
        &clean.visual_signature,
        &accepted.visual_signature,
    );
    push_if_different(
        fields,
        "render.raster_count",
        &clean.raster_count,
        &accepted.raster_count,
    );
    if matches!(
        profile,
        CleanFidelityProfile::CoreCabinetFlow
            | CleanFidelityProfile::PlayerControlFlow
            | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        return;
    }

    push_if_different(
        fields,
        "render.sprite_count",
        &clean.sprite_count,
        &accepted.sprite_count,
    );
    push_if_different(fields, "render.layers", &clean.layers, &accepted.layers);
}

fn compare_sound_events(
    profile: CleanFidelityProfile,
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &[SoundEvent],
    accepted: &[SoundEvent],
) {
    if matches!(
        profile,
        CleanFidelityProfile::CoreCabinetFlow
            | CleanFidelityProfile::PlayerControlFlow
            | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        let clean = cabinet_sound_events(clean);
        let accepted = cabinet_sound_events(accepted);
        push_if_different(fields, "sound_events", &clean, &accepted);
        return;
    }

    push_if_different(fields, "sound_events", &clean, &accepted);
}

fn compare_gameplay_events(
    profile: CleanFidelityProfile,
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    clean: &[crate::game::GameEvent],
    accepted: &[crate::game::GameEvent],
) {
    if matches!(
        profile,
        CleanFidelityProfile::CoreCabinetFlow
            | CleanFidelityProfile::PlayerControlFlow
            | CleanFidelityProfile::LongPlayfieldFlow
    ) {
        let clean = cabinet_gameplay_events(clean);
        let accepted = cabinet_gameplay_events(accepted);
        push_if_different(fields, "gameplay_events", &clean, &accepted);
        return;
    }

    push_if_different(fields, "gameplay_events", &clean, &accepted);
}

fn cabinet_gameplay_events(events: &[crate::game::GameEvent]) -> Vec<crate::game::GameEvent> {
    events
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                crate::game::GameEvent::CreditAdded | crate::game::GameEvent::GameStarted
            )
        })
        .collect()
}

fn cabinet_sound_events(events: &[SoundEvent]) -> Vec<SoundEvent> {
    events
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                SoundEvent::Startup | SoundEvent::CreditAdded | SoundEvent::GameStarted
            )
        })
        .collect()
}

fn push_if_different<T>(
    fields: &mut Vec<CleanFidelityFieldMismatch>,
    field: &'static str,
    clean: &T,
    accepted: &T,
) where
    T: PartialEq + fmt::Debug,
{
    if clean == accepted {
        return;
    }

    fields.push(CleanFidelityFieldMismatch {
        field,
        clean: format!("{clean:?}"),
        accepted: format!("{accepted:?}"),
    });
}

fn tsv_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use crate::game::{
        EXPANDED_OBJECT_DETAIL_LIMIT, ExpandedObjectDetailSnapshot, ExpandedObjectEvidenceSnapshot,
        ExpandedObjectKind, Game, GameEvent, GameInput, GameOverSnapshot, GamePhase,
        HighScoreEntrySnapshot, HighScoreSubmissionSnapshot, OBJECT_EVIDENCE_DETAIL_LIMIT,
        ObjectEvidenceCategory, ObjectEvidenceDetailSnapshot, ObjectEvidenceList,
        ObjectEvidenceSnapshot, SoundEvent, TerrainBlowSnapshot, TerrainBlowStage,
    };
    use crate::renderer::SpriteId;

    use super::{
        CleanFidelityFieldMismatch, CleanFidelityMismatch, CleanFidelityProfile,
        CleanFidelityReport, cabinet_gameplay_events, cabinet_sound_events, compare_scenario,
        compare_state, parse_expanded_input_text, profile_for_scenario,
    };

    const PHASE_ONE_SCENARIOS: &str = concat!(
        "attract_boot start_game first_300_frames firing thrust_reverse ",
        "smart_bomb hyperspace abduction death wave_advance planet_destruction ",
        "high_score_entry"
    );

    #[test]
    fn input_program_parser_supports_manifest_actions_and_combinations() {
        let inputs = parse_expanded_input_text(
            "none;coin;start_one;reverse,thrust;fire,smart_bomb,hyperspace;initial_A\n",
        )
        .expect("parse expanded inputs");

        assert_eq!(inputs.len(), 6);
        assert_eq!(inputs[0], GameInput::NONE);
        assert!(inputs[1].coin);
        assert!(inputs[2].start_one);
        assert!(inputs[3].reverse);
        assert!(inputs[3].thrust);
        assert!(inputs[4].fire);
        assert!(inputs[4].smart_bomb);
        assert!(inputs[4].hyperspace);
        assert_eq!(inputs[5].high_score_initial, Some('A'));
    }

    #[test]
    fn clean_fidelity_report_serializes_first_mismatch_as_tsv() {
        let report = CleanFidelityReport {
            scenario: "attract_start_probe".to_owned(),
            input_frames: 8,
            compared_frames: 1,
            first_mismatch: Some(CleanFidelityMismatch {
                frame_index: 1,
                frame: 1,
                fields: vec![CleanFidelityFieldMismatch {
                    field: "render.visual_signature",
                    clean: "None".to_owned(),
                    accepted: "Some(2962226826)".to_owned(),
                }],
            }),
        };
        let mismatch = report.first_mismatch.as_ref().expect("mismatch");

        assert_eq!(report.input_frames, 8);
        assert_eq!(report.compared_frames, 1);
        assert_eq!(mismatch.frame_index, 1);
        assert_eq!(mismatch.frame, 1);
        assert!(
            mismatch
                .fields
                .iter()
                .any(|field| field.field == "render.visual_signature")
        );

        let tsv = report.to_tsv();
        assert!(tsv.starts_with("scenario\tinput_frames\tcompared_frames\tstatus"));
        assert!(tsv.contains("attract_start_probe\t8\t1\tmismatch\t1\t1"));
        assert!(tsv.contains("\trender.visual_signature\t"));
    }

    #[test]
    fn clean_fidelity_profiles_milestone_scenarios() {
        assert_eq!(
            profile_for_scenario("attract_boot"),
            CleanFidelityProfile::CoreCabinetFlow
        );
        assert_eq!(
            profile_for_scenario("start_game"),
            CleanFidelityProfile::CoreCabinetFlow
        );
        assert_eq!(
            profile_for_scenario("firing"),
            CleanFidelityProfile::PlayerControlFlow
        );
        assert_eq!(
            profile_for_scenario("hyperspace"),
            CleanFidelityProfile::PlayerControlFlow
        );
        assert_eq!(
            profile_for_scenario("abduction"),
            CleanFidelityProfile::LongPlayfieldFlow
        );
        assert_eq!(
            profile_for_scenario("planet_destruction"),
            CleanFidelityProfile::LongPlayfieldFlow
        );
        assert_eq!(
            profile_for_scenario("high_score_entry"),
            CleanFidelityProfile::LongPlayfieldFlow
        );
    }

    #[test]
    fn clean_fidelity_compares_high_score_session_state() {
        let mut clean = Game::new().state();
        let mut accepted = clean.clone();
        clean.wave_profile.landers = 15;
        accepted.wave_profile.landers = 20;
        clean.high_score_entry = Some(HighScoreEntrySnapshot {
            score: 30_000,
            rank: 1,
        });
        accepted.high_score_entry = Some(HighScoreEntrySnapshot {
            score: 30_000,
            rank: 2,
        });
        clean.high_score_submission = Some(HighScoreSubmissionSnapshot {
            player: 1,
            score: 30_000,
        });
        accepted.high_score_submission = Some(HighScoreSubmissionSnapshot {
            player: 2,
            score: 30_000,
        });
        clean.high_score_tables.all_time[0].score = 30_000;
        accepted.high_score_tables.all_time[0].score = 29_000;
        clean.game_over.player_death_sleep_remaining = Some(40);
        accepted.game_over.player_death_sleep_remaining = Some(39);

        let mut fields = Vec::new();
        compare_state(
            CleanFidelityProfile::LongPlayfieldFlow,
            &mut fields,
            &clean,
            &accepted,
        );

        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.wave_profile")
        );
        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.high_score_entry")
        );
        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.high_score_submission")
        );
        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.high_score_tables")
        );
        assert!(fields.iter().any(|field| field.field == "state.game_over"));
    }

    #[test]
    fn clean_fidelity_ignores_generic_attract_hall_of_fame_timer() {
        let clean = Game::new().state();
        let mut accepted = clean.clone();
        accepted.phase = GamePhase::Attract;
        accepted.game_over = GameOverSnapshot {
            player_death_sleep_remaining: None,
            player_switch_sleep_remaining: None,
            player_switch_from: None,
            player_switch_to: None,
            no_entry_delay_remaining: None,
            hall_of_fame_stall_remaining: Some(44),
        };

        let mut fields = Vec::new();
        compare_state(
            CleanFidelityProfile::LongPlayfieldFlow,
            &mut fields,
            &clean,
            &accepted,
        );

        assert!(fields.iter().all(|field| field.field != "state.game_over"));
    }

    #[test]
    fn clean_fidelity_ignores_wave_profile_when_profile_ignores_wave_drift() {
        let mut clean = Game::new().state();
        let accepted = clean.clone();
        clean.wave = 2;
        clean.wave_profile = crate::WaveProfileSnapshot::for_wave(2);

        let mut fields = Vec::new();
        compare_state(
            CleanFidelityProfile::PlayerControlFlow,
            &mut fields,
            &clean,
            &accepted,
        );

        assert!(
            fields
                .iter()
                .all(|field| field.field != "state.wave_profile")
        );
    }

    #[test]
    fn clean_fidelity_full_profile_compares_object_evidence() {
        let mut clean = Game::new().state();
        let mut accepted = clean.clone();
        clean.world.object_evidence = ObjectEvidenceSnapshot {
            active_count: 2,
            inactive_count: 0,
            projectile_count: 0,
            visible_count: 2,
            evidence_crc32: None,
            detail_count: 1,
            details: {
                let mut details =
                    [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                details[0] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    object_category: Some(ObjectEvidenceCategory::Human),
                    address: None,
                    slot: None,
                    screen_position: Some(crate::systems::ScreenPosition::new(10, 20)),
                    world_position: None,
                    velocity: None,
                    picture_address: None,
                    picture_label: None,
                    picture_size: None,
                    primary_image_address: None,
                    alternate_image_address: None,
                    mapped_sprite: Some(SpriteId::HUMAN),
                    object_type: None,
                    scanner_color: Some(0x6666),
                };
                details
            },
        };
        accepted.world.object_evidence = ObjectEvidenceSnapshot {
            active_count: 2,
            inactive_count: 0,
            projectile_count: 0,
            visible_count: 2,
            evidence_crc32: None,
            detail_count: 1,
            details: {
                let mut details =
                    [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT];
                details[0] = ObjectEvidenceDetailSnapshot {
                    list: ObjectEvidenceList::Active,
                    object_category: None,
                    address: Some(0xA23C),
                    slot: Some(0),
                    screen_position: Some(crate::systems::ScreenPosition::new(10, 20)),
                    world_position: Some((0x1000, 0x2000)),
                    velocity: Some((0x0001, 0xFFFF)),
                    picture_address: Some(0x9000),
                    picture_label: Some("LNDP1"),
                    picture_size: Some((5, 8)),
                    primary_image_address: Some(0xCCE0),
                    alternate_image_address: Some(0xCD08),
                    mapped_sprite: Some(SpriteId::ENEMY_LANDER),
                    object_type: Some(0x11),
                    scanner_color: Some(0x4433),
                };
                details
            },
        };

        let mut fields = Vec::new();
        compare_state(CleanFidelityProfile::Full, &mut fields, &clean, &accepted);

        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.world.object_evidence")
        );
    }

    #[test]
    fn clean_fidelity_full_profile_compares_expanded_object_evidence() {
        let mut clean = Game::new().state();
        let mut accepted = clean.clone();
        clean.world.expanded_objects = ExpandedObjectEvidenceSnapshot::default();
        accepted.world.expanded_objects = ExpandedObjectEvidenceSnapshot {
            active_count: 1,
            last_slot_address: Some(0x9C00),
            detail_count: 1,
            details: {
                let mut details =
                    [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT];
                details[0] = ExpandedObjectDetailSnapshot {
                    kind: ExpandedObjectKind::Explosion,
                    slot_address: Some(0x9C00),
                    size: 0x01AA,
                    descriptor_address: Some(0xF951),
                    picture_label: Some("BXPIC"),
                    picture_size: Some((4, 8)),
                    mapped_sprite: Some(SpriteId::BOMB_EXPLOSION),
                    erase_address: Some(0x9C40),
                    center: Some(crate::systems::ScreenPosition::new(0x14, 0x05)),
                    top_left: Some(crate::systems::ScreenPosition::new(0x0C, 0x05)),
                    object_address: None,
                    score_popup_lifetime_ticks: None,
                    score_popup_value: None,
                    explosion_frame: Some(1),
                    explosion_lifetime_frames: Some(crate::game::SOURCE_EXPLOSION_LIFETIME_FRAMES),
                };
                details
            },
        };

        let mut fields = Vec::new();
        compare_state(CleanFidelityProfile::Full, &mut fields, &clean, &accepted);

        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.world.expanded_objects")
        );
    }

    #[test]
    fn clean_fidelity_full_profile_compares_terrain_blow_evidence() {
        let clean = Game::new().state();
        let mut accepted = clean.clone();
        accepted.world.terrain_blow = Some(TerrainBlowSnapshot {
            stage: TerrainBlowStage::ExplosionPassSleeping,
            status_terrain_blown: true,
            source_iteration: 0,
            source_iteration_limit: 16,
            source_sleep_remaining: Some(2),
            source_pseudo_color: 0x3C,
            source_overload_counter: 8,
            terrain_erase_entries: 0x98,
            scanner_terrain_erase_entries: 0x40,
            terrain_words_remaining: 0,
            scanner_terrain_words_remaining: 0,
            explosions_per_pass: 2,
        });

        let mut fields = Vec::new();
        compare_state(CleanFidelityProfile::Full, &mut fields, &clean, &accepted);

        assert!(
            fields
                .iter()
                .any(|field| field.field == "state.world.terrain_blow")
        );
    }

    #[test]
    fn cabinet_sound_events_keep_only_r3_cabinet_sounds() {
        let events = cabinet_sound_events(&[
            SoundEvent::CreditAdded,
            SoundEvent::UnmappedSoundCommand { command: 234 },
            SoundEvent::GameStarted,
        ]);

        assert_eq!(events, [SoundEvent::CreditAdded, SoundEvent::GameStarted]);
    }

    #[test]
    fn cabinet_gameplay_events_keep_only_shared_cabinet_events() {
        let events = cabinet_gameplay_events(&[
            GameEvent::CreditAdded,
            GameEvent::FirePressed,
            GameEvent::GameStarted,
        ]);

        assert_eq!(events, [GameEvent::CreditAdded, GameEvent::GameStarted]);
    }

    #[test]
    #[ignore = "developer gate: run through make clean-fidelity"]
    fn clean_fidelity_reports_selected_scenarios() {
        let scenarios = selected_scenarios();
        assert!(
            !scenarios.is_empty(),
            "CLEAN_FIDELITY_SCENARIOS must name at least one scenario"
        );

        for scenario in scenarios {
            let report = compare_scenario(&scenario).expect("compare clean scenario");

            print!("{}", report.to_tsv());
            assert!(report.input_frames > 0);
            assert!(report.compared_frames > 0);
            assert!(report.compared_frames <= report.input_frames);
            assert!(
                report.first_mismatch.is_none(),
                "clean fidelity mismatch for {scenario}"
            );
        }
    }

    fn selected_scenarios() -> Vec<String> {
        let configured = std::env::var("CLEAN_FIDELITY_SCENARIOS")
            .unwrap_or_else(|_| PHASE_ONE_SCENARIOS.to_owned());

        configured
            .split_whitespace()
            .map(str::trim)
            .filter(|scenario| !scenario.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}
