    #[test]
    fn smart_bomb_input_without_stock_does_not_clear_hostiles() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let report = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(report.sounds.is_empty());
        assert!(
            !report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::SmartBomb { .. }))
        );
    }

    #[test]
    fn xyzzy_overlay_smart_bomb_does_not_consume_stock() {
        let mut driver = started_driver();
        driver.smart_bombs = 0;

        let pressed = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                overlay_smart_bomb: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(pressed.smart_bombs, 0);
        assert_eq!(
            pressed.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert!(pressed.survivor_bonus.is_none());
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(pressed.sounds.is_empty());
        assert!(pressed.commands.contains(&GameCommand::SmartBomb {
            consume_stock: false,
        }));

        let detonated = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(detonated.score, LANDER_SCORE * 5);
        assert_eq!(detonated.smart_bombs, 0);
        assert_eq!(
            detonated.smart_bomb_flash_steps_remaining,
            SMART_BOMB_FLASH_STEPS
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        let blocked_restore = driver.step(GameInput::NONE);
        assert_eq!(blocked_restore.enemy_reserve, detonated.enemy_reserve);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        let restored = step_until_driver_reserve_activation_spawns_lander(&mut driver);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
    }

    #[test]
    fn default_wave_script_uses_wave_tuning_table_values() {
        let script = ActorWaveScript::default_progression();
        assert_eq!(script.name(), "actor-reference-wave-table");
        let first_wave_tuning = ActorWaveTuning::for_wave(1);
        assert_eq!(first_wave_tuning.baiter_delay, 192);
        assert_eq!(first_wave_tuning.baiter_shot_time, 10);
        assert_eq!(first_wave_tuning.baiter_seek_probability, 200);
        assert_eq!(first_wave_tuning.mutant_random_y, 1);
        assert_eq!(first_wave_tuning.mutant_y_velocity, 0x0080);
        assert_eq!(first_wave_tuning.mutant_x_velocity, 32);
        assert_eq!(first_wave_tuning.mutant_shot_time, 32);

        let first = script.profile_for_wave(1);
        assert_eq!(first.wave_tuning, Some(first_wave_tuning));
        let first_lander = first
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(first.lander_spawns.len(), 5);
        assert_eq!(
            first.lander_spawn_points(),
            vec![
                Point::new(0xFB, 0x2C),
                Point::new(0x3F, 0x2C),
                Point::new(0x67, 0x2C),
                Point::new(0x0D, 0x2C),
                Point::new(0x41, 0x2C),
            ]
        );
        assert_eq!(first.human_spawns.len(), 10);
        assert!(first.bomber_spawns.is_empty());
        assert!(first.pod_spawns.is_empty());
        assert_eq!(
            first.human_spawn_points(),
            vec![
                Point::new(0x18, 0xE0),
                Point::new(0x1C, 0xE1),
                Point::new(0x4E, 0xE0),
                Point::new(0x57, 0xE0),
                Point::new(0x9B, 0xE0),
                Point::new(0x9D, 0xE0),
                Point::new(0xCE, 0xE0),
                Point::new(0xD7, 0xE0),
                Point::new(0xD2, 0xE0),
                Point::new(0xE8, 0xE0),
            ]
        );
        assert_eq!(
            first.human_spawns[1].reference_state,
            Some(HumanReferenceState {
                x_fraction: 0x81,
                y_fraction: 0x00,
                animation_frame: crate::SpriteFrameIndex::new(3),
                target_slot_index: 1,
            })
        );
        assert_eq!(
            first.lander_spawns[0].reference_state,
            Some(LanderReferenceState {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x04,
                animation_frame: crate::SpriteFrameIndex::new(1),
                target_human_index: Some(1),
            })
        );
        assert_eq!(
            first.lander_spawns[3].reference_state,
            Some(LanderReferenceState {
                x_fraction: 0x11,
                y_fraction: 0x70,
                x_velocity: 0x0014,
                y_velocity: 0x0070,
                shot_timer: 0x3C,
                sleep_ticks: 0x04,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: Some(4),
            })
        );
        assert_eq!(first_lander.lander_seek_speed, 2);
        assert_eq!(first_lander.lander_fire_period_steps, 64);

        let second = script.profile_for_wave(2);
        assert_eq!(
            second.wave_tuning,
            Some(ActorWaveTuning::for_wave(2))
        );
        let second_lander = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        let second_bomber = second
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Bomber);
        assert_eq!(second.lander_spawns.len(), 3);
        assert_eq!(
            second.lander_spawn_points(),
            vec![
                Point::new(0xD2, 0x2C),
                Point::new(0x1A, 0x2C),
                Point::new(0xE3, 0x2C),
            ]
        );
        assert_eq!(
            second.lander_spawns[0].reference_state,
            Some(LanderReferenceState {
                x_fraction: 0xAD,
                y_fraction: 0,
                x_velocity: 0x001E,
                y_velocity: 0x00B0,
                shot_timer: 0x21,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: Some(1),
            })
        );
        assert_eq!(
            second.lander_spawns[1].reference_state,
            Some(LanderReferenceState {
                x_fraction: 0x55,
                y_fraction: 0,
                x_velocity: 0xFFDE,
                y_velocity: 0x00B0,
                shot_timer: 0x2F,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: Some(2),
            })
        );
        assert_eq!(
            second.lander_spawns[2].reference_state,
            Some(LanderReferenceState {
                x_fraction: 0x4A,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0x00B0,
                shot_timer: 0x1D,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: Some(3),
            })
        );
        assert_eq!(second.bomber_spawns.len(), 1);
        assert_eq!(second.bomber_spawn_points(), vec![Point::new(228, 104)]);
        assert_eq!(
            second.bomber_spawns[0].reference_state,
            Some(BomberReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0xFFD8,
                y_velocity: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: 1,
            })
        );
        assert_eq!(second.pod_spawns.len(), 1);
        assert_eq!(second.pod_spawn_points(), vec![Point::new(184, 72)]);
        assert_eq!(
            second.pod_spawns[0].reference_state,
            Some(PodReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
            })
        );
        assert_eq!(
            second.human_spawn_points(),
            vec![
                Point::new(0x12, 0xE0),
                Point::new(0x09, 0xE0),
                Point::new(0x54, 0xE0),
                Point::new(0x5A, 0xE0),
                Point::new(0x8D, 0xE0),
                Point::new(0x86, 0xE0),
                Point::new(0xC3, 0xE0),
                Point::new(0xD1, 0xE0),
                Point::new(0x09, 0xE0),
                Point::new(0x14, 0xE0),
            ]
        );
        assert_eq!(
            second.human_spawns[0].reference_state,
            Some(HumanReferenceState {
                x_fraction: 0xAD,
                y_fraction: 0,
                animation_frame: crate::SpriteFrameIndex::new(2),
                target_slot_index: 0,
            })
        );
        assert_eq!(
            second.human_spawns[9].reference_state,
            Some(HumanReferenceState {
                x_fraction: 0x69,
                y_fraction: 0,
                animation_frame: crate::SpriteFrameIndex::new(2),
                target_slot_index: 9,
            })
        );
        assert_eq!(
            second.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(second_lander.lander_seek_speed, 2);
        assert_eq!(second_lander.lander_fire_period_steps, 48);
        assert_eq!(second_bomber.bomber_drift_speed, 1);

        let fifth = script.profile_for_wave(5);
        assert_eq!(fifth.wave_tuning, Some(ActorWaveTuning::for_wave(5)));
        let fifth_lander = fifth
            .behavior_script
            .behavior_for(ActorId::new(1), ActorKind::Lander);
        assert_eq!(fifth_lander.lander_seek_speed, 3);
        assert_eq!(fifth_lander.lander_fire_period_steps, 30);
    }

    #[test]
    fn first_wave_early_reserve_lander_spawns_match_reference_rows() {
        let rows = ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS
            .iter()
            .copied()
            .map(reference_lander_spawn_row_for_test)
            .collect::<Vec<_>>();

        assert_eq!(
            rows,
            vec![
                (0x689A, 0x2C70, 0x001E, 0x0070, 0x10, 0, 1, Some(7)),
                (0x43D3, 0x2C70, 0xFFEC, 0x0070, 0x3A, 0, 1, Some(9)),
                (0x1F51, 0x2C70, 0x0014, 0x0070, 0x13, 0, 0, Some(8)),
                (0xFA03, 0x2C70, 0x0016, 0x0070, 0x26, 0, 1, Some(7)),
                (0xCF34, 0x2CE0, 0, 0, 0x34, 1, 0, Some(6)),
            ]
        );
        assert_eq!(FIRST_WAVE_EARLY_RESERVE_TARGET2_SHOT_PHASE_DELAY, 2);
    }

    #[test]
    fn first_wave_refill_lander_spawns_match_reference_rows() {
        let rows = ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS
            .iter()
            .copied()
            .map(reference_lander_spawn_row_for_test)
            .collect::<Vec<_>>();

        assert_eq!(
            rows,
            vec![
                (0xBC29, 0x2CFD, 0x001C, 0x0090, 0x36, 6, 1, Some(7)),
                (0xE14C, 0x2CAE, 0x000E, 0x0090, 0x2F, 0, 0, Some(4)),
                (0x0A63, 0x2CF0, 0xFFF4, 0x0090, 0x23, 1, 0, Some(3)),
                (0x531B, 0x2CC0, 0xFFF6, 0x0090, 0x30, 1, 0, Some(2)),
                (0x98D9, 0x2CB8, 0x001A, 0x0090, 0x1F, 1, 0, Some(1)),
            ]
        );
        assert_eq!(FIRST_WAVE_LANDER_REFILL_DELAY_STEPS, 47);
        assert_eq!(FIRST_WAVE_LANDER_REFILL_APPEAR_SOUND_DELAY_STEPS, 1);
    }

    #[test]
    fn embedded_actor_wave_script_expands_wave_tuning_range() {
        let parsed = ActorWaveScript::parse_text(ACTOR_WAVE_SCRIPT)
            .expect("embedded actor wave script should parse");

        assert_eq!(
            ActorWaveScript::default_progression().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            ActorWaveScript::reference_table_progression().manifest(),
            parsed.manifest()
        );
        assert_eq!(parsed.name(), "actor-reference-wave-table");
        assert_eq!(
            parsed.manifest().waves.len(),
            usize::from(ACTOR_DATA_BACKED_WAVES)
        );
        assert_eq!(
            parsed.manifest().waves[0].wave_tuning,
            Some(ActorWaveTuning::for_wave(1))
        );
        assert_eq!(
            parsed.manifest().waves[4].wave_tuning,
            Some(ActorWaveTuning::for_wave(5))
        );
        assert_eq!(parsed.profile_for_wave(1).lander_spawns.len(), 5);
        assert_eq!(
            parsed.profile_for_wave(1).enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(parsed.profile_for_wave(2).bomber_spawns.len(), 1);
        assert_eq!(parsed.profile_for_wave(2).pod_spawns.len(), 1);
    }

    #[test]
    fn second_wave_tuning_spawns_bomber_and_pod_actor_families() {
        let (driver, live) = started_wave_tuning_driver(2);
        assert_eq!(live.wave, 2);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 3);
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        assert_eq!(
            live.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(live.game_state().world.enemy_reserve, live.enemy_reserve);
        let bomber_snapshot = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomber)
            .expect("second wave should publish bomber runtime snapshot");
        let (expected_bomber_position, expected_bomber_reference_state) =
            expected_bomber_after_runtime_motion(
                Point::new(228, 104),
                BomberReferenceState {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0xFFD8,
                    y_velocity: 0,
                    animation_frame: crate::SpriteFrameIndex::new(0),
                    cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                    sleep_ticks: 0,
                    slot: 1,
                },
                live.step,
                bomber_snapshot.id,
                live.actor_rng,
                None,
            );
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Lander
                && snapshot.position == Point::new(0xD2, 0x2C)
                && snapshot.reference_state.as_lander()
                    == Some(LanderReferenceState {
                        x_fraction: 0xCB,
                        y_fraction: 0xB0,
                        x_velocity: 0x001E,
                        y_velocity: 0x00B0,
                        shot_timer: 0x20,
                        sleep_ticks: 0,
                        animation_frame: crate::SpriteFrameIndex::new(0),
                        target_human_index: Some(1),
                    })
        }));
        assert_eq!(bomber_snapshot.position, expected_bomber_position);
        assert_eq!(
            bomber_snapshot.reference_state.as_bomber(),
            Some(expected_bomber_reference_state)
        );
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Pod
                && snapshot.reference_state.as_pod()
                    == Some(PodReferenceState {
                        x_fraction: 0x20,
                        y_fraction: 0,
                        x_velocity: 0x0020,
                        y_velocity: 0,
                    })
        }));
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Bomber
                    && matches!(
                        draw.effect,
                        VisualEffect::BomberSpriteFrame { animation_frame }
                            if animation_frame == expected_bomber_reference_state.animation_frame
                    ))
        );
        assert!(
            live.draws.iter().any(|draw| draw.sprite == SpriteKey::Pod
                && matches!(draw.effect, VisualEffect::PodSprite))
        );
    }

    #[test]
    fn actor_runtime_reserve_landers_activate_before_wave_clear() {
        let (mut driver, live) = started_wave_tuning_driver(2);
        assert_eq!(
            live.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );

        destroy_wave_hostiles(&mut driver, &live);
        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.phase, Phase::Playing);
        assert!(
            !restored
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 3 })
        );
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 12,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            restored.game_state().world.enemy_reserve,
            restored.enemy_reserve
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            MAX_ACTIVE_WAVE_ENEMIES
        );
        let runtime_landers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        assert_eq!(runtime_landers.len(), MAX_ACTIVE_WAVE_ENEMIES);
        assert!(
            runtime_landers
                .iter()
                .all(|snapshot| snapshot.reference_state.as_lander().is_some())
        );
        assert!(runtime_landers.iter().any(|snapshot| {
            snapshot
                .reference_state.as_lander()
                .is_some_and(|reference_state| reference_state.target_human_index == Some(4))
        }));
    }

    #[test]
    fn actor_first_wave_early_lander_reserve_materializes_on_reference_cadence() {
        let mut driver = started_driver();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_takes_enemy_collision_damage: false,
                ..ActorBehaviorProfile::default()
            },
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(
            driver.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            driver.first_wave_early_reserve_steps_remaining,
            Some(FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS)
        );

        let mut materialized = None;
        for offset in 1..=FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            let spawn_count = report
                .commands
                .iter()
                .filter(|command| {
                    matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. }))
                })
                .count();
            if spawn_count > 0 || report.sounds.contains(&SoundCue::HyperspaceMaterialize) {
                materialized = Some((offset, report));
                break;
            }

            assert_eq!(spawn_count, 0);
            assert!(!report.sounds.contains(&SoundCue::HyperspaceMaterialize));
        }

        let (offset, report) = materialized.unwrap_or_else(|| {
            panic!(
                "first-wave early lander reserve should materialize on reference cadence; \
                 ready={} cooldown={} early={:?} reserve={:?} hostiles={} phase={:?}",
                driver.reserve_activation_ready,
                driver.reserve_activation_cooldown_steps,
                driver.first_wave_early_reserve_steps_remaining,
                driver.enemy_reserve,
                driver.wave_hostile_snapshot_count(),
                driver.phase
            )
        });
        assert_eq!(offset, FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS);
        assert!(report.sounds.contains(&SoundCue::HyperspaceMaterialize));
        assert!(
            !report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        assert_eq!(
            report
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            ACTOR_FIRST_WAVE_EARLY_RESERVE_LANDER_SPAWNS.len()
        );
        assert_eq!(
            report.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            report.game_state().world.enemy_reserve,
            report.enemy_reserve
        );
        assert_eq!(
            report
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            10
        );
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Lander
                && snapshot.reference_state.as_lander().is_some_and(|reference_state| {
                    reference_state.target_human_index == Some(FIRST_WAVE_EARLY_RESERVE_TARGET_CURSOR_SLOT)
                        && reference_state.x_velocity == 0
                        && reference_state.y_velocity == 0
                })
        }));
    }

    #[test]
    fn actor_first_wave_lander_refill_keeps_hidden_lanes_suppressed() {
        let mut driver = started_driver();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_takes_enemy_collision_damage: false,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_mode: LanderBehaviorMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let early_reserve = step_until_first_wave_early_reserve_materializes(&mut driver);
        assert_eq!(
            early_reserve.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            early_reserve
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            10
        );

        let destroy_three_landers = early_reserve
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .take(3)
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&destroy_three_landers);

        let scheduled = driver.step(GameInput::NONE);
        assert_eq!(
            scheduled
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            7
        );
        assert_eq!(
            driver.first_wave_lander_refill_steps_remaining,
            Some(FIRST_WAVE_LANDER_REFILL_DELAY_STEPS)
        );

        let mut materialized = None;
        for offset in 1..=FIRST_WAVE_LANDER_REFILL_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            let spawn_count = report
                .commands
                .iter()
                .filter(|command| {
                    matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. }))
                })
                .count();
            if spawn_count > 0 {
                materialized = Some((offset, report));
                break;
            }

            assert_eq!(spawn_count, 0);
        }

        let (offset, report) =
            materialized.expect("first-wave refill should materialize on reference cadence");
        assert_eq!(offset, FIRST_WAVE_LANDER_REFILL_DELAY_STEPS);
        assert_eq!(
            report
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            ACTOR_FIRST_WAVE_REFILL_LANDER_SPAWNS.len()
        );
        assert_eq!(report.enemy_reserve, EnemyReserveSnapshot::default());

        let refill_landers = report
            .snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.kind == ActorKind::Lander
                    && snapshot
                        .reference_state.as_lander()
                        .is_some_and(|reference_state| reference_state.y_velocity == 0x0090)
            })
            .collect::<Vec<_>>();
        assert_eq!(refill_landers.len(), 5);
        assert_eq!(
            refill_landers
                .iter()
                .filter(|snapshot| snapshot.bounds.is_some())
                .count(),
            1
        );
        let visible_refill = refill_landers
            .iter()
            .find(|snapshot| snapshot.bounds.is_some())
            .expect("target-3 refill lane should be visible");
        assert_eq!(
            visible_refill.reference_state.as_lander(),
            Some(LanderReferenceState {
                x_fraction: 0x63,
                y_fraction: 0xF0,
                x_velocity: 0xFFF4,
                y_velocity: 0x0090,
                shot_timer: 0x23,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: Some(3),
            })
        );
        assert_eq!(
            report
                .game_state()
                .world
                .enemies
                .iter()
                .filter(|enemy| {
                    matches!(enemy.kind, CleanEnemyKind::Lander)
                        && enemy
                            .lander_reference_state
                            .is_some_and(|reference_state| reference_state.y_velocity == 0x0090)
                })
                .count(),
            1
        );

        let refill_ids = refill_landers
            .iter()
            .map(|snapshot| snapshot.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            report
                .draws
                .iter()
                .filter(|draw| refill_ids.contains(&draw.actor) && draw.sprite == SpriteKey::Lander)
                .count(),
            1
        );

        let delayed_sound = driver.step(GameInput::NONE);
        assert!(
            delayed_sound
                .sounds
                .contains(&SoundCue::SoundBoardCommand(APPEARANCE_SOUND_COMMAND))
        );
    }

    #[test]
    fn hidden_first_wave_refill_lanes_do_not_block_wave_advance() {
        let mut driver = started_driver();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_takes_enemy_collision_damage: false,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_mode: LanderBehaviorMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let early_reserve = step_until_first_wave_early_reserve_materializes(&mut driver);
        let destroy_three_landers = early_reserve
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .take(3)
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&destroy_three_landers);
        driver.step(GameInput::NONE);

        let mut refill = None;
        for _ in 0..=FIRST_WAVE_LANDER_REFILL_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. })))
            {
                refill = Some(report);
                break;
            }
        }
        let refill = refill.expect("first-wave refill should materialize before clear proof");
        assert!(
            refill.snapshots.iter().any(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.bounds.is_none()
            })
        );

        let destroy_visible_hostiles = refill
            .snapshots
            .iter()
            .filter(|snapshot| snapshot_blocks_wave_clear(snapshot))
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        assert!(!destroy_visible_hostiles.is_empty());
        driver.apply_commands(&destroy_visible_hostiles);

        let cleared = driver.step(GameInput::NONE);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        assert!(cleared.survivor_bonus.is_some());
        assert!(
            cleared.snapshots.iter().all(|snapshot| {
                snapshot.kind != ActorKind::Lander || snapshot.bounds.is_some()
            })
        );
        assert!(cleared.game_state().world.enemies.iter().all(|enemy| {
            !matches!(enemy.kind, CleanEnemyKind::Lander)
                || enemy
                    .lander_reference_state
                    .is_none_or(|reference_state| reference_state.y_velocity != 0x0090)
        }));

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            next_wave
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        let next_scene = next_wave.render_scene();
        assert!(next_scene.sprites.iter().any(|sprite| {
            sprite.layer == RenderLayer::Terrain && sprite.tint == wave_tuning_landscape_tint(2)
        }));
        assert!(next_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD
                && sprite.position != screen_position_from_cell(crate::ScreenAddress::new(0x4C07))
                && sprite.position != screen_position_from_cell(crate::ScreenAddress::new(0x4C28))
                && sprite.tint == wave_tuning_landscape_tint(2)
        }));
    }

    #[test]
    fn actor_lander_reserve_without_humans_restores_runtime_mutants() {
        let (mut driver, live) = started_wave_tuning_driver(2);
        let player_position = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        let clear_playfield = live
            .snapshots
            .iter()
            .filter(|snapshot| is_hostile(snapshot.kind) || snapshot.kind == ActorKind::Human)
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&clear_playfield);
        driver.enemy_reserve = EnemyReserveSnapshot {
            landers: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.actor_rng = ActorRng {
            seed: 0x20,
            hseed: 0x66,
            lseed: 0x99,
        };
        driver.background_left = 0x3400;
        let mut expected_rng = driver.actor_rng;
        let first_spawn = ActorMutantSpawn::from_wave_restore(
            &mut expected_rng,
            ActorWaveTuning::for_wave(2),
            driver.background_left,
        );
        let second_spawn = ActorMutantSpawn::from_wave_restore(
            &mut expected_rng,
            ActorWaveTuning::for_wave(2),
            driver.background_left,
        );

        let restored = driver.step(GameInput::NONE);

        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(restored.background_left, 0x3400);
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            0
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Mutant { .. })
                ))
                .count(),
            2
        );
        assert_eq!(
            restored
                .actor_rng
                .expect("playing report should carry actor rng"),
            expected_rng.advance().snapshot()
        );

        let prompt = mutant_reference_state_prompt_for_test(
            restored.step,
            restored.wave,
            restored
                .actor_rng
                .expect("playing report should carry actor rng"),
            player_position,
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let runtime_mutants = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
            .collect::<Vec<_>>();
        assert_eq!(runtime_mutants.len(), 2);
        for (snapshot, spawn) in runtime_mutants.iter().zip([first_spawn, second_spawn]) {
            let (expected_position, expected_reference_state, _) = expected_mutant_reference_state_after_motion(
                spawn.position,
                spawn.reference_state.expect("mutant wave restore state"),
                snapshot.id,
                &prompt,
                behavior,
            );
            assert_eq!(snapshot.position, expected_position);
            assert_eq!(snapshot.reference_state.as_mutant(), Some(expected_reference_state));
            assert!(snapshot.reference_state.as_lander().is_none());
        }

        let followup = driver.step(GameInput::NONE);
        assert_eq!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Lander)
                .count(),
            0
        );
        assert_eq!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
                .count(),
            2
        );
        assert!(
            followup
                .snapshots
                .iter()
                .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
                .all(|snapshot| snapshot.reference_state.as_mutant().is_some())
        );
    }

    #[test]
    fn actor_mutant_reserves_use_wave_restore_state() {
        let (mut driver, seeded) = started_wave_tuning_driver(2);
        let player_position = seeded
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        destroy_wave_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            mutants: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.actor_rng = ActorRng {
            seed: 0x37,
            hseed: 0x5A,
            lseed: 0x91,
        };
        driver.background_left = 0x5420;
        let profile = ActorWaveTuning::for_wave(2);
        let mut expected_rng = driver.actor_rng;
        let first_spawn = ActorMutantSpawn::from_wave_restore(
            &mut expected_rng,
            profile,
            driver.background_left,
        );
        let second_spawn = ActorMutantSpawn::from_wave_restore(
            &mut expected_rng,
            profile,
            driver.background_left,
        );

        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.enemy_reserve, EnemyReserveSnapshot::default());
        assert_eq!(restored.background_left, 0x5420);
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Mutant { .. })
                ))
                .count(),
            2
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Lander { .. })
                ))
                .count(),
            0
        );
        assert_eq!(
            restored
                .actor_rng
                .expect("playing report should carry actor rng"),
            expected_rng.advance().snapshot()
        );

        let prompt = mutant_reference_state_prompt_for_test(
            restored.step,
            restored.wave,
            restored
                .actor_rng
                .expect("playing report should carry actor rng"),
            player_position,
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let mut runtime_mutants = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Mutant)
            .collect::<Vec<_>>();
        runtime_mutants.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(runtime_mutants.len(), 2);
        for (snapshot, spawn) in runtime_mutants.iter().zip([first_spawn, second_spawn]) {
            let (expected_position, expected_reference_state, _) = expected_mutant_reference_state_after_motion(
                spawn.position,
                spawn.reference_state.expect("mutant wave restore state"),
                snapshot.id,
                &prompt,
                behavior,
            );
            assert_eq!(snapshot.position, expected_position);
            assert_eq!(snapshot.reference_state.as_mutant(), Some(expected_reference_state));
            assert!(snapshot.reference_state.as_lander().is_none());
        }
    }

    #[test]
    fn actor_bomber_and_pod_reserves_use_wave_restore_state() {
        let (mut driver, seeded) = started_wave_tuning_driver(2);
        let player_position = seeded
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .expect("seed step should publish the player")
            .position;
        destroy_wave_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            bombers: 5,
            pods: 2,
            ..EnemyReserveSnapshot::default()
        };
        driver.actor_rng = ActorRng {
            seed: 0x12,
            hseed: 0x6D,
            lseed: 0x80,
        };
        let mut expected_rng = driver.actor_rng;
        let mut expected_pod = ActorPodSpawn::from_wave_restore(&mut expected_rng);
        if let Some(reference_state) = &mut expected_pod.reference_state {
            let (x, x_fraction) = step_motion_axis(
                expected_pod.position.x,
                reference_state.x_fraction,
                reference_state.x_velocity,
            );
            let (y, y_fraction) = step_wrapping_motion_y(
                expected_pod.position.y,
                reference_state.y_fraction,
                reference_state.y_velocity,
            );
            expected_pod.position = Point::new(x, y);
            reference_state.x_fraction = x_fraction;
            reference_state.y_fraction = y_fraction;
        }

        let restored = driver.step(GameInput::NONE);

        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                bombers: 1,
                pods: 1,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Bomber { .. })
                ))
                .count(),
            4
        );
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Pod { .. })))
                .count(),
            1
        );
        let bombers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomber)
            .collect::<Vec<_>>();
        assert_eq!(bombers.len(), 4);
        assert!(
            bombers
                .iter()
                .all(|snapshot| snapshot.reference_state.as_bomber().is_some())
        );
        assert!(bombers.iter().any(|snapshot| {
            let reference_state = snapshot.reference_state.as_bomber().expect("bomber runtime state");
            reference_state.x_velocity
                == actor_sign_extend_u8_to_u16(
                    ActorWaveTuning::for_wave(2).bomber_x_velocity,
                )
                && reference_state.slot == 0
        }));
        assert!(restored.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Pod
                && snapshot.position == expected_pod.position
                && snapshot.reference_state.as_pod() == expected_pod.reference_state
        }));
        assert!(
            restored
                .snapshots
                .iter()
                .filter_map(|snapshot| snapshot.reference_state.as_bomber())
                .any(|reference_state| {
                    let expected_spawn = ActorBomberSpawn::wave_restore_batch(
                        ActorWaveTuning::for_wave(2),
                        absolute_world_x(player_position, 0),
                        1,
                    )[0];
                    let expected_reference_state =
                        expected_spawn
                            .reference_state
                            .expect("expected bomber runtime state");
                    let (_, x_fraction) = step_motion_axis(
                        expected_spawn.position.x,
                        expected_reference_state.x_fraction,
                        expected_reference_state.x_velocity,
                    );
                    reference_state.x_fraction == x_fraction
                })
        );
    }

    #[test]
    fn actor_swarmer_reserves_use_wave_restore_state() {
        let (mut driver, seeded) = started_wave_tuning_driver(2);
        destroy_wave_hostiles(&mut driver, &seeded);
        driver.enemy_reserve = EnemyReserveSnapshot {
            swarmers: 4,
            ..EnemyReserveSnapshot::default()
        };
        driver.actor_rng = ActorRng {
            seed: 0x20,
            hseed: 0x41,
            lseed: 0xC0,
        };
        let profile = ActorWaveTuning::for_wave(2);
        let mut expected_rng = driver.actor_rng;
        let expected_spawns =
            ActorSwarmerSpawn::wave_restore_batch(&mut expected_rng, profile, 4);

        let restored = driver.step(GameInput::NONE);

        assert_eq!(restored.enemy_reserve, EnemyReserveSnapshot::default());
        assert_eq!(
            restored
                .commands
                .iter()
                .filter(|command| matches!(
                    command,
                    GameCommand::Spawn(SpawnRequest::Swarmer { .. })
                ))
                .count(),
            4
        );
        assert_eq!(
            restored
                .actor_rng
                .expect("playing report should carry actor rng"),
            expected_rng.advance().snapshot()
        );

        let mut runtime_swarmers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Swarmer)
            .collect::<Vec<_>>();
        runtime_swarmers.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(runtime_swarmers.len(), 4);
        for (snapshot, spawn) in runtime_swarmers.iter().zip(expected_spawns) {
            let mut expected_reference_state =
                spawn.reference_state.expect("swarmer wave restore metadata");
            expected_reference_state.sleep_ticks =
                expected_reference_state.sleep_ticks.saturating_sub(1);
            assert_eq!(snapshot.position, spawn.position);
            assert_eq!(snapshot.reference_state.as_swarmer(), Some(expected_reference_state));
        }
        assert!(runtime_swarmers.iter().all(|snapshot| {
            let reference_state = snapshot.reference_state.as_swarmer().expect("swarmer runtime state");
            snapshot.position == runtime_swarmers[0].position
                && reference_state.x_fraction == MINI_SWARMER_RESTORE_X_LOW
                && reference_state.y_fraction == 0
        }));
    }

    #[test]
    fn pod_y_motion_wraps_through_reference_playfield_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let top = driver.spawn_pod_from_spawn(ActorPodSpawn {
            position: Point::new(0xD0, i16::from(PLAYFIELD_TOP_EDGE_Y)),
            reference_state: Some(PodReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
            }),
        });
        let bottom = driver.spawn_pod_from_spawn(ActorPodSpawn {
            position: Point::new(0xE0, i16::from(PLAYFIELD_BOTTOM_EDGE_Y)),
            reference_state: Some(PodReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&report, top).position,
            Point::new(0xD0, i16::from(PLAYFIELD_BOTTOM_EDGE_Y))
        );
        assert_eq!(
            snapshot_for(&report, top).reference_state.as_pod(),
            Some(PodReferenceState {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
            })
        );
        assert_eq!(
            snapshot_for(&report, bottom).position,
            Point::new(0xE0, i16::from(PLAYFIELD_TOP_EDGE_Y))
        );
        assert_eq!(
            snapshot_for(&report, bottom).reference_state.as_pod(),
            Some(PodReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
            })
        );
    }

    #[test]
    fn hostile_y_motion_wraps_through_reference_playfield_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let lander = driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(0x70, i16::from(PLAYFIELD_TOP_EDGE_Y)),
            reference_state: Some(LanderReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 8,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: None,
            }),
            spawn_visibility: LanderSpawnVisibility::Normal,
        });
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: Point::new(0x80, i16::from(PLAYFIELD_BOTTOM_EDGE_Y)),
            reference_state: Some(SwarmerReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0x0100,
                acceleration: 0,
                sleep_ticks: 0,
                shot_timer: 3,
                horizontal_seek_pending: true,
            }),
        });
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(0x90, i16::from(PLAYFIELD_TOP_EDGE_Y)),
            reference_state: Some(BaiterReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 3,
                sleep_ticks: 1,
                animation_frame: crate::SpriteFrameIndex::new(0),
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&report, lander).position,
            Point::new(0x70, i16::from(PLAYFIELD_BOTTOM_EDGE_Y))
        );
        assert_eq!(
            snapshot_for(&report, lander).reference_state.as_lander(),
            Some(LanderReferenceState {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 7,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
                target_human_index: None,
            })
        );
        assert_eq!(
            snapshot_for(&report, swarmer).position,
            Point::new(0x7F, i16::from(PLAYFIELD_TOP_EDGE_Y))
        );
        assert_eq!(
            snapshot_for(&report, swarmer).reference_state.as_swarmer(),
            Some(SwarmerReferenceState {
                x_fraction: 0xE0,
                y_fraction: 0,
                x_velocity: 0xFFE0,
                y_velocity: 0x0100,
                acceleration: 0,
                sleep_ticks: MINI_SWARMER_LOOP_SLEEP_TICKS,
                shot_timer: 3,
                horizontal_seek_pending: false,
            })
        );
        assert_eq!(
            snapshot_for(&report, baiter).position,
            Point::new(0x90, i16::from(PLAYFIELD_BOTTOM_EDGE_Y))
        );
        assert_eq!(
            snapshot_for(&report, baiter).reference_state.as_baiter(),
            Some(BaiterReferenceState {
                x_fraction: 0,
                y_fraction: 0xFF,
                x_velocity: 0,
                y_velocity: 0xFFFF,
                shot_timer: 3,
                sleep_ticks: 0,
                animation_frame: crate::SpriteFrameIndex::new(0),
            })
        );
    }

    #[test]
    fn first_wave_landers_publish_reference_state_and_animation_frames() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let live = step_until_driver_player_start_completes(&mut driver, 1);
        let lander = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.position == Point::new(0xFB, 0x2C)
            })
            .expect("reference-state first-wave lander should publish its restore position");

        assert_eq!(
            lander.reference_state.as_lander(),
            Some(LanderReferenceState {
                x_fraction: 0x33,
                y_fraction: 0xE0,
                x_velocity: 0xFFDE,
                y_velocity: 0x0070,
                shot_timer: 0x27,
                sleep_ticks: 0x03,
                animation_frame: crate::SpriteFrameIndex::new(1),
                target_human_index: Some(1),
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == lander.id
                && draw.sprite == SpriteKey::Lander
                && matches!(
                    draw.effect,
                    VisualEffect::LanderSpriteFrame { animation_frame }
                        if animation_frame == crate::SpriteFrameIndex::new(1)
                )
        }));
    }

    #[test]
    fn lander_sleep_ticks_delay_first_wave_motion() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let initial = Point::new(0xFB, 0x2C);
        let mut lander_id = None;
        let mut current_report = Some(step_until_driver_player_start_completes(&mut driver, 1));
        for expected_sleep in [3, 2, 1, 0] {
            let sleeping = current_report
                .take()
                .unwrap_or_else(|| driver.step(GameInput::NONE));
            let lander = sleeping
                .snapshots
                .iter()
                .find(|snapshot| {
                    let matches_known_lander = match lander_id {
                        Some(id) => snapshot.id == id,
                        None => snapshot.position == initial,
                    };
                    snapshot.kind == ActorKind::Lander && matches_known_lander
                })
                .expect("sleeping reference-state lander should stay visible");
            lander_id = Some(lander.id);
            assert_eq!(lander.position, initial);
            assert_eq!(
                lander
                    .reference_state.as_lander()
                    .map(|reference_state| reference_state.sleep_ticks),
                Some(expected_sleep)
            );
        }

        let awake = driver.step(GameInput::NONE);
        let lander = snapshot_for(
            &awake,
            lander_id.expect("reference-state lander id should be known"),
        );
        assert_eq!(lander.position, Point::new(0xFB, 0x2D));
        assert_eq!(
            lander
                .reference_state.as_lander()
                .map(|reference_state| (reference_state.x_fraction, reference_state.y_fraction)),
            Some((0x11, 0x50))
        );
        assert_eq!(
            lander
                .reference_state.as_lander()
                .map(|reference_state| reference_state.sleep_ticks),
            Some(0)
        );
    }

    #[test]
    fn lander_shot_timer_controls_first_wave_sound_board_command() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut first_laser_step = None;
        let live = step_until_driver_player_start_completes(&mut driver, 1);
        if live.sounds.contains(&SoundCue::LanderShot) {
            first_laser_step = Some(1);
        }
        for live_step in 2..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.sounds.contains(&SoundCue::LanderShot) {
                first_laser_step = Some(live_step);
                break;
            }
        }

        assert_eq!(first_laser_step, Some(39));
    }

    #[test]
    fn runtime_lander_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        let mut shot_report = None;
        let live = step_until_driver_player_start_completes(&mut driver, 1);
        if live
            .commands
            .iter()
            .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::EnemyLaser { .. })))
        {
            shot_report = Some(live);
        }
        for _ in 2..=50 {
            let report = driver.step(GameInput {
                xyzzy: XyzzyMode {
                    active: true,
                    invincible: true,
                    ..XyzzyMode::INACTIVE
                },
                ..GameInput::NONE
            });
            if report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::EnemyLaser { .. }))
            }) {
                shot_report = Some(report);
                break;
            }
        }
        let shot_report = shot_report.expect("runtime lander should spawn a hostile shot");

        assert!(shot_report.sounds.contains(&SoundCue::LanderShot));
        let (shot_position, shot_velocity, shot_reference_state) = shot_report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    reference_state,
                }) => Some((*position, *velocity, *reference_state)),
                _ => None,
            })
            .expect("runtime lander should emit a hostile shot command");
        let lander_reference_state = shot_report
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Lander && snapshot.position == shot_position
            })
            .and_then(|snapshot| snapshot.reference_state.as_lander())
            .expect("runtime lander snapshot should own shot fractions");
        assert_eq!(
            shot_reference_state,
            Some(EnemyProjectileReferenceState {
                x_fraction: lander_reference_state.x_fraction,
                y_fraction: lander_reference_state.y_fraction,
                x_velocity: projectile_velocity_word(shot_velocity.dx),
                y_velocity: projectile_velocity_word(shot_velocity.dy),
                lifetime_ticks: projectile_lifetime_ticks(LANDER_SHOT_LIFETIME),
            })
        );
        let settled = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });
        let enemy_laser = settled
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .expect("spawned hostile shot should publish an actor snapshot");
        let projectile_reference_state = enemy_laser
            .reference_state.as_enemy_projectile()
            .expect("hostile shot should publish projectile runtime metadata");
        assert_eq!(
            projectile_reference_state.x_velocity,
            projectile_velocity_word(enemy_laser.velocity.dx)
        );
        assert_eq!(
            projectile_reference_state.y_velocity,
            projectile_velocity_word(enemy_laser.velocity.dy)
        );
        assert!(projectile_reference_state.lifetime_ticks > 0);
        assert!(
            settled
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::EnemyLaser)
        );
    }

    #[test]
    fn enemy_laser_actor_advances_fixed_point_motion_state() {
        let behavior = ActorBehaviorProfile {
            lander_shot_lifetime_steps: 4,
            ..ActorBehaviorProfile::default()
        };
        let prompt = StepPrompt {
            step: 1,
            phase: Phase::Playing,
            input: GameInput::NONE,
            wave: 1,
            wave_tuning: ActorWaveTuning::for_wave(1),
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_pending: false,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [0; 5],
            high_score_initials: HighScoreInitialsState::EMPTY,
            snapshots: Vec::new(),
            behavior_script: ActorBehaviorScript::default()
                .with_kind_behavior(ActorKind::EnemyLaser, behavior),
            background_left: 0,
            actor_rng: None,
            human_walk_target_slot: None,
            projectile_scan_tick: false,
        };
        let mut shot = EnemyLaserShot::new(
            ActorId::new(101),
            Point::new(10, 80),
            Velocity::new(1, -1),
            None,
        );
        shot.reference_state.x_velocity = 0x0180;
        shot.reference_state.y_velocity = 0xFF80;

        let first = shot.update(&prompt);

        assert_eq!(first.snapshot.position, Point::new(11, 79));
        assert_eq!(first.snapshot.velocity, Velocity::new(1, -1));
        assert_eq!(
            first.snapshot.reference_state.as_enemy_projectile(),
            Some(EnemyProjectileReferenceState {
                x_fraction: 0x80,
                y_fraction: 0x80,
                x_velocity: 0x0180,
                y_velocity: 0xFF80,
                lifetime_ticks: 4,
            })
        );

        let mut tick_prompt = prompt;
        tick_prompt.projectile_scan_tick = true;
        let second = shot.update(&tick_prompt);

        assert_eq!(second.snapshot.position, Point::new(13, 79));
        assert_eq!(second.snapshot.velocity, Velocity::new(2, 0));
        assert_eq!(
            second.snapshot.reference_state.as_enemy_projectile(),
            Some(EnemyProjectileReferenceState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0180,
                y_velocity: 0xFF80,
                lifetime_ticks: 3,
            })
        );
    }

    #[test]
    fn driver_applies_enemy_projectile_scan_lifetime_cadence() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::EnemyLaser,
            ActorBehaviorProfile {
                lander_shot_lifetime_steps: 20,
                ..ActorBehaviorProfile::default()
            },
        );
        let shot =
            driver.spawn_enemy_laser_from_spawn(Point::new(80, 120), Velocity::new(0, 0), None);

        let lifetimes = (0..=ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                snapshot_for(&report, shot)
                    .reference_state.as_enemy_projectile()
                    .expect("enemy laser should publish projectile runtime state")
                    .lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(lifetimes, vec![20, 20, 20, 20, 20, 20, 19]);
    }

    #[test]
    fn enemy_laser_collision_consumes_life_and_respawns_player() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 2);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let respawned = driver.step(GameInput::NONE);
        assert_eq!(respawned.phase, Phase::Playing);
        assert_eq!(respawned.lives, 2);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn enemy_laser_collision_on_final_life_enters_game_over() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::Explosion));
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn hyperspace_clears_enemy_projectiles_without_spending_stock_or_life() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        driver.spawn_bomb_for_test(Point::new(90, 120));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_eq!(report.lives, 3);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        assert!(!report.commands.contains(&GameCommand::PlayerKilled));
        assert!(report.sounds.contains(&SoundCue::Hyperspace));
        assert!(!report.sounds.contains(&SoundCue::GameOver));
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_leaves_hostiles_and_player_lasers_active() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 3;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        let player = driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(90, 120));
        driver.spawn_laser(Point::new(10, 40), Direction::Right, player);
        driver.spawn_enemy_laser_from_spawn(Point::new(70, 120), Velocity::new(0, 0), None);
        driver.spawn_bomb_for_test(Point::new(120, 120));

        let report = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert!(report.commands.contains(&GameCommand::Hyperspace));
        let emitted_smart_bomb = report
            .commands
            .iter()
            .any(|command| matches!(command, GameCommand::SmartBomb { .. }));
        assert!(!emitted_smart_bomb);
        assert_eq!(driver.snapshot_count(ActorKind::EnemyLaser), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Laser), 1);
        assert_eq!(report.score, 0);
        assert_eq!(report.smart_bombs, INITIAL_SMART_BOMBS);
    }

    #[test]
    fn hyperspace_hides_player_until_scripted_rematerialization() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 2,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&entered, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!entered.draws.iter().any(|draw| draw.actor == player));
        assert!(entered.sounds.contains(&SoundCue::Hyperspace));
        assert!(!entered.sounds.contains(&SoundCue::HyperspaceMaterialize));

        let still_hidden = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });

        let hidden_player = snapshot_for(&still_hidden, player);
        assert_eq!(hidden_player.bounds, None);
        assert!(!still_hidden.draws.iter().any(|draw| draw.actor == player));
        assert!(!still_hidden.sounds.contains(&SoundCue::Thrust));
        assert!(
            !still_hidden.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let rematerialized = driver.step(GameInput::NONE);
        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(player_snapshot.position, Point::new(150, 92));
        assert!(player_snapshot.bounds.is_some());
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == Point::new(150, 92)
                && matches!(draw.sprite, SpriteKey::PlayerRight)
        }));
    }

    #[test]
    fn hyperspace_lseed_high_enters_delayed_player_death_path() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 2,
                player_hyperspace_death_lseed: HYPERSPACE_DEATH_LOW_SEED_THRESHOLD + 1,
                ..ActorBehaviorProfile::default()
            },
        );

        let entered = driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        assert!(entered.commands.contains(&GameCommand::Hyperspace));
        assert!(!entered.commands.contains(&GameCommand::PlayerKilled));

        let rematerialized = driver.step(GameInput::NONE);
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(rematerialized.lives, 3);

        let pending_death = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert!(!pending_death.commands.contains(&GameCommand::PlayerKilled));
        assert!(!pending_death.sounds.contains(&SoundCue::Thrust));
        assert!(
            !pending_death.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Laser { .. }))
            })
        );

        let destroyed = driver.step(GameInput::NONE);

        assert_eq!(destroyed.phase, Phase::Playing);
        assert_eq!(destroyed.lives, 2);
        assert!(destroyed.commands.contains(&GameCommand::Destroy(player)));
        assert!(destroyed.commands.contains(&GameCommand::PlayerKilled));
        assert!(destroyed.sounds.contains(&SoundCue::Explosion));
        assert!(destroyed.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Player,
                    ..
                })
            )
        }));
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }
