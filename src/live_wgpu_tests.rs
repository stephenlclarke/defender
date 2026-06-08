#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        GameInput,
        actor_game::{GameInput as ActorGameInput, Phase, PlayerStartReport},
    };
    use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

    use super::{
        ActorScriptCheckExplosionSample, ActorScriptCheckPlayerLaserSample,
        ActorScriptCheckSourceActorSample, ActorScriptCheckSourceProjectileSample,
        ActorScriptCheckSpawnedActorSample, ActorScriptCheckSpawnedCounts, LiveInputState,
        LiveSmokeReport, actor_live_runtime_from_script_path, actor_runtime_from_script_path,
        run_actor_live, run_actor_script_check, run_actor_wgpu_smoke, run_smoke,
    };

    #[test]
    fn live_smoke_report_formats_current_cli_output() {
        let report = LiveSmokeReport {
            frame_source: "actor_game",
            legacy_presenter_used: false,
            window_created: false,
            rendered_frames: 3,
            first_frame_size: Some((640, 480)),
            distinct_frame_signatures: 2,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 1,
            credit_visual_frames: 1,
            playing_visual_frames: 1,
            attract_distinct_frame_signatures: 1,
            credit_distinct_frame_signatures: 1,
            playing_distinct_frame_signatures: 1,
            clean_game_frames: 0,
            actor_frames: 3,
            sprite_frames: 3,
            sprite_instances: 12,
            sprite_draw_commands: 4,
            temporary_raster_frames: 0,
            temporary_raster_commands: 0,
            offscreen_wgpu_frames: 3,
            offscreen_non_blank_frames: 3,
            offscreen_distinct_frame_signatures: 2,
            offscreen_first_frame_signature: Some(0x1234_ABCD),
            offscreen_last_frame_signature: Some(0xABCD_1234),
            injected_inputs: vec![String::from("coin"), String::from("start_one")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "wgpu live smoke passed\n",
                "  frame_source: actor_game\n",
                "  legacy_presenter_used: false\n",
                "  window_created: false\n",
                "  rendered_frames: 3\n",
                "  first_frame_size: 640x480\n",
                "  distinct_frame_signatures: 2\n",
                "  saw_non_blank_frame: true\n",
                "  saw_attract: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_credit: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_playing: true (visual_frames: 1, visual_signatures: 1)\n",
                "  clean_game_frames: 0\n",
                "  actor_frames: 3\n",
                "  sprite_frames: 3\n",
                "  sprite_instances: 12\n",
                "  sprite_draw_commands: 4\n",
                "  temporary_raster_frames: 0\n",
                "  temporary_raster_commands: 0\n",
                "  offscreen_wgpu_frames: 3\n",
                "  offscreen_non_blank_frames: 3\n",
                "  offscreen_distinct_frame_signatures: 2\n",
                "  offscreen_first_frame_signature: 000000001234abcd\n",
                "  offscreen_last_frame_signature: 00000000abcd1234\n",
                "  injected_inputs: coin,start_one\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn live_smoke_uses_actor_frame_source() {
        let report = run_smoke(super::LiveInputProfile::Test, None).expect("actor live smoke");

        assert_eq!(report.frame_source, "actor_game");
        assert!(!report.legacy_presenter_used);
        assert!(!report.window_created);
        assert_eq!(report.clean_game_frames, 0);
        assert_eq!(report.actor_frames, report.rendered_frames);
        assert_eq!(report.temporary_raster_frames, 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert!(report.sprite_frames > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
    }

    #[test]
    fn actor_wgpu_smoke_uses_actor_frame_source() {
        let report = run_actor_wgpu_smoke().expect("actor wgpu smoke");

        assert_eq!(report.frame_source, "actor_game");
        assert!(!report.legacy_presenter_used);
        assert!(!report.window_created);
        assert_eq!(report.clean_game_frames, 0);
        assert_eq!(report.actor_frames, report.rendered_frames);
        assert_eq!(report.temporary_raster_frames, 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert!(report.sprite_frames > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
    }

    #[test]
    fn actor_live_entrypoint_is_available_under_tests() {
        run_actor_live(
            super::LiveInputProfile::Test,
            crate::audio::LiveAudioMode::Null,
            None,
            None,
        )
        .expect("actor live entrypoint should be wired");
    }

    #[test]
    fn actor_live_runtime_admits_fresh_start_buttons_without_manual_coin() {
        let mut one_player =
            actor_live_runtime_from_script_path(None).expect("live runtime should construct");

        let one_started = one_player.step(ActorGameInput {
            start_one: true,
            ..ActorGameInput::NONE
        });

        assert_eq!(one_started.report.phase, Phase::Playing);
        assert_eq!(one_started.report.credits, 0);
        assert_eq!(one_started.report.player_count, 1);
        assert!(matches!(
            one_started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: 1..,
                player: 1,
            })
        ));

        let mut two_player =
            actor_live_runtime_from_script_path(None).expect("live runtime should construct");

        let two_started = two_player.step(ActorGameInput {
            start_two: true,
            ..ActorGameInput::NONE
        });

        assert_eq!(two_started.report.phase, Phase::Playing);
        assert_eq!(two_started.report.credits, 0);
        assert_eq!(two_started.report.player_count, 2);
        assert!(matches!(
            two_started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: 1..,
                player: 1,
            })
        ));
    }

    #[test]
    fn actor_live_entrypoint_loads_sectioned_script_file_under_tests() {
        let path = write_actor_script_file(
            "live-entrypoint",
            "\
            [attract]\n\
            text 1 forever 12 20 LIVE SCRIPT\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            name live script waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        );

        run_actor_live(
            super::LiveInputProfile::Test,
            crate::audio::LiveAudioMode::Null,
            None,
            Some(&path),
        )
        .expect("actor live entrypoint should parse script file under tests");

        let mut runtime = actor_runtime_from_script_path(Some(&path))
            .expect("actor script path should build a runtime");
        let frame = runtime.step(crate::actor_game::GameInput::NONE);

        assert_eq!(
            runtime.driver().script_manifest().wave_script.name,
            "live script waves"
        );
        assert!(frame.report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("LIVE SCRIPT")
                && draw.position == crate::actor_game::Point::new(12, 20)
        }));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_file_loader_reports_read_and_parse_context() {
        let missing = unique_actor_script_path("missing-script");
        let read_error = match actor_runtime_from_script_path(Some(&missing)) {
            Ok(_) => panic!("missing script should fail"),
            Err(error) => error,
        };
        assert!(
            read_error
                .to_string()
                .contains("reading actor driver script")
        );
        assert!(
            read_error
                .to_string()
                .contains(&missing.display().to_string())
        );

        let malformed = write_actor_script_file(
            "malformed-script",
            "\
            [attract]\n\
            text 1 forever 12 20 BAD SCRIPT\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            lander 80 214\n",
        );
        let parse_error = match actor_runtime_from_script_path(Some(&malformed)) {
            Ok(_) => panic!("malformed script should fail"),
            Err(error) => error,
        };
        assert!(
            parse_error
                .to_string()
                .contains("parsing actor driver script")
        );
        assert!(format!("{parse_error:#}").contains("actor driver wave script line 6"));

        let _ = fs::remove_file(malformed);
    }

    #[test]
    fn actor_script_check_report_summarizes_example_driver_script() {
        let path = std::path::Path::new("examples/actor-custom-attract.script");
        let report = run_actor_script_check(path).expect("example actor script should check");

        assert_eq!(report.path, path.display().to_string());
        assert_eq!(report.attract_events, 8);
        let attract_cycle = report
            .attract_cycle
            .as_ref()
            .expect("example script should declare a checked attract cycle");
        assert_eq!(attract_cycle.cycle_steps, 96);
        assert_eq!(attract_cycle.sampled_steps, 96);
        assert_eq!(attract_cycle.attract_frames, 96);
        assert_eq!(attract_cycle.non_attract_frames, 0);
        assert_eq!(attract_cycle.draw_commands, 193);
        assert_eq!(attract_cycle.scene_sprites, 22385);
        assert!(attract_cycle.saw_williams_reveal);
        assert!(attract_cycle.saw_defender_coalescence);
        assert!(attract_cycle.saw_hall_of_fame);
        assert!(attract_cycle.saw_scoring_surface);
        assert!(attract_cycle.saw_final_scoring_label);
        assert!(attract_cycle.saw_cycle_return);
        assert!(report.attract_cycle_unavailable_reason.is_none());
        assert_eq!(report.behavior_kind_profiles, 2);
        assert_eq!(report.behavior_actor_profiles, 0);
        assert_eq!(report.wave_profiles, 1);
        assert_eq!(report.first_frame_phase, "Attract");
        assert_eq!(report.first_frame_draws, 1);
        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_wave_size, 5);
        assert_eq!(report.first_playing_source_landers, 15);
        assert_eq!(report.first_playing_source_bombers, 0);
        assert_eq!(report.first_playing_source_pods, 0);
        assert_eq!(report.first_playing_source_mutants, 0);
        assert_eq!(report.first_playing_source_swarmers, 0);
        assert_eq!(report.first_playing_world_enemies, 2);
        assert_eq!(report.first_playing_world_humans, 2);
        assert_eq!(report.first_playing_reserve_landers, 0);
        assert_eq!(report.first_playing_reserve_bombers, 0);
        assert_eq!(report.first_playing_reserve_pods, 0);
        assert_eq!(report.first_playing_reserve_mutants, 0);
        assert_eq!(report.first_playing_reserve_swarmers, 0);
        assert_eq!(report.first_playing_source_background_left, 0);
        assert_eq!(report.first_playing_source_rng_seed, Some(0xbe));
        assert_eq!(report.first_playing_source_rng_hseed, Some(0xb1));
        assert_eq!(report.first_playing_source_rng_lseed, Some(0x06));
        assert!(report.first_playing_source_actor_samples.is_empty());
        assert!(report.first_playing_source_projectile_samples.is_empty());
        assert_eq!(report.first_playing_sound_commands, [0xea]);
        assert_eq!(
            report
                .first_player_laser
                .as_ref()
                .expect("example checker should sample player laser"),
            &super::ActorScriptCheckFirstPlayerLaserSummary {
                sample_steps: 2,
                samples: vec![ActorScriptCheckPlayerLaserSample {
                    x: 62,
                    y: 120,
                    velocity_dx: 8,
                    velocity_dy: 0,
                    direction: "right".to_string(),
                }],
                sound_commands: vec![0xeb],
            }
        );
        assert!(report.first_player_laser_unavailable_reason.is_none());
        assert!(report.first_player_laser_hit.is_none());
        assert_eq!(
            report.first_player_laser_hit_unavailable_reason.as_deref(),
            Some("player_laser_hit_not_observed_after_512_steps")
        );
        assert!(report.first_source_projectile.is_none());
        assert_eq!(
            report.first_source_projectile_unavailable_reason.as_deref(),
            Some("source_projectile_not_observed_after_512_steps")
        );
        assert!(report.first_playing_player_takes_enemy_collision_damage);
        assert_eq!(report.first_playing_player_laser_cooldown_steps, 6);
        assert_eq!(report.first_playing_lander_mode, "drift");
        assert_eq!(report.first_playing_lander_seek_speed, 1);
        assert_eq!(report.first_playing_lander_drift_speed, 3);
        assert_eq!(report.first_playing_lander_fire_period_steps, 96);
        assert_eq!(report.first_playing_mutant_mode, "chase_player");
        assert_eq!(report.first_playing_bomber_mode, "drift");
        assert_eq!(report.first_playing_pod_mode, "drift");
        assert_eq!(report.first_playing_swarmer_mode, "chase_player");
        assert_eq!(report.first_playing_baiter_mode, "chase_player");
        assert_eq!(report.first_playing_swarmer_fire_period_steps, 58);
        assert_eq!(report.first_playing_baiter_fire_period_steps, 42);
        assert_eq!(report.next_playing_assist_steps, Some(140));
        let next_playing = report
            .next_playing
            .as_ref()
            .expect("example script should reach the second wave");
        assert_eq!(next_playing.wave, 2);
        assert_eq!(next_playing.wave_size, 5);
        assert_eq!(next_playing.source_landers, 20);
        assert_eq!(next_playing.source_bombers, 3);
        assert_eq!(next_playing.source_pods, 1);
        assert_eq!(next_playing.source_mutants, 0);
        assert_eq!(next_playing.source_swarmers, 0);
        assert_eq!(next_playing.world_enemies, 2);
        assert_eq!(next_playing.world_humans, 2);
        assert_eq!(next_playing.lander_mode, "drift");
        let wave_clear = report
            .wave_clear
            .as_ref()
            .expect("example script should report wave clear interstitial");
        assert_eq!(wave_clear.assist_steps, 4);
        assert_eq!(wave_clear.next_wave, 2);
        assert_eq!(wave_clear.score, 400);
        assert_eq!(wave_clear.world_enemies, 0);
        assert_eq!(wave_clear.world_humans, 2);
        assert_eq!(wave_clear.total_survivors, Some(2));
        assert_eq!(wave_clear.visible_icons, Some(1));
        assert_eq!(wave_clear.remaining_awards, Some(1));
        assert_eq!(wave_clear.awarded_points, Some(100));
        assert_eq!(wave_clear.astronaut_sleep_steps_remaining, Some(4));
        assert_eq!(wave_clear.wave_advance_sleep_steps_remaining, None);
        let wave_sleep = report
            .wave_clear_advance_sleep
            .as_ref()
            .expect("example script should report wave advance sleep");
        assert_eq!(wave_sleep.assist_steps, 12);
        assert_eq!(wave_sleep.next_wave, 2);
        assert_eq!(wave_sleep.score, 500);
        assert_eq!(wave_sleep.world_enemies, 0);
        assert_eq!(wave_sleep.world_humans, 2);
        assert_eq!(wave_sleep.total_survivors, Some(2));
        assert_eq!(wave_sleep.visible_icons, Some(2));
        assert_eq!(wave_sleep.remaining_awards, Some(0));
        assert_eq!(wave_sleep.awarded_points, None);
        assert_eq!(wave_sleep.astronaut_sleep_steps_remaining, Some(0));
        assert_eq!(wave_sleep.wave_advance_sleep_steps_remaining, Some(128));
        assert!(report.wave_clear_unavailable_reason.is_none());
        assert!(report.wave_clear_advance_sleep_unavailable_reason.is_none());
        assert!(report.reserve_activation_batches.is_empty());
        assert_eq!(
            report.reserve_activation_status,
            "next_playing_has_no_reserve"
        );
        assert!(report.post_reserve_wave_clear.is_none());
        assert_eq!(
            report.post_reserve_wave_clear_unavailable_reason.as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert!(report.post_reserve_wave_clear_advance_sleep.is_none());
        assert_eq!(
            report
                .post_reserve_wave_clear_advance_sleep_unavailable_reason
                .as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert_eq!(report.post_reserve_next_playing_assist_steps, None);
        assert!(report.post_reserve_next_playing.is_none());
        assert_eq!(
            report
                .post_reserve_next_playing_unavailable_reason
                .as_deref(),
            Some("next_playing_has_no_reserve")
        );
        assert!(report.clean_exit);
        assert_eq!(
            report.to_text(),
            concat!(
                "actor script check passed\n",
                "  path: examples/actor-custom-attract.script\n",
                "  attract_events: 8\n",
                "  attract_cycle_steps: 96\n",
                "  attract_cycle_sampled_steps: 96\n",
                "  attract_cycle_frames: attract=96,non_attract=0\n",
                "  attract_cycle_draws: 193\n",
                "  attract_cycle_scene_sprites: 22385\n",
                "  attract_cycle_milestones: williams_reveal=true,defender_coalescence=true,hall_of_fame=true,scoring_surface=true,final_scoring_label=true,cycle_return=true\n",
                "  behavior_kind_profiles: 2\n",
                "  behavior_actor_profiles: 0\n",
                "  wave_profiles: 1\n",
                "  first_frame_phase: Attract\n",
                "  first_frame_draws: 1\n",
                "  first_playing_wave: 1\n",
                "  first_playing_wave_size: 5\n",
                "  first_playing_source_counts: landers=15,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  first_playing_world_counts: enemies=2,humans=2\n",
                "  first_playing_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  first_playing_source_state: background_left=0x0000,rng=seed=0xbe,hseed=0xb1,lseed=0x06\n",
                "  first_playing_source_actor_samples: none\n",
                "  first_playing_source_projectile_samples: none\n",
                "  first_playing_sound_commands: 0xea\n",
                "  first_playing_player_behavior: takes_enemy_collision_damage=true,laser_cooldown_steps=6\n",
                "  first_playing_lander_behavior: mode=drift,seek_speed=1,drift_speed=3,fire_period_steps=96\n",
                "  first_playing_hostile_modes: mutant=chase_player,bomber=drift,pod=drift,swarmer=chase_player,baiter=chase_player\n",
                "  first_playing_hostile_fire: swarmer_period_steps=58,baiter_period_steps=42\n",
                "  first_player_laser_sample_steps: 2\n",
                "  first_player_laser_samples: laser@62,120[velocity=8/0,direction=right]\n",
                "  first_player_laser_sound_commands: 0xeb\n",
                "  first_player_laser_hit: unavailable,reason=player_laser_hit_not_observed_after_512_steps\n",
                "  hostile_laser_hit_matrix: ",
                "lander@2[score_delta=150,score=150,explosions=lander@62,120[source_center=none],sounds=0xf9,spawns=none];",
                "mutant@2[score_delta=150,score=150,explosions=mutant@62,120[source_center=none],sounds=0xe8,spawns=none];",
                "bomber@2[score_delta=250,score=250,explosions=bomber@62,120[source_center=none],sounds=0xfe,spawns=none];",
                "pod@2[score_delta=1000,score=1000,explosions=pod@62,120[source_center=none],sounds=0xfa,spawns=landers=0,bombers=0,pods=0,mutants=0,swarmers=6];",
                "swarmer@2[score_delta=150,score=150,explosions=swarmer@62,120[source_center=none],sounds=0xf8,spawns=none];",
                "baiter@2[score_delta=200,score=200,explosions=baiter@62,120[source_center=none],sounds=0xf8,spawns=none]\n",
                "  hostile_projectile_matrix: ",
                "lander@1[samples=enemy_laser@210,45[velocity=-3/3,source=frac=0xe9/0x60,vel=0xfd00/0x0300,life=90],sounds=0xfc];",
                "mutant@454[samples=enemy_laser@0,222[velocity=1/-1,source=frac=0x50/0x00,vel=0x009c/0xfe5c,life=90],sounds=0xf6];",
                "swarmer@0[samples=enemy_laser@62,120[velocity=3/0,source=none],sounds=0xf3];",
                "baiter@79[samples=enemy_laser@28,120[velocity=1/-1,source=frac=0x00/0x00,vel=0x002c/0xffc4,life=20],sounds=0xfc]\n",
                "  first_source_projectile: unavailable,reason=source_projectile_not_observed_after_512_steps\n",
                "  wave_clear_assist_steps: 4\n",
                "  wave_clear_next_wave: 2\n",
                "  wave_clear_score: 400\n",
                "  wave_clear_world_counts: enemies=0,humans=2\n",
                "  wave_clear_survivor_bonus: total=2,visible_icons=1,remaining_awards=1,awarded_points=100\n",
                "  wave_clear_sleep: astronaut_steps=4,wave_advance_steps=none\n",
                "  wave_clear_advance_sleep_assist_steps: 12\n",
                "  wave_clear_advance_sleep_next_wave: 2\n",
                "  wave_clear_advance_sleep_score: 500\n",
                "  wave_clear_advance_sleep_world_counts: enemies=0,humans=2\n",
                "  wave_clear_advance_sleep_survivor_bonus: total=2,visible_icons=2,remaining_awards=0,awarded_points=none\n",
                "  wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128\n",
                "  next_playing_assist_steps: 140\n",
                "  next_playing_wave: 2\n",
                "  next_playing_wave_size: 5\n",
                "  next_playing_source_counts: landers=20,bombers=3,pods=1,mutants=0,swarmers=0\n",
                "  next_playing_world_counts: enemies=2,humans=2\n",
                "  next_playing_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0\n",
                "  next_playing_source_state: background_left=0x0000,rng=seed=0x82,hseed=0x35,lseed=0x88\n",
                "  next_playing_source_actor_samples: none\n",
                "  next_playing_source_projectile_samples: none\n",
                "  next_playing_sound_commands: none\n",
                "  next_playing_player_behavior: takes_enemy_collision_damage=true,laser_cooldown_steps=6\n",
                "  next_playing_lander_behavior: mode=drift,seek_speed=1,drift_speed=3,fire_period_steps=96\n",
                "  next_playing_hostile_modes: mutant=chase_player,bomber=drift,pod=drift,swarmer=chase_player,baiter=chase_player\n",
                "  next_playing_hostile_fire: swarmer_period_steps=58,baiter_period_steps=42\n",
                "  reserve_activation_batches: 0\n",
                "  reserve_activation_status: next_playing_has_no_reserve\n",
                "  post_reserve_wave_clear: unavailable,reason=next_playing_has_no_reserve\n",
                "  post_reserve_wave_clear_advance_sleep: unavailable,reason=next_playing_has_no_reserve\n",
                "  post_reserve_next_playing: unavailable,reason=next_playing_has_no_reserve\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn actor_script_check_reports_custom_attract_cycle_milestones() {
        let path = write_actor_script_file(
            "actor-script-attract-cycle-check",
            concat!(
                "[attract]\n",
                "cycle 12\n",
                "williams_logo 1 forever 12 20\n",
                "defender_wordmark 2 4 48 36\n",
                "message 3 3 HALLD_TITLE 0x3854\n",
                "scoring_surface 4 6\n",
                "message 5 4 SWARMV 0x5CA8\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name attract cycle check waves\n",
                "wave 1\n",
                "lander 80 214\n",
                "human 100 214\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("attract cycle script should check");
        let summary = report
            .attract_cycle
            .as_ref()
            .expect("declared cycle should be sampled");

        assert_eq!(summary.cycle_steps, 12);
        assert_eq!(summary.sampled_steps, 12);
        assert_eq!(summary.attract_frames, 12);
        assert_eq!(summary.non_attract_frames, 0);
        assert!(summary.draw_commands > 0);
        assert!(summary.scene_sprites > 0);
        assert!(summary.saw_williams_reveal);
        assert!(summary.saw_defender_coalescence);
        assert!(summary.saw_hall_of_fame);
        assert!(summary.saw_scoring_surface);
        assert!(summary.saw_final_scoring_label);
        assert!(summary.saw_cycle_return);
        assert!(report.attract_cycle_unavailable_reason.is_none());
        assert!(report.to_text().contains("attract_cycle_steps: 12"));
        assert!(report.to_text().contains(
            "attract_cycle_milestones: williams_reveal=true,defender_coalescence=true,hall_of_fame=true,scoring_surface=true,final_scoring_label=true,cycle_return=true"
        ));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_check_reports_player_laser_hit_explosion_and_sound() {
        let path = write_actor_script_file(
            "actor-script-player-laser-hit-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 HIT CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "kind lander lander_drift_speed 0\n",
                "[wave]\n",
                "name hit check waves\n",
                "wave 1\n",
                "lander 62 120\n",
                "human 100 214\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("hit script should check");
        let first_hit = report
            .first_player_laser_hit
            .as_ref()
            .expect("checker should sample the first player laser hit");

        assert_eq!(first_hit.sample_steps, 2);
        assert_eq!(first_hit.score, 250);
        assert_eq!(first_hit.sound_commands, [0xf9]);
        assert_eq!(
            first_hit.explosion_samples,
            vec![ActorScriptCheckExplosionSample {
                kind: "lander".to_string(),
                x: 62,
                y: 120,
                source_center_x: None,
                source_center_y: None,
            }]
        );
        assert!(report.first_player_laser_hit_unavailable_reason.is_none());
        assert!(
            report
                .to_text()
                .contains("first_player_laser_hit_explosions: lander@62,120[source_center=none]")
        );
        assert!(
            report
                .to_text()
                .contains("first_player_laser_hit_sound_commands: 0xf9")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_check_reports_hostile_laser_hit_matrix() {
        let path = std::path::Path::new("examples/actor-custom-attract.script");
        let report = run_actor_script_check(path).expect("example actor script should check");

        let expected = [
            (
                "lander",
                150,
                0xf9,
                ActorScriptCheckSpawnedCounts::default(),
            ),
            (
                "mutant",
                150,
                0xe8,
                ActorScriptCheckSpawnedCounts::default(),
            ),
            (
                "bomber",
                250,
                0xfe,
                ActorScriptCheckSpawnedCounts::default(),
            ),
            (
                "pod",
                1000,
                0xfa,
                ActorScriptCheckSpawnedCounts {
                    swarmers: 6,
                    ..ActorScriptCheckSpawnedCounts::default()
                },
            ),
            (
                "swarmer",
                150,
                0xf8,
                ActorScriptCheckSpawnedCounts::default(),
            ),
            (
                "baiter",
                200,
                0xf8,
                ActorScriptCheckSpawnedCounts::default(),
            ),
        ];

        assert_eq!(report.hostile_laser_hit_matrix.len(), expected.len());
        for (kind, score_delta, sound_command, spawned_counts) in expected {
            let sample = report
                .hostile_laser_hit_matrix
                .iter()
                .find(|sample| sample.kind == kind)
                .unwrap_or_else(|| panic!("missing hostile hit matrix sample for {kind}"));
            assert_eq!(sample.sample_steps, 2, "{kind} sample step");
            assert_eq!(sample.score_delta, score_delta, "{kind} score delta");
            assert_eq!(sample.score, score_delta, "{kind} cumulative score");
            assert_eq!(sample.sound_commands, [sound_command], "{kind} sound");
            assert_eq!(
                sample.explosion_samples,
                vec![ActorScriptCheckExplosionSample {
                    kind: kind.to_string(),
                    x: 62,
                    y: 120,
                    source_center_x: None,
                    source_center_y: None,
                }],
                "{kind} explosion"
            );
            assert_eq!(
                sample.spawned_counts, spawned_counts,
                "{kind} spawned counts"
            );
        }

        let text = report.to_text();
        assert!(text.contains(
            "hostile_laser_hit_matrix: lander@2[score_delta=150,score=150,explosions=lander@62,120[source_center=none],sounds=0xf9,spawns=none]"
        ));
        assert!(text.contains(
            "pod@2[score_delta=1000,score=1000,explosions=pod@62,120[source_center=none],sounds=0xfa,spawns=landers=0,bombers=0,pods=0,mutants=0,swarmers=6]"
        ));
    }

    #[test]
    fn actor_script_check_reports_hostile_projectile_matrix() {
        let path = std::path::Path::new("examples/actor-custom-attract.script");
        let report = run_actor_script_check(path).expect("example actor script should check");

        let expected = [
            ("lander", 0xfc),
            ("mutant", 0xf6),
            ("swarmer", 0xf3),
            ("baiter", 0xfc),
        ];

        assert_eq!(report.hostile_projectile_matrix.len(), expected.len());
        for (kind, sound_command) in expected {
            let sample = report
                .hostile_projectile_matrix
                .iter()
                .find(|sample| sample.kind == kind)
                .unwrap_or_else(|| panic!("missing hostile projectile matrix sample for {kind}"));
            assert_eq!(sample.sound_commands, [sound_command], "{kind} sound");
            assert!(
                !sample.samples.is_empty(),
                "{kind} should publish a projectile sample"
            );
            assert!(
                sample
                    .samples
                    .iter()
                    .all(|projectile| projectile.kind == "enemy_laser"),
                "{kind} projectile kind"
            );
            if kind == "swarmer" {
                assert!(
                    sample
                        .samples
                        .iter()
                        .all(|projectile| projectile.lifetime_ticks.is_none()),
                    "{kind} should be a clean scripted shot"
                );
            } else {
                assert!(
                    sample
                        .samples
                        .iter()
                        .all(|projectile| projectile.lifetime_ticks.unwrap_or_default() > 0),
                    "{kind} source projectile metadata"
                );
            }
        }

        let text = report.to_text();
        assert!(text.contains("hostile_projectile_matrix: lander@"));
        assert!(text.contains("sounds=0xfc"));
        assert!(text.contains("swarmer@"));
        assert!(text.contains("sounds=0xf3"));
    }

    #[test]
    fn actor_script_check_reports_player_laser_sample_and_sound() {
        let path = write_actor_script_file(
            "actor-script-player-laser-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 LASER CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "kind laser laser_speed 3\n",
                "kind laser laser_lifetime_steps 5\n",
                "[wave]\n",
                "name laser check waves\n",
                "wave 1\n",
                "lander 90 120\n",
                "human 100 214\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("laser script should check");
        let first_laser = report
            .first_player_laser
            .as_ref()
            .expect("checker should sample the first player laser");

        assert_eq!(first_laser.sample_steps, 2);
        assert_eq!(first_laser.sound_commands, [0xeb]);
        assert_eq!(
            first_laser.samples,
            vec![ActorScriptCheckPlayerLaserSample {
                x: 57,
                y: 120,
                velocity_dx: 3,
                velocity_dy: 0,
                direction: "right".to_string(),
            }]
        );
        assert!(report.first_player_laser_unavailable_reason.is_none());
        assert!(
            report
                .to_text()
                .contains("first_player_laser_samples: laser@57,120[velocity=3/0,direction=right]")
        );
        assert!(
            report
                .to_text()
                .contains("first_player_laser_sound_commands: 0xeb")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_check_reports_source_projectile_and_sound_samples() {
        let path = write_actor_script_file(
            "actor-script-source-projectile-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 PROJECTILE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name projectile check waves\n",
                "arcade_wave 1 wave_size 1 landers 0 bombers 0 pods 0 mutants 1 swarmers 0 ",
                "mutant_shot_time 1 mutant_x_velocity 48 mutant_random_y 2\n",
                "behavior kind mutant mutant_mode drift\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("projectile script should check");
        let first_projectile = report
            .first_source_projectile
            .as_ref()
            .expect("checker should sample the first source projectile");

        assert_eq!(first_projectile.sample_steps, 455);
        assert_eq!(first_projectile.sound_commands, [0xf6]);
        assert_eq!(
            first_projectile.samples,
            vec![ActorScriptCheckSourceProjectileSample {
                kind: "enemy_laser".to_string(),
                x: 0,
                y: 220,
                x_subpixel: 0xec,
                y_subpixel: 0x5c,
                x_velocity_word: 0x009c,
                y_velocity_word: 0xfe5c,
                lifetime_ticks: 90,
            }]
        );
        assert!(report.first_source_projectile_unavailable_reason.is_none());
        assert!(report.to_text().contains(
            "first_source_projectile_samples: enemy_laser@0,220[frac=0xec/0x5c,vel=0x009c/0xfe5c,life=90]"
        ));
        assert!(
            report
                .to_text()
                .contains("first_source_projectile_sound_commands: 0xf6")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn actor_script_check_reports_arcade_wave_overrides_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-source-wave-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 SOURCE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name source check waves\n",
                "arcade_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
                "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
                "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("arcade wave script should check");

        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_wave_size, 5);
        assert_eq!(report.first_playing_source_landers, 1);
        assert_eq!(report.first_playing_source_bombers, 1);
        assert_eq!(report.first_playing_source_pods, 1);
        assert_eq!(report.first_playing_source_mutants, 1);
        assert_eq!(report.first_playing_source_swarmers, 1);
        assert_eq!(report.first_playing_world_enemies, 5);
        assert_eq!(report.first_playing_world_humans, 10);
        assert_eq!(report.first_playing_reserve_landers, 0);
        assert_eq!(report.first_playing_reserve_bombers, 0);
        assert_eq!(report.first_playing_reserve_pods, 0);
        assert_eq!(report.first_playing_reserve_mutants, 0);
        assert_eq!(report.first_playing_reserve_swarmers, 0);
        assert_eq!(
            report.first_playing_source_actor_samples,
            vec![
                ActorScriptCheckSourceActorSample {
                    kind: "lander".to_string(),
                    x: 251,
                    y: 44,
                    x_subpixel: 0x33,
                    y_subpixel: 0xe0,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "bomber".to_string(),
                    x: 227,
                    y: 104,
                    x_subpixel: 0xe0,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "pod".to_string(),
                    x: 184,
                    y: 72,
                    x_subpixel: 0x20,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "mutant".to_string(),
                    x: 148,
                    y: 96,
                    x_subpixel: 0x00,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "swarmer".to_string(),
                    x: 236,
                    y: 66,
                    x_subpixel: 0x00,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "human".to_string(),
                    x: 24,
                    y: 224,
                    x_subpixel: 0xc3,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "human".to_string(),
                    x: 28,
                    y: 225,
                    x_subpixel: 0x81,
                    y_subpixel: 0x00,
                },
                ActorScriptCheckSourceActorSample {
                    kind: "human".to_string(),
                    x: 78,
                    y: 224,
                    x_subpixel: 0x30,
                    y_subpixel: 0x00,
                },
            ]
        );
        assert!(report.to_text().contains(
            "first_playing_source_counts: landers=1,bombers=1,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_world_counts: enemies=5,humans=10")
        );
        assert!(report.to_text().contains(
            "first_playing_source_actor_samples: lander@251,44[frac=0x33/0xe0];bomber@227,104[frac=0xe0/0x00];pod@184,72[frac=0x20/0x00];mutant@148,96[frac=0x00/0x00];swarmer@236,66[frac=0x00/0x00]"
        ));
    }

    #[test]
    fn actor_script_check_reports_reserve_and_source_state_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-reserve-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 RESERVE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name reserve check waves\n",
                "arcade_wave 1 wave_size 2 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
                "reserve_full 3 2 1 1 1\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("reserve script should check");

        assert_eq!(report.first_playing_world_enemies, 2);
        assert_eq!(report.first_playing_world_humans, 10);
        assert_eq!(report.first_playing_reserve_landers, 3);
        assert_eq!(report.first_playing_reserve_bombers, 2);
        assert_eq!(report.first_playing_reserve_pods, 1);
        assert_eq!(report.first_playing_reserve_mutants, 1);
        assert_eq!(report.first_playing_reserve_swarmers, 1);
        assert_eq!(report.first_playing_source_background_left, 0);
        assert!(report.first_playing_source_rng_seed.is_some());
        assert!(report.to_text().contains(
            "first_playing_reserve_counts: landers=3,bombers=2,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_source_state: background_left=0x0000,rng=seed=")
        );
    }

    #[test]
    fn actor_script_check_reports_next_wave_progression_after_assisted_clear() {
        let path = write_actor_script_file(
            "actor-script-next-wave-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 NEXT WAVE CHECK\n",
                "[behavior]\n",
                "kind lander lander_mode drift\n",
                "[wave]\n",
                "name next wave check waves\n",
                "arcade_wave 1 wave_size 1 landers 1 bombers 0 pods 0 mutants 0 swarmers 0\n",
                "behavior kind lander lander_mode drift\n",
                "behavior kind lander lander_drift_speed 2\n",
                "arcade_wave 2 wave_size 3 landers 1 bombers 1 pods 1 mutants 0 swarmers 0\n",
                "reserve_full 2 1 1 1 1\n",
                "behavior kind lander lander_mode chase_player\n",
                "behavior kind lander lander_seek_speed 7\n",
                "behavior kind swarmer swarmer_fire_period_steps 23\n",
                "behavior kind baiter baiter_fire_period_steps 31\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("next wave script should check");
        let next_playing = report
            .next_playing
            .as_ref()
            .expect("checker should reach wave two with assist");
        let wave_clear = report
            .wave_clear
            .as_ref()
            .expect("checker should report the assisted wave-clear interstitial");
        let wave_sleep = report
            .wave_clear_advance_sleep
            .as_ref()
            .expect("checker should report the arcade wave advance sleep");
        let post_reserve_wave_clear = report
            .post_reserve_wave_clear
            .as_ref()
            .expect("checker should report wave clear after reserve exhaustion");
        let post_reserve_wave_sleep = report
            .post_reserve_wave_clear_advance_sleep
            .as_ref()
            .expect("checker should report post-reserve wave advance sleep");
        let post_reserve_next_playing = report
            .post_reserve_next_playing
            .as_ref()
            .expect("checker should report playable wave after post-reserve sleep");
        assert_eq!(report.reserve_activation_batches.len(), 3);
        let first_activation = &report.reserve_activation_batches[0];
        let second_activation = &report.reserve_activation_batches[1];
        let third_activation = &report.reserve_activation_batches[2];

        assert_eq!(report.first_playing_wave, 1);
        assert_eq!(report.first_playing_world_enemies, 1);
        assert_eq!(wave_clear.assist_steps, 4);
        assert_eq!(wave_clear.next_wave, 2);
        assert_eq!(wave_clear.score, 250);
        assert_eq!(wave_clear.world_enemies, 0);
        assert_eq!(wave_clear.world_humans, 10);
        assert_eq!(wave_clear.total_survivors, Some(10));
        assert_eq!(wave_clear.visible_icons, Some(1));
        assert_eq!(wave_clear.remaining_awards, Some(9));
        assert_eq!(wave_clear.awarded_points, Some(100));
        assert_eq!(wave_clear.astronaut_sleep_steps_remaining, Some(4));
        assert_eq!(wave_clear.wave_advance_sleep_steps_remaining, None);
        assert!(report.wave_clear_unavailable_reason.is_none());
        assert_eq!(wave_sleep.assist_steps, 44);
        assert_eq!(wave_sleep.next_wave, 2);
        assert_eq!(wave_sleep.score, 1150);
        assert_eq!(wave_sleep.world_enemies, 0);
        assert_eq!(wave_sleep.world_humans, 10);
        assert_eq!(wave_sleep.total_survivors, Some(10));
        assert_eq!(wave_sleep.visible_icons, Some(10));
        assert_eq!(wave_sleep.remaining_awards, Some(0));
        assert_eq!(wave_sleep.awarded_points, None);
        assert_eq!(wave_sleep.astronaut_sleep_steps_remaining, Some(0));
        assert_eq!(wave_sleep.wave_advance_sleep_steps_remaining, Some(128));
        assert!(report.wave_clear_advance_sleep_unavailable_reason.is_none());
        assert_eq!(next_playing.wave, 2);
        assert_eq!(next_playing.wave_size, 3);
        assert_eq!(next_playing.source_landers, 1);
        assert_eq!(next_playing.source_bombers, 1);
        assert_eq!(next_playing.source_pods, 1);
        assert_eq!(next_playing.source_mutants, 0);
        assert_eq!(next_playing.source_swarmers, 0);
        assert_eq!(next_playing.world_enemies, 3);
        assert_eq!(next_playing.world_humans, 10);
        assert_eq!(next_playing.reserve_landers, 2);
        assert_eq!(next_playing.reserve_bombers, 1);
        assert_eq!(next_playing.reserve_pods, 1);
        assert_eq!(next_playing.reserve_mutants, 1);
        assert_eq!(next_playing.reserve_swarmers, 1);
        assert_eq!(next_playing.lander_mode, "chase_player");
        assert_eq!(next_playing.lander_seek_speed, 7);
        assert_eq!(next_playing.swarmer_fire_period_steps, 23);
        assert_eq!(next_playing.baiter_fire_period_steps, 31);
        assert!(report.next_playing_assist_steps.is_some_and(
            |steps| steps > 0 && steps < super::ACTOR_SCRIPT_CHECK_NEXT_WAVE_STEP_LIMIT as u32
        ));
        assert_eq!(first_activation.assist_steps, 244);
        assert_eq!(first_activation.spawned_counts.landers, 2);
        assert_eq!(first_activation.spawned_counts.bombers, 0);
        assert_eq!(first_activation.spawned_counts.pods, 0);
        assert_eq!(first_activation.spawned_counts.mutants, 0);
        assert_eq!(first_activation.spawned_counts.swarmers, 0);
        assert_eq!(
            first_activation.spawned_samples,
            vec![
                ActorScriptCheckSpawnedActorSample {
                    kind: "lander".to_string(),
                    x: 222,
                    y: 44,
                },
                ActorScriptCheckSpawnedActorSample {
                    kind: "lander".to_string(),
                    x: 251,
                    y: 44,
                },
            ]
        );
        assert_eq!(first_activation.playing.wave, 2);
        assert_eq!(first_activation.playing.world_enemies, 2);
        assert_eq!(first_activation.playing.world_humans, 10);
        assert_eq!(first_activation.playing.reserve_landers, 0);
        assert_eq!(first_activation.playing.reserve_bombers, 1);
        assert_eq!(first_activation.playing.reserve_pods, 1);
        assert_eq!(first_activation.playing.reserve_mutants, 1);
        assert_eq!(first_activation.playing.reserve_swarmers, 1);
        assert_eq!(first_activation.playing.lander_mode, "chase_player");
        assert_eq!(first_activation.playing.lander_seek_speed, 7);

        assert!(second_activation.assist_steps > first_activation.assist_steps);
        assert_eq!(second_activation.spawned_counts.landers, 0);
        assert_eq!(second_activation.spawned_counts.bombers, 1);
        assert_eq!(second_activation.spawned_counts.pods, 1);
        assert_eq!(second_activation.spawned_counts.mutants, 1);
        assert_eq!(second_activation.spawned_counts.swarmers, 0);
        assert_eq!(
            second_activation.spawned_samples,
            vec![
                ActorScriptCheckSpawnedActorSample {
                    kind: "bomber".to_string(),
                    x: 171,
                    y: 80,
                },
                ActorScriptCheckSpawnedActorSample {
                    kind: "pod".to_string(),
                    x: 36,
                    y: 135,
                },
                ActorScriptCheckSpawnedActorSample {
                    kind: "mutant".to_string(),
                    x: 106,
                    y: 141,
                },
            ]
        );
        assert_eq!(second_activation.playing.wave, 2);
        assert_eq!(second_activation.playing.world_enemies, 3);
        assert_eq!(second_activation.playing.reserve_landers, 0);
        assert_eq!(second_activation.playing.reserve_bombers, 0);
        assert_eq!(second_activation.playing.reserve_pods, 0);
        assert_eq!(second_activation.playing.reserve_mutants, 0);
        assert_eq!(second_activation.playing.reserve_swarmers, 1);

        assert!(third_activation.assist_steps > second_activation.assist_steps);
        assert_eq!(third_activation.spawned_counts.landers, 0);
        assert_eq!(third_activation.spawned_counts.bombers, 0);
        assert_eq!(third_activation.spawned_counts.pods, 0);
        assert_eq!(third_activation.spawned_counts.mutants, 0);
        assert_eq!(third_activation.spawned_counts.swarmers, 1);
        assert_eq!(
            third_activation.spawned_samples,
            vec![ActorScriptCheckSpawnedActorSample {
                kind: "swarmer".to_string(),
                x: 173,
                y: 124,
            }]
        );
        assert_eq!(third_activation.playing.wave, 2);
        assert_eq!(third_activation.playing.world_enemies, 1);
        assert_eq!(third_activation.playing.reserve_landers, 0);
        assert_eq!(third_activation.playing.reserve_bombers, 0);
        assert_eq!(third_activation.playing.reserve_pods, 0);
        assert_eq!(third_activation.playing.reserve_mutants, 0);
        assert_eq!(third_activation.playing.reserve_swarmers, 0);
        assert_eq!(report.reserve_activation_status, "reserve_empty");
        assert_eq!(post_reserve_wave_clear.assist_steps, 736);
        assert_eq!(post_reserve_wave_clear.next_wave, 3);
        assert_eq!(post_reserve_wave_clear.score, 4600);
        assert_eq!(post_reserve_wave_clear.world_enemies, 0);
        assert_eq!(post_reserve_wave_clear.world_humans, 10);
        assert_eq!(post_reserve_wave_clear.total_survivors, Some(10));
        assert_eq!(post_reserve_wave_clear.visible_icons, Some(1));
        assert_eq!(post_reserve_wave_clear.remaining_awards, Some(9));
        assert_eq!(post_reserve_wave_clear.awarded_points, Some(200));
        assert_eq!(
            post_reserve_wave_clear.astronaut_sleep_steps_remaining,
            Some(4)
        );
        assert_eq!(
            post_reserve_wave_clear.wave_advance_sleep_steps_remaining,
            None
        );
        assert!(report.post_reserve_wave_clear_unavailable_reason.is_none());
        assert_eq!(post_reserve_wave_sleep.assist_steps, 776);
        assert_eq!(post_reserve_wave_sleep.next_wave, 3);
        assert_eq!(post_reserve_wave_sleep.score, 6400);
        assert_eq!(post_reserve_wave_sleep.world_enemies, 0);
        assert_eq!(post_reserve_wave_sleep.world_humans, 10);
        assert_eq!(post_reserve_wave_sleep.total_survivors, Some(10));
        assert_eq!(post_reserve_wave_sleep.visible_icons, Some(10));
        assert_eq!(post_reserve_wave_sleep.remaining_awards, Some(0));
        assert_eq!(post_reserve_wave_sleep.awarded_points, None);
        assert_eq!(
            post_reserve_wave_sleep.astronaut_sleep_steps_remaining,
            Some(0)
        );
        assert_eq!(
            post_reserve_wave_sleep.wave_advance_sleep_steps_remaining,
            Some(128)
        );
        assert!(
            report
                .post_reserve_wave_clear_advance_sleep_unavailable_reason
                .is_none()
        );
        assert_eq!(report.post_reserve_next_playing_assist_steps, Some(904));
        assert_eq!(post_reserve_next_playing.wave, 3);
        assert_eq!(post_reserve_next_playing.wave_size, 3);
        assert_eq!(post_reserve_next_playing.source_landers, 1);
        assert_eq!(post_reserve_next_playing.source_bombers, 1);
        assert_eq!(post_reserve_next_playing.source_pods, 1);
        assert_eq!(post_reserve_next_playing.source_mutants, 0);
        assert_eq!(post_reserve_next_playing.source_swarmers, 0);
        assert_eq!(post_reserve_next_playing.world_enemies, 3);
        assert_eq!(post_reserve_next_playing.world_humans, 10);
        assert_eq!(post_reserve_next_playing.reserve_landers, 2);
        assert_eq!(post_reserve_next_playing.reserve_bombers, 1);
        assert_eq!(post_reserve_next_playing.reserve_pods, 1);
        assert_eq!(post_reserve_next_playing.reserve_mutants, 1);
        assert_eq!(post_reserve_next_playing.reserve_swarmers, 1);
        assert_eq!(post_reserve_next_playing.lander_mode, "chase_player");
        assert_eq!(post_reserve_next_playing.lander_seek_speed, 7);
        assert_eq!(post_reserve_next_playing.swarmer_fire_period_steps, 23);
        assert_eq!(post_reserve_next_playing.baiter_fire_period_steps, 31);
        assert!(
            report
                .post_reserve_next_playing_unavailable_reason
                .is_none()
        );
        assert!(report.to_text().contains(
            "next_playing_source_counts: landers=1,bombers=1,pods=1,mutants=0,swarmers=0"
        ));
        assert!(report.to_text().contains(
            "wave_clear_survivor_bonus: total=10,visible_icons=1,remaining_awards=9,awarded_points=100"
        ));
        assert!(report.to_text().contains(
            "wave_clear_advance_sleep_survivor_bonus: total=10,visible_icons=10,remaining_awards=0,awarded_points=none"
        ));
        assert!(
            report.to_text().contains(
                "wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128"
            )
        );
        assert!(report.to_text().contains(
            "next_playing_reserve_counts: landers=2,bombers=1,pods=1,mutants=1,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("next_playing_lander_behavior: mode=chase_player,seek_speed=7")
        );
        assert!(
            report.to_text().contains(
                "next_playing_hostile_fire: swarmer_period_steps=23,baiter_period_steps=31"
            )
        );
        assert!(report.to_text().contains("reserve_activation_batches: 3"));
        assert!(report.to_text().contains(
            "reserve_activation_1_spawned_counts: landers=2,bombers=0,pods=0,mutants=0,swarmers=0"
        ));
        assert!(
            report
                .to_text()
                .contains("reserve_activation_1_spawned_samples: lander@222,44;lander@251,44")
        );
        assert!(report.to_text().contains(
            "reserve_activation_2_spawned_counts: landers=0,bombers=1,pods=1,mutants=1,swarmers=0"
        ));
        assert!(report.to_text().contains(
            "reserve_activation_2_spawned_samples: bomber@171,80;pod@36,135;mutant@106,141"
        ));
        assert!(report.to_text().contains(
            "reserve_activation_3_spawned_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=1"
        ));
        assert!(
            report
                .to_text()
                .contains("reserve_activation_3_spawned_samples: swarmer@173,124")
        );
        assert!(report.to_text().contains(
            "reserve_activation_3_reserve_counts: landers=0,bombers=0,pods=0,mutants=0,swarmers=0"
        ));
        assert!(
            report
                .to_text()
                .contains("reserve_activation_status: reserve_empty")
        );
        assert!(
            report
                .to_text()
                .contains("post_reserve_wave_clear_next_wave: 3")
        );
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_survivor_bonus: total=10,visible_icons=1,remaining_awards=9,awarded_points=200"
        ));
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_advance_sleep_survivor_bonus: total=10,visible_icons=10,remaining_awards=0,awarded_points=none"
        ));
        assert!(report.to_text().contains(
            "post_reserve_wave_clear_advance_sleep_sleep: astronaut_steps=0,wave_advance_steps=128"
        ));
        assert!(
            report
                .to_text()
                .contains("post_reserve_next_playing_assist_steps: 904")
        );
        assert!(report.to_text().contains(
            "post_reserve_next_playing_source_counts: landers=1,bombers=1,pods=1,mutants=0,swarmers=0"
        ));
    }

    #[test]
    fn actor_script_check_reports_effective_behavior_overrides_at_play_start() {
        let path = write_actor_script_file(
            "actor-script-behavior-check",
            concat!(
                "[attract]\n",
                "text 1 forever 12 20 BEHAVIOR CHECK\n",
                "[behavior]\n",
                "kind player player_takes_enemy_collision_damage false\n",
                "kind player player_laser_cooldown_steps 5\n",
                "[wave]\n",
                "name behavior check waves\n",
                "arcade_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1\n",
                "behavior kind lander lander_mode chase_player\n",
                "behavior kind lander lander_seek_speed 4\n",
                "behavior kind mutant mutant_mode drift\n",
                "behavior kind bomber bomber_mode chase_player\n",
                "behavior kind pod pod_mode chase_player\n",
                "behavior kind swarmer swarmer_mode drift\n",
                "behavior kind swarmer swarmer_fire_period_steps 17\n",
                "behavior kind baiter baiter_mode drift\n",
                "behavior kind baiter baiter_fire_period_steps 19\n",
                "spawn_behavior lander 0 lander_seek_speed 9\n",
            ),
        );

        let report = run_actor_script_check(&path).expect("behavior script should check");

        assert!(!report.first_playing_player_takes_enemy_collision_damage);
        assert_eq!(report.first_playing_player_laser_cooldown_steps, 5);
        assert_eq!(report.first_playing_lander_mode, "chase_player");
        assert_eq!(report.first_playing_lander_seek_speed, 9);
        assert_eq!(report.first_playing_mutant_mode, "drift");
        assert_eq!(report.first_playing_bomber_mode, "chase_player");
        assert_eq!(report.first_playing_pod_mode, "chase_player");
        assert_eq!(report.first_playing_swarmer_mode, "drift");
        assert_eq!(report.first_playing_baiter_mode, "drift");
        assert_eq!(report.first_playing_swarmer_fire_period_steps, 17);
        assert_eq!(report.first_playing_baiter_fire_period_steps, 19);
        assert!(report.to_text().contains(
            "first_playing_player_behavior: takes_enemy_collision_damage=false,laser_cooldown_steps=5"
        ));
        assert!(
            report
                .to_text()
                .contains("first_playing_lander_behavior: mode=chase_player,seek_speed=9")
        );
        assert!(report.to_text().contains(
            "first_playing_hostile_modes: mutant=drift,bomber=chase_player,pod=chase_player,swarmer=drift,baiter=drift"
        ));
    }

    #[test]
    fn actor_live_uses_actor_derived_game_frame_handoff() {
        let source = include_str!("live_wgpu_window.rs");

        assert!(source.contains("let actor_frame = self.runtime.step_clean_input(input, xyzzy);"));
        assert!(source.contains("let frame = actor_frame.game_frame();"));
        assert!(source.contains("self.audio.submit_game_frame(&frame);"));
        let old_batch_call = [
            "LiveAudioEventBatch::new(",
            "frame.report.step",
            ", frame.events.sounds())",
        ]
        .concat();
        assert!(!source.contains(&old_batch_call));
    }

    #[test]
    fn live_input_state_carries_xyzzy_mode_for_actor_runtime() {
        let mut input = LiveInputState::default();
        for character in ['X', 'Y', 'Z', 'Z', 'Y'] {
            input.apply(super::LiveControl::HighScoreInitial(character), true);
        }
        input.apply(super::LiveControl::HighScoreInitial('F'), true);
        input.apply(super::LiveControl::HighScoreInitial('G'), true);
        input.apply(super::LiveControl::SmartBomb, true);

        let clean_input = input.drain_game_input();
        let xyzzy = input.drain_xyzzy_mode();

        assert!(clean_input.smart_bomb);
        assert!(xyzzy.active);
        assert!(xyzzy.auto_fire);
        assert!(xyzzy.invincible);
        assert!(xyzzy.overlay_smart_bomb);
        assert!(!input.drain_xyzzy_mode().overlay_smart_bomb);
    }

    #[test]
    fn live_input_state_emits_edge_pulses_and_held_gameplay_controls() {
        let mut input = LiveInputState::default();
        input.apply(super::LiveControl::Coin, true);
        input.apply(super::LiveControl::StartOne, true);
        input.apply(super::LiveControl::StartTwo, true);
        input.apply(super::LiveControl::Thrust, true);
        input.apply(super::LiveControl::AltitudeUp, true);
        input.apply(super::LiveControl::AltitudeDown, true);
        input.apply(super::LiveControl::Reverse, true);
        input.apply(super::LiveControl::Fire, true);
        input.apply(super::LiveControl::SmartBomb, true);
        input.apply(super::LiveControl::Hyperspace, true);
        input.apply(super::LiveControl::ServiceAutoUp, true);
        input.apply(super::LiveControl::ServiceAdvance, true);
        input.apply(super::LiveControl::HighScoreReset, true);
        input.apply(super::LiveControl::HighScoreInitial('A'), true);
        input.apply(super::LiveControl::HighScoreBackspace, true);
        input.apply(super::LiveControl::Quit, true);

        assert_eq!(
            input.drain_game_input(),
            GameInput {
                coin: true,
                start_one: true,
                start_two: true,
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                service_advance: true,
                high_score_reset: true,
                high_score_initial: Some('A'),
                high_score_backspace: true,
                ..GameInput::NONE
            }
        );
        assert_eq!(
            input.drain_game_input(),
            GameInput {
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                ..GameInput::NONE
            }
        );

        input.apply(super::LiveControl::Thrust, false);
        input.apply(super::LiveControl::AltitudeUp, false);
        input.apply(super::LiveControl::AltitudeDown, false);
        input.apply(super::LiveControl::Reverse, false);
        input.apply(super::LiveControl::Fire, false);
        input.apply(super::LiveControl::SmartBomb, false);
        input.apply(super::LiveControl::Hyperspace, false);
        input.apply(super::LiveControl::ServiceAutoUp, false);
        assert_eq!(input.drain_game_input(), GameInput::NONE);
    }

    #[test]
    fn live_input_state_emits_reverse_as_press_pulse() {
        let mut input = LiveInputState::default();

        input.apply(super::LiveControl::Reverse, true);
        assert!(input.drain_game_input().reverse);
        assert!(!input.drain_game_input().reverse);
        assert!(!input.drain_game_input().reverse);

        input.apply(super::LiveControl::Reverse, false);
        assert!(!input.drain_game_input().reverse);

        input.apply(super::LiveControl::Reverse, true);
        assert!(input.drain_game_input().reverse);
    }

    #[test]
    fn planetoid_live_keymap_binds_shift_to_reverse_and_space_to_thrust() {
        assert_eq!(
            super::physical_control(
                super::LiveInputProfile::Planetoid,
                &PhysicalKey::Code(KeyCode::ShiftLeft),
            ),
            Some(super::LiveControl::Reverse)
        );
        assert_eq!(
            super::physical_control(
                super::LiveInputProfile::Planetoid,
                &PhysicalKey::Code(KeyCode::ShiftRight),
            ),
            Some(super::LiveControl::Reverse)
        );
        assert_eq!(
            super::physical_control(
                super::LiveInputProfile::Planetoid,
                &PhysicalKey::Code(KeyCode::Space),
            ),
            Some(super::LiveControl::Thrust)
        );
        assert_eq!(
            super::logical_control(
                super::LiveInputProfile::Planetoid,
                &Key::Named(NamedKey::Shift),
            ),
            Some(super::LiveControl::Reverse)
        );
        assert_eq!(
            super::character_control(super::LiveInputProfile::Planetoid, " "),
            Some(super::LiveControl::Thrust)
        );
    }

    fn write_actor_script_file(label: &str, source: &str) -> std::path::PathBuf {
        let path = unique_actor_script_path(label);
        fs::write(&path, source).expect("write actor script");
        path
    }

    fn unique_actor_script_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "defender-{label}-{}-{nanos}.script",
            std::process::id()
        ))
    }
}
