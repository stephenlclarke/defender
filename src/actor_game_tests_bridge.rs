    fn arcade_projectile_at_screen(
        position: Point,
        background_left: u16,
    ) -> (Point, EnemyProjectileArcadeState) {
        let screen_x = u16::try_from(position.x).expect("test screen x should be non-negative");
        let absolute_x = background_left.wrapping_add(screen_x << OBJECT_WORLD_TO_SCREEN_SHIFT);
        let [x, x_fraction] = absolute_x.to_be_bytes();
        (
            Point::new(i16::from(x), position.y),
            EnemyProjectileArcadeState {
                x_fraction,
                y_fraction: 0,
                x_velocity: 0,
                y_velocity: 0,
                lifetime_ticks: 0,
            },
        )
    }

    fn spawn_enemy_laser_at_screen(driver: &mut ActorGameDriver, position: Point) -> ActorId {
        let (arcade_position, arcade_state) =
            arcade_projectile_at_screen(position, driver.background_left);
        driver.spawn_enemy_laser_from_spawn(
            arcade_position,
            Velocity::new(0, 0),
            Some(arcade_state),
        )
    }

    fn spawn_bomb_at_screen(driver: &mut ActorGameDriver, position: Point) -> ActorId {
        let (arcade_position, arcade_state) =
            arcade_projectile_at_screen(position, driver.background_left);
        driver.spawn_bomb(arcade_position, Some(arcade_state))
    }

    #[test]
    fn actor_sound_cues_expose_sound_board_commands() {
        let expected = [
            (SoundCue::Credit, 0xE6),
            (SoundCue::Start, 0xF5),
            (SoundCue::Thrust, 0xE9),
            (SoundCue::Laser, 0xEB),
            (SoundCue::SmartBomb, 0xEE),
            (SoundCue::PlayerAppear, 0xEA),
            (SoundCue::HyperspaceMaterialize, 0xEA),
            (SoundCue::Explosion, 0xEE),
            (SoundCue::LanderHit, 0xF9),
            (SoundCue::LanderPickup, 0xF4),
            (SoundCue::HumanPulled, 0xF1),
            (SoundCue::HumanReleased, 0xE5),
            (SoundCue::HumanRescued, 0xF7),
            (SoundCue::HumanSafeLanding, 0xE0),
            (SoundCue::HumanLost, 0xEE),
            (SoundCue::MutantSpawn, 0xEE),
            (SoundCue::MutantHit, 0xE8),
            (SoundCue::BomberHit, 0xFE),
            (SoundCue::BombHit, 0xEE),
            (SoundCue::PodHit, 0xFA),
            (SoundCue::SwarmerHit, 0xF8),
            (SoundCue::LanderShot, 0xFC),
            (SoundCue::MutantShot, 0xF6),
            (SoundCue::SwarmerShot, 0xF3),
            (SoundCue::BaiterHit, 0xF8),
            (SoundCue::BaiterShot, 0xFC),
            (SoundCue::GameOver, 0xEC),
            (SoundCue::SoundBoardCommand(0xE8), 0xE8),
        ];

        for (cue, command) in expected {
            assert_eq!(cue.sound_board_command(), Some(command), "{cue:?}");
        }
        for cue in [SoundCue::Hyperspace, SoundCue::AttractPulse] {
            assert_eq!(cue.sound_board_command(), None, "{cue:?}");
        }
    }

    #[test]
    fn actor_sound_cues_map_to_clean_sound_events() {
        assert_eq!(
            SoundCue::Credit.sound_event(),
            Some(SoundEvent::CreditAdded)
        );
        assert_eq!(SoundCue::Start.sound_event(), Some(SoundEvent::GameStarted));
        assert_eq!(
            SoundCue::Thrust.sound_event(),
            Some(SoundEvent::ThrustStarted)
        );
        assert_eq!(
            SoundCue::Laser.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xEB })
        );
        assert_eq!(
            SoundCue::PlayerAppear.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xEA })
        );
        assert_eq!(
            SoundCue::LanderShot.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xFC })
        );
        assert_eq!(
            SoundCue::MutantShot.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xF6 })
        );
        assert_eq!(
            SoundCue::SoundBoardCommand(0xE8).sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xE8 })
        );
        assert_eq!(
            SoundCue::HumanReleased.sound_event(),
            Some(SoundEvent::UnmappedSoundCommand { command: 0xE5 })
        );
        assert_eq!(SoundCue::Hyperspace.sound_event(), None);
    }

    #[test]
    fn actor_sound_event_bridge_emits_clean_audio_events_and_thrust_edges() {
        let mut bridge = ActorSoundEventBridge::new();

        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Credit, SoundCue::Laser]),
            [
                SoundEvent::CreditAdded,
                SoundEvent::UnmappedSoundCommand { command: 0xEB },
            ]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Thrust, SoundCue::LanderShot]),
            [
                SoundEvent::ThrustStarted,
                SoundEvent::UnmappedSoundCommand { command: 0xFC },
            ]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::Thrust, SoundCue::SwarmerShot]),
            [SoundEvent::UnmappedSoundCommand { command: 0xF3 }]
        );
        assert_eq!(
            ActorSoundEventBridge::new().sound_events_for_cues(&[SoundCue::MutantShot]),
            [SoundEvent::UnmappedSoundCommand { command: 0xF6 }]
        );
        assert_eq!(
            ActorSoundEventBridge::new().sound_events_for_cues(&[SoundCue::SoundBoardCommand(0xE8)]),
            [SoundEvent::UnmappedSoundCommand { command: 0xE8 }]
        );
        assert_eq!(
            bridge.sound_events_for_cues(&[SoundCue::HumanReleased]),
            [
                SoundEvent::UnmappedSoundCommand { command: 0xE5 },
                SoundEvent::ThrustStopped,
            ]
        );
        assert!(bridge.sound_events_for_cues(&[]).is_empty());
    }

    #[test]
    fn step_report_uses_actor_sound_event_bridge() {
        let mut driver = started_driver();
        let mut bridge = ActorSoundEventBridge::new();

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert_eq!(
            fired.sound_events(&mut bridge),
            [SoundEvent::UnmappedSoundCommand { command: 0xEB }]
        );

        let thrusting = driver.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert_eq!(
            thrusting.sound_events(&mut bridge),
            [SoundEvent::ThrustStarted]
        );

        let coasting = driver.step(GameInput::NONE);
        assert_eq!(
            coasting.sound_events(&mut bridge),
            [SoundEvent::ThrustStopped]
        );
    }

    #[test]
    fn actor_render_scene_bridge_projects_attract_source_pixels() {
        let mut driver = ActorGameDriver::new();

        let williams = driver.step(GameInput::NONE);
        let williams_scene = williams.render_scene();
        assert_eq!(williams_scene.frame, williams.step);
        assert_eq!(williams_scene.surface, ACTOR_RENDER_SURFACE);
        assert!(williams_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(
            !williams_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO)
        );
        assert!(!williams_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H && sprite.layer == RenderLayer::Overlay
        }));

        let mut coalescing = None;
        for _ in 0..DEFENDER_WORDMARK_START_STEP {
            let report = driver.step(GameInput::NONE);
            if report
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
            {
                coalescing = Some(report);
                break;
            }
        }
        let coalescing = coalescing.expect("Defender wordmark should coalesce");
        let coalescing_scene = coalescing.render_scene();
        assert!(coalescing_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(
            !coalescing_scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO)
        );

        let mut hall = None;
        for _ in ATTRACT_DEFENDER_WORDMARK_START_STEP..ATTRACT_HALL_OF_FAME_START_STEP {
            hall = Some(driver.step(GameInput::NONE));
        }
        let hall_scene = hall
            .expect("hall-of-fame page boundary should be reached")
            .render_scene();
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn actor_render_scene_bridge_projects_playing_actors_and_status_text() {
        let mut driver = started_driver();

        let report = driver.step(GameInput::NONE);
        let state = report.game_state();
        let scene = report.render_scene();

        assert!(state.world.scanner.enabled);
        assert!(state.world.scanner.scan_left.is_some());
        assert!(state.world.scanner.player_blip.is_some());
        assert!(state.world.object_evidence.detail_count > 0);
        assert_eq!(scene.frame, report.step);
        assert!(scene.sprites.iter().any(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT
            ) && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            SpriteId::SCORE_DIGITS.contains(&sprite.sprite) && sprite.layer == RenderLayer::Hud
        }));
        assert!(
            scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
                .count()
                >= TOP_DISPLAY_BORDER_SEGMENTS.len()
        );
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD
                && sprite.tint == VISUAL_STATE.top_display_scanner_marker_tint()
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL
                && sprite.layer == RenderLayer::Hud
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP && sprite.layer == RenderLayer::Hud
        }));
    }

    #[test]
    fn actor_render_scene_bridge_ignores_armed_terrain_flash_until_erased() {
        let mut driver = started_driver();
        let mut report = driver.step(GameInput::NONE);
        let mut terrain_blow = TerrainBlowSnapshot::source_armed_terrain_visible();
        terrain_blow.source_elapsed_frames = 2;
        report.terrain_blow = Some(terrain_blow);

        let scene = report.render_scene();

        assert!(!terrain_blow.terrain_erased());
        assert_ne!(
            source_terrain_blow_flash_tint(terrain_blow.source_elapsed_frames),
            Color { rgba: [0; 4] }
        );
        assert_eq!(scene.clear_color, Color { rgba: [0; 4] });
    }

    #[test]
    fn actor_wave_clear_delays_next_wave_and_draws_source_survivor_bonus_scene() {
        let wave_script = ActorWaveScript::new(
            "wave-clear-interstitial",
            vec![
                ActorWaveProfile::with_spawns(
                    1,
                    ActorBehaviorScript::default(),
                    vec![ActorLanderSpawn::new(Point::new(100, 80))],
                    vec![
                        ActorHumanSpawn::new(Point::new(40, HUMAN_GROUND_Y), HumanMode::Grounded),
                        ActorHumanSpawn::new(Point::new(64, HUMAN_GROUND_Y), HumanMode::Grounded),
                    ],
                ),
                ActorWaveProfile::with_spawns(
                    2,
                    ActorBehaviorScript::default(),
                    vec![ActorLanderSpawn::new(Point::new(120, 80))],
                    Vec::new(),
                ),
            ],
        );
        let mut runtime =
            ActorRuntimeAdapter::with_driver(ActorGameDriver::with_wave_script(wave_script));
        runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_player_start_completes(&mut runtime, 1);

        let pressed = runtime.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(pressed.report.score, 0);
        assert!(
            pressed
                .events
                .gameplay()
                .contains(&GameEvent::SmartBombPressed)
        );
        assert!(!pressed.events.gameplay().contains(&GameEvent::WaveCleared));
        assert_eq!(pressed.state.world.enemies.len(), 1);

        let cleared = step_until_smart_bomb_detonates(&mut runtime);

        assert_eq!(cleared.state.wave, 1);
        assert!(cleared.state.world.enemies.is_empty());
        assert_eq!(cleared.state.world.humans.len(), 2);
        assert_eq!(cleared.report.score, 250);
        assert_eq!(
            cleared.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 2,
                multiplier: 1,
                total_survivors: 2,
                visible_icons: 1,
                remaining_awards: 1,
                awarded_points: Some(100),
                astronaut_sleep_steps_remaining: SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(
            cleared
                .report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        assert!(cleared.events.gameplay().contains(&GameEvent::WaveCleared));
        assert!(!cleared.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_A
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [112.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_C
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [122.0, 96.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_B
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [120.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [202.0, 80.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(cleared.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_1
                && sprite.layer == RenderLayer::Overlay
                && sprite.position == [176.0, 144.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(scene_has_survivor_bonus_icon(
            &cleared.scene,
            [120.0, 160.0]
        ));
        assert!(!scene_has_survivor_bonus_icon(
            &cleared.scene,
            [128.0, 160.0]
        ));

        for expected_sleep in [3, 2, 1] {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(sleep.state.wave, 1);
            assert_eq!(sleep.report.score, 250);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus should remain active")
                    .astronaut_sleep_steps_remaining,
                expected_sleep
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let second_survivor = runtime.step(GameInput::NONE);

        assert_eq!(second_survivor.state.wave, 1);
        assert_eq!(second_survivor.report.score, 350);
        assert_eq!(
            second_survivor.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 2,
                multiplier: 1,
                total_survivors: 2,
                visible_icons: 2,
                remaining_awards: 0,
                awarded_points: Some(100),
                astronaut_sleep_steps_remaining: SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(scene_has_survivor_bonus_icon(
            &second_survivor.scene,
            [120.0, 160.0]
        ));
        assert!(scene_has_survivor_bonus_icon(
            &second_survivor.scene,
            [128.0, 160.0]
        ));

        for expected_sleep in [3, 2, 1] {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus should remain active")
                    .astronaut_sleep_steps_remaining,
                expected_sleep
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let wave_sleep = runtime.step(GameInput::NONE);
        assert_eq!(
            wave_sleep
                .report
                .survivor_bonus
                .expect("survivor bonus should enter wave sleep")
                .wave_advance_sleep_steps_remaining,
            Some(SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS)
        );
        assert!(
            !wave_sleep
                .events
                .gameplay()
                .contains(&GameEvent::WaveStarted)
        );

        for expected_sleep in (1..SURVIVOR_BONUS_WAVE_ADVANCE_SLEEP_STEPS).rev() {
            let sleep = runtime.step(GameInput::NONE);
            assert_eq!(
                sleep
                    .report
                    .survivor_bonus
                    .expect("survivor bonus wave sleep should remain active")
                    .wave_advance_sleep_steps_remaining,
                Some(expected_sleep)
            );
            assert!(!sleep.events.gameplay().contains(&GameEvent::WaveStarted));
        }

        let next_wave = runtime.step(GameInput::NONE);

        assert_eq!(next_wave.state.wave, 2);
        assert!(
            next_wave
                .events
                .gameplay()
                .contains(&GameEvent::WaveStarted)
        );
        assert!(
            next_wave
                .report
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        assert_eq!(next_wave.report.survivor_bonus, None);
        assert_eq!(next_wave.state.world.humans.len(), 0);
        assert!(
            next_wave
                .state
                .world
                .enemies
                .iter()
                .any(|enemy| enemy.kind == CleanEnemyKind::Lander)
        );
    }

    #[test]
    fn actor_score_awards_replay_stock_and_bonus_event() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.score = 9_900;
        driver.next_bonus = REPLAY_BONUS_SCORE;
        driver.lives = 3;
        driver.smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        runtime.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let scored = runtime.step(GameInput::NONE);

        assert_eq!(scored.report.score, 10_050);
        assert_eq!(scored.report.next_bonus, 20_000);
        assert!(scored.report.bonus_awarded);
        assert_eq!(scored.state.scores.player_one, 10_050);
        assert_eq!(scored.state.scores.high_score, 10_050);
        assert_eq!(scored.state.scores.next_bonus, 20_000);
        assert_eq!(scored.state.player.lives, 4);
        assert_eq!(scored.state.player.smart_bombs, 2);
        assert_eq!(
            scored.state.player_stocks[0],
            PlayerStockSnapshot::new(4, 2)
        );
        assert!(scored.events.gameplay().contains(&GameEvent::BonusAwarded));
    }

    #[test]
    fn actor_survivor_bonus_uses_arcade_wave_multiplier_and_replay_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 3;
        driver.score = 9_700;
        driver.next_bonus = REPLAY_BONUS_SCORE;
        driver.lives = 3;
        driver.smart_bombs = 3;
        driver.spawn_player();
        driver.spawn_human_for_test(Point::new(40, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(64, HUMAN_GROUND_Y));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let cleared = runtime.step(GameInput::NONE);

        assert!(
            cleared
                .report
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 4 })
        );
        assert_eq!(cleared.report.score, 10_000);
        assert_eq!(cleared.report.next_bonus, 20_000);
        assert!(cleared.report.bonus_awarded);
        assert_eq!(
            cleared.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 4,
                multiplier: 3,
                total_survivors: 2,
                visible_icons: 1,
                remaining_awards: 1,
                awarded_points: Some(300),
                astronaut_sleep_steps_remaining: SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert_eq!(cleared.state.player.lives, 4);
        assert_eq!(cleared.state.player.smart_bombs, 4);
        assert!(cleared.events.gameplay().contains(&GameEvent::BonusAwarded));

        for _ in 0..SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS - 1 {
            runtime.step(GameInput::NONE);
        }
        let second_survivor = runtime.step(GameInput::NONE);

        assert_eq!(second_survivor.report.score, 10_300);
        assert_eq!(
            second_survivor.report.survivor_bonus,
            Some(SurvivorBonusReport {
                next_wave: 4,
                multiplier: 3,
                total_survivors: 2,
                visible_icons: 2,
                remaining_awards: 0,
                awarded_points: Some(300),
                astronaut_sleep_steps_remaining: SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS,
                wave_advance_sleep_steps_remaining: None,
            })
        );
        assert!(!second_survivor.report.bonus_awarded);
    }

    #[test]
    fn actor_render_scene_bridge_maps_projectiles_and_explosion_variants() {
        let report = StepReport {
            step: 99,
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
            background_left: 0,
            arcade_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws: vec![
                DrawCommand::sprite(ActorId(101), SpriteKey::Laser, Point::new(40, 80)),
                DrawCommand::sprite(ActorId(102), SpriteKey::EnemyLaser, Point::new(90, 82)),
                DrawCommand::sprite_with_effect(
                    ActorId(103),
                    SpriteKey::Explosion,
                    Point::new(104, 82),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Lander,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(104),
                    SpriteKey::Explosion,
                    Point::new(108, 84),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Mutant,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(105),
                    SpriteKey::Explosion,
                    Point::new(112, 86),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomber,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(106),
                    SpriteKey::Explosion,
                    Point::new(116, 88),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Pod,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(107),
                    SpriteKey::Explosion,
                    Point::new(120, 90),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Swarmer,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(108),
                    SpriteKey::Explosion,
                    Point::new(122, 92),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Baiter,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(109),
                    SpriteKey::Explosion,
                    Point::new(124, 94),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Bomb,
                        age: 2,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(110),
                    SpriteKey::Explosion,
                    Point::new(124, 96),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Human,
                        age: 1,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite_with_effect(
                    ActorId(111),
                    SpriteKey::Explosion,
                    Point::new(128, 100),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Player,
                        age: 1,
                        explosion_anchor: None,
                    },
                ),
            ],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let bridge = ActorRenderSceneBridge::with_surface(SurfaceSize::new(320, 240));
        let scene = report.render_scene_with(&bridge);

        assert_eq!(bridge.surface(), SurfaceSize::new(320, 240));
        assert_eq!(scene.surface, SurfaceSize::new(320, 240));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_PROJECTILE_SCENE_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_BOMB && sprite.layer == RenderLayer::Projectiles
        }));
        for sprite_id in [
            SpriteId::ENEMY_LANDER,
            SpriteId::ENEMY_MUTANT,
            SpriteId::ENEMY_BOMBER,
            SpriteId::ENEMY_POD,
            SpriteId::SWARMER_EXPLOSION,
            SpriteId::ENEMY_BAITER,
        ] {
            assert!(
                !scene.sprites.iter().any(
                    |sprite| sprite.sprite == sprite_id && sprite.layer == RenderLayer::Objects
                ),
                "source-family explosion should use pixel cloud, not {sprite_id:?}"
            );
        }
        let source_cloud_pixels = scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                    && sprite.layer == RenderLayer::Objects
            })
            .count();
        assert!(
            source_cloud_pixels > 1,
            "source-family explosions should project expanded-object pixels"
        );
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::BOMB_EXPLOSION && sprite.layer == RenderLayer::Objects
        }));
        let bomb_explosion = scene
            .sprites
            .iter()
            .find(|sprite| sprite.sprite == SpriteId::BOMB_EXPLOSION)
            .expect("bomb explosion sprite should be projected");
        assert_eq!(bomb_explosion.size, [16.0, 16.0]);
        assert_eq!(bomb_explosion.position, [120.0, 90.0]);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ASTRONAUT_EXPLOSION && sprite.layer == RenderLayer::Objects
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
        }));
    }

    #[test]
    fn actor_render_projects_arcade_world_objects_against_scrolling_background() {
        let still = arcade_world_projection_report_for_test(0).render_scene();
        let scrolled = arcade_world_projection_report_for_test(0x0100).render_scene();

        assert_eq!(
            sprite_position_for_test(&still, SpriteId::PLAYER_SHIP, RenderLayer::Objects),
            Some([128.0, 100.0])
        );
        assert_eq!(
            sprite_position_for_test(&scrolled, SpriteId::PLAYER_SHIP, RenderLayer::Objects),
            Some([128.0, 100.0])
        );
        assert_eq!(
            sprite_position_for_test(&still, SpriteId::ENEMY_LANDER, RenderLayer::Objects),
            Some([192.0, 80.0])
        );
        assert_eq!(
            sprite_position_for_test(&scrolled, SpriteId::ENEMY_LANDER, RenderLayer::Objects),
            Some([188.0, 80.0])
        );

        let still_projectiles =
            sprite_positions_for_test(&still, SpriteId::ENEMY_BOMB, RenderLayer::Projectiles);
        let scrolled_projectiles =
            sprite_positions_for_test(&scrolled, SpriteId::ENEMY_BOMB, RenderLayer::Projectiles);
        assert!(still_projectiles.contains(&[196.0, 96.0]));
        assert!(still_projectiles.contains(&[196.0, 104.0]));
        assert!(scrolled_projectiles.contains(&[192.0, 96.0]));
        assert!(scrolled_projectiles.contains(&[192.0, 104.0]));
    }

    #[test]
    fn actor_render_projects_arcade_world_humans_against_scrolling_background() {
        let still_report = arcade_world_projection_report_for_test(0);
        let scrolled_report = arcade_world_projection_report_for_test(0x0100);

        let still_state = still_report.game_state();
        let scrolled_state = scrolled_report.game_state();
        assert_eq!(
            still_state.world.humans[0].position,
            ScreenPosition::new(0x2E, 220)
        );
        assert_eq!(
            scrolled_state.world.humans[0].position,
            ScreenPosition::new(0x2E, 220)
        );
        assert_eq!(still_state.world.humans[0].x_subpixel, 0x80);
        assert_eq!(scrolled_state.world.humans[0].x_subpixel, 0x80);

        let still = still_report.render_scene();
        let scrolled = scrolled_report.render_scene();
        assert_eq!(
            sprite_position_for_test(&still, SpriteId::HUMAN, RenderLayer::Objects),
            Some([186.0, 220.0])
        );
        assert_eq!(
            sprite_position_for_test(&scrolled, SpriteId::HUMAN, RenderLayer::Objects),
            Some([182.0, 220.0])
        );

        let human_runtime = still_report
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Human)
            .expect("arcade-state-backed human snapshot should be present");
        let projected = actor_collision_body_for_snapshot(human_runtime, 0)
            .expect("arcade-state-backed human should be projected while visible");
        assert_eq!(projected.position, Point::new(186, 220));
    }

    #[test]
    fn actor_collisions_project_arcade_world_hostiles_against_background() {
        let mut lander = actor_snapshot(2, ActorKind::Lander, Point::new(0x30, 80));
        lander.bounds = Some(Rect::from_center(lander.position, 8, 8));
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

        let projected = actor_collision_body_for_snapshot(&lander, 0)
            .expect("arcade-state-backed lander should be projected while visible");
        assert_eq!(projected.position, Point::new(192, 80));
        assert!(
            projected
                .bounds
                .intersects(Rect::from_center(Point::new(192, 80), 10, 2))
        );
        assert!(
            actor_collision_body_for_snapshot(&lander, 0x4000).is_none(),
            "offscreen arcade-state-backed hostiles should not collide with screen-space player/laser bodies"
        );
    }

    #[test]
    fn arcade_state_falling_human_rescue_uses_projected_world_position() {
        let mut human = Human::with_source(
            ActorId::new(7),
            Point::new(0x40, 100),
            HumanMode::Falling { velocity: 0 },
            Some(HumanArcadeState {
                x_fraction: 0,
                y_fraction: 0,
                picture_frame: 0,
                target_slot_index: 0,
            }),
        );
        let mut player = actor_snapshot(1, ActorKind::Player, Point::new(128, 101));
        player.bounds = Some(Rect::from_center(player.position, 18, 10));
        let mut prompt = playing_player_prompt_for_test(GameInput::NONE, 0x2000);
        prompt.snapshots = vec![player];

        let reply = human.update(&prompt);

        assert_eq!(reply.snapshot.position, Point::new(0x40, 101));
        assert_eq!(
            reply.snapshot.human_runtime.map(|source| source.x_fraction),
            Some(0)
        );
        assert!(
            reply
                .commands
                .contains(&GameCommand::Destroy(ActorId::new(7)))
        );
        assert!(
            reply
                .commands
                .contains(&GameCommand::AddScore(HUMAN_RESCUE_SCORE))
        );
        assert!(
            reply
                .commands
                .contains(&GameCommand::PlaySound(SoundCue::HumanRescued))
        );
    }

    #[test]
    fn actor_source_explosion_render_scale_uses_source_size_curve() {
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(0)),
            1.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(1)),
            1.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(2)),
            2.0
        );
        assert_eq!(
            actor_source_explosion_render_scale(source_explosion_size_for_age(18)),
            3.0
        );
    }

    #[test]
    fn actor_explosion_anchor_reaches_state_and_render_bridges() {
        let top_left = Point::new(0x20, 0xA2);
        let explosion_anchor = Point::new(0x21, 0xA9);
        let report = StepReport {
            step: 7,
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
            background_left: 0,
            arcade_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws: vec![DrawCommand::sprite_with_effect(
                ActorId(101),
                SpriteKey::Explosion,
                top_left,
                VisualEffect::ExplosionCloud {
                    kind: ExplosionKind::Mutant,
                    age: 2,
                    explosion_anchor: Some(explosion_anchor),
                },
            )],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let state = ActorStateBridge::new().state_for_report(&report);
        assert_eq!(state.world.explosions.len(), 1);
        assert_eq!(state.world.explosions[0].kind, CleanExplosionKind::Mutant);
        assert_eq!(
            state.world.explosions[0].position,
            ScreenPosition::new(0x20, 0xA2)
        );
        assert_eq!(
            state.world.explosions[0].explosion_anchor,
            Some(ScreenPosition::new(0x21, 0xA9))
        );
        assert_eq!(
            state.world.explosions[0].source_size,
            source_explosion_size_for_age(2)
        );

        let scene = report.render_scene();
        let mut expected = RenderScene::empty(report.step, ACTOR_RENDER_SURFACE);
        assert!(push_explosion_cloud_pixels(
            &mut expected,
            CleanExplosionKind::Mutant,
            ScreenPosition::new(0x20, 0xA2),
            Some(ScreenPosition::new(0x21, 0xA9)),
            source_explosion_size_for_age(2),
        ));
        let object_sprites = scene
            .sprites
            .iter()
            .filter(|sprite| sprite.layer == RenderLayer::Objects)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(object_sprites, expected.sprites);
    }

    #[test]
    fn actor_state_bridge_maps_report_snapshots_and_draw_effects_to_clean_state() {
        let mut player = actor_snapshot(11, ActorKind::Player, Point::new(40, 70));
        player.bounds = Some(Rect::from_center(player.position, 16, 6));
        player.velocity = Velocity::new(3, -1);
        player.direction = Some(Direction::Right);
        let mut lander = actor_snapshot(12, ActorKind::Lander, Point::new(0x3F, 0x2C));
        lander.velocity = Velocity::new(-2, 1);
        lander.lander_runtime = Some(LanderArcadeState {
            x_fraction: 0x4A,
            y_fraction: 0xE0,
            x_velocity: 0xFFEE,
            y_velocity: 0x0070,
            shot_timer: 0x3B,
            sleep_ticks: 0x04,
            picture_frame: 1,
            target_human_index: Some(2),
        });
        let mut human = actor_snapshot(13, ActorKind::Human, Point::new(0x1C, 0xE1));
        human.human_runtime = Some(HumanArcadeState {
            x_fraction: 0x81,
            y_fraction: 0,
            picture_frame: 3,
            target_slot_index: 1,
        });
        let mut laser = actor_snapshot(14, ActorKind::Laser, Point::new(80, 72));
        laser.velocity = Velocity::new(8, 0);
        laser.direction = Some(Direction::Right);
        let mut enemy_laser = actor_snapshot(15, ActorKind::EnemyLaser, Point::new(90, 80));
        enemy_laser.velocity = Velocity::new(-3, 2);
        enemy_laser.enemy_projectile_runtime = Some(EnemyProjectileArcadeState {
            x_fraction: 0x22,
            y_fraction: 0x77,
            x_velocity: 0xFD00,
            y_velocity: 0x0200,
            lifetime_ticks: 17,
        });
        let mut bomb = actor_snapshot(16, ActorKind::Bomb, Point::new(100, 84));
        bomb.enemy_projectile_runtime = Some(EnemyProjectileArcadeState {
            x_fraction: 0x44,
            y_fraction: 0x55,
            x_velocity: 0,
            y_velocity: 0,
            lifetime_ticks: 9,
        });

        let report = StepReport {
            step: 77,
            phase: Phase::HighScoreEntry,
            wave: 2,
            current_player: 1,
            player_count: 1,
            score: 12_000,
            player_scores: [12_000, 0],
            credits: 1,
            lives: 2,
            smart_bombs: 1,
            smart_bomb_flash_steps_remaining: 0,
            player_stocks: [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(3, 3),
            ],
            next_bonus: 20_000,
            player_death_sleep_remaining: None,
            game_over_hall_of_fame_stall_remaining: None,
            player_switch: None,
            player_start: None,
            high_scores: [12_000, 10_000, 7_500, 5_000, 2_500],
            arcade_wave: ArcadeWaveProfile::for_wave(2),
            high_score_initials: HighScoreInitialsState {
                initials: [Some('R'), None, None],
                cursor: 1,
            },
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            background_left: 0,
            arcade_rng: None,
            terrain_blow: None,
            snapshots: vec![player, lander, human, laser, enemy_laser, bomb],
            draws: vec![
                DrawCommand::sprite(ActorId(11), SpriteKey::PlayerLeft, Point::new(40, 70)),
                DrawCommand::sprite(ActorId(13), SpriteKey::HumanCarried, Point::new(0x1C, 0xE1)),
                DrawCommand::sprite_with_effect(
                    ActorId(17),
                    SpriteKey::Explosion,
                    Point::new(120, 90),
                    VisualEffect::ExplosionCloud {
                        kind: ExplosionKind::Lander,
                        age: 0,
                        explosion_anchor: None,
                    },
                ),
                DrawCommand::sprite(ActorId(18), SpriteKey::Score500, Point::new(122, 88)),
            ],
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let state = report.game_state();

        assert_eq!(state.frame, 77);
        assert_eq!(state.phase, GamePhase::HighScoreEntry);
        assert_eq!(state.credits, 1);
        assert_eq!(state.wave, 2);
        assert_eq!(state.player.direction, CleanDirection::Right);
        assert_eq!(state.player.position.0.subpixels(), 40 * 256);
        assert_eq!(state.player.velocity.0.subpixels(), 3 * 256);
        assert_eq!(state.player.velocity.1.subpixels(), -256);
        assert_eq!(state.player_stocks[0], PlayerStockSnapshot::new(2, 1));
        assert_eq!(state.scores.player_one, 12_000);
        assert_eq!(state.scores.high_score, 12_000);
        assert_eq!(state.high_score_initials.initials, [Some('R'), None, None]);
        assert_eq!(
            state.high_score_entry,
            Some(HighScoreEntrySnapshot {
                score: 12_000,
                rank: 1
            })
        );
        assert_eq!(state.high_score_tables.all_time[0].score, 12_000);

        assert_eq!(state.world.enemies.len(), 1);
        assert_eq!(state.world.enemies[0].kind, CleanEnemyKind::Lander);
        assert_eq!(state.world.enemies[0].velocity, ScreenVelocity::new(-2, 1));
        assert_eq!(
            state.world.enemies[0].lander_runtime,
            Some(LanderRuntimeSnapshot {
                x_fraction: 0x4A,
                y_fraction: 0xE0,
                x_velocity: 0xFFEE,
                y_velocity: 0x0070,
                shot_timer: 0x3B,
                sleep_ticks: 0x04,
                picture_frame: 1,
                target_human_index: Some(2),
            })
        );
        assert_eq!(state.world.humans.len(), 1);
        assert!(state.world.humans[0].carried);
        assert_eq!(state.world.humans[0].picture_frame, 3);
        assert_eq!(state.world.projectiles.len(), 1);
        assert_eq!(
            state.world.projectiles[0].velocity,
            ScreenVelocity::new(8, 0)
        );
        assert_eq!(state.world.enemy_projectiles.len(), 2);
        let fireball = state
            .world
            .enemy_projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::Fireball)
            .expect("actor enemy laser should bridge as a source fireball");
        assert_eq!(fireball.velocity, ScreenVelocity::new(-3, 2));
        assert_eq!(fireball.x_subpixel, 0x22);
        assert_eq!(fireball.y_subpixel, 0x77);
        assert_eq!(fireball.x_velocity_word, 0xFD00);
        assert_eq!(fireball.y_velocity_word, 0x0200);
        assert_eq!(fireball.lifetime_ticks, 17);
        let bomb_shell = state
            .world
            .enemy_projectiles
            .iter()
            .find(|projectile| projectile.kind == EnemyProjectileKind::BomberBombShell)
            .expect("actor bomb should bridge as a source bomb shell");
        assert_eq!(bomb_shell.x_subpixel, 0x44);
        assert_eq!(bomb_shell.y_subpixel, 0x55);
        assert_eq!(bomb_shell.x_velocity_word, 0);
        assert_eq!(bomb_shell.y_velocity_word, 0);
        assert_eq!(bomb_shell.lifetime_ticks, 9);
        assert!(
            state
                .world
                .enemy_projectiles
                .iter()
                .any(|projectile| projectile.velocity == ScreenVelocity::new(-3, 2))
        );
        assert!(state.world.enemy_projectiles.iter().any(|projectile| {
            projectile.kind == EnemyProjectileKind::BomberBombShell
        }));
        assert_eq!(state.world.explosions.len(), 1);
        assert_eq!(state.world.explosions[0].kind, CleanExplosionKind::Lander);
        assert_eq!(state.world.score_popups.len(), 1);
        assert_eq!(
            state.world.score_popups[0].kind,
            CleanScorePopupKind::Points500
        );
    }

    #[test]
    fn actor_state_bridge_preserves_enemy_family_explosion_kinds() {
        let draws = [
            (ExplosionKind::Lander, Point::new(20, 40)),
            (ExplosionKind::Mutant, Point::new(24, 44)),
            (ExplosionKind::Bomber, Point::new(28, 48)),
            (ExplosionKind::Pod, Point::new(32, 52)),
            (ExplosionKind::Swarmer, Point::new(36, 56)),
            (ExplosionKind::Baiter, Point::new(40, 60)),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (kind, position))| {
            DrawCommand::sprite_with_effect(
                ActorId(200 + index as u64),
                SpriteKey::Explosion,
                position,
                VisualEffect::ExplosionCloud {
                    kind,
                    age: 0,
                    explosion_anchor: None,
                },
            )
        })
        .collect::<Vec<_>>();

        let report = StepReport {
            step: 101,
            phase: Phase::Playing,
            wave: 3,
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
            arcade_wave: ArcadeWaveProfile::for_wave(3),
            high_score_initials: HighScoreInitialsState::EMPTY,
            high_score_initial_accepted: false,
            high_score_submitted: false,
            bonus_awarded: false,
            survivor_bonus: None,
            behavior_script: ActorBehaviorScript::default().manifest(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            background_left: 0,
            arcade_rng: None,
            terrain_blow: None,
            snapshots: Vec::new(),
            draws,
            sounds: Vec::new(),
            commands: Vec::new(),
        };

        let kinds = report
            .game_state()
            .world
            .explosions
            .iter()
            .map(|explosion| explosion.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                CleanExplosionKind::Lander,
                CleanExplosionKind::Mutant,
                CleanExplosionKind::Bomber,
                CleanExplosionKind::Pod,
                CleanExplosionKind::Swarmer,
                CleanExplosionKind::Baiter,
            ]
        );
    }

    #[test]
    fn actor_input_converts_clean_live_key_contract_with_xyzzy_overlay() {
        let xyzzy = XyzzyMode {
            active: true,
            auto_fire: true,
            invincible: true,
            overlay_smart_bomb: true,
        };
        let input = GameInput::from_clean_input(
            CleanGameInput {
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
                high_score_initial: Some('A'),
                high_score_backspace: true,
                tilt: true,
            },
            xyzzy,
        );

        assert!(input.coin);
        assert!(input.coin_two);
        assert!(input.coin_three);
        assert!(input.start_one);
        assert!(input.start_two);
        assert!(input.altitude_up);
        assert!(input.altitude_down);
        assert!(input.reverse);
        assert!(input.thrust);
        assert!(input.fire);
        assert!(input.smart_bomb);
        assert!(input.hyperspace);
        assert!(input.service_advance);
        assert!(input.high_score_reset);
        assert_eq!(input.high_score_initial, Some('A'));
        assert!(input.high_score_backspace);
        assert!(input.auto_up_manual_down);
        assert!(input.tilt);
        assert_eq!(input.xyzzy, xyzzy);
    }

    #[test]
    fn actor_runtime_adapter_bundles_report_events_audio_and_scene() {
        let mut runtime = ActorRuntimeAdapter::new();

        let credited = runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.report.phase, Phase::Attract);
        assert_eq!(credited.report.credits, 1);
        assert_eq!(credited.state.phase, GamePhase::Attract);
        assert_eq!(credited.state.credits, 1);
        assert_eq!(credited.events.gameplay(), &[GameEvent::CreditAdded]);
        assert_eq!(credited.events.sounds(), &[SoundEvent::CreditAdded]);
        assert_eq!(credited.scene.frame, credited.report.step);
        assert!(credited.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));

        let started = runtime.step_clean_input(
            CleanGameInput {
                start_one: true,
                ..CleanGameInput::NONE
            },
            XyzzyMode::INACTIVE,
        );
        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.state.world.enemies.is_empty());
        assert_no_arcade_message(&started.report, MessageId::PlayerOne, PLAYER_START_PROMPT_SCREEN_ADDRESS);
        assert!(started.events.sounds().is_empty());

        let start_sound = runtime.step(GameInput::NONE);
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            start_sound.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );

        let settled = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(settled.state.phase, GamePhase::Playing);
        assert_eq!(settled.state.wave, 1);
        assert_eq!(
            settled.state.player_stocks[0],
            PlayerStockSnapshot::new(3, 3)
        );
        assert_eq!(settled.state.world.humans.len(), 10);
        assert_eq!(
            settled
                .state
                .world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == CleanEnemyKind::Lander)
                .count(),
            5
        );
        let clean_frame = settled.game_frame();
        assert_eq!(clean_frame.state, settled.state);
        assert_eq!(clean_frame.events, settled.events);
        assert_eq!(clean_frame.scene, settled.scene);
        assert!(settled.scene.sprites.iter().any(|sprite| {
            matches!(
                sprite.sprite,
                SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT
            )
        }));
        assert!(
            settled
                .scene
                .sprites
                .iter()
                .any(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
        );
    }

    #[test]
    fn actor_runtime_adapter_carries_audio_edge_state_between_frames() {
        let mut runtime = ActorRuntimeAdapter::new();
        runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_player_start_completes(&mut runtime, 1);

        let thrusting = runtime.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert_eq!(thrusting.events.sounds(), &[SoundEvent::ThrustStarted]);

        let still_thrusting = runtime.step(GameInput {
            thrust: true,
            ..GameInput::NONE
        });
        assert!(still_thrusting.events.sounds().is_empty());

        let coasting = runtime.step(GameInput::NONE);
        assert_eq!(coasting.events.sounds(), &[SoundEvent::ThrustStopped]);
    }

    #[test]
    fn player_reverse_input_flips_once_until_released() {
        let mut runtime = ActorRuntimeAdapter::new();
        runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.state.player.direction, CleanDirection::Right);

        let reversed = runtime.step(GameInput {
            reverse: true,
            ..GameInput::NONE
        });
        assert_eq!(reversed.state.player.direction, CleanDirection::Left);

        for _ in 0..6 {
            let held = runtime.step(GameInput {
                reverse: true,
                ..GameInput::NONE
            });
            assert_eq!(held.state.player.direction, CleanDirection::Left);
        }

        let released = runtime.step(GameInput::NONE);
        assert_eq!(released.state.player.direction, CleanDirection::Left);

        let reversed_again = runtime.step(GameInput {
            reverse: true,
            ..GameInput::NONE
        });
        assert_eq!(reversed_again.state.player.direction, CleanDirection::Right);
    }

    #[test]
    fn player_thrust_scrolls_wrapping_background_after_reaching_center() {
        let player_id = ActorId::new(1);
        let mut player = PlayerShip::new(player_id, Point::new(PLAYER_SCROLL_CENTER_X, 120));

        let right = player.update(&playing_player_prompt_for_test(
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            0xFE00,
        ));
        assert_eq!(right.snapshot.position.x, PLAYER_SCROLL_CENTER_X);
        assert!(
            right
                .commands
                .contains(&GameCommand::SetWorldScrollLeft(0xFF00))
        );

        let reversed = player.update(&playing_player_prompt_for_test(
            GameInput {
                reverse: true,
                ..GameInput::NONE
            },
            0x0100,
        ));
        assert_eq!(reversed.snapshot.direction, Some(Direction::Left));

        let left = player.update(&playing_player_prompt_for_test(
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            0x0100,
        ));
        assert_eq!(left.snapshot.position.x, PLAYER_SCROLL_CENTER_X);
        assert!(
            left.commands
                .contains(&GameCommand::SetWorldScrollLeft(0x0000))
        );
    }

    #[test]
    fn player_altitude_up_clamps_to_playfield_below_hud() {
        let player_id = ActorId::new(1);
        let mut player = PlayerShip::new(
            player_id,
            Point::new(PLAYER_SCROLL_CENTER_X, PLAYER_PLAYFIELD_TOP_Y + 1),
        );

        for _ in 0..8 {
            let report = player.update(&playing_player_prompt_for_test(
                GameInput {
                    altitude_up: true,
                    ..GameInput::NONE
                },
                0,
            ));
            assert_eq!(report.snapshot.position.y, PLAYER_PLAYFIELD_TOP_Y);
            assert!(report.draws.iter().any(|draw| {
                matches!(draw.sprite, SpriteKey::PlayerRight | SpriteKey::PlayerLeft)
                    && draw.position.y == PLAYER_PLAYFIELD_TOP_Y
            }));
        }
    }

    #[test]
    fn player_default_thrust_moves_at_half_previous_speed() {
        let player_id = ActorId::new(1);
        let mut player = PlayerShip::new(player_id, Point::new(42, 120));

        let report = player.update(&playing_player_prompt_for_test(
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            0,
        ));

        assert_eq!(ActorBehaviorProfile::default().player_speed, 1);
        assert_eq!(report.snapshot.position.x, 43);
        assert_eq!(report.snapshot.velocity, Velocity::new(1, 0));
        assert!(
            report
                .commands
                .contains(&GameCommand::PlaySound(SoundCue::Thrust))
        );
    }
