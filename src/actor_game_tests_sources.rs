    #[test]
    fn hyperspace_lseed_at_source_threshold_survives() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_death_delay_steps: 1,
                player_hyperspace_death_lseed: HYPERSPACE_DEATH_LOW_SEED_THRESHOLD,
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);
        let settled = driver.step(GameInput::NONE);

        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
        assert!(!settled.commands.contains(&GameCommand::PlayerKilled));
        assert_eq!(settled.lives, 3);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
    }

    #[test]
    fn hyperspace_arcade_seed_controls_rematerialization_position_and_direction() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_actor_behavior(
            player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                player_hyperspace_arcade_seed: Some(ActorHyperspaceArcadeSeed {
                    seed: 0x12,
                    hseed: 0x34,
                    lseed: 0,
                }),
                ..ActorBehaviorProfile::default()
            },
        );

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);

        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(
            player_snapshot.position,
            Point::new(0x70, i16::from((0x34_u8 >> 1) + PLAYFIELD_TOP_EDGE_Y))
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == player_snapshot.position
                && matches!(draw.sprite, SpriteKey::PlayerLeft)
        }));
        assert!(
            rematerialized
                .sounds
                .contains(&SoundCue::HyperspaceMaterialize)
        );
        assert_eq!(rematerialized.background_left, 0x1234);
        assert!(
            rematerialized
                .commands
                .contains(&GameCommand::SetWorldScrollLeft(0x1234))
        );
        assert!(!rematerialized.commands.contains(&GameCommand::PlayerKilled));
    }

    #[test]
    fn driver_advances_hyperspace_arcade_rng_for_default_player_behavior() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        driver.set_kind_behavior(
            ActorKind::Player,
            ActorBehaviorProfile {
                player_hyperspace_hidden_steps: 1,
                player_hyperspace_rematerialize_x: 150,
                player_hyperspace_rematerialize_y: 92,
                ..ActorBehaviorProfile::default()
            },
        );
        let mut expected_arcade_rng = PLAYFIELD_START_RNG;
        expected_arcade_rng.advance();
        expected_arcade_rng.advance();

        driver.step(GameInput {
            hyperspace: true,
            ..GameInput::NONE
        });
        let rematerialized = driver.step(GameInput::NONE);

        let expected_y =
            i16::from((expected_arcade_rng.hseed >> 1).wrapping_add(PLAYFIELD_TOP_EDGE_Y));
        let expected_position = if expected_arcade_rng.hseed & 1 != 0 {
            Point::new(0x20, expected_y)
        } else {
            Point::new(0x70, expected_y)
        };
        let player_snapshot = snapshot_for(&rematerialized, player);
        assert_eq!(player_snapshot.position, expected_position);
        assert_eq!(
            rematerialized.background_left,
            u16::from_be_bytes([expected_arcade_rng.seed, expected_arcade_rng.hseed])
        );
        assert!(rematerialized.draws.iter().any(|draw| {
            draw.actor == player
                && draw.position == expected_position
                && matches!(
                    (expected_arcade_rng.hseed & 1 != 0, draw.sprite),
                    (true, SpriteKey::PlayerRight) | (false, SpriteKey::PlayerLeft)
                )
        }));
    }

    #[test]
    fn playing_step_report_carries_driver_arcade_rng_snapshot() {
        let mut driver = ActorGameDriver::new();

        let attract = driver.step(GameInput::NONE);
        assert_eq!(attract.arcade_rng, None);

        driver.phase = Phase::Playing;
        let mut expected_arcade_rng = PLAYFIELD_START_RNG;
        let expected_snapshot = expected_arcade_rng.advance().snapshot();

        let playing = driver.step(GameInput::NONE);

        assert_eq!(playing.arcade_rng, Some(expected_snapshot));
        assert_eq!(
            playing.game_state().world.arcade_rng,
            clean_arcade_rng(expected_snapshot)
        );
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_enemy_laser_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn first_wave_humans_publish_arcade_state_and_picture_frames() {
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
        let human = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.kind == ActorKind::Human && snapshot.position == Point::new(0x1C, 0xE1)
            })
            .expect("arcade-state first-wave human should publish its restore position");

        assert_eq!(
            human.human_runtime,
            Some(HumanArcadeState {
                x_fraction: 0x81,
                y_fraction: 0x00,
                picture_frame: 3,
                target_slot_index: 1,
            })
        );
        assert!(live.draws.iter().any(|draw| {
            draw.actor == human.id
                && draw.sprite == SpriteKey::Human
                && matches!(draw.effect, VisualEffect::HumanSpriteFrame { frame: 3 })
        }));
    }

    #[test]
    fn human_walk_uses_arcade_seeded_left_branch_and_terrain_y_target() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let human_id =
            driver.spawn_human_from_spawn(arcade_human_spawn_for_test(Point::new(64, 220), 1, 0));
        driver.step(GameInput::NONE);
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };

        let walked = driver.step(GameInput::NONE);
        let human = snapshot_for(&walked, human_id);

        assert_eq!(
            walked.arcade_rng.map(|arcade_rng| arcade_rng.seed),
            Some(17)
        );
        assert_eq!(human.position, Point::new(63, 221));
        assert_eq!(
            human
                .human_runtime
                .map(|arcade_state| (arcade_state.x_fraction, arcade_state.picture_frame)),
            Some((0xE0, 1))
        );
    }

    #[test]
    fn human_walk_turns_on_low_arcade_seed_without_y_step() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let human_id =
            driver.spawn_human_from_spawn(arcade_human_spawn_for_test(Point::new(64, 220), 1, 0));
        driver.step(GameInput::NONE);
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 222,
        };

        let turned = driver.step(GameInput::NONE);
        let human = snapshot_for(&turned, human_id);

        assert_eq!(turned.arcade_rng.map(|arcade_rng| arcade_rng.seed), Some(0));
        assert_eq!(human.position, Point::new(64, 220));
        assert_eq!(
            human
                .human_runtime
                .map(|arcade_state| (arcade_state.x_fraction, arcade_state.picture_frame)),
            Some((0x20, 2))
        );
    }

    #[test]
    fn human_walk_process_moves_only_selected_target_slot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let slot0 =
            driver.spawn_human_from_spawn(arcade_human_spawn_for_test(Point::new(48, 220), 0, 0));
        let slot1 =
            driver.spawn_human_from_spawn(arcade_human_spawn_for_test(Point::new(64, 220), 1, 0));
        let slot2 =
            driver.spawn_human_from_spawn(arcade_human_spawn_for_test(Point::new(80, 220), 2, 0));

        driver.step(GameInput::NONE);
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let walked = driver.step(GameInput::NONE);

        assert_eq!(driver.human_walk_cursor, Some(1));
        assert_eq!(
            driver.human_walk_sleep_ticks,
            ASTRONAUT_PROCESS_SLEEP_TICKS
        );
        assert_eq!(snapshot_for(&walked, slot0).position, Point::new(48, 220));
        assert_eq!(snapshot_for(&walked, slot1).position, Point::new(63, 221));
        assert_eq!(snapshot_for(&walked, slot2).position, Point::new(80, 220));

        let sleeping = driver.step(GameInput::NONE);
        assert_eq!(driver.human_walk_sleep_ticks, 1);
        assert_eq!(snapshot_for(&sleeping, slot1).position, Point::new(63, 221));
        assert_eq!(snapshot_for(&sleeping, slot2).position, Point::new(80, 220));
    }

    #[test]
    fn human_walk_process_suppresses_inactive_first_wave_slots() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let mut human_ids = Vec::new();
        for slot in 0..usize::from(START_HUMAN_COUNT) {
            human_ids.push(driver.spawn_human_from_spawn(arcade_human_spawn_for_test(
                Point::new(40 + i16::try_from(slot).expect("slot fits i16") * 8, 220),
                slot,
                0,
            )));
        }

        driver.step(GameInput::NONE);
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot1_walked = driver.step(GameInput::NONE);
        assert_eq!(
            snapshot_for(&slot1_walked, human_ids[1]).position,
            Point::new(47, 221)
        );

        driver.human_walk_sleep_ticks = 0;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot2_suppressed = driver.step(GameInput::NONE);

        assert_eq!(driver.human_walk_cursor, Some(2));
        assert_eq!(
            snapshot_for(&slot2_suppressed, human_ids[2]).position,
            Point::new(56, 220)
        );
    }

    #[test]
    fn human_walk_process_suppression_counts_plain_humans() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let mut arcade_state_ids = Vec::new();
        for slot in 0..9usize {
            arcade_state_ids.push(driver.spawn_human_from_spawn(arcade_human_spawn_for_test(
                Point::new(40 + i16::try_from(slot).expect("slot fits i16") * 8, 220),
                slot,
                0,
            )));
        }
        driver.spawn_human_for_test(Point::new(128, 220));

        driver.step(GameInput::NONE);
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot1_walked = driver.step(GameInput::NONE);
        assert_eq!(
            snapshot_for(&slot1_walked, arcade_state_ids[1]).position,
            Point::new(47, 221)
        );

        driver.human_walk_sleep_ticks = 0;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        let slot2_suppressed = driver.step(GameInput::NONE);

        assert_eq!(driver.human_walk_cursor, Some(2));
        assert_eq!(
            snapshot_for(&slot2_suppressed, arcade_state_ids[2]).position,
            Point::new(56, 220)
        );
    }

    #[test]
    fn source_lander_prefers_configured_target_human_slot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Lander,
            ActorBehaviorProfile {
                lander_seek_speed: 4,
                lander_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let lander_id = driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(100, 100),
            source: Some(LanderArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: u8::MAX,
                sleep_ticks: 0,
                picture_frame: 0,
                target_human_index: Some(7),
            }),
        });
        driver.spawn_human_for_test(Point::new(90, 100));
        driver.spawn_human_from_spawn(ActorHumanSpawn {
            position: Point::new(160, 100),
            mode: HumanMode::Grounded,
            source: Some(HumanArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame: 0,
                target_slot_index: 7,
            }),
        });

        driver.step(GameInput::NONE);
        let targeted = driver.step(GameInput::NONE);

        assert_eq!(
            snapshot_for(&targeted, lander_id).position,
            Point::new(104, 100)
        );
    }

    #[test]
    fn behavior_script_can_define_level_wide_actor_motion() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));

        let first = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&first, lander_id).position.x, 79);

        let seeking = driver.step(GameInput::NONE);
        assert_eq!(snapshot_for(&seeking, lander_id).position.x, 83);
    }

    #[test]
    fn behavior_script_can_define_bomber_and_pod_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 3,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 4,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(100, 80));
        let pod = driver.spawn_pod_for_test(Point::new(100, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(97, 80));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(104, 88));
    }

    #[test]
    fn behavior_script_can_choose_mutant_drift_mode() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Mutant,
            ActorBehaviorProfile {
                mutant_seek_speed: 4,
                mutant_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let mutant = driver.spawn_mutant(Point::new(100, 100));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, mutant).position, Point::new(96, 100));
    }

    #[test]
    fn behavior_script_can_choose_bomber_and_pod_targeting_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 5,
                bomber_bomb_period_steps: u64::MAX,
                bomber_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Pod,
            ActorBehaviorProfile {
                pod_drift_speed: 6,
                pod_mode: HostileMovementMode::ChasePlayer,
                ..ActorBehaviorProfile::default()
            },
        );
        let bomber = driver.spawn_bomber_for_test(Point::new(70, 80));
        let pod = driver.spawn_pod_for_test(Point::new(70, 88));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, bomber).position, Point::new(65, 85));
        assert_eq!(snapshot_for(&report, pod).position, Point::new(64, 94));
    }

    #[test]
    fn bomber_actor_lays_scripted_bomb_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_for_test(Point::new(100, 80));

        let report = driver.step(GameInput::NONE);

        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position: Point { x: 100, y: 80 },
                    source: None,
                })
            )
        }));

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Bomb), 1);
        let bomb = live
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomb)
            .expect("spawned bomb should publish an actor snapshot");
        let projectile_arcade_state = bomb
            .enemy_projectile_runtime
            .expect("bomb should publish enemy projectile arcade metadata");
        assert_eq!(projectile_arcade_state.x_velocity, 0);
        assert_eq!(projectile_arcade_state.y_velocity, 0);
        assert!(projectile_arcade_state.lifetime_ticks > 0);
        assert!(live.draws.iter().any(|draw| draw.sprite == SpriteKey::Bomb));
    }

    #[test]
    fn bomber_bomb_spawn_carries_enemy_projectile_fractions() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Bomb,
            ActorBehaviorProfile {
                bomb_lifetime_steps: 5,
                ..ActorBehaviorProfile::default()
            },
        );
        let initial_source = BomberArcadeState {
            x_fraction: 0x6D,
            y_fraction: 0x7B,
            x_velocity: 0,
            y_velocity: 0,
            picture_frame: 0,
            cruise_altitude: BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            slot: 0,
        };
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(100, 80),
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_bomber_after_arcade_motion(
            Point::new(100, 80),
            initial_source,
            report.step,
            bomber,
            report.arcade_rng,
            None,
        );
        let expected_lifetime_ticks = report
            .arcade_rng
            .map(bomber_bomb_lifetime_ticks)
            .expect("playing report should carry arcade rng");
        let bomber_snapshot = snapshot_for(&report, bomber);
        assert_eq!(bomber_snapshot.position, expected_position);
        assert_eq!(bomber_snapshot.bomber_runtime, Some(expected_source));

        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Bomb {
                    position,
                    source: Some(EnemyProjectileArcadeState {
                        x_fraction,
                        y_fraction,
                        x_velocity: 0,
                        y_velocity: 0,
                        lifetime_ticks,
                    }),
                }) if *position == Point::new(100, 80)
                    && *x_fraction == initial_source.x_fraction
                    && *y_fraction == initial_source.y_fraction
                    && *lifetime_ticks == expected_lifetime_ticks
            )
        }));

        let live = driver.step(GameInput::NONE);
        let bomb = live
            .snapshots
            .iter()
            .find(|snapshot| {
                snapshot.enemy_projectile_runtime.is_some_and(|source| {
                    source.x_fraction == initial_source.x_fraction
                        && source.y_fraction == initial_source.y_fraction
                })
            })
            .expect("arcade-backed bomber bomb should publish enemy projectile fractions");

        assert_eq!(
            bomb.enemy_projectile_runtime,
            Some(EnemyProjectileArcadeState {
                x_fraction: initial_source.x_fraction,
                y_fraction: initial_source.y_fraction,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: expected_lifetime_ticks,
            })
        );
    }

    #[test]
    fn bomber_bomb_spawn_uses_arcade_rng_gate() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 14,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(100, 80),
            source: Some(BomberArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let arcade_rng = report
            .arcade_rng
            .expect("playing report should carry arcade rng");

        assert_ne!(arcade_rng.lseed & 0x07, 0);
        assert!(
            !report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
    }

    #[test]
    fn bomber_bomb_spawn_respects_enemy_projectile_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 0,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_drift_speed: 0,
                bomber_bomb_period_steps: 1,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X, 80),
            source: Some(BomberArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: 0,
            }),
        });
        driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: Point::new(
                ENEMY_PROJECTILE_MAX_SCREEN_X - 1,
                i16::from(PLAYFIELD_TOP_EDGE_Y),
            ),
            source: Some(BomberArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                picture_frame: 0,
                cruise_altitude: BOMBER_CRUISE_ALTITUDE,
                sleep_ticks: 0,
                slot: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(
            report.commands.iter().all(|command| {
                !matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
        let live = driver.step(GameInput::NONE);
        assert_eq!(bomb_projectile_snapshot_count(&live), 0);
    }

    #[test]
    fn bomber_motion_uses_seeded_sprite_frame_and_y_velocity() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_bomb_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.spawn_player();
        let player_report = driver.step(GameInput::NONE);
        let player_position = player_report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Player)
            .map(|snapshot| snapshot.position)
            .expect("player should publish a prompt snapshot");
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 10,
        };
        let initial_source = BomberArcadeState {
            x_fraction: 0x10,
            y_fraction: 0x20,
            x_velocity: 0x0100,
            y_velocity: 0,
            picture_frame: 2,
            cruise_altitude: BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            slot: 3,
        };
        let bomber_position = Point::new(96, player_position.y - 8);
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: bomber_position,
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_bomber_after_arcade_motion(
            bomber_position,
            initial_source,
            report.step,
            bomber,
            report.arcade_rng,
            Some(player_position),
        );
        let snapshot = snapshot_for(&report, bomber);

        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.bomber_runtime, Some(expected_source));
        assert_ne!(expected_source.y_velocity, 0);
        assert!(report.draws.iter().any(|draw| {
            draw.actor == bomber
                && matches!(
                    draw.effect,
                    VisualEffect::BomberSpriteFrame { frame }
                        if frame == expected_source.picture_frame
                )
        }));
        assert!(
            !report.commands.iter().any(|command| {
                matches!(command, GameCommand::Spawn(SpawnRequest::Bomb { .. }))
            })
        );
    }

    #[test]
    fn bomber_offscreen_motion_adjusts_cruise_altitude() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.arcade_rng = ActorArcadeRng {
            seed: 0,
            hseed: 0,
            lseed: 13,
        };
        driver.set_kind_behavior(
            ActorKind::Bomber,
            ActorBehaviorProfile {
                bomber_bomb_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let initial_source = BomberArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            picture_frame: 1,
            cruise_altitude: BOMBER_CRUISE_ALTITUDE,
            sleep_ticks: 0,
            slot: 3,
        };
        let bomber_position = Point::new(100, 0);
        let bomber = driver.spawn_bomber_from_spawn(ActorBomberSpawn {
            position: bomber_position,
            source: Some(initial_source),
        });

        let report = driver.step(GameInput::NONE);
        let (expected_position, expected_source) = expected_bomber_after_arcade_motion(
            bomber_position,
            initial_source,
            report.step,
            bomber,
            report.arcade_rng,
            None,
        );
        let snapshot = snapshot_for(&report, bomber);

        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.bomber_runtime, Some(expected_source));
        assert_ne!(expected_source.cruise_altitude, BOMBER_CRUISE_ALTITUDE);
        assert_ne!(expected_source.y_velocity, 0);
    }

    #[test]
    fn source_bomb_spawn_commands_respect_getshl_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X, 100),
                source: Some(EnemyProjectileArcadeState {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: 0,
                }),
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(
                    ENEMY_PROJECTILE_MAX_SCREEN_X - 1,
                    i16::from(PLAYFIELD_TOP_EDGE_Y),
                ),
                source: Some(EnemyProjectileArcadeState {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    lifetime_ticks: 0,
                }),
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X, 108),
                source: None,
            }),
        ]);
        let report = driver.step(GameInput::NONE);

        let bombs = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
            .collect::<Vec<_>>();
        assert_eq!(bombs.len(), 1);
        assert_eq!(
            bombs[0].position,
            Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X, 108)
        );
    }

    #[test]
    fn source_bomb_spawn_preserves_scripted_lifetime_ticks() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Bomb,
            ActorBehaviorProfile {
                bomb_lifetime_steps: 40,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.apply_commands(&[GameCommand::Spawn(SpawnRequest::Bomb {
            position: Point::new(80, 100),
            source: Some(EnemyProjectileArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 9,
            }),
        })]);

        let lifetimes = (0..=ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                report
                    .snapshots
                    .iter()
                    .find(|snapshot| snapshot.kind == ActorKind::Bomb)
                    .and_then(|snapshot| snapshot.enemy_projectile_runtime)
                    .expect("source-backed bomb should publish source projectile metadata")
                    .lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(lifetimes, vec![9, 9, 9, 9, 9, 9, 8]);
    }

    #[test]
    fn source_enemy_laser_spawn_commands_respect_getshl_bounds() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(
                    ENEMY_PROJECTILE_MAX_SCREEN_X - 1,
                    i16::from(PLAYFIELD_TOP_EDGE_Y),
                ),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X - 1, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
        ]);
        let report = driver.step(GameInput::NONE);

        let shots = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .collect::<Vec<_>>();
        assert_eq!(shots.len(), 1);
        assert_eq!(
            shots[0].position,
            Point::new(ENEMY_PROJECTILE_MAX_SCREEN_X - 1, 100)
        );
    }

    #[test]
    fn source_enemy_laser_spawn_preserves_scripted_projectile_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::EnemyLaser,
            ActorBehaviorProfile {
                lander_shot_lifetime_steps: 40,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.apply_commands(&[GameCommand::Spawn(SpawnRequest::EnemyLaser {
            position: Point::new(80, 100),
            velocity: Velocity::new(0, 0),
            source: Some(EnemyProjectileArcadeState {
                x_fraction: 0x55,
                y_fraction: 0x66,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 7,
            }),
        })]);

        let mut first_source = None;
        let lifetimes = (0..=ENEMY_PROJECTILE_SCAN_INITIAL_DELAY_STEPS)
            .map(|_| {
                let report = driver.step(GameInput::NONE);
                let source = report
                    .snapshots
                    .iter()
                    .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
                    .and_then(|snapshot| snapshot.enemy_projectile_runtime)
                    .expect("scripted enemy laser should publish source metadata");
                first_source.get_or_insert(source);
                source.lifetime_ticks
            })
            .collect::<Vec<_>>();

        assert_eq!(
            first_source,
            Some(EnemyProjectileArcadeState {
                x_fraction: 0x55,
                y_fraction: 0x66,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 7,
            })
        );
        assert_eq!(lifetimes, vec![7, 7, 7, 7, 7, 7, 6]);
    }

    #[test]
    fn enemy_projectile_cap_blocks_and_releases_slots() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        for index in 0..ENEMY_PROJECTILE_SLOT_LIMIT {
            driver.spawn_enemy_laser_from_spawn(
                Point::new(40 + index as i16, 120),
                Velocity::new(0, 0),
                None,
            );
        }
        let filled = driver.step(GameInput::NONE);
        assert_eq!(
            enemy_projectile_snapshot_count(&filled),
            ENEMY_PROJECTILE_SLOT_LIMIT
        );

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(96, 96),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(100, 100),
                source: None,
            }),
        ]);
        let capped = driver.step(GameInput::NONE);
        assert_eq!(
            enemy_projectile_snapshot_count(&capped),
            ENEMY_PROJECTILE_SLOT_LIMIT
        );
        assert!(
            capped
                .snapshots
                .iter()
                .all(|snapshot| snapshot.kind != ActorKind::Bomb)
        );

        let freed_shell = capped
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .expect("filled enemy projectile list should contain enemy lasers")
            .id;
        driver.apply_commands(&[
            GameCommand::Destroy(freed_shell),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(100, 100),
                source: None,
            }),
        ]);
        let refilled = driver.step(GameInput::NONE);

        assert_eq!(
            enemy_projectile_snapshot_count(&refilled),
            ENEMY_PROJECTILE_SLOT_LIMIT
        );
        assert!(
            refilled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Bomb)
        );
    }

    #[test]
    fn bomb_projectile_cap_blocks_bombs_without_blocking_enemy_lasers() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        for index in 0..ACTIVE_BOMBER_BOMB_LIMIT {
            driver.spawn_bomb_for_test(Point::new(40 + (index as i16 * 4), 120));
        }
        let filled = driver.step(GameInput::NONE);
        assert_eq!(bomb_projectile_snapshot_count(&filled), ACTIVE_BOMBER_BOMB_LIMIT);
        assert!(
            enemy_projectile_snapshot_count(&filled) < ENEMY_PROJECTILE_SLOT_LIMIT,
            "bomb projectile cap should fill before the shared enemy projectile cap"
        );

        driver.apply_commands(&[
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(112, 100),
                source: None,
            }),
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position: Point::new(116, 100),
                velocity: Velocity::new(0, 0),
                source: None,
            }),
        ]);
        let capped = driver.step(GameInput::NONE);
        assert_eq!(bomb_projectile_snapshot_count(&capped), ACTIVE_BOMBER_BOMB_LIMIT);
        assert_eq!(enemy_laser_snapshot_count(&capped), 1);
        assert!(
            capped
                .snapshots
                .iter()
                .all(|snapshot| snapshot.kind != ActorKind::Bomb
                    || snapshot.position != Point::new(112, 100))
        );

        let freed_bomb = capped
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Bomb)
            .expect("filled bomb projectile list should contain bombs")
            .id;
        driver.apply_commands(&[
            GameCommand::Destroy(freed_bomb),
            GameCommand::Spawn(SpawnRequest::Bomb {
                position: Point::new(112, 100),
                source: None,
            }),
        ]);
        let refilled = driver.step(GameInput::NONE);

        assert_eq!(
            bomb_projectile_snapshot_count(&refilled),
            ACTIVE_BOMBER_BOMB_LIMIT
        );
        assert_eq!(enemy_laser_snapshot_count(&refilled), 1);
        assert!(refilled.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Bomb && snapshot.position == Point::new(112, 100)
        }));
    }

    #[test]
    fn behavior_script_can_define_swarmer_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 5,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(65, 120));
    }

    #[test]
    fn behavior_script_can_define_baiter_motion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 6,
                baiter_fire_period_steps: u64::MAX,
                ..ActorBehaviorProfile::default()
            },
        );
        let baiter = driver.spawn_baiter_for_test(Point::new(70, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, baiter).position, Point::new(64, 120));
    }

    #[test]
    fn behavior_script_can_choose_swarmer_and_baiter_drift_modes() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(
            ActorKind::Swarmer,
            ActorBehaviorProfile {
                swarmer_seek_speed: 4,
                swarmer_fire_period_steps: u64::MAX,
                swarmer_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        driver.set_kind_behavior(
            ActorKind::Baiter,
            ActorBehaviorProfile {
                baiter_seek_speed: 5,
                baiter_fire_period_steps: u64::MAX,
                baiter_mode: HostileMovementMode::Drift,
                ..ActorBehaviorProfile::default()
            },
        );
        let swarmer = driver.spawn_swarmer_for_test(Point::new(70, 120));
        let baiter = driver.spawn_baiter_for_test(Point::new(80, 124));

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, swarmer).position, Point::new(66, 120));
        assert_eq!(snapshot_for(&report, baiter).position, Point::new(75, 124));
    }

    #[test]
    fn baiter_timer_spawns_source_baiter_from_wave_profile() {
        let mut driver = started_driver();

        driver.set_baiter_timer_for_test(1);
        let report = driver.step(GameInput::NONE);

        let baiter_spawn = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Baiter { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .expect("expired baiter timer should spawn a baiter");
        assert_eq!(
            baiter_spawn,
            (
                Point::new(228, 144),
                Some(BaiterArcadeState {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0xFFC0,
                    y_velocity: 0xFF80,
                    shot_timer: BAITER_INITIAL_SHOT_TIMER,
                    sleep_ticks: 0,
                    picture_frame: 0,
                })
            )
        );

        let live = driver.step(GameInput::NONE);
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 1);
        assert!(live.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Baiter
                && snapshot.position == Point::new(227, 143)
                && snapshot.baiter_runtime
                    == Some(BaiterArcadeState {
                        x_fraction: 0,
                        y_fraction: 0x80,
                        x_velocity: 0xFFC0,
                        y_velocity: 0xFF80,
                        shot_timer: 7,
                        sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
                        picture_frame: 1,
                    })
        }));
        assert!(live.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::Baiter
                && matches!(draw.effect, VisualEffect::BaiterSpriteFrame { frame: 1 })
        }));
    }

    #[test]
    fn source_swarmer_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let start = Point::new(25, 100);
        let source = SwarmerArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0020,
            y_velocity: 0,
            acceleration: 0,
            sleep_ticks: 0,
            shot_timer: 1,
            horizontal_seek_pending: false,
        };
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let report_arcade_rng = report
            .arcade_rng
            .expect("playing report should carry arcade rng");
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report_arcade_rng,
            Point::new(42, 120),
            Velocity::default(),
        );
        let mut expected_source = source;
        expected_source.y_velocity = source_mini_swarmer_y_velocity(
            source.y_velocity,
            source.acceleration,
            120,
            start.y,
            report_arcade_rng.seed,
        );
        let (expected_x, expected_x_fraction) =
            actor_source_axis_step(start.x, source.x_fraction, expected_source.x_velocity);
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            start.y,
            source.y_fraction,
            expected_source.y_velocity,
        );
        let expected_position = Point::new(expected_x, expected_y);
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        expected_source.shot_timer = source_rmax(
            clamped_source_swarmer_shot_reset(ArcadeWaveProfile::for_wave(report.wave)),
            report_arcade_rng.seed,
        );
        expected_source.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;
        let (expected_velocity, expected_projectile_source) =
            actor_source_mini_swarmer_fireball(expected_position, &prompt, expected_source)
                .expect("expected source swarmer fireball");

        assert!(report.sounds.contains(&SoundCue::SwarmerShot));
        let swarmer_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => Some((*position, *velocity, *source)),
                _ => None,
            })
            .expect("source swarmer should emit a hostile shot command");
        assert_eq!(
            swarmer_shot,
            (
                expected_position,
                expected_velocity,
                Some(expected_projectile_source)
            )
        );
        assert_eq!(
            snapshot_for(&report, swarmer).swarmer_runtime,
            Some(expected_source)
        );
    }

    #[test]
    fn swarmer_shot_direction_gate_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        let start = Point::new(48, 100);
        let source = SwarmerArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0x0020,
            y_velocity: 0,
            acceleration: 0,
            sleep_ticks: 0,
            shot_timer: 1,
            horizontal_seek_pending: false,
        };
        let swarmer = driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let report_arcade_rng = report
            .arcade_rng
            .expect("playing report should carry arcade rng");
        let mut expected_source = source;
        expected_source.y_velocity = source_mini_swarmer_y_velocity(
            source.y_velocity,
            source.acceleration,
            120,
            start.y,
            report_arcade_rng.seed,
        );
        let (expected_x, expected_x_fraction) =
            actor_source_axis_step(start.x, source.x_fraction, expected_source.x_velocity);
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            start.y,
            source.y_fraction,
            expected_source.y_velocity,
        );
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        expected_source.shot_timer = source_rmax(
            clamped_source_swarmer_shot_reset(ArcadeWaveProfile::for_wave(report.wave)),
            report_arcade_rng.seed,
        );
        expected_source.sleep_ticks = MINI_SWARMER_LOOP_SLEEP_TICKS;

        assert!(!report.sounds.contains(&SoundCue::SwarmerShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, swarmer).position,
            Point::new(expected_x, expected_y)
        );
        assert_eq!(
            snapshot_for(&report, swarmer).swarmer_runtime,
            Some(expected_source)
        );
    }

    #[test]
    fn swarmer_full_shell_cap_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        for index in 0..ENEMY_PROJECTILE_SLOT_LIMIT {
            let id = ActorId::new(10_000 + index as u64);
            driver.snapshots.insert(
                id,
                actor_snapshot(id.value(), ActorKind::EnemyLaser, Point::new(64, 120)),
            );
        }
        driver.spawn_swarmer_from_spawn(ActorSwarmerSpawn {
            position: Point::new(25, 100),
            source: Some(SwarmerArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0x0020,
                y_velocity: 0,
                acceleration: 0,
                sleep_ticks: 0,
                shot_timer: 1,
                horizontal_seek_pending: false,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert!(!report.sounds.contains(&SoundCue::SwarmerShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
    }

    #[test]
    fn source_baiter_shot_timer_spawns_hostile_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_player();
        driver.wave = 0;
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 120),
            source: Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let report_arcade_rng = report
            .arcade_rng
            .expect("playing report should carry arcade rng");
        let prompt = source_mutant_prompt_for_test(
            report.step,
            report.wave,
            report_arcade_rng,
            Point::new(42, 120),
            Velocity::default(),
        );
        let mut expected_source = BaiterArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: actor_source_baiter_shot_reset(
                ArcadeWaveProfile::for_wave(report.wave),
                report_arcade_rng.seed,
            ),
            sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
            picture_frame: 1,
        };
        let (expected_velocity, expected_projectile_source) = actor_source_baiter_fireball(
            Point::new(70, 120),
            &prompt,
            expected_source,
            report_arcade_rng,
        )
        .expect("expected source baiter fireball");

        assert!(report.sounds.contains(&SoundCue::BaiterShot));
        let baiter_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => Some((*position, *velocity, *source)),
                _ => None,
            })
            .expect("source baiter should emit a hostile shot command");
        assert_eq!(
            baiter_shot,
            (
                Point::new(70, 120),
                expected_velocity,
                Some(expected_projectile_source)
            )
        );
        let (expected_x, expected_x_fraction) = actor_source_axis_step(
            70,
            expected_source.x_fraction,
            actor_source_baiter_screen_x_velocity(expected_source.x_velocity),
        );
        let (expected_y, expected_y_fraction) = actor_source_active_object_y_step(
            120,
            expected_source.y_fraction,
            expected_source.y_velocity,
        );
        expected_source.x_fraction = expected_x_fraction;
        expected_source.y_fraction = expected_y_fraction;
        assert_eq!(
            snapshot_for(&report, baiter).position,
            Point::new(expected_x, expected_y)
        );
        assert_eq!(
            snapshot_for(&report, baiter).baiter_runtime,
            Some(expected_source)
        );
    }

    #[test]
    fn baiter_full_shell_cap_suppresses_fireball_and_sound() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        let player = driver.spawn_player();
        driver.snapshots.insert(
            player,
            actor_snapshot(player.value(), ActorKind::Player, Point::new(42, 120)),
        );
        for index in 0..ENEMY_PROJECTILE_SLOT_LIMIT {
            let id = ActorId::new(20_000 + index as u64);
            driver.snapshots.insert(
                id,
                actor_snapshot(id.value(), ActorKind::EnemyLaser, Point::new(64, 120)),
            );
        }
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 120),
            source: Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: 0,
                picture_frame: 0,
            }),
        });

        let report = driver.step(GameInput::NONE);
        let report_arcade_rng = report
            .arcade_rng
            .expect("playing report should carry arcade rng");

        assert!(!report.sounds.contains(&SoundCue::BaiterShot));
        assert!(!report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    source: Some(_),
                    ..
                })
            )
        }));
        assert_eq!(
            snapshot_for(&report, baiter).baiter_runtime,
            Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: actor_source_baiter_shot_reset(
                    ArcadeWaveProfile::for_wave(report.wave),
                    report_arcade_rng.seed,
                ),
                sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 1,
            })
        );
    }

    #[test]
    fn source_baiter_fireball_adds_player_velocity_when_seed_is_high() {
        let arcade_rng = ActorArcadeRngSnapshot {
            seed: 0x90,
            hseed: 0,
            lseed: 0x44,
        };
        let prompt = source_mutant_prompt_for_test(
            7,
            2,
            arcade_rng,
            Point::new(80, 120),
            Velocity::new(5, -2),
        );
        let source = BaiterArcadeState {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            picture_frame: 0,
        };

        let (velocity, projectile) =
            actor_source_baiter_fireball(Point::new(70, 100), &prompt, source, arcade_rng)
                .expect("high-seed baiter shot should allocate");

        let expected_x_velocity = actor_sign_extend_u8_to_u16(
            (arcade_rng.seed & 0x1F)
                .wrapping_sub(0x10)
                .wrapping_add(80)
                .wrapping_sub(70),
        )
        .wrapping_shl(2)
        .wrapping_add(actor_source_velocity_word(5).wrapping_shl(2));
        let expected_y_velocity = actor_sign_extend_u8_to_u16(
            (arcade_rng.lseed & 0x1F)
                .wrapping_sub(0x10)
                .wrapping_add(120)
                .wrapping_sub(100),
        )
        .wrapping_shl(2);

        assert_eq!(
            projectile,
            EnemyProjectileArcadeState {
                x_fraction: source.x_fraction,
                y_fraction: source.y_fraction,
                x_velocity: expected_x_velocity,
                y_velocity: expected_y_velocity,
                lifetime_ticks: ENEMY_PROJECTILE_LIFETIME_TICKS,
            }
        );
        assert_eq!(
            velocity,
            actor_source_screen_velocity(expected_x_velocity, expected_y_velocity)
        );
    }

    #[test]
    fn baiter_retarget_uses_driver_arcade_rng_snapshot() {
        fn step_baiter_after_arcade_seed(seed: u8) -> (StepReport, ActorId) {
            let mut driver = ActorGameDriver::new();
            driver.phase = Phase::Playing;
            driver.wave = 1;
            driver.spawn_player();
            driver.spawn_lander_for_test(Point::new(220, 80));
            driver.step(GameInput::NONE);
            driver.arcade_rng = ActorArcadeRng {
                seed,
                hseed: 0,
                lseed: 0,
            };
            let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
                position: Point::new(70, 120),
                source: Some(BaiterArcadeState {
                    x_fraction: 0,
                    y_fraction: 0,
                    x_velocity: 0,
                    y_velocity: 0,
                    shot_timer: 2,
                    sleep_ticks: 0,
                    picture_frame: 2,
                }),
            });

            (driver.step(GameInput::NONE), baiter)
        }

        let (held, held_baiter) = step_baiter_after_arcade_seed(0);
        assert_eq!(held.arcade_rng.map(|arcade_rng| arcade_rng.seed), Some(17));
        assert_eq!(
            snapshot_for(&held, held_baiter).position,
            Point::new(70, 120)
        );
        assert_eq!(
            snapshot_for(&held, held_baiter).baiter_runtime,
            Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );

        let (retargeted, retargeted_baiter) = step_baiter_after_arcade_seed(70);
        assert_eq!(
            retargeted.arcade_rng.map(|arcade_rng| arcade_rng.seed),
            Some(227)
        );
        assert_eq!(
            snapshot_for(&retargeted, retargeted_baiter).position,
            Point::new(69, 120)
        );
        assert_eq!(
            snapshot_for(&retargeted, retargeted_baiter).baiter_runtime,
            Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0xFFC0,
                y_velocity: 0,
                shot_timer: 1,
                sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );
    }

    #[test]
    fn source_baiter_retarget_adds_player_velocity() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.arcade_rng = ActorArcadeRng {
            seed: 70,
            hseed: 0,
            lseed: 0,
        };
        let player_id = ActorId::new(90);
        let mut player = actor_snapshot(player_id.value(), ActorKind::Player, Point::new(42, 112));
        player.velocity = Velocity::new(8, 4);
        driver.snapshots.insert(player_id, player);
        let baiter = driver.spawn_baiter_from_spawn(ActorBaiterSpawn {
            position: Point::new(70, 140),
            source: Some(BaiterArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                shot_timer: 2,
                sleep_ticks: 0,
                picture_frame: 2,
            }),
        });

        let report = driver.step(GameInput::NONE);

        assert_eq!(
            report.arcade_rng.map(|arcade_rng| arcade_rng.seed),
            Some(227)
        );
        assert_eq!(snapshot_for(&report, baiter).position, Point::new(69, 139));
        assert_eq!(
            snapshot_for(&report, baiter).baiter_runtime,
            Some(BaiterArcadeState {
                x_fraction: 0x08,
                y_fraction: 0x82,
                x_velocity: 0xFFC2,
                y_velocity: 0xFF82,
                shot_timer: 1,
                sleep_ticks: BAITER_LOOP_SLEEP_TICKS,
                picture_frame: 0,
            })
        );
    }

    #[test]
    fn baiter_does_not_block_wave_completion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.reset_baiter_timer();
        let baiter = driver.spawn_baiter_for_test(Point::new(100, 100));

        let cleared = driver.step(GameInput::NONE);

        assert!(snapshot_for(&cleared, baiter).alive);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            !next_wave
                .snapshots
                .iter()
                .any(|snapshot| snapshot.id == baiter)
        );
    }

    #[test]
    fn behavior_script_manifest_exports_resolution_order() {
        let default_behavior = ActorBehaviorProfile {
            player_speed: 3,
            ..ActorBehaviorProfile::default()
        };
        let lander_behavior = ActorBehaviorProfile {
            lander_drift_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let actor = ActorId::new(42);
        let actor_behavior = ActorBehaviorProfile {
            lander_drift_speed: 7,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        let script = ActorBehaviorScript::new(default_behavior)
            .with_kind_behavior(ActorKind::Lander, lander_behavior)
            .with_actor_behavior(actor, actor_behavior);

        let manifest = script.manifest();

        assert_eq!(manifest.default_profile, default_behavior);
        assert_eq!(
            manifest.kind_profiles,
            [ActorKindBehaviorProfile {
                kind: ActorKind::Lander,
                profile: lander_behavior
            }]
        );
        assert_eq!(
            manifest.actor_profiles,
            [ActorInstanceBehaviorProfile {
                actor,
                profile: actor_behavior
            }]
        );
        assert_eq!(
            manifest.behavior_for(actor, ActorKind::Lander),
            actor_behavior
        );
        assert_eq!(
            manifest.behavior_for(ActorId::new(99), ActorKind::Lander),
            lander_behavior
        );
        assert_eq!(
            manifest.behavior_for(ActorId::new(99), ActorKind::Bomber),
            default_behavior
        );
    }

    #[test]
    fn behavior_script_text_parser_updates_default_kind_and_actor_profiles() {
        let script = ActorBehaviorScript::parse_text(
            "\
            # Movement and behavior script\n\
            default player_speed 5\n\
            default player_takes_enemy_collision_damage off\n\
            kind lander lander_mode chase_player\n\
            kind lander lander_seek_speed 6\n\
            actor 42 lander_drift_speed 7\n\
            actor 42 player_hyperspace_arcade_seed 0x52 0x62 0x0c\n",
        )
        .expect("behavior script text should parse");

        let manifest = script.manifest();

        assert_eq!(manifest.default_profile.player_speed, 5);
        assert!(!manifest.default_profile.player_takes_enemy_collision_damage);
        let lander = manifest
            .kind_profile(ActorKind::Lander)
            .expect("lander kind profile should be parsed");
        assert_eq!(lander.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(lander.lander_seek_speed, 6);
        let actor = manifest
            .actor_profile(ActorId::new(42))
            .expect("actor profile should be parsed");
        assert_eq!(actor.lander_drift_speed, 7);
        assert_eq!(
            actor.player_hyperspace_arcade_seed,
            Some(ActorHyperspaceArcadeSeed {
                seed: 0x52,
                hseed: 0x62,
                lseed: 0x0C,
            })
        );
    }

    #[test]
    fn parsed_behavior_script_drives_motion_and_damage_policy() {
        let script = "\
            kind lander lander_mode chase_player\n\
            kind lander lander_seek_speed 4\n\
            kind player player_takes_enemy_collision_damage false\n"
            .parse::<ActorBehaviorScript>()
            .expect("behavior script should parse");
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.behavior_script = script;
        driver.spawn_player();
        let lander = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));

        driver.step(GameInput::NONE);
        let chasing = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&chasing, lander).position.x, 75);

        let script = "kind player player_takes_enemy_collision_damage false"
            .parse::<ActorBehaviorScript>()
            .expect("damage behavior script should parse");
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.behavior_script = script;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));
        let protected = driver.step(GameInput::NONE);

        assert_eq!(protected.phase, Phase::Playing);
        assert_eq!(protected.lives, 3);
        assert!(
            !protected
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::PlayerKilled))
        );
    }

    #[test]
    fn behavior_script_text_parser_reports_line_errors() {
        let error = ActorBehaviorScript::parse_text("kind lander no_such_field 1\n")
            .expect_err("unknown behavior field should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));

        let error = ActorBehaviorScript::parse_text("kind no_such_kind lander_seek_speed 1\n")
            .expect_err("unknown actor kind should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown actor kind"));
    }
