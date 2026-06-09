    #[test]
    fn falling_human_rescue_queues_sound_board_command_tail() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_falling_human_for_test(Point::new(42, 120), 0);
        driver.spawn_human_for_test(Point::new(200, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let rescued = driver.step(GameInput::NONE);
        assert_eq!(rescued.sounds, [SoundCue::HumanRescued]);

        let mut observed_tail = Vec::new();
        for offset in 1..=20u8 {
            let report = driver.step(GameInput::NONE);
            if !report.sounds.is_empty() {
                observed_tail.push((offset, report.sounds.clone()));
            }
        }

        assert_eq!(
            observed_tail,
            vec![
                (
                    10,
                    vec![SoundCue::SoundBoardCommand(ASTRONAUT_CATCH_SOUND_COMMAND)]
                ),
                (
                    20,
                    vec![SoundCue::SoundBoardCommand(ASTRONAUT_CATCH_SOUND_COMMAND)]
                ),
            ]
        );
    }

    #[test]
    fn slow_falling_human_lands_safely_for_250_points() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 1);

        let landed = driver.step(GameInput::NONE);

        assert_eq!(landed.score, HUMAN_SAFE_LANDING_SCORE);
        assert!(landed.sounds.contains(&SoundCue::HumanSafeLanding));
        assert!(
            landed
                .commands
                .contains(&GameCommand::AddScore(HUMAN_SAFE_LANDING_SCORE))
        );
        assert!(
            landed
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Human)
        );
    }

    #[test]
    fn actor_playing_state_and_render_bridge_projects_terrain_until_blow() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_player();

        let report = driver.step(GameInput::NONE);
        let state = report.game_state();
        let scene = report.render_scene();

        assert_eq!(state.world.terrain, playfield_terrain_segments());
        assert!(state.world.terrain_blow.is_none());
        assert!(state.world.scanner.enabled);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE && sprite.layer == RenderLayer::Terrain
        }));
    }

    #[test]
    fn last_human_loss_starts_actor_terrain_blow() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_pod_for_test(Point::new(180, 80));
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 4);

        let report = driver.step(GameInput::NONE);

        assert!(driver.snapshot_count(ActorKind::Human) == 0);
        assert!(report.sounds.is_empty());
        let terrain_blow = report.terrain_blow.expect("terrain blow should start");
        assert!(terrain_blow.status_terrain_blown);
        assert_eq!(terrain_blow.stage, TerrainBlowStage::ExplosionPassSleeping);
        assert_eq!(terrain_blow.elapsed_ticks, 0);
        assert_eq!(terrain_blow.explosion_pass, 0);
        assert_eq!(terrain_blow.sleep_ticks_remaining, Some(1));
        assert_eq!(
            terrain_blow.overload_counter,
            TERRAIN_BLOW_OVERLOAD_COUNTER
        );
        assert!(terrain_blow.terrain_erased());
        assert!(terrain_blow.scanner_terrain_erased());

        let state = report.game_state();
        assert!(state.world.terrain.is_empty());
        assert_eq!(state.world.terrain_blow, Some(terrain_blow));
        assert!(!state.world.scanner.enabled);
        assert!(state.world.explosions.iter().any(|explosion| {
            explosion.kind == CleanExplosionKind::Terrain
                && explosion.position == ScreenPosition::new(0x4C, 0xC2)
                && explosion.picture_label == "TEREX"
                && explosion.mapped_sprite == SpriteId::TERRAIN_EXPLOSION
        }));

        let scene = report.render_scene();
        assert!(!scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE && sprite.layer == RenderLayer::Terrain
        }));
    }

    #[test]
    fn actor_terrain_blow_advances_flash_explosions_and_sound_tail() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.spawn_pod_for_test(Point::new(180, 80));
        driver.spawn_falling_human_for_test(Point::new(100, HUMAN_GROUND_Y - 1), 4);
        let start = driver.step(GameInput::NONE);
        assert!(start.terrain_blow.is_some());

        let mut observed_sounds = Vec::new();
        let mut saw_completion = false;
        for offset in 1..=TERRAIN_BLOW_COMPLETE_FRAME + 26 {
            let report = driver.step(GameInput::NONE);
            if !report.sounds.is_empty() {
                observed_sounds.push((offset, report.sounds.clone()));
            }
            let terrain_blow = report
                .terrain_blow
                .expect("terrain blow should remain published");
            if offset == 2 {
                assert_eq!(
                    report.render_scene().clear_color,
                    terrain_blow_flash_tint(terrain_blow.elapsed_ticks)
                );
            }
            if offset == 4 {
                assert!(
                    report
                        .game_state()
                        .world
                        .explosions
                        .iter()
                        .any(|explosion| {
                            explosion.kind == CleanExplosionKind::Terrain
                                && explosion.position == ScreenPosition::new(0x14, 0xE2)
                        })
                );
            }
            if terrain_blow.stage == TerrainBlowStage::Completed {
                saw_completion = true;
                assert_eq!(
                    terrain_blow.explosion_pass,
                    TERRAIN_BLOW_START_SOUND_FRAMES.len() as u8
                );
                assert_eq!(terrain_blow.sleep_ticks_remaining, None);
            }
        }

        assert!(saw_completion);
        assert_eq!(observed_sounds, terrain_blow_sound_board_cues());
    }

    #[test]
    fn completed_abduction_consumes_human_and_spawns_mutant() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);
        driver.step(GameInput::NONE);

        let mut converted = None;
        for _ in 0..120 {
            let step = driver.step(GameInput::NONE);
            if step.sounds.contains(&SoundCue::MutantSpawn) {
                converted = Some(step);
                break;
            }
        }
        let converted = converted.expect("carried human should convert into a mutant");

        assert_eq!(driver.snapshot_count(ActorKind::Human), 0);
        assert!(
            converted
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Mutant { .. })))
        );
        let settled = driver.step(GameInput::NONE);
        assert!(
            settled
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Mutant)
        );
    }

    #[test]
    fn arcade_lander_abduction_spawns_mutant_arcade_state() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        let lander_runtime = LanderArcadeState {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: u8::MAX,
            sleep_ticks: 0,
            picture_frame: 3,
            target_human_index: None,
        };
        driver.spawn_lander_from_spawn(ActorLanderSpawn {
            position: Point::new(100, HUMAN_GROUND_Y),
            source: Some(lander_runtime),
        });
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);
        driver.step(GameInput::NONE);

        let (converted, mutant_runtime) = (0..120)
            .filter_map(|_| {
                let report = driver.step(GameInput::NONE);
                report.commands.iter().find_map(|command| {
                    if let GameCommand::Spawn(SpawnRequest::Mutant {
                        source: Some(source),
                        ..
                    }) = command
                    {
                        Some((report.clone(), *source))
                    } else {
                        None
                    }
                })
            })
            .next()
            .expect("arcade lander should spawn a mutant with arcade state");
        let expected_arcade = MutantArcadeState {
            x_fraction: lander_runtime.x_fraction,
            y_fraction: lander_runtime.y_fraction,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: ArcadeWaveProfile::for_wave(converted.wave)
                .mutant_shot_time
                .min(u32::from(u8::MAX)) as u8,
            sleep_ticks: 0,
            hop_rng: converted
                .arcade_rng
                .expect("playing report should expose arcade rng"),
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        assert_eq!(mutant_runtime, expected_arcade);

        let settled = driver.step(GameInput::NONE);
        let mutant = settled
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Mutant)
            .expect("mutant with arcade state should become a live actor");
        assert_eq!(mutant.mutant_runtime, Some(expected_arcade));

        let clean_state = settled.game_state();
        let clean_mutant = clean_state
            .world
            .enemies
            .iter()
            .find(|enemy| enemy.kind == CleanEnemyKind::Mutant)
            .expect("actor bridge should expose a clean mutant");
        assert_eq!(
            clean_mutant.mutant_runtime,
            Some(MutantRuntimeSnapshot {
                x_fraction: expected_arcade.x_fraction,
                y_fraction: expected_arcade.y_fraction,
                x_velocity: expected_arcade.x_velocity,
                y_velocity: expected_arcade.y_velocity,
                shot_timer: expected_arcade.shot_timer,
                sleep_ticks: expected_arcade.sleep_ticks,
                hop_rng: clean_arcade_rng(expected_arcade.hop_rng),
                render_x_correction: expected_arcade.render_x_correction,
                target6_first_shot_deferred: expected_arcade.target6_first_shot_deferred,
            })
        );
    }

    #[test]
    fn arcade_mutant_actor_advances_wave_velocity_and_hop_rng() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = MutantArcadeState {
            x_fraction: 0x20,
            y_fraction: 0x40,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 9,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(100, 80);
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let prompt = mutant_arcade_prompt_for_test(
            report.step,
            report.wave,
            report
                .arcade_rng
                .expect("playing report should carry source rng"),
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, shot) =
            expected_mutant_arcade_after_motion(start, source, mutant, &prompt, behavior);

        assert_eq!(shot, None);
        let snapshot = snapshot_for(&report, mutant);
        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.mutant_runtime, Some(expected_source));
        assert_eq!(
            expected_source.x_velocity,
            mutant_arcade_x_velocity(
                ArcadeWaveProfile::for_wave(1).mutant_x_velocity,
                arcade_absolute_x(Point::new(42, 120), 0),
                arcade_absolute_x(start, source.x_fraction),
            )
        );
        assert_ne!(expected_source.hop_rng, source.hop_rng);
    }

    #[test]
    fn arcade_mutant_actor_uses_prompt_arcade_wave_profile() {
        let actor = ActorId::new(1001);
        let default_profile = ArcadeWaveProfile::for_wave(1);
        let mut custom_arcade_profile = default_profile;
        custom_arcade_profile.mutant_x_velocity = 0x48;
        custom_arcade_profile.mutant_y_velocity_msb = 0x00;
        custom_arcade_profile.mutant_y_velocity_lsb = 0x40;
        custom_arcade_profile.mutant_random_y = 2;
        custom_arcade_profile.mutant_shot_time = 12;
        assert_ne!(
            custom_arcade_profile.mutant_x_velocity,
            default_profile.mutant_x_velocity
        );

        let source = MutantArcadeState {
            x_fraction: 0x20,
            y_fraction: 0x40,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 9,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(100, 80);
        let prompt = mutant_arcade_prompt_with_arcade_wave_for_test(
            12,
            1,
            custom_arcade_profile,
            ActorArcadeRngSnapshot {
                seed: 0x52,
                hseed: 0x34,
                lseed: 0x12,
            },
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, _shot) =
            expected_mutant_arcade_after_motion(start, source, actor, &prompt, behavior);
        let default_x_velocity = mutant_arcade_x_velocity(
            default_profile.mutant_x_velocity,
            arcade_absolute_x(Point::new(42, 120), 0),
            arcade_absolute_x(start, source.x_fraction),
        );

        let mut mutant = Mutant::from_spawn(
            actor,
            ActorMutantSpawn {
                position: start,
                source: Some(source),
            },
        );
        let reply = mutant.update(&prompt);
        let updated_source = reply
            .snapshot
            .mutant_runtime
            .expect("source mutant should keep source metadata");

        assert_ne!(updated_source.x_velocity, default_x_velocity);
        assert_eq!(reply.snapshot.position, expected_position);
        assert_eq!(updated_source, expected_source);
    }

    #[test]
    fn target6_arcade_lander_conversion_sets_mutant_render_correction() {
        let profile = ArcadeWaveProfile::for_wave(1);
        let hop_rng = ActorArcadeRngSnapshot {
            seed: 0x33,
            hseed: 0x44,
            lseed: 0x55,
        };
        let lander_runtime = LanderArcadeState {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 0,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: Some(6),
        };

        let mutant_runtime =
            MutantArcadeState::from_lander_conversion(lander_runtime, profile, hop_rng);

        assert_eq!(
            mutant_runtime.render_x_correction,
            TARGET6_MUTANT_CONVERSION_X_CORRECTION
        );
        assert_eq!(mutant_runtime.x_fraction, lander_runtime.x_fraction);
        assert_eq!(mutant_runtime.y_fraction, lander_runtime.y_fraction);

        let moving_lander = LanderArcadeState {
            x_velocity: 0x0030,
            ..lander_runtime
        };
        assert_eq!(
            MutantArcadeState::from_lander_conversion(moving_lander, profile, hop_rng)
                .render_x_correction,
            0
        );
    }

    #[test]
    fn target6_arcade_mutant_defers_first_entry_shot() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = MutantArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x81,
                hseed: 0x22,
                lseed: 0x44,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: false,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(4, 0x50),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);

        assert!(!report.sounds.contains(&SoundCue::MutantShot));
        assert!(first_enemy_laser_command(&report).is_none());
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .mutant_runtime
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(source.shot_timer, TARGET6_MUTANT_DEFERRED_SHOT_TIMER);
        assert_eq!(source.sleep_ticks, 0);
    }

    #[test]
    fn target6_arcade_mutant_visible_entry_shot_uses_projected_position() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = MutantArcadeState {
            x_fraction: 0x7C,
            y_fraction: 0x80,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: TARGET6_MUTANT_DEFERRED_SHOT_TIMER,
            sleep_ticks: MUTANT_LOOP_SLEEP_TICKS,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x44,
                hseed: 0x55,
                lseed: 0x66,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: false,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(0x03, 0x33),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let shot = first_enemy_laser_command(&report)
            .expect("visible target6 entry should emit a mutant shot");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        assert_eq!(shot.0, Point::new(0x13, 0x46));
        assert_eq!(shot.2.x_fraction, source.x_fraction);
        assert_eq!(shot.2.y_fraction, source.y_fraction);
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .mutant_runtime
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(source.shot_timer, TARGET6_MUTANT_DEFERRED_SHOT_TIMER);
        assert_eq!(source.sleep_ticks, MUTANT_LOOP_SLEEP_TICKS - 1);
    }

    #[test]
    fn target6_arcade_mutant_fire2524_sleep_shot_uses_exact_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = MutantArcadeState {
            x_fraction: 0x2C,
            y_fraction: 0x60,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: MUTANT_LOOP_SLEEP_TICKS,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x11,
                hseed: 0x22,
                lseed: 0x33,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: Point::new(0x08, 0x51),
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let shot =
            first_enemy_laser_command(&report).expect("fire2524 target6 row should force a shot");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        assert_eq!(shot.0, Point::new(0x1E, 0x54));
        assert_eq!(shot.1, arcade_screen_velocity(0xFFE0, 0x0138));
        assert_eq!(
            shot.2,
            EnemyProjectileArcadeState {
                x_fraction: 0x33,
                y_fraction: 0x56,
                x_velocity: 0xFFE0,
                y_velocity: 0x0138,
                lifetime_ticks: arcade_projectile_lifetime_ticks(MUTANT_SHOT_LIFETIME),
            }
        );
        let snapshot = snapshot_for(&report, mutant);
        let source = snapshot
            .mutant_runtime
            .expect("target6 mutant should keep source metadata");
        assert!(source.target6_first_shot_deferred);
        assert_eq!(
            source.shot_timer,
            TARGET6_MUTANT_FIRE2524_PENDING_SHOT_TIMER
        );
        assert_eq!(source.sleep_ticks, MUTANT_LOOP_SLEEP_TICKS - 1);
    }

    #[test]
    fn target6_arcade_mutant_shot_position_uses_dive_anchor_overrides() {
        let source = MutantArcadeState {
            x_fraction: 0x8C,
            y_fraction: 0xB0,
            x_velocity: 0,
            y_velocity: 0x0090,
            shot_timer: 0,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };

        assert_eq!(
            target6_mutant_arcade_shot_position(Point::new(0x08, 0x61), source),
            Point::new(0x1E, 0x70)
        );
        assert_eq!(
            target6_mutant_arcade_shot_position(
                Point::new(0x07, 0x78),
                MutantArcadeState {
                    x_fraction: 0xFC,
                    y_fraction: 0x00,
                    ..source
                },
            ),
            Point::new(0x21, 0x87)
        );
        assert_eq!(
            target6_mutant_arcade_shot_position(
                Point::new(0x03, 0x33),
                MutantArcadeState {
                    x_fraction: 0x7C,
                    y_fraction: 0x80,
                    ..source
                },
            ),
            Point::new(0x13, 0x46)
        );
    }

    #[test]
    fn target6_arcade_mutant_collision_position_offsets_dive_projection() {
        let source = MutantArcadeState {
            x_fraction: 0x8C,
            y_fraction: 0xB0,
            x_velocity: 0,
            y_velocity: 0x0090,
            shot_timer: 0,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };

        assert_eq!(
            target6_mutant_arcade_scene_position(Point::new(0x08, 0x61), Some(source)),
            Point::new(0x1E, 0x71)
        );
        assert_eq!(
            target6_mutant_arcade_collision_position(Point::new(0x08, 0x61), Some(source)),
            Point::new(0x1E, 0x72)
        );
    }

    #[test]
    fn target6_arcade_mutant_waits_for_fire2524_collision_window() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player_id = ActorId::new(100);
        let mutant_id = ActorId::new(101);
        let raw_position = Point::new(0x08, 0x99);
        let source = MutantArcadeState {
            x_fraction: 0x5C,
            y_fraction: 0xE0,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        let collision_position =
            target6_mutant_arcade_collision_position(raw_position, Some(source));
        driver.snapshots.insert(
            player_id,
            actor_snapshot_with_bounds(
                player_id,
                ActorKind::Player,
                collision_position,
                Rect::from_center(collision_position, 18, 10),
            ),
        );
        driver.snapshots.insert(
            mutant_id,
            mutant_arcade_snapshot_with_bounds(
                mutant_id,
                raw_position,
                source,
                Rect::from_center(collision_position, 14, 12),
            ),
        );

        let mut commands = Vec::new();
        driver.resolve_collisions(&ActorBehaviorScript::default(), &mut commands);

        assert!(
            commands.is_empty(),
            "pending fire2524 target6 mutant should not collide yet: {commands:?}"
        );
    }

    #[test]
    fn target6_arcade_mutant_fire2524_collision_projects_enemy_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player_id = ActorId::new(100);
        let mutant_id = ActorId::new(101);
        let raw_position = Point::new(0x08, 0xA5);
        let player_position = TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER;
        let source = MutantArcadeState {
            x_fraction: 0x00,
            y_fraction: 0x00,
            x_velocity: 0x0030,
            y_velocity: 0x0090,
            shot_timer: 0x80,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0,
                hseed: 0,
                lseed: 0,
            },
            render_x_correction: TARGET6_MUTANT_CONVERSION_X_CORRECTION,
            target6_first_shot_deferred: true,
        };
        driver.snapshots.insert(
            player_id,
            actor_snapshot_with_bounds(
                player_id,
                ActorKind::Player,
                player_position,
                Rect::from_center(player_position, 18, 10),
            ),
        );
        driver.snapshots.insert(
            mutant_id,
            mutant_arcade_snapshot_with_bounds(
                mutant_id,
                raw_position,
                source,
                Rect::from_center(player_position, 14, 12),
            ),
        );

        let mut commands = Vec::new();
        driver.resolve_collisions(&ActorBehaviorScript::default(), &mut commands);

        assert!(commands.contains(&GameCommand::Destroy(player_id)));
        assert!(commands.contains(&GameCommand::Destroy(mutant_id)));
        assert!(commands.contains(&GameCommand::AddScore(MUTANT_SCORE)));
        assert!(commands.contains(&GameCommand::PlaySound(SoundCue::MutantHit)));
        assert!(commands.contains(&GameCommand::PlayerKilled));
        let explosions = commands
            .iter()
            .filter_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Explosion {
                    position,
                    kind,
                    explosion_anchor,
                }) => Some((*position, *kind, *explosion_anchor)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(explosions.contains(&(
            TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_TOP_LEFT,
            ExplosionKind::Mutant,
            Some(TARGET6_MUTANT_FIRE2524_COLLISION_EXPLOSION_CENTER),
        )));
        assert!(explosions.contains(&(player_position, ExplosionKind::Player, None)));
    }

    #[test]
    fn arcade_mutant_shot_timer_spawns_arcade_projectile() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.step(GameInput::NONE);
        driver.wave = 1;
        let source = MutantArcadeState {
            x_fraction: 0x12,
            y_fraction: 0x34,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 1,
            sleep_ticks: 0,
            hop_rng: ActorArcadeRngSnapshot {
                seed: 0x71,
                hseed: 0x44,
                lseed: 0x88,
            },
            render_x_correction: 0,
            target6_first_shot_deferred: false,
        };
        let start = Point::new(70, 120);
        let mutant = driver.spawn_mutant_from_spawn(ActorMutantSpawn {
            position: start,
            source: Some(source),
        });

        let report = driver.step(GameInput::NONE);
        let prompt = mutant_arcade_prompt_for_test(
            report.step,
            report.wave,
            report
                .arcade_rng
                .expect("playing report should carry source rng"),
            Point::new(42, 120),
            Velocity::default(),
        );
        let behavior = ActorBehaviorProfile::default();
        let (expected_position, expected_source, expected_shot) =
            expected_mutant_arcade_after_motion(start, source, mutant, &prompt, behavior);
        let expected_shot = expected_shot.expect("shot timer should emit a mutant fireball");

        assert!(report.sounds.contains(&SoundCue::MutantShot));
        let mutant_shot = report
            .commands
            .iter()
            .find_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::EnemyLaser {
                    position,
                    velocity,
                    source,
                }) => source.map(|source| (*position, *velocity, source)),
                _ => None,
            })
            .expect("source mutant should emit a hostile shot command");
        assert_eq!(mutant_shot, expected_shot);
        assert_eq!(
            mutant_shot.2.lifetime_ticks,
            arcade_projectile_lifetime_ticks(MUTANT_SHOT_LIFETIME)
        );
        let snapshot = snapshot_for(&report, mutant);
        assert_eq!(snapshot.position, expected_position);
        assert_eq!(snapshot.mutant_runtime, Some(expected_source));
    }

    #[test]
    fn driver_resolves_laser_lander_collision_with_score_sound_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, 150);
        assert!(collision.sounds.contains(&SoundCue::LanderHit));
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert!(collision.commands.contains(&GameCommand::AddScore(150)));
        assert!(collision.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Lander,
                    ..
                })
            )
        }));
    }

    #[test]
    fn driver_resolves_laser_mutant_collision_with_sound_board_score_command() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_mutant_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, MUTANT_SCORE);
        assert!(collision.sounds.contains(&SoundCue::MutantHit));
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(MUTANT_SCORE))
        );
    }

    #[test]
    fn driver_resolves_laser_bomber_collision_with_sound_board_score_command_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_bomber_for_test(Point::new(62, 120));

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        assert!(fired.sounds.contains(&SoundCue::Laser));

        let collision = driver.step(GameInput::NONE);
        assert_eq!(collision.score, BOMBER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BomberHit));
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BOMBER_SCORE))
        );
    }

    #[test]
    fn driver_resolves_laser_pod_collision_with_sound_board_score_command_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let mut expected_rng = driver.arcade_rng;
        expected_rng.advance();
        let expected_first_swarmer = ActorSwarmerSpawn::from_pod_release(
            &mut expected_rng,
            ArcadeWaveProfile::for_wave(2),
            Point::new(64, 120),
        );
        for _ in 1..POD_SWARMER_REQUEST_LIMIT {
            ActorSwarmerSpawn::from_pod_release(
                &mut expected_rng,
                ArcadeWaveProfile::for_wave(2),
                Point::new(64, 120),
            );
        }
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, POD_SCORE);
        assert!(collision.sounds.contains(&SoundCue::PodHit));
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(POD_SCORE))
        );
        let swarmer_spawns = collision
            .commands
            .iter()
            .filter_map(|command| match command {
                GameCommand::Spawn(SpawnRequest::Swarmer { position, source }) => {
                    Some((*position, *source))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(swarmer_spawns.len(), POD_SWARMER_REQUEST_LIMIT);
        assert_eq!(
            swarmer_spawns[0],
            (
                expected_first_swarmer.position,
                expected_first_swarmer.source
            )
        );
        assert_eq!(driver.arcade_rng, expected_rng);

        let live = driver.step(GameInput::NONE);
        assert_eq!(
            driver.snapshot_count(ActorKind::Swarmer),
            POD_SWARMER_REQUEST_LIMIT
        );
        assert!(
            live.draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::Swarmer)
        );
    }

    #[test]
    fn driver_resolves_laser_baiter_collision_with_sound_board_score_command_and_explosion() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_baiter_for_test(Point::new(62, 120));

        driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let collision = driver.step(GameInput::NONE);

        assert_eq!(collision.score, BAITER_SCORE);
        assert!(collision.sounds.contains(&SoundCue::BaiterHit));
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 0);
        assert!(
            collision
                .commands
                .contains(&GameCommand::AddScore(BAITER_SCORE))
        );
    }

    #[test]
    fn bomb_collision_enters_game_over_with_sound_board_bomb_command() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.spawn_player();
        spawn_bomb_at_screen(&mut driver, Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::GameOver);
        assert!(report.sounds.contains(&SoundCue::BombHit));
        assert!(report.sounds.contains(&SoundCue::GameOver));
        assert!(report.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Explosion {
                    kind: ExplosionKind::Bomb,
                    ..
                })
            )
        }));
    }

    #[test]
    fn explosion_actor_draws_variant_metadata() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let explosion = driver.spawn_explosion(Point::new(90, 80), ExplosionKind::Bomb);

        let report = driver.step(GameInput::NONE);

        assert!(report.draws.iter().any(|draw| {
            draw.actor == explosion
                && draw.sprite == SpriteKey::Explosion
                && matches!(
                    draw.effect,
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomb,
                        age: 0,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn smart_bomb_pod_score_does_not_spawn_swarmers() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.smart_bombs = INITIAL_SMART_BOMBS;
        driver.spawn_player();
        driver.spawn_pod_for_test(Point::new(120, 120));

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        let report = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(report.score, POD_SCORE);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 0);
        assert!(!report.commands.iter().any(|command| {
            matches!(command, GameCommand::Spawn(SpawnRequest::Swarmer { .. }))
        }));
    }

    #[test]
    fn high_score_entry_is_driver_owned_phase() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.next_bonus = 20_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::HighScoreEntry);
        assert_eq!(report.lives, 0);
        assert!(report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn xyzzy_invincibility_keeps_player_alive_on_contact() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

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
    fn threaded_asset_is_prompted_once_per_driver_step() {
        let mut driver = ActorGameDriver::new();
        let first = driver.step(GameInput::NONE);
        let second = driver.step(GameInput::NONE);

        assert_eq!(first.step, 1);
        assert_eq!(second.step, 2);
        assert!(
            second
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::AttractDirector)
        );
    }

    fn started_driver() -> ActorGameDriver {
        let mut driver = ActorGameDriver::new();
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);
        driver
    }

    fn started_arcade_wave_driver(wave: u16) -> (ActorGameDriver, StepReport) {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = wave.max(1);
        driver.arcade_rng = PLAYFIELD_START_RNG;
        driver.apply_wave_profile();
        driver.spawn_player();
        driver.spawn_wave_hostiles();
        driver.spawn_initial_humans();
        driver.arm_first_wave_early_lander_reserve_delay();
        let report = driver.step(GameInput::NONE);
        (driver, report)
    }

    fn arcade_lander_spawn_row_for_test(
        spawn: ActorLanderSpawn,
    ) -> (u16, u16, u16, u16, u8, u8, u8, Option<usize>) {
        let arcade_state = spawn
            .source
            .expect("arcade lander spawn should carry state");
        let x16 = u16::from_be_bytes([spawn.position.x as u8, arcade_state.x_fraction]);
        let y16 = u16::from_be_bytes([spawn.position.y as u8, arcade_state.y_fraction]);
        (
            x16,
            y16,
            arcade_state.x_velocity,
            arcade_state.y_velocity,
            arcade_state.shot_timer,
            arcade_state.sleep_ticks,
            arcade_state.picture_frame,
            arcade_state.target_human_index,
        )
    }

    fn destroy_wave_hostiles(driver: &mut ActorGameDriver, report: &StepReport) {
        let commands = report
            .snapshots
            .iter()
            .filter(|snapshot| is_hostile(snapshot.kind))
            .map(|snapshot| GameCommand::Destroy(snapshot.id))
            .collect::<Vec<_>>();
        driver.apply_commands(&commands);
    }

    fn scene_has_survivor_bonus_icon(scene: &RenderScene, position: [f32; 2]) -> bool {
        scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == position
                && sprite.size == SURVIVOR_BONUS_HUMAN_SIZE
                && sprite.tint == Color::WHITE
        })
    }

    fn step_until_wave_started(driver: &mut ActorGameDriver, wave: u16) -> StepReport {
        for _ in 0..=256 {
            let report = driver.step(GameInput::NONE);
            if report.commands.contains(&GameCommand::AdvanceWave { wave }) {
                return report;
            }
        }

        panic!("wave {wave} should start after survivor bonus cadence");
    }

    fn step_until_driver_player_start_completes(
        driver: &mut ActorGameDriver,
        player: u8,
    ) -> StepReport {
        let mut previous_delay = PLAYER_START_PLAYFIELD_DELAY_STEPS.saturating_add(1);
        for _ in 0..=PLAYER_START_PLAYFIELD_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if let Some(player_start) = report.player_start {
                assert_eq!(player_start.player, player);
                assert!(player_start.delay_steps_remaining < previous_delay);
                previous_delay = player_start.delay_steps_remaining;
                assert!(
                    !report
                        .commands
                        .contains(&GameCommand::AdvanceWave { wave: 1 })
                );
                assert_no_arcade_message(&report, MessageId::PlayerOne, PLAYER_START_PROMPT_SCREEN_ADDRESS);
                continue;
            }

            assert_eq!(report.phase, Phase::Playing);
            assert_eq!(report.current_player, player);
            assert!(
                report
                    .commands
                    .contains(&GameCommand::AdvanceWave { wave: 1 })
            );
            assert_eq!(report.sounds, [SoundCue::PlayerAppear]);
            return report;
        }

        panic!("player {player} start should complete after source delay");
    }

    fn step_until_driver_smart_bomb_detonates(driver: &mut ActorGameDriver) -> StepReport {
        for _ in 0..=SMART_BOMB_DETONATION_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report.smart_bomb_flash_steps_remaining == SMART_BOMB_FLASH_STEPS {
                return report;
            }
        }

        panic!("source smart bomb should detonate after the source delay");
    }

    fn step_until_final_game_over_sleep_returns_to_attract(
        runtime: &mut ActorRuntimeAdapter,
    ) -> ActorFrame {
        for expected_sleep in (1..FINAL_GAME_OVER_DELAY_STEPS).rev() {
            let waiting = runtime.step(GameInput::NONE);
            assert_eq!(waiting.report.phase, Phase::GameOver);
            assert_eq!(
                waiting.report.player_death_sleep_remaining,
                Some(expected_sleep)
            );
            assert_eq!(
                waiting.state.game_over.player_death_sleep_remaining,
                Some(expected_sleep)
            );
            assert_arcade_message(&waiting.report, MessageId::GameOver, FINAL_GAME_OVER_SCREEN_ADDRESS);
            assert_arcade_message_scene(&waiting.scene, MessageId::GameOver, FINAL_GAME_OVER_SCREEN_ADDRESS);
            assert!(
                !waiting
                    .report
                    .draws
                    .iter()
                    .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
            );
        }

        let returned = runtime.step(GameInput::NONE);
        assert_no_arcade_message(&returned.report, MessageId::GameOver, FINAL_GAME_OVER_SCREEN_ADDRESS);
        returned
    }

    fn step_until_player_switch_completes(
        runtime: &mut ActorRuntimeAdapter,
        to_player: u8,
    ) -> ActorFrame {
        let from_player = if to_player == 1 { 2 } else { 1 };
        for expected_sleep in (1..PLAYER_SWITCH_DELAY_STEPS).rev() {
            let waiting = runtime.step(GameInput::NONE);
            assert_eq!(waiting.report.phase, Phase::GameOver);
            assert_eq!(
                waiting.report.player_switch,
                Some(PlayerSwitchReport {
                    sleep_steps_remaining: expected_sleep,
                    from_player,
                    to_player,
                })
            );
            assert_eq!(
                waiting.state.game_over.player_switch_sleep_remaining,
                Some(expected_sleep)
            );
            assert!(!waiting.events.gameplay().contains(&GameEvent::GameOver));
            assert!(!waiting.report.sounds.contains(&SoundCue::GameOver));
            assert_arcade_message(
                &waiting.report,
                player_message(from_player),
                PLAYER_SWITCH_LABEL_SCREEN_ADDRESS,
            );
            assert_arcade_message(
                &waiting.report,
                MessageId::GameOver,
                PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS,
            );
            assert_arcade_message_scene(
                &waiting.scene,
                player_message(from_player),
                PLAYER_SWITCH_LABEL_SCREEN_ADDRESS,
            );
            assert_arcade_message_scene(
                &waiting.scene,
                MessageId::GameOver,
                PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS,
            );
            assert_no_arcade_message(
                &waiting.report,
                player_message(to_player),
                PLAYER_START_PROMPT_SCREEN_ADDRESS,
            );
            assert!(
                !waiting
                    .report
                    .draws
                    .iter()
                    .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
            );
        }

        let switched = runtime.step(GameInput::NONE);
        assert_no_arcade_message(
            &switched.report,
            player_message(from_player),
            PLAYER_SWITCH_LABEL_SCREEN_ADDRESS,
        );
        assert_no_arcade_message(
            &switched.report,
            MessageId::GameOver,
            PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS,
        );
        assert_arcade_message(
            &switched.report,
            player_message(to_player),
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
        assert_arcade_message_scene(
            &switched.scene,
            player_message(to_player),
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
        switched
    }

    fn step_until_player_start_completes(
        runtime: &mut ActorRuntimeAdapter,
        player: u8,
    ) -> ActorFrame {
        let mut previous_delay = PLAYER_START_PLAYFIELD_DELAY_STEPS.saturating_add(1);
        for _ in 0..=PLAYER_START_PLAYFIELD_DELAY_STEPS {
            let frame = runtime.step(GameInput::NONE);
            if let Some(player_start) = frame.report.player_start {
                assert_eq!(player_start.player, player);
                assert!(player_start.delay_steps_remaining < previous_delay);
                previous_delay = player_start.delay_steps_remaining;
                assert!(!frame.events.gameplay().contains(&GameEvent::WaveStarted));
                assert!(frame.state.world.enemies.is_empty());
                if frame.report.player_count > 1 {
                    assert_arcade_message(
                        &frame.report,
                        player_message(player),
                        PLAYER_START_PROMPT_SCREEN_ADDRESS,
                    );
                    assert_no_arcade_message(
                        &frame.report,
                        MessageId::GameOver,
                        PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS,
                    );
                } else {
                    assert_no_arcade_message(
                        &frame.report,
                        player_message(player),
                        PLAYER_START_PROMPT_SCREEN_ADDRESS,
                    );
                }
                continue;
            }

            assert_eq!(frame.report.phase, Phase::Playing);
            assert_eq!(frame.report.current_player, player);
            assert_no_arcade_message(
                &frame.report,
                player_message(player),
                PLAYER_START_PROMPT_SCREEN_ADDRESS,
            );
            assert_no_arcade_message(&frame.report, MessageId::GameOver, PLAYER_SWITCH_GAME_OVER_SCREEN_ADDRESS);
            assert!(frame.events.gameplay().contains(&GameEvent::WaveStarted));
            assert_eq!(
                frame.events.sounds(),
                &[SoundEvent::UnmappedSoundCommand { command: 0xEA }]
            );
            assert_eq!(frame.report.sounds, [SoundCue::PlayerAppear]);
            return frame;
        }

        panic!("player {player} start should complete after source delay");
    }

    fn step_until_smart_bomb_detonates(runtime: &mut ActorRuntimeAdapter) -> ActorFrame {
        for _ in 0..=SMART_BOMB_DETONATION_DELAY_STEPS {
            let frame = runtime.step(GameInput::NONE);
            if frame.report.smart_bomb_flash_steps_remaining == SMART_BOMB_FLASH_STEPS {
                return frame;
            }
        }

        panic!("source smart bomb should detonate after the source delay");
    }

    fn smart_bomb_sound_board_cues() -> Vec<SoundCue> {
        SMART_BOMB_SOUND_SEQUENCE
            .iter()
            .map(|(_, command)| SoundCue::SoundBoardCommand(*command))
            .collect()
    }

    fn terrain_blow_sound_board_cues() -> Vec<(u16, Vec<SoundCue>)> {
        TERRAIN_BLOW_START_SOUND_FRAMES
            .iter()
            .copied()
            .map(|frame| {
                (
                    frame,
                    vec![SoundCue::SoundBoardCommand(SMART_BOMB_SOUND_COMMAND)],
                )
            })
            .chain(std::iter::once((
                TERRAIN_BLOW_COMPLETE_FRAME,
                vec![SoundCue::SoundBoardCommand(TERRAIN_BLOW_SOUND_COMMAND)],
            )))
            .chain(
                TERRAIN_BLOW_SOUND_TAIL_SEQUENCE
                    .iter()
                    .copied()
                    .map(|(offset, command)| {
                        (
                            TERRAIN_BLOW_COMPLETE_FRAME + u16::from(offset),
                            vec![SoundCue::SoundBoardCommand(command)],
                        )
                    }),
            )
            .collect()
    }

    fn collect_driver_smart_bomb_sound_sequence(driver: &mut ActorGameDriver) -> Vec<SoundCue> {
        let mut sounds = Vec::new();
        let last_step = SMART_BOMB_SOUND_SEQUENCE
            .last()
            .expect("smart bomb sound sequence should not be empty")
            .0;
        for _ in 0..last_step {
            sounds.extend(driver.step(GameInput::NONE).sounds);
        }
        sounds
    }

    fn step_until_driver_reserve_activation_spawns_lander(
        driver: &mut ActorGameDriver,
    ) -> StepReport {
        for _ in 0..=SMART_BOMB_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. })))
            {
                return report;
            }
        }

        panic!("enemy reserve should reactivate after smart-bomb cooldown");
    }

    fn step_until_first_wave_early_reserve_materializes(
        driver: &mut ActorGameDriver,
    ) -> StepReport {
        for _ in 0..=FIRST_WAVE_EARLY_RESERVE_DELAY_STEPS {
            let report = driver.step(GameInput::NONE);
            if report.sounds.contains(&SoundCue::HyperspaceMaterialize) {
                return report;
            }
        }

        panic!("first-wave early reserve should materialize on arcade cadence");
    }

    fn snapshot_for(report: &StepReport, id: ActorId) -> &ActorSnapshot {
        report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.id == id)
            .expect("actor snapshot should be present")
    }

    fn arcade_human_spawn_for_test(
        position: Point,
        target_slot_index: usize,
        picture_frame: u8,
    ) -> ActorHumanSpawn {
        ActorHumanSpawn {
            position,
            mode: HumanMode::Grounded,
            source: Some(HumanArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame,
                target_slot_index,
            }),
        }
    }

    fn expected_bomber_after_arcade_motion(
        position: Point,
        mut arcade_state: BomberArcadeState,
        _step: u64,
        _id: ActorId,
        arcade_rng: Option<ActorArcadeRngSnapshot>,
        player_position: Option<Point>,
    ) -> (Point, BomberArcadeState) {
        if let Some(arcade_rng) = arcade_rng
            && arcade_state.slot == arcade_tie_selected_slot(arcade_rng.seed)
        {
            if arcade_state.sleep_ticks > 0 {
                arcade_state.sleep_ticks = arcade_state.sleep_ticks.saturating_sub(1);
            } else {
                arcade_state.picture_frame =
                    bomber_sprite_frame_after_arcade_seed(arcade_rng.seed, arcade_state.picture_frame);
                arcade_state.y_velocity =
                    bomber_seeded_y_velocity(arcade_state.y_velocity, arcade_rng.seed);
                if position.y == 0 {
                    arcade_state.y_velocity = bomber_cruise_y_velocity(
                        arcade_state.y_velocity,
                        &mut arcade_state.cruise_altitude,
                        position.y,
                        arcade_rng.seed,
                    );
                } else if let Some(player) = player_position
                    && let Some(delta) =
                        bomber_player_tracking_y_velocity_delta(position.y, player.y)
                {
                    arcade_state.y_velocity = arcade_state.y_velocity.wrapping_add(delta);
                }
                arcade_state.sleep_ticks = BOMBER_LOOP_SLEEP_TICKS;
            }
        }

        let (x, x_fraction) =
            arcade_axis_step(position.x, arcade_state.x_fraction, arcade_state.x_velocity);
        let (y, y_fraction) =
            arcade_active_object_y_step(position.y, arcade_state.y_fraction, arcade_state.y_velocity);
        arcade_state.x_fraction = x_fraction;
        arcade_state.y_fraction = y_fraction;
        (Point::new(x, y), arcade_state)
    }

    fn actor_snapshot(id: u64, kind: ActorKind, position: Point) -> ActorSnapshot {
        actor_snapshot_with_bounds(
            ActorId(id),
            kind,
            position,
            Rect::from_center(position, 4, 4),
        )
    }

    fn actor_snapshot_with_bounds(
        id: ActorId,
        kind: ActorKind,
        position: Point,
        bounds: Rect,
    ) -> ActorSnapshot {
        ActorSnapshot {
            id,
            kind,
            position,
            velocity: Velocity::default(),
            direction: None,
            bounds: Some(bounds),
            alive: true,
            lander_runtime: None,
            bomber_runtime: None,
            pod_runtime: None,
            swarmer_runtime: None,
            baiter_runtime: None,
            mutant_runtime: None,
            human_runtime: None,
            enemy_projectile_runtime: None,
        }
    }

    fn mutant_arcade_snapshot_with_bounds(
        id: ActorId,
        position: Point,
        source: MutantArcadeState,
        bounds: Rect,
    ) -> ActorSnapshot {
        let mut snapshot = actor_snapshot_with_bounds(id, ActorKind::Mutant, position, bounds);
        snapshot.mutant_runtime = Some(source);
        snapshot
    }

    fn actor_snapshot_with_velocity(
        id: u64,
        kind: ActorKind,
        position: Point,
        velocity: Velocity,
    ) -> ActorSnapshot {
        let mut snapshot = actor_snapshot(id, kind, position);
        snapshot.velocity = velocity;
        snapshot
    }

    fn arcade_world_projection_report_for_test(background_left: u16) -> StepReport {
        let mut player = actor_snapshot(1, ActorKind::Player, Point::new(128, 100));
        player.direction = Some(Direction::Right);

        let mut lander = actor_snapshot(2, ActorKind::Lander, Point::new(0x30, 80));
        lander.lander_runtime = Some(LanderArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            shot_timer: 0,
            sleep_ticks: 0,
            picture_frame: 0,
            target_human_index: None,
        });

        let mut enemy_laser = actor_snapshot(3, ActorKind::EnemyLaser, Point::new(0x31, 96));
        enemy_laser.enemy_projectile_runtime = Some(EnemyProjectileArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 12,
        });

        let mut bomb = actor_snapshot(4, ActorKind::Bomb, Point::new(0x31, 104));
        bomb.enemy_projectile_runtime = Some(EnemyProjectileArcadeState {
            x_fraction: 0,
            y_fraction: 0,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 12,
        });

        let mut human = actor_snapshot(5, ActorKind::Human, Point::new(0x2E, 220));
        human.human_runtime = Some(HumanArcadeState {
            x_fraction: 0x80,
            y_fraction: 0,
            picture_frame: 2,
            target_slot_index: 4,
        });

        StepReport {
            step: 123,
            phase: Phase::Playing,
            wave: 1,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: 3,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [PlayerStockSnapshot::new(3, 3); 2],
            next_bonus: REPLAY_BONUS_SCORE,
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [10_000, 7_500, 5_000, 2_500, 1_000],
            arcade_wave: ArcadeWaveProfile::for_wave(1),
            high_score_initials: HighScoreInitialsState::EMPTY,
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            background_left,
            arcade_rng: None,
            terrain_blow: None,
            snapshots: vec![player, lander, enemy_laser, bomb, human],
            draws: vec![
                DrawCommand::sprite(
                    ActorId::new(1),
                    SpriteKey::PlayerRight,
                    Point::new(128, 100),
                ),
                DrawCommand::sprite(ActorId::new(2), SpriteKey::Lander, Point::new(0x30, 80)),
                DrawCommand::sprite(ActorId::new(3), SpriteKey::EnemyLaser, Point::new(0x31, 96)),
                DrawCommand::sprite(ActorId::new(4), SpriteKey::Bomb, Point::new(0x31, 104)),
                DrawCommand::sprite_with_effect(
                    ActorId::new(5),
                    SpriteKey::Human,
                    Point::new(0x2E, 220),
                    VisualEffect::HumanSpriteFrame { frame: 2 },
                ),
            ],
            sounds: Vec::new(),
            commands: Vec::new(),
        }
    }

    fn sprite_position_for_test(
        scene: &RenderScene,
        sprite: SpriteId,
        layer: RenderLayer,
    ) -> Option<[f32; 2]> {
        scene
            .sprites
            .iter()
            .find(|candidate| candidate.sprite == sprite && candidate.layer == layer)
            .map(|candidate| candidate.position)
    }

    fn sprite_positions_for_test(
        scene: &RenderScene,
        sprite: SpriteId,
        layer: RenderLayer,
    ) -> Vec<[f32; 2]> {
        scene
            .sprites
            .iter()
            .filter(|candidate| candidate.sprite == sprite && candidate.layer == layer)
            .map(|candidate| candidate.position)
            .collect()
    }

    fn mutant_arcade_prompt_for_test(
        step: u64,
        wave: u16,
        arcade_rng: ActorArcadeRngSnapshot,
        player_position: Point,
        player_velocity: Velocity,
    ) -> StepPrompt {
        mutant_arcade_prompt_with_arcade_wave_for_test(
            step,
            wave,
            ArcadeWaveProfile::for_wave(wave),
            arcade_rng,
            player_position,
            player_velocity,
        )
    }

    fn playing_player_prompt_for_test(input: GameInput, background_left: u16) -> StepPrompt {
        StepPrompt {
            step: 1,
            phase: Phase::Playing,
            input,
            wave: 1,
            arcade_wave: ArcadeWaveProfile::for_wave(1),
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: INITIAL_PLAYER_LIVES,
            smart_bombs: INITIAL_SMART_BOMBS,
            smart_bomb_pending: false,
            player_stocks: [PlayerStockSnapshot::new(INITIAL_PLAYER_LIVES, INITIAL_SMART_BOMBS); 2],
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [0; 5],
            high_score_initials: HighScoreInitialsState::EMPTY,
            snapshots: Vec::new(),
            behavior_script: ActorBehaviorScript::default(),
            background_left,
            arcade_rng: None,
            human_walk_target_slot: None,
            projectile_scan_tick: false,
        }
    }

    fn mutant_arcade_prompt_with_arcade_wave_for_test(
        step: u64,
        wave: u16,
        arcade_wave: ArcadeWaveProfile,
        arcade_rng: ActorArcadeRngSnapshot,
        player_position: Point,
        player_velocity: Velocity,
    ) -> StepPrompt {
        StepPrompt {
            step,
            phase: Phase::Playing,
            input: GameInput::NONE,
            wave,
            arcade_wave,
            current_player: 1,
            player_count: 1,
            score: 0,
            player_scores: [0, 0],
            credits: 0,
            lives: 3,
            smart_bombs: INITIAL_SMART_BOMBS,
            smart_bomb_pending: false,
            player_stocks: [PlayerStockSnapshot::new(3, INITIAL_SMART_BOMBS); 2],
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [0; 5],
            high_score_initials: HighScoreInitialsState::EMPTY,
            snapshots: vec![actor_snapshot_with_velocity(
                999,
                ActorKind::Player,
                player_position,
                player_velocity,
            )],
            behavior_script: ActorBehaviorScript::default(),
            background_left: 0,
            arcade_rng: Some(arcade_rng),
            human_walk_target_slot: None,
            projectile_scan_tick: false,
        }
    }

    fn expected_mutant_arcade_after_motion(
        mut position: Point,
        mut source: MutantArcadeState,
        actor: ActorId,
        prompt: &StepPrompt,
        behavior: ActorBehaviorProfile,
    ) -> (
        Point,
        MutantArcadeState,
        Option<(Point, Velocity, EnemyProjectileArcadeState)>,
    ) {
        if source.sleep_ticks > 0 {
            source.sleep_ticks = source.sleep_ticks.saturating_sub(1);
            return (position, source, None);
        }

        let player_position = prompt
            .player_position()
            .expect("arcade mutant expected helper needs a player");
        let profile = prompt.arcade_wave;
        let player_absolute_x = arcade_absolute_x(player_position, 0);
        let object_absolute_x = arcade_absolute_x(position, source.x_fraction);
        source.x_velocity = mutant_arcade_x_velocity(
            profile.mutant_x_velocity,
            player_absolute_x,
            object_absolute_x,
        );
        source.y_velocity = mutant_arcade_y_velocity(
            profile,
            player_position.y,
            player_absolute_x,
            object_absolute_x,
            position,
        );

        let mut shot = None;
        if mutant_arcade_should_hop_and_shoot(player_absolute_x, object_absolute_x, position) {
            let mut hop_rng = arcade_rng_from_snapshot(source.hop_rng);
            let hop_state = hop_rng.advance();
            source.hop_rng = hop_state.snapshot();
            position.y = mutant_arcade_hop_y(position.y, profile.mutant_random_y, hop_state.seed);
            source.shot_timer = source.shot_timer.wrapping_sub(1);
            if source.shot_timer == 0 {
                let shot_rng = mutant_arcade_shot_rng(prompt, actor, position);
                source.shot_timer = mutant_arcade_shot_reset(profile, shot_rng.seed);
                shot = mutant_arcade_fireball(position, prompt, behavior, source, shot_rng)
                    .map(|(velocity, projectile_source)| (position, velocity, projectile_source));
            }
        }

        let (x, x_fraction) =
            arcade_axis_step(position.x, source.x_fraction, source.x_velocity);
        let (y, y_fraction) =
            arcade_active_object_y_step(position.y, source.y_fraction, source.y_velocity);
        source.x_fraction = x_fraction;
        source.y_fraction = y_fraction;
        source.sleep_ticks = MUTANT_LOOP_SLEEP_TICKS;
        (Point::new(x, y), source, shot)
    }

    fn enemy_projectile_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| is_enemy_projectile_kind(snapshot.kind))
            .count()
    }

    fn bomb_projectile_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
            .count()
    }

    fn first_enemy_laser_command(
        report: &StepReport,
    ) -> Option<(Point, Velocity, EnemyProjectileArcadeState)> {
        report.commands.iter().find_map(|command| match command {
            GameCommand::Spawn(SpawnRequest::EnemyLaser {
                position,
                velocity,
                source: Some(source),
            }) => Some((*position, *velocity, *source)),
            _ => None,
        })
    }

    fn enemy_laser_snapshot_count(report: &StepReport) -> usize {
        report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::EnemyLaser)
            .count()
    }

    fn assert_text(report: &StepReport, value: &str) {
        assert!(
            report
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(value)),
            "expected draw text {value:?}"
        );
    }

    fn assert_no_text(report: &StepReport, value: &str) {
        assert!(
            !report
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(value)),
            "unexpected draw text {value:?}"
        );
    }

    fn assert_arcade_message(
        report: &StepReport,
        message: MessageId,
        top_left_screen_address: u16,
    ) {
        let scene = report.render_scene();
        assert_arcade_message_scene(&scene, message, top_left_screen_address);
    }

    fn assert_arcade_message_scene(
        scene: &RenderScene,
        message: MessageId,
        top_left_screen_address: u16,
    ) {
        for (sprite_id, position, size) in
            expected_plain_arcade_message_sprites(message, top_left_screen_address)
        {
            assert!(
                scene.sprites.iter().any(|sprite| {
                    sprite.sprite == sprite_id
                        && sprite.layer == RenderLayer::Overlay
                        && sprite.position == position
                        && sprite.size == size
                        && sprite.tint == Color::WHITE
                }),
                "expected full arcade message {message:?} glyph {sprite_id:?} at {top_left_screen_address:#06x}"
            );
        }
    }

    fn expected_plain_arcade_message_sprites(
        message: MessageId,
        top_left_screen_address: u16,
    ) -> Vec<(SpriteId, [f32; 2], [f32; 2])> {
        let text = crate::arcade_assets::message_text(message);
        let mut cursor = top_left_screen_address;
        let mut expected = Vec::new();
        for character in text.chars() {
            let size = SpriteId::message_glyph_size(character)
                .expect("test arcade prompt should use clean message glyphs");
            if character != ' ' {
                let sprite =
                    SpriteId::message_glyph(character).expect("visible prompt glyph should exist");
                expected.push((
                    sprite,
                    source_screen_position(cursor),
                    [size[0] as f32, size[1] as f32],
                ));
            }
            cursor = arcade_test_text_cursor_after_glyph(cursor, size[0]);
        }
        assert!(
            !expected.is_empty(),
            "arcade message {message:?} should contain visible glyphs"
        );
        expected
    }

    fn arcade_test_text_cursor_after_glyph(cursor: u16, width_pixels: u32) -> u16 {
        let [column, row] = cursor.to_be_bytes();
        let width_columns =
            u8::try_from(width_pixels / 2).expect("arcade glyph width should fit in u8");
        u16::from_be_bytes([column.wrapping_add(width_columns).wrapping_add(1), row])
    }

    fn assert_no_arcade_message(
        report: &StepReport,
        message: MessageId,
        top_left_screen_address: u16,
    ) {
        let text = crate::arcade_assets::message_text(message);
        let first_glyph = text
            .chars()
            .find_map(SpriteId::message_glyph)
            .expect("arcade message should contain a visible glyph");
        let position = source_screen_position(top_left_screen_address);
        let scene = report.render_scene();
        assert!(
            scene.sprites.iter().all(|sprite| {
                sprite.sprite != first_glyph
                    || sprite.layer != RenderLayer::Overlay
                    || sprite.position != position
            }),
            "unexpected arcade message {message:?} at {top_left_screen_address:#06x}"
        );
    }
