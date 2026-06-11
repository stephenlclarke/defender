#[cfg(test)]
mod tests {
    use super::{
        ACTOR_SMOKE_COIN_STEP, ACTOR_SMOKE_FIRE_STEP, ACTOR_SMOKE_STEPS,
        ACTOR_SMOKE_START_STEP, ACTOR_SMOKE_THRUST_STEP, ActorAttractCycleSmokeReport,
        ActorPostGameSmokeReport, ActorSmokeReport, attract_cycle_report,
        default_attract_cycle_report, default_post_game_report, smoke_actor_input,
        smoke_step_count, smoke_input, smoke_report,
    };

    #[test]
    fn smoke_report_exercises_actor_runtime_and_native_draw_plans() {
        let report = smoke_report(ACTOR_SMOKE_STEPS).expect("actor smoke report");

        assert_eq!(report.steps, ACTOR_SMOKE_STEPS);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
        assert!(report.actor_event_steps > 0);
        assert!(report.actor_sound_events > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_render_commands > 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.clean_exit);
    }

    #[test]
    fn smoke_report_rejects_zero_steps() {
        let error = smoke_report(0).expect_err("zero-step smoke should fail");

        assert_eq!(
            error.to_string(),
            "actor smoke step count must be positive"
        );
    }

    #[test]
    fn attract_cycle_report_exercises_default_actor_attract_loop() {
        let report = default_attract_cycle_report().expect("actor attract cycle smoke report");

        assert_eq!(report.steps, 3479);
        assert_eq!(report.cycle_steps, 3479);
        assert_eq!(report.attract_steps, report.steps);
        assert_eq!(report.playing_steps, 0);
        assert_eq!(report.game_over_steps, 0);
        assert_eq!(report.high_score_entry_steps, 0);
        assert_eq!(report.actor_event_steps, 0);
        assert_eq!(report.actor_sound_steps, 0);
        assert_eq!(report.actor_sound_events, 0);
        assert!(report.distinct_scene_signatures >= 8);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_render_commands > 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.saw_williams_reveal);
        assert!(report.saw_defender_coalescence);
        assert!(report.saw_hall_of_fame);
        assert!(report.saw_scoring_surface);
        assert!(report.saw_final_scoring_instruction);
        assert!(report.saw_cycle_return);
        assert!(report.clean_exit);
    }

    #[test]
    fn attract_cycle_report_rejects_zero_steps() {
        let error = attract_cycle_report(0).expect_err("zero-step attract smoke should fail");

        assert_eq!(
            error.to_string(),
            "actor attract smoke step count must be positive"
        );
    }

    #[test]
    fn post_game_report_exercises_high_score_hall_return() {
        let report = default_post_game_report().expect("actor post-game smoke report");

        assert!(report.steps >= 60);
        assert!(report.playing_steps > 0);
        assert!(report.high_score_entry_steps > 0);
        assert_eq!(report.game_over_steps, 60);
        assert_eq!(report.hall_stall_steps, 60);
        assert!(report.attract_steps > 0);
        assert_eq!(report.forced_player_collisions, 3);
        assert!(report.final_score >= 3_000);
        assert_eq!(report.final_lives, 0);
        assert!(report.player_destroyed_events >= 3);
        assert!(report.game_over_events > 0);
        assert!(report.high_score_entry_events > 0);
        assert_eq!(report.high_score_initial_accept_events, 3);
        assert_eq!(report.high_score_submit_events, 1);
        assert!(report.game_over_sound_events > 0);
        assert!(report.saw_game_over_hall_stall);
        assert!(report.saw_attract_return);
        assert!(report.saw_return_williams_reveal);
        assert!(report.saw_player_sprite);
        assert!(report.saw_pod_sprite);
        assert!(report.saw_explosion_pixels);
        assert!(report.saw_hall_of_fame);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.wgpu_render_commands > 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert!(report.clean_exit);
    }

    #[test]
    fn attract_cycle_report_validates_required_default_milestones() {
        let mut report = ActorAttractCycleSmokeReport {
            steps: 3479,
            cycle_steps: 3479,
            distinct_scene_signatures: 8,
            attract_steps: 3479,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            wgpu_render_commands: 1,
            saw_williams_reveal: true,
            saw_defender_coalescence: true,
            saw_hall_of_fame: true,
            saw_scoring_surface: true,
            saw_final_scoring_instruction: true,
            saw_cycle_return: true,
            clean_exit: true,
            ..ActorAttractCycleSmokeReport::default()
        };

        report.saw_cycle_return = false;
        let error = report
            .validate()
            .expect_err("missing cycle return should fail");
        assert_eq!(
            error.to_string(),
            "actor attract smoke did not return to Williams after cycle boundary"
        );

        report.saw_cycle_return = true;
        report.actor_sound_steps = 1;
        let error = report
            .validate()
            .expect_err("attract sound event should fail");
        assert_eq!(
            error.to_string(),
            "actor attract smoke unexpectedly produced sound events"
        );
    }

    #[test]
    fn post_game_report_validates_high_score_return_contract() {
        let mut report = ActorPostGameSmokeReport {
            steps: 72,
            distinct_scene_signatures: 6,
            playing_steps: 10,
            high_score_entry_steps: 3,
            game_over_steps: 60,
            attract_steps: 1,
            forced_player_collisions: 3,
            final_score: 3_000,
            player_destroyed_events: 3,
            game_over_events: 1,
            high_score_entry_events: 1,
            high_score_initial_accept_events: 3,
            high_score_submit_events: 1,
            actor_sound_steps: 4,
            actor_sound_events: 6,
            game_over_sound_events: 1,
            saw_game_over_hall_stall: true,
            hall_stall_steps: 60,
            saw_attract_return: true,
            saw_return_williams_reveal: true,
            saw_player_sprite: true,
            saw_pod_sprite: true,
            saw_explosion_pixels: true,
            saw_hall_of_fame: true,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            wgpu_render_commands: 1,
            clean_exit: true,
            ..ActorPostGameSmokeReport::default()
        };

        report.hall_stall_steps = 59;
        let error = report.validate().expect_err("short Hall stall should fail");
        assert_eq!(
            error.to_string(),
            "actor post-game smoke did not observe the 60-step Hall-of-Fame stall"
        );

        report.hall_stall_steps = 60;
        report.saw_attract_return = false;
        let error = report
            .validate()
            .expect_err("missing attract return should fail");
        assert_eq!(
            error.to_string(),
            "actor post-game smoke did not return to attract"
        );
    }

    #[test]
    fn smoke_report_validates_required_actor_play_states() {
        let mut report = ActorSmokeReport {
            steps: 1,
            initial_surface_size: Some((292, 240)),
            distinct_scene_signatures: 3,
            sprite_steps: 1,
            sprite_instances: 1,
            sprite_draw_commands: 1,
            object_sprites: 1,
            projectile_sprites: 1,
            hud_sprites: 1,
            overlay_sprites: 1,
            actor_event_steps: 1,
            actor_sound_steps: 1,
            actor_sound_events: 1,
            covered_sprites: super::REQUIRED_SPRITES
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            overlay_draw_commands: 1,
            covered_pipelines: super::REQUIRED_PIPELINES
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            wgpu_render_commands: 1,
            injected_inputs: super::REQUIRED_INPUTS
                .iter()
                .map(|label| (*label).to_owned())
                .collect(),
            clean_exit: true,
            ..ActorSmokeReport::default()
        };

        let error = report
            .validate()
            .expect_err("missing attract should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe attract steps"
        );

        report.saw_attract = true;
        report.attract_steps = 1;
        let error = report
            .validate()
            .expect_err("missing credited attract should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe a credited attract step"
        );

        report.saw_credit = true;
        report.credited_steps = 1;
        let error = report
            .validate()
            .expect_err("missing playing should fail validation");
        assert_eq!(
            error.to_string(),
            "actor smoke did not observe playing steps"
        );
    }

    #[test]
    fn smoke_script_uses_release_frames_between_edge_inputs() {
        assert!(smoke_input(ACTOR_SMOKE_COIN_STEP).value.coin);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_COIN_STEP + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_START_STEP).value.start_one);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_START_STEP + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_FIRE_STEP).value.fire);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_FIRE_STEP + 1).value,
            crate::actor_game::GameInput::NONE
        );
        assert!(smoke_input(ACTOR_SMOKE_THRUST_STEP).value.thrust);
        assert_eq!(
            smoke_input(ACTOR_SMOKE_THRUST_STEP + 1).value,
            crate::actor_game::GameInput::NONE
        );
    }

    #[test]
    fn smoke_script_helpers_match_current_actor_smoke_contract() {
        assert_eq!(smoke_step_count(), ACTOR_SMOKE_STEPS);
        assert!(smoke_actor_input(ACTOR_SMOKE_COIN_STEP).coin);
        assert!(smoke_actor_input(ACTOR_SMOKE_START_STEP).start_one);
        assert!(smoke_actor_input(ACTOR_SMOKE_FIRE_STEP).fire);
        assert_eq!(
            smoke_actor_input(ACTOR_SMOKE_FIRE_STEP + 1),
            crate::actor_game::GameInput::NONE
        );
    }

    #[test]
    fn smoke_report_formats_current_cli_output() {
        let report = ActorSmokeReport {
            steps: 3,
            initial_surface_size: Some((292, 240)),
            distinct_scene_signatures: 2,
            saw_attract: true,
            attract_steps: 1,
            saw_credit: true,
            credited_steps: 1,
            saw_playing: true,
            playing_steps: 2,
            actor_event_steps: 2,
            actor_sound_steps: 2,
            actor_sound_events: 3,
            sprite_steps: 3,
            sprite_instances: 12,
            sprite_draw_commands: 4,
            object_sprites: 2,
            projectile_sprites: 1,
            hud_sprites: 3,
            overlay_sprites: 6,
            covered_sprites: vec!["player_ship".to_owned(), "enemy_lander".to_owned()],
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            overlay_draw_commands: 1,
            covered_pipelines: vec!["sprites".to_owned(), "hud_text".to_owned()],
            wgpu_render_commands: 9,
            injected_inputs: vec!["coin".to_owned(), "start_one".to_owned()],
            clean_exit: true,
            ..ActorSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor smoke passed\n"));
        assert!(text.contains("initial_surface_size: 292x240"));
        assert!(text.contains("covered_sprites: player_ship,enemy_lander"));
        assert!(text.contains("injected_inputs: coin,start_one"));
    }

    #[test]
    fn attract_cycle_report_formats_current_cli_output() {
        let report = ActorAttractCycleSmokeReport {
            steps: 3479,
            cycle_steps: 3479,
            distinct_scene_signatures: 42,
            attract_steps: 3479,
            sprite_instances: 1200,
            sprite_draw_commands: 340,
            wgpu_render_commands: 680,
            saw_williams_reveal: true,
            saw_defender_coalescence: true,
            saw_hall_of_fame: true,
            saw_scoring_surface: true,
            saw_final_scoring_instruction: true,
            saw_cycle_return: true,
            clean_exit: true,
            ..ActorAttractCycleSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor attract smoke passed\n"));
        assert!(text.contains("cycle_steps: 3479"));
        assert!(text.contains("saw_scoring_surface: true"));
        assert!(text.contains("saw_cycle_return: true"));
    }

    #[test]
    fn post_game_report_formats_current_cli_output() {
        let report = ActorPostGameSmokeReport {
            steps: 72,
            distinct_scene_signatures: 9,
            playing_steps: 8,
            high_score_entry_steps: 3,
            game_over_steps: 60,
            attract_steps: 1,
            forced_player_collisions: 3,
            final_score: 3_000,
            player_destroyed_events: 3,
            game_over_events: 1,
            high_score_entry_events: 3,
            high_score_initial_accept_events: 3,
            high_score_submit_events: 1,
            actor_sound_steps: 4,
            actor_sound_events: 6,
            game_over_sound_events: 1,
            hall_stall_steps: 60,
            saw_attract_return: true,
            saw_return_williams_reveal: true,
            saw_player_sprite: true,
            saw_pod_sprite: true,
            saw_explosion_pixels: true,
            saw_hall_of_fame: true,
            sprite_instances: 44,
            sprite_draw_commands: 22,
            wgpu_render_commands: 66,
            clean_exit: true,
            ..ActorPostGameSmokeReport::default()
        };

        let text = report.to_text();

        assert!(text.starts_with("actor post-game smoke passed\n"));
        assert!(text.contains("forced_player_collisions: 3"));
        assert!(text.contains("hall_stall_steps: 60"));
        assert!(text.contains("saw_attract_return: true"));
    }
}
