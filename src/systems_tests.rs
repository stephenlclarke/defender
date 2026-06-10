#[cfg(test)]
mod tests {
    use crate::game::{Direction, GameInput, ScoreSnapshot, WorldVector};

    use super::{
        CollisionBox, CollisionSystem, EnemyMotionSystem, Fixed24, FixedStepAccumulator, FrameRate,
        HighScoreEntrySystem, HighScoreInitialsState, OperatorActionTriggers,
        OperatorControlSystem, PlayerActionTriggers, PlayerControlIntent, PlayerControlSystem,
        PlayerDamageSystem, PlayerMotionState, PlayerMotionSystem, PlayerStock,
        ProjectileLaunchOutcome, ProjectileMotionSystem, ProjectileState, ProjectileSystem,
        ScoreSystem, ScreenPosition, ScreenVelocity, SmartBombSystem, VerticalControl, WaveState,
        WaveStatus, WaveSystem, clamp_camera_velocity_word, next_vertical_velocity,
        scroll_adjusted_x, thrust_acceleration,
    };

    #[test]
    fn frame_rate_uses_rounded_microsecond_duration() {
        assert_eq!(FrameRate::CABINET.millihz(), 60_100);
        assert_eq!(FrameRate::CABINET.frame_duration_micros(), 16_639);
    }

    #[test]
    fn fixed_step_accumulator_consumes_bounded_steps() {
        let mut accumulator = FixedStepAccumulator::new(FrameRate::from_millihz(1_000));
        accumulator.add_elapsed_micros(3_500_000);

        assert_eq!(accumulator.consume_due_steps(2), 2);
        assert_eq!(accumulator.accumulated_micros(), 1_500_000);
        assert_eq!(accumulator.consume_due_steps(8), 1);
        assert_eq!(accumulator.accumulated_micros(), 500_000);
    }

    #[test]
    fn player_control_intent_keeps_held_controls_separate_from_edges() {
        let intent = PlayerControlIntent::from_input(GameInput {
            altitude_up: true,
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(intent.vertical, VerticalControl::Up);
        assert!(intent.reverse);
        assert!(intent.thrust);
        assert!(intent.fire);
        assert!(intent.smart_bomb);
        assert!(intent.hyperspace);
    }

    #[test]
    fn player_control_system_requires_two_clear_samples_for_new_triggers() {
        let mut controls = PlayerControlSystem::new();
        let fire = GameInput {
            fire: true,
            ..GameInput::NONE
        };

        assert!(controls.step(fire).triggers.fire);
        assert_eq!(controls.step(fire).triggers, PlayerActionTriggers::NONE);
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert_eq!(controls.step(fire).triggers, PlayerActionTriggers::NONE);
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            PlayerActionTriggers::NONE
        );
        assert!(controls.step(fire).triggers.fire);
    }

    #[test]
    fn player_control_system_reports_all_playing_control_triggers() {
        let mut controls = PlayerControlSystem::new();
        let step = controls.step(GameInput {
            altitude_down: true,
            reverse: true,
            thrust: true,
            fire: true,
            smart_bomb: true,
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(
            step.triggers,
            PlayerActionTriggers {
                fire: true,
                thrust: true,
                smart_bomb: true,
                hyperspace: true,
                reverse: true,
                altitude_down: true,
            }
        );
        assert!(step.triggers.any());
    }

    #[test]
    fn operator_control_system_reports_edges_without_repeating_held_inputs() {
        let mut controls = OperatorControlSystem::new();
        let input = GameInput {
            service_auto_up: true,
            service_advance: true,
            high_score_reset: true,
            ..GameInput::NONE
        };

        assert_eq!(
            controls.step(input).triggers,
            OperatorActionTriggers {
                diagnostics: true,
                audits: true,
                high_score_reset: true,
            }
        );
        assert_eq!(controls.step(input).triggers, OperatorActionTriggers::NONE);
        assert!(!controls.step(input).triggers.any());
        assert_eq!(
            controls.step(GameInput::NONE).triggers,
            OperatorActionTriggers::NONE
        );
        assert_eq!(
            controls.step(input).triggers,
            OperatorActionTriggers {
                diagnostics: true,
                audits: true,
                high_score_reset: true,
            }
        );
    }

    #[test]
    fn player_motion_applies_thrust_damping_scroll_and_world_position() {
        let step = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                thrust: true,
                ..PlayerControlIntent::default()
            },
        );

        assert_eq!(word(step.state.position.0), 0x2000);
        assert_eq!(word(step.state.velocity.0), 0x0300);
        assert_eq!(word(step.state.camera_left), 0x0003);
        assert_eq!(word(step.world_x), 0x0803);
        assert_eq!(step.screen_position, ScreenPosition::new(0x20, 0x80));
        assert!(!step.blocked_by_vertical_limit);
    }

    #[test]
    fn player_motion_applies_vertical_priority_acceleration_and_limits() {
        let upward = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                vertical: VerticalControl::Up,
                ..PlayerControlIntent::default()
            },
        );

        assert_eq!(word(upward.state.velocity.1), 0xFF00);
        assert_eq!(word(upward.state.position.1), 0x7F00);
        assert_eq!(upward.screen_position, ScreenPosition::new(0x20, 0x7F));

        let blocked = PlayerMotionSystem::step(
            player_motion_state(0x2000, 0xEE00, 0, 0, Direction::Right, 0),
            PlayerControlIntent {
                vertical: VerticalControl::Down,
                ..PlayerControlIntent::default()
            },
        );

        assert!(blocked.blocked_by_vertical_limit);
        assert_eq!(word(blocked.state.position.1), 0xEE00);
        assert_eq!(word(blocked.state.velocity.1), 0);
    }

    #[test]
    fn player_motion_helpers_cover_direction_scroll_and_velocity_limits() {
        assert_eq!(Fixed24::new(0x0080_0000).to_bytes(), [0x80, 0x00, 0x00]);
        assert_eq!(
            Fixed24::from_bytes([0x01, 0x00, 0x00]).damped().to_bytes(),
            [0x00, 0xFC, 0x00]
        );
        assert_eq!(
            Fixed24::new(-0x0300).calculated_screen_x(Direction::Left),
            0x6F80
        );
        assert_eq!(
            Fixed24::new(0x0300).calculated_screen_x(Direction::Left),
            0x7000
        );
        assert_eq!(thrust_acceleration(Direction::Left), -0x0300);

        assert_eq!(scroll_adjusted_x(0x2000, 0x2080), (0x2080, 0));
        assert_eq!(scroll_adjusted_x(0x2000, 0x2200), (0x2100, 0x0040));
        assert_eq!(scroll_adjusted_x(0x2000, 0x1F80), (0x1F80, 0));
        assert_eq!(scroll_adjusted_x(0x2000, 0x1E00), (0x1F00, 0xFFC0));

        assert_eq!(clamp_camera_velocity_word(0x0200), 0x0100);
        assert_eq!(clamp_camera_velocity_word(0xFE00), 0xFF00);
        assert_eq!(clamp_camera_velocity_word(0x0080), 0x0080);

        assert_eq!(next_vertical_velocity(43, 0, VerticalControl::Up), None);
        assert_eq!(
            next_vertical_velocity(0x80, 0xFF00, VerticalControl::Up),
            Some(0xFEF8)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0xFE00, VerticalControl::Up),
            Some(0xFE00)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0, VerticalControl::Down),
            Some(0x0100)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0x0100, VerticalControl::Down),
            Some(0x0108)
        );
        assert_eq!(
            next_vertical_velocity(0x80, 0x0200, VerticalControl::Down),
            Some(0x0200)
        );
    }

    #[test]
    fn projectile_launch_uses_player_edge_and_caps_active_projectiles() {
        let started = ProjectileSystem::try_launch(
            ProjectileState::new(0),
            ScreenPosition::new(0x40, 0x50),
            Direction::Right,
        );

        assert_eq!(
            started,
            ProjectileLaunchOutcome::Started {
                state: ProjectileState::new(1),
                direction: Direction::Right,
                spawn: ScreenPosition::new(0x47, 0x54),
            }
        );
        assert_eq!(
            ProjectileSystem::try_launch(
                ProjectileState::new(3),
                ScreenPosition::new(0x40, 0x50),
                Direction::Left,
            ),
            ProjectileLaunchOutcome::Started {
                state: ProjectileState::new(4),
                direction: Direction::Left,
                spawn: ScreenPosition::new(0x40, 0x54),
            }
        );
        assert_eq!(
            ProjectileSystem::try_launch(
                ProjectileState::new(ProjectileSystem::MAX_ACTIVE_PROJECTILES),
                ScreenPosition::new(0x40, 0x50),
                Direction::Right,
            ),
            ProjectileLaunchOutcome::CapacityReached {
                state: ProjectileState::new(4),
            }
        );
    }

    #[test]
    fn projectile_motion_system_advances_directional_velocity_and_culls_screen_exit() {
        assert_eq!(
            ProjectileMotionSystem::velocity_for_direction(Direction::Right),
            ScreenVelocity::new(4, 0)
        );
        assert_eq!(
            ProjectileMotionSystem::velocity_for_direction(Direction::Left),
            ScreenVelocity::new(-4, 0)
        );

        let moved = ProjectileMotionSystem::step(
            ScreenPosition::new(0x40, 0x50),
            ScreenVelocity::new(4, 0),
        );

        assert_eq!(moved.position, ScreenPosition::new(0x44, 0x50));
        assert_eq!(moved.velocity, ScreenVelocity::new(4, 0));
        assert!(moved.active);

        let right_edge = ProjectileMotionSystem::step(
            ScreenPosition::new(0xFB, 0x50),
            ScreenVelocity::new(4, 0),
        );
        assert_eq!(right_edge.position, ScreenPosition::new(0xFF, 0x50));
        assert!(right_edge.active);

        let left_edge = ProjectileMotionSystem::step(
            ScreenPosition::new(0x04, 0x50),
            ScreenVelocity::new(-4, 0),
        );
        assert_eq!(left_edge.position, ScreenPosition::new(0x00, 0x50));
        assert!(left_edge.active);

        let off_right = ProjectileMotionSystem::step(
            ScreenPosition::new(0xFE, 0x50),
            ScreenVelocity::new(4, 0),
        );
        let off_left = ProjectileMotionSystem::step(
            ScreenPosition::new(0x01, 0x50),
            ScreenVelocity::new(-4, 0),
        );

        assert!(!off_right.active);
        assert!(!off_left.active);
    }

    #[test]
    fn enemy_motion_system_advances_and_wraps_screen_positions() {
        let moved =
            EnemyMotionSystem::step(ScreenPosition::new(204, 84), ScreenVelocity::new(-1, 2));

        assert_eq!(moved.position, ScreenPosition::new(203, 86));
        assert_eq!(moved.velocity, ScreenVelocity::new(-1, 2));

        let wrapped =
            EnemyMotionSystem::step(ScreenPosition::new(0, 255), ScreenVelocity::new(-2, 1));

        assert_eq!(wrapped.position, ScreenPosition::new(254, 0));
    }

    #[test]
    fn collision_boxes_detect_overlap_without_touching_edges() {
        let projectile = CollisionBox::new(ScreenPosition::new(40, 50), (8, 2));

        assert!(projectile.overlaps(CollisionBox::new(ScreenPosition::new(47, 51), (12, 8))));
        assert!(!projectile.overlaps(CollisionBox::new(ScreenPosition::new(48, 51), (12, 8))));
        assert!(!projectile.overlaps(CollisionBox::new(ScreenPosition::new(47, 52), (12, 8))));
    }

    #[test]
    fn collision_system_reports_first_projectile_enemy_hit() {
        let projectiles = [
            CollisionBox::new(ScreenPosition::new(10, 10), (8, 2)),
            CollisionBox::new(ScreenPosition::new(40, 50), (8, 2)),
        ];
        let enemies = [
            CollisionBox::new(ScreenPosition::new(80, 80), (12, 8)),
            CollisionBox::new(ScreenPosition::new(47, 51), (12, 8)),
        ];

        assert_eq!(
            CollisionSystem::first_projectile_enemy_hit(&projectiles, &enemies),
            Some(super::ProjectileEnemyHit {
                projectile_index: 1,
                enemy_index: 1,
            })
        );
        assert_eq!(
            CollisionSystem::first_projectile_enemy_hit(&projectiles[..1], &enemies[..1]),
            None
        );
    }

    #[test]
    fn collision_system_reports_first_player_enemy_hit() {
        let player = CollisionBox::new(ScreenPosition::new(30, 40), (16, 8));
        let enemies = [
            CollisionBox::new(ScreenPosition::new(2, 2), (12, 8)),
            CollisionBox::new(ScreenPosition::new(42, 44), (12, 8)),
        ];

        let hit = CollisionSystem::first_player_enemy_hit(player, &enemies)
            .expect("player should overlap second enemy");

        assert_eq!(hit.enemy_index, 1);
        assert_eq!(
            CollisionSystem::first_player_enemy_hit(
                CollisionBox::new(ScreenPosition::new(100, 100), (16, 8)),
                &enemies
            ),
            None
        );
    }

    #[test]
    fn wave_system_reports_progress_or_next_wave() {
        assert_eq!(
            WaveSystem::evaluate(WaveState::new(1, 2)),
            WaveStatus::InProgress
        );
        assert_eq!(
            WaveSystem::evaluate(WaveState::new(1, 0)),
            WaveStatus::Cleared { next_wave: 2 }
        );
        assert_eq!(
            WaveSystem::evaluate(WaveState::new(0, 0)),
            WaveStatus::Cleared { next_wave: 1 }
        );
        assert_eq!(
            WaveSystem::evaluate(WaveState::new(u8::MAX, 0)),
            WaveStatus::Cleared { next_wave: u8::MAX }
        );
    }

    #[test]
    fn score_system_awards_points_to_current_player_and_tracks_high_score() {
        let scores = ScoreSnapshot {
            player_one: 1_000,
            player_two: 2_000,
            high_score: 2_000,
            next_bonus: 10_000,
        };

        let player_one = ScoreSystem::award_points(scores, PlayerStock::new(3, 3), 1, 150);
        assert_eq!(player_one.scores.player_one, 1_150);
        assert_eq!(player_one.scores.player_two, 2_000);
        assert_eq!(player_one.scores.high_score, 2_000);
        assert_eq!(player_one.bonus_awards, 0);

        let player_two = ScoreSystem::award_points(scores, PlayerStock::new(3, 3), 2, 150);
        assert_eq!(player_two.scores.player_one, 1_000);
        assert_eq!(player_two.scores.player_two, 2_150);
        assert_eq!(player_two.scores.high_score, 2_150);
    }

    #[test]
    fn score_system_awards_bonus_stock_when_thresholds_are_crossed() {
        let scores = ScoreSnapshot {
            player_one: 9_900,
            player_two: 0,
            high_score: 9_900,
            next_bonus: 10_000,
        };

        let step = ScoreSystem::award_points(scores, PlayerStock::new(3, 2), 1, 20_250);

        assert_eq!(step.scores.player_one, 30_150);
        assert_eq!(step.scores.high_score, 30_150);
        assert_eq!(step.scores.next_bonus, 40_000);
        assert_eq!(step.stock, PlayerStock::new(6, 5));
        assert_eq!(step.bonus_awards, 3);
    }

    #[test]
    fn score_system_saturates_scores_bonus_stock_and_thresholds() {
        let scores = ScoreSnapshot {
            player_one: u32::MAX - 10,
            player_two: 0,
            high_score: u32::MAX - 20,
            next_bonus: u32::MAX - 1,
        };

        let step = ScoreSystem::award_points(scores, PlayerStock::new(u8::MAX, 254), 1, 50);

        assert_eq!(step.scores.player_one, u32::MAX);
        assert_eq!(step.scores.high_score, u32::MAX);
        assert_eq!(step.scores.next_bonus, u32::MAX);
        assert_eq!(step.stock, PlayerStock::new(u8::MAX, u8::MAX));
        assert_eq!(step.bonus_awards, 1);

        let max_bonus = ScoreSystem::award_points(step.scores, PlayerStock::new(3, 3), 1, 1_000);
        assert_eq!(max_bonus.bonus_awards, 0);
        assert_eq!(max_bonus.stock, PlayerStock::new(3, 3));
    }

    #[test]
    fn high_score_entry_system_qualifies_positive_scores_above_high_score() {
        assert!(!HighScoreEntrySystem::evaluate(10_000, 10_000).qualifies);
        assert!(HighScoreEntrySystem::evaluate(10_100, 10_000).qualifies);
        assert!(!HighScoreEntrySystem::evaluate(9_900, 10_000).qualifies);
        assert!(!HighScoreEntrySystem::evaluate(0, 0).qualifies);
    }

    #[test]
    fn high_score_entry_system_accepts_backspaces_and_submits_initials() {
        let first =
            HighScoreEntrySystem::enter_initial(HighScoreInitialsState::EMPTY, Some('a'), false);
        assert_eq!(first.state.initials, [Some('A'), None, None]);
        assert_eq!(first.state.cursor, 1);
        assert!(first.accepted);
        assert!(!first.submitted);

        let ignored = HighScoreEntrySystem::enter_initial(first.state, Some('1'), false);
        assert_eq!(ignored.state, first.state);
        assert!(!ignored.accepted);
        assert!(!ignored.submitted);

        let erased = HighScoreEntrySystem::enter_initial(ignored.state, None, true);
        assert_eq!(erased.state, HighScoreInitialsState::EMPTY);
        assert!(!erased.accepted);
        assert!(!erased.submitted);

        let second = HighScoreEntrySystem::enter_initial(erased.state, Some('b'), false).state;
        let third = HighScoreEntrySystem::enter_initial(second, Some('c'), false).state;
        let submitted = HighScoreEntrySystem::enter_initial(third, Some('d'), false);
        assert_eq!(submitted.state.initials, [Some('B'), Some('C'), Some('D')]);
        assert_eq!(submitted.state.cursor, 3);
        assert!(submitted.accepted);
        assert!(submitted.submitted);
    }

    #[test]
    fn smart_bomb_system_reports_all_active_enemies_destroyed() {
        assert_eq!(SmartBombSystem::detonate(3).destroyed_enemies, 3);
        assert_eq!(SmartBombSystem::detonate(0).destroyed_enemies, 0);
    }

    #[test]
    fn player_damage_system_decrements_lives_and_reports_game_over() {
        let survived = PlayerDamageSystem::apply_hit(PlayerStock::new(3, 2));
        assert_eq!(survived.stock, PlayerStock::new(2, 2));
        assert!(!survived.game_over);

        let final_life = PlayerDamageSystem::apply_hit(PlayerStock::new(1, 2));
        assert_eq!(final_life.stock, PlayerStock::new(0, 2));
        assert!(final_life.game_over);

        let already_empty = PlayerDamageSystem::apply_hit(PlayerStock::new(0, 2));
        assert_eq!(already_empty.stock, PlayerStock::new(0, 2));
        assert!(already_empty.game_over);
    }

    fn player_motion_state(
        x: u16,
        y: u16,
        x_velocity: i32,
        y_velocity: i16,
        direction: Direction,
        camera_left: u16,
    ) -> PlayerMotionState {
        PlayerMotionState::new(
            (unsigned_vector(x), unsigned_vector(y)),
            (
                WorldVector::from_subpixels(x_velocity << 8),
                WorldVector::from_subpixels(i32::from(y_velocity) << 8),
            ),
            direction,
            unsigned_vector(camera_left),
        )
    }

    fn unsigned_vector(word: u16) -> WorldVector {
        WorldVector::from_subpixels(i32::from(word) << 8)
    }

    fn word(vector: WorldVector) -> u16 {
        (vector.subpixels() >> 8) as u16
    }
}
