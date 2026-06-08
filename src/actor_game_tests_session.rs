    #[test]
    fn actor_two_player_start_requires_two_credits() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 1;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let blocked = runtime.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(blocked.report.phase, Phase::Attract);
        assert_eq!(blocked.report.credits, 1);
        assert_eq!(blocked.state.phase, GamePhase::Attract);
        assert_eq!(blocked.state.credits, 1);
        assert_eq!(blocked.state.current_player, 1);
        assert_eq!(blocked.state.player_count, 1);
        assert_eq!(blocked.state.scores.player_two, 0);
        assert!(!blocked.events.gameplay().contains(&GameEvent::GameStarted));
        assert!(!blocked.report.sounds.contains(&SoundCue::Start));
    }

    #[test]
    fn actor_credit_gated_start_buttons_block_without_credit() {
        let mut runtime = ActorRuntimeAdapter::new();

        let one_blocked = runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        assert_eq!(one_blocked.report.phase, Phase::Attract);
        assert_eq!(one_blocked.report.credits, 0);
        assert!(
            !one_blocked
                .events
                .gameplay()
                .contains(&GameEvent::GameStarted)
        );

        let two_blocked = runtime.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(two_blocked.report.phase, Phase::Attract);
        assert_eq!(two_blocked.report.credits, 0);
        assert_eq!(two_blocked.report.player_count, 1);
        assert!(
            !two_blocked
                .events
                .gameplay()
                .contains(&GameEvent::GameStarted)
        );
    }

    #[test]
    fn actor_free_play_admission_starts_without_inserted_credit() {
        let mut one_player = ActorRuntimeAdapter::new_with_free_play_admission();

        let one_started = one_player.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        assert_eq!(one_started.report.phase, Phase::Playing);
        assert_eq!(one_started.report.credits, 0);
        assert_eq!(one_started.report.player_count, 1);
        assert!(
            one_started
                .events
                .gameplay()
                .contains(&GameEvent::GameStarted)
        );
        assert_eq!(
            one_started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );

        let mut two_player = ActorRuntimeAdapter::new_with_free_play_admission();

        let two_started = two_player.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(two_started.report.phase, Phase::Playing);
        assert_eq!(two_started.report.credits, 0);
        assert_eq!(two_started.report.player_count, 2);
        assert!(
            two_started
                .events
                .gameplay()
                .contains(&GameEvent::GameStarted)
        );
        assert_eq!(
            two_started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_source_message(
            &two_started.report,
            "PLYR1",
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
    }

    #[test]
    fn actor_one_player_start_accepts_same_step_credit_and_start_button() {
        let mut runtime = ActorRuntimeAdapter::new();

        let started = runtime.step(GameInput {
            coin: true,
            start_one: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 1);
        assert!(started.events.gameplay().contains(&GameEvent::CreditAdded));
        assert!(started.events.gameplay().contains(&GameEvent::GameStarted));
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.state.world.enemies.is_empty());

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.phase, Phase::Playing);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(!active.state.world.enemies.is_empty());
    }

    #[test]
    fn actor_one_player_start_uses_source_playfield_delay_without_source_prompt() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 1;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let started = runtime.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 1);
        assert_eq!(started.events.gameplay(), &[GameEvent::GameStarted]);
        assert!(!started.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(started.events.sounds().is_empty());
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.report.snapshots.iter().all(|snapshot| !matches!(
            snapshot.kind,
            ActorKind::Player
                | ActorKind::Lander
                | ActorKind::Bomber
                | ActorKind::Pod
                | ActorKind::Human
        )));
        assert!(started.state.world.enemies.is_empty());
        assert!(started.state.world.humans.is_empty());
        assert_no_source_message(&started.report, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let start_sound = runtime.step(GameInput::NONE);
        assert_eq!(start_sound.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            start_sound.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert!(start_sound.state.world.enemies.is_empty());
        assert_no_source_message(
            &start_sound.report,
            "PLYR1",
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );

        let active = step_until_player_start_completes(&mut runtime, 1);

        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 1);
        assert_eq!(active.report.player_count, 1);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert_eq!(
            active.events.sounds(),
            &[SoundEvent::UnmappedSoundCommand { command: 0xEA }]
        );
        assert_eq!(active.state.world.humans.len(), 10);
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_two_player_start_accepts_two_key_after_second_credit() {
        let mut runtime = ActorRuntimeAdapter::new();

        let credited = runtime.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.report.phase, Phase::Attract);
        assert_eq!(credited.report.credits, 1);

        let started = runtime.step(GameInput {
            coin: true,
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 2);
        assert!(started.events.gameplay().contains(&GameEvent::CreditAdded));
        assert!(started.events.gameplay().contains(&GameEvent::GameStarted));
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_source_message(&started.report, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.player_count, 2);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(!active.state.world.enemies.is_empty());
    }

    #[test]
    fn actor_two_player_start_initializes_session_state_bridge() {
        let mut driver = ActorGameDriver::new();
        driver.credits = 2;
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let started = runtime.step(GameInput {
            start_two: true,
            ..GameInput::NONE
        });

        assert_eq!(started.report.phase, Phase::Playing);
        assert_eq!(started.report.credits, 0);
        assert_eq!(started.report.current_player, 1);
        assert_eq!(started.report.player_count, 2);
        assert_eq!(started.report.player_scores, [0, 0]);
        assert_eq!(
            started.report.player_stocks,
            [PlayerStockSnapshot::new(3, 3); 2]
        );
        assert_eq!(started.state.phase, GamePhase::Playing);
        assert_eq!(started.state.credits, 0);
        assert_eq!(started.state.current_player, 1);
        assert_eq!(started.state.player_count, 2);
        assert_eq!(started.state.scores.player_one, 0);
        assert_eq!(started.state.scores.player_two, 0);
        assert_eq!(
            started.state.player_stocks,
            [PlayerStockSnapshot::new(3, 3); 2]
        );
        assert!(!started.report.sounds.contains(&SoundCue::Start));
        assert!(started.events.sounds().is_empty());
        assert!(started.events.gameplay().contains(&GameEvent::GameStarted));
        assert!(!started.events.gameplay().contains(&GameEvent::WaveStarted));
        assert_eq!(
            started.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert!(started.state.world.enemies.is_empty());
        assert_source_message(&started.report, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);
        assert_source_message_scene(&started.scene, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let status = runtime.step(GameInput::NONE);
        assert_text(&status.report, "WAVE 01");
        assert_text(&status.report, "CREDIT 00");
        assert_eq!(status.state.scores.player_two, 0);
        assert_eq!(status.events.sounds(), &[SoundEvent::GameStarted]);
        assert_eq!(
            status.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert_source_message(&status.report, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);
        assert_source_message_scene(&status.scene, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 1);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_score_awards_follow_current_player_two_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 2;
        driver.player_count = 2;
        driver.score = 900;
        driver.player_two_score = 9_900;
        driver.next_bonus = REPLAY_BONUS_SCORE;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.player_two_lives = 3;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(62, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        runtime.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });
        let scored = runtime.step(GameInput::NONE);

        assert_eq!(scored.report.current_player, 2);
        assert_eq!(scored.report.player_scores, [900, 10_050]);
        assert_eq!(scored.report.score, 10_050);
        assert_eq!(scored.report.next_bonus, 20_000);
        assert_eq!(
            scored.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(4, 2),
            ]
        );
        assert_eq!(scored.state.scores.player_one, 900);
        assert_eq!(scored.state.scores.player_two, 10_050);
        assert_eq!(scored.state.scores.high_score, 10_050);
        assert_eq!(
            scored.state.player_stocks[1],
            PlayerStockSnapshot::new(4, 2)
        );
        assert!(scored.events.gameplay().contains(&GameEvent::BonusAwarded));
    }

    #[test]
    fn actor_one_player_non_final_death_stays_in_play_and_respawns() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 1;
        driver.lives = 2;
        driver.smart_bombs = 3;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::Playing);
        assert_eq!(killed.state.phase, GamePhase::Playing);
        assert_eq!(killed.report.lives, 1);
        assert_eq!(
            killed.state.player_stocks[0],
            PlayerStockSnapshot::new(1, 3)
        );
        assert!(
            killed
                .events
                .gameplay()
                .contains(&GameEvent::PlayerDestroyed)
        );
        assert!(!killed.events.gameplay().contains(&GameEvent::GameOver));
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));
        assert!(killed.report.player_switch.is_none());
        assert!(killed.report.player_start.is_none());
        assert!(
            !killed
                .report
                .draws
                .iter()
                .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
        assert!(!killed.scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.layer == RenderLayer::Overlay
        }));
        assert_eq!(runtime.driver().snapshot_count(ActorKind::Player), 0);

        let respawned = runtime.step(GameInput::NONE);

        assert_eq!(respawned.report.phase, Phase::Playing);
        assert_eq!(respawned.state.phase, GamePhase::Playing);
        assert_eq!(respawned.report.lives, 1);
        assert_eq!(
            respawned.state.player_stocks[0],
            PlayerStockSnapshot::new(1, 3)
        );
        assert_eq!(runtime.driver().snapshot_count(ActorKind::Player), 1);
        assert!(!respawned.events.gameplay().contains(&GameEvent::GameOver));
        assert!(!respawned.report.sounds.contains(&SoundCue::GameOver));
    }

    #[test]
    fn actor_two_player_non_final_death_starts_player_switch_sleep() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 2;
        driver.smart_bombs = 3;
        driver.player_two_lives = 3;
        driver.player_two_smart_bombs = 3;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.current_player, 1);
        assert_eq!(killed.report.lives, 1);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: PLAYER_SWITCH_DELAY_STEPS,
                from_player: 1,
                to_player: 2,
            })
        );
        assert_eq!(
            killed.state.game_over,
            GameOverSnapshot {
                player_switch_sleep_remaining: Some(PLAYER_SWITCH_DELAY_STEPS),
                player_switch_from: Some(1),
                player_switch_to: Some(2),
                ..GameOverSnapshot::NONE
            }
        );
        assert!(
            killed
                .events
                .gameplay()
                .contains(&GameEvent::PlayerDestroyed)
        );
        assert!(!killed.events.gameplay().contains(&GameEvent::GameOver));
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 2);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 2);
        assert_eq!(switched.report.lives, 3);
        assert_eq!(switched.report.smart_bombs, 3);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(1, 3),
                PlayerStockSnapshot::new(3, 3),
            ]
        );
        assert!(switched.report.player_switch.is_none());
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 2,
            })
        );
        assert!(switched.state.world.enemies.is_empty());
        assert_source_message(
            &switched.report,
            "PLYR2",
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
        assert_source_message_scene(&switched.scene, "PLYR2", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let active = step_until_player_start_completes(&mut runtime, 2);
        assert_eq!(active.report.phase, Phase::Playing);
        assert_eq!(active.report.current_player, 2);
        assert!(active.report.player_start.is_none());
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
        assert!(active.state.world.enemies.iter().any(|enemy| {
            enemy.kind == CleanEnemyKind::Lander
                || enemy.kind == CleanEnemyKind::Bomber
                || enemy.kind == CleanEnemyKind::Pod
        }));
    }

    #[test]
    fn actor_two_player_final_death_switches_to_other_stocked_player() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 1;
        driver.smart_bombs = 2;
        driver.player_two_lives = 2;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.lives, 0);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(0, 2),
                PlayerStockSnapshot::new(2, 1),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: PLAYER_SWITCH_DELAY_STEPS,
                from_player: 1,
                to_player: 2,
            })
        );
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));
        assert!(!killed.events.gameplay().contains(&GameEvent::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 2);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 2);
        assert_eq!(switched.report.lives, 2);
        assert_eq!(switched.report.smart_bombs, 1);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(0, 2),
                PlayerStockSnapshot::new(2, 1),
            ]
        );
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 2,
            })
        );
        assert_source_message(
            &switched.report,
            "PLYR2",
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
        assert_source_message_scene(&switched.scene, "PLYR2", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let active = step_until_player_start_completes(&mut runtime, 2);
        assert_eq!(active.report.current_player, 2);
        assert_eq!(active.report.lives, 2);
        assert_eq!(active.report.smart_bombs, 1);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
    }

    #[test]
    fn actor_two_player_final_death_enters_game_over_when_no_other_stock() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 1;
        driver.player_count = 2;
        driver.lives = 1;
        driver.smart_bombs = 2;
        driver.player_two_lives = 0;
        driver.player_two_smart_bombs = 1;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.lives, 0);
        assert!(killed.report.player_switch.is_none());
        assert!(killed.report.sounds.contains(&SoundCue::GameOver));
        assert!(killed.events.gameplay().contains(&GameEvent::GameOver));
        assert_eq!(
            killed.state.game_over,
            GameOverSnapshot {
                player_death_sleep_remaining: Some(FINAL_GAME_OVER_DELAY_STEPS),
                ..GameOverSnapshot::NONE
            }
        );
        assert_source_message(&killed.report, "GO", FINAL_GAME_OVER_SCREEN_ADDRESS);
        assert_source_message_scene(&killed.scene, "GO", FINAL_GAME_OVER_SCREEN_ADDRESS);
        assert!(
            !killed
                .report
                .draws
                .iter()
                .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );

        let returned = step_until_final_game_over_sleep_returns_to_attract(&mut runtime);

        assert_eq!(returned.report.phase, Phase::Attract);
        assert_eq!(returned.state.phase, GamePhase::Attract);
        assert_eq!(returned.state.game_over, GameOverSnapshot::NONE);
        assert!(returned.report.player_death_sleep_remaining.is_none());
        assert!(
            returned
                .report
                .draws
                .iter()
                .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
    }

    #[test]
    fn actor_two_player_second_player_final_death_switches_back_to_player_one() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.wave = 1;
        driver.current_player = 2;
        driver.player_count = 2;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.player_two_lives = 1;
        driver.player_two_smart_bombs = 2;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));
        let mut runtime = ActorRuntimeAdapter::with_driver(driver);

        let killed = runtime.step(GameInput::NONE);

        assert_eq!(killed.report.phase, Phase::GameOver);
        assert_eq!(killed.report.current_player, 2);
        assert_eq!(killed.report.lives, 0);
        assert_eq!(
            killed.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(0, 2),
            ]
        );
        assert_eq!(
            killed.report.player_switch,
            Some(PlayerSwitchReport {
                sleep_steps_remaining: PLAYER_SWITCH_DELAY_STEPS,
                from_player: 2,
                to_player: 1,
            })
        );
        assert!(!killed.report.sounds.contains(&SoundCue::GameOver));

        let switched = step_until_player_switch_completes(&mut runtime, 1);

        assert_eq!(switched.report.phase, Phase::Playing);
        assert_eq!(switched.report.current_player, 1);
        assert_eq!(switched.report.lives, 2);
        assert_eq!(switched.report.smart_bombs, 1);
        assert_eq!(
            switched.report.player_stocks,
            [
                PlayerStockSnapshot::new(2, 1),
                PlayerStockSnapshot::new(0, 2),
            ]
        );
        assert_eq!(
            switched.report.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_source_message(
            &switched.report,
            "PLYR1",
            PLAYER_START_PROMPT_SCREEN_ADDRESS,
        );
        assert_source_message_scene(&switched.scene, "PLYR1", PLAYER_START_PROMPT_SCREEN_ADDRESS);

        let active = step_until_player_start_completes(&mut runtime, 1);
        assert_eq!(active.report.current_player, 1);
        assert_eq!(active.report.lives, 2);
        assert_eq!(active.report.smart_bombs, 1);
        assert!(active.events.gameplay().contains(&GameEvent::WaveStarted));
    }

    #[test]
    fn status_display_actor_draws_play_state_from_prompt() {
        let mut driver = started_driver();
        driver.score = 9_875;
        driver.wave = 7;
        driver.lives = 2;
        driver.smart_bombs = 1;
        driver.credits = 3;

        let report = driver.step(GameInput::NONE);

        assert!(
            report
                .snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::StatusDisplay)
        );
        assert_text(&report, "WAVE 07");
        assert_text(&report, "CREDIT 03");
        assert_no_text(&report, "1UP 009875");
        assert_no_text(&report, "HIGH 010000");
        assert_no_text(&report, "LIVES 02");
        assert_no_text(&report, "BOMBS 01");

        let scene = report.render_scene();
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_9
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [34.0, 21.0]
                && sprite.size == ACTOR_HUD_SCORE_DIGIT_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_8
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [42.0, 21.0]
                && sprite.size == ACTOR_HUD_SCORE_DIGIT_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_7
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [50.0, 21.0]
                && sprite.size == ACTOR_HUD_SCORE_DIGIT_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_5
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [58.0, 21.0]
                && sprite.size == ACTOR_HUD_SCORE_DIGIT_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_LIFE_STOCK
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [18.0, 13.0]
                && sprite.size == ACTOR_HUD_PLAYER_LIFE_STOCK_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_LIFE_STOCK
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [30.0, 13.0]
                && sprite.size == ACTOR_HUD_PLAYER_LIFE_STOCK_SIZE
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SMART_BOMB_STOCK
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [70.0, 20.0]
                && sprite.size == ACTOR_HUD_SMART_BOMB_STOCK_SIZE
        }));
        assert!(
            !scene.sprites.iter().any(|sprite| {
                SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite)
                    && sprite.layer == RenderLayer::Hud
                    && sprite.position[0] >= 94.0
                    && sprite.position[0] < 224.0
                    && sprite.position[1] < 40.0
            }),
            "gameplay status text must not overlap the scanner frame"
        );
    }

    #[test]
    fn status_display_actor_draws_high_score_entry_state() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.spawn_player();
        spawn_enemy_laser_at_screen(&mut driver, Point::new(42, 120));

        let game_over = driver.step(GameInput::NONE);
        assert_eq!(game_over.phase, Phase::HighScoreEntry);

        let entry = driver.step(GameInput::NONE);
        assert_text(&entry, "FINAL SCORE 012000");
        assert_text(&entry, "HIGH SCORES");
        assert_text(&entry, "INITIALS ___");
        assert_text(&entry, "1. 012000");
        assert_text(&entry, "2. 010000");
        assert_text(&entry, "ENTER INITIALS");
    }

    #[test]
    fn high_score_entry_accepts_initials_and_backspace_from_actor_input() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.lives = 1;
        driver.score = 12_000;
        driver.next_bonus = 20_000;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));

        assert_eq!(driver.step(GameInput::NONE).phase, Phase::HighScoreEntry);

        let first = driver.step(GameInput {
            high_score_initial: Some('a'),
            ..GameInput::NONE
        });
        assert_eq!(first.phase, Phase::HighScoreEntry);
        assert!(first.high_score_initial_accepted);
        assert!(!first.high_score_submitted);
        assert_text(&first, "INITIALS A__");

        let erased = driver.step(GameInput {
            high_score_backspace: true,
            ..GameInput::NONE
        });
        assert_eq!(erased.phase, Phase::HighScoreEntry);
        assert_text(&erased, "INITIALS ___");

        let second = driver.step(GameInput {
            high_score_initial: Some('B'),
            ..GameInput::NONE
        });
        assert!(second.high_score_initial_accepted);
        assert_text(&second, "INITIALS B__");
        let third = driver.step(GameInput {
            high_score_initial: Some('C'),
            ..GameInput::NONE
        });
        assert!(third.high_score_initial_accepted);
        assert_text(&third, "INITIALS BC_");
        let submitted = driver.step(GameInput {
            high_score_initial: Some('D'),
            ..GameInput::NONE
        });

        assert_eq!(submitted.phase, Phase::GameOver);
        assert!(submitted.high_score_initial_accepted);
        assert!(submitted.high_score_submitted);
        assert_eq!(
            submitted.game_over_hall_of_fame_stall_remaining,
            Some(HIGH_SCORE_HALL_STALL_STEPS)
        );
        assert_eq!(
            submitted.game_state().game_over,
            GameOverSnapshot {
                hall_of_fame_stall_remaining: Some(HIGH_SCORE_HALL_STALL_STEPS),
                ..GameOverSnapshot::NONE
            }
        );
        assert_text(&submitted, "HALL OF FAME");
        assert!(!submitted.draws.iter().any(|draw| {
            draw.text
                .as_deref()
                .is_some_and(|text| text.contains("INITIALS"))
        }));

        for expected_timer in (1..HIGH_SCORE_HALL_STALL_STEPS).rev() {
            let waiting = driver.step(GameInput::NONE);
            assert_eq!(waiting.phase, Phase::GameOver);
            assert_eq!(
                waiting.game_over_hall_of_fame_stall_remaining,
                Some(expected_timer)
            );
            assert_text(&waiting, "HALL OF FAME");
        }

        let returned = driver.step(GameInput::NONE);
        assert_eq!(returned.phase, Phase::Attract);
        assert_eq!(returned.game_over_hall_of_fame_stall_remaining, None);
        assert!(
            returned
                .draws
                .iter()
                .any(|draw| matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
    }

    #[test]
    fn planetoid_mapper_matches_current_live_key_contract() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Planetoid);
        let mut step = KeyboardPoll::default();

        for key in [
            KeyboardKey::Character('5'),
            KeyboardKey::Character('6'),
            KeyboardKey::Character('7'),
            KeyboardKey::Enter,
            KeyboardKey::Character('2'),
            KeyboardKey::Character('A'),
            KeyboardKey::Character('Z'),
            KeyboardKey::LeftShift,
            KeyboardKey::Character(' '),
            KeyboardKey::Tab,
            KeyboardKey::Character('H'),
            KeyboardKey::Function(2),
            KeyboardKey::Function(3),
            KeyboardKey::Function(4),
            KeyboardKey::Function(5),
        ] {
            mapper.map_event(KeyboardEvent::press(key), &mut step);
        }
        mapper.finish_poll(&mut step);

        assert!(step.input.coin);
        assert!(step.input.coin_two);
        assert!(step.input.coin_three);
        assert!(step.input.start_one);
        assert!(step.input.start_two);
        assert!(step.input.fire);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.thrust);
        assert!(step.input.reverse);
        assert!(step.input.smart_bomb);
        assert!(step.input.hyperspace);
        assert!(step.input.service_advance);
        assert!(step.input.high_score_reset);
        assert!(step.input.auto_up_manual_down);
        assert!(step.input.tilt);
    }

    #[test]
    fn planetoid_mapper_binds_shift_to_reverse_and_space_to_thrust() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Planetoid);

        let mut shift_poll = KeyboardPoll::default();
        mapper.map_event(
            KeyboardEvent::press(KeyboardKey::LeftShift),
            &mut shift_poll,
        );
        mapper.finish_poll(&mut shift_poll);
        assert!(shift_poll.input.reverse);
        assert!(!shift_poll.input.thrust);

        let mut no_repeat_poll = KeyboardPoll::default();
        mapper.finish_poll(&mut no_repeat_poll);
        assert!(!no_repeat_poll.input.reverse);
        assert!(!no_repeat_poll.input.thrust);

        let mut space_poll = KeyboardPoll::default();
        mapper.map_event(
            KeyboardEvent::press(KeyboardKey::Character(' ')),
            &mut space_poll,
        );
        mapper.finish_poll(&mut space_poll);
        assert!(space_poll.input.thrust);
        assert!(!space_poll.input.reverse);

        let mut held_space_poll = KeyboardPoll::default();
        mapper.finish_poll(&mut held_space_poll);
        assert!(held_space_poll.input.thrust);
        assert!(!held_space_poll.input.reverse);

        let mut release_poll = KeyboardPoll::default();
        mapper.map_event(
            KeyboardEvent::release(KeyboardKey::Character(' ')),
            &mut release_poll,
        );
        let mut released_space_poll = KeyboardPoll::default();
        mapper.finish_poll(&mut released_space_poll);
        assert!(!released_space_poll.input.thrust);
        assert!(!released_space_poll.input.reverse);
    }

    #[test]
    fn cabinet_mapper_keeps_enter_out_of_the_start_binding() {
        let mut mapper = KeyboardMapper::new(KeyboardProfile::Cabinet);
        let mut enter_poll = KeyboardPoll::default();

        mapper.map_event(KeyboardEvent::press(KeyboardKey::Enter), &mut enter_poll);
        mapper.finish_poll(&mut enter_poll);
        assert!(!enter_poll.input.start_one);
        assert!(!enter_poll.input.fire);

        let mut step = KeyboardPoll::default();
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('1')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('2')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowUp), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::ArrowDown), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('R')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('T')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('F')), &mut step);
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Character('B')), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.start_one);
        assert!(step.input.start_two);
        assert!(step.input.altitude_up);
        assert!(step.input.altitude_down);
        assert!(step.input.reverse);
        assert!(step.input.thrust);
        assert!(step.input.fire);
        assert!(step.input.smart_bomb);
    }

    #[test]
    fn xyzzy_overlay_toggles_auto_fire_invincibility_and_overlay_bomb() {
        let mut mapper = KeyboardMapper::default();
        let mut step = KeyboardPoll::default();

        for character in ['x', 'y', 'z', 'z', 'y', 'f', 'g'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut step,
            );
        }
        mapper.map_event(KeyboardEvent::press(KeyboardKey::Tab), &mut step);
        mapper.finish_poll(&mut step);

        assert!(step.input.xyzzy.active);
        assert!(step.input.xyzzy.auto_fire);
        assert!(step.input.xyzzy.invincible);
        assert!(step.input.xyzzy.overlay_smart_bomb);
        assert!(step.input.fire);

        for character in ['x', 'y', 'z', 'z', 'y'] {
            mapper.map_event(
                KeyboardEvent::press(KeyboardKey::Character(character)),
                &mut KeyboardPoll::default(),
            );
        }
        assert_eq!(mapper.xyzzy_mode(), XyzzyMode::INACTIVE);
    }

    #[test]
    fn player_actor_emits_spawn_and_sound_when_prompted_to_fire() {
        let mut driver = started_driver();

        let fired = driver.step(GameInput {
            fire: true,
            ..GameInput::NONE
        });

        assert!(fired.sounds.contains(&SoundCue::Laser));
        assert!(fired.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::Laser {
                    direction: Direction::Right,
                    ..
                })
            )
        }));

        let next = driver.step(GameInput::NONE);
        assert!(
            next.snapshots
                .iter()
                .any(|snapshot| snapshot.kind == ActorKind::Laser)
        );
    }

    #[test]
    fn smart_bomb_clears_hostiles_with_explosions_and_score() {
        let mut driver = started_driver();

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });

        assert_eq!(pressed.score, 0);
        assert_eq!(pressed.smart_bombs, INITIAL_SMART_BOMBS - 1);
        assert_eq!(
            pressed.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 10,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert!(pressed.survivor_bonus.is_none());
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
        assert!(pressed.sounds.is_empty());
        assert!(pressed.commands.contains(&GameCommand::SmartBomb {
            consume_stock: true,
        }));

        let held_during_delay = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert_eq!(held_during_delay.score, 0);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert!(
            !held_during_delay
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::SmartBomb { .. }))
        );

        let detonated = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(detonated.score, LANDER_SCORE * 5);
        assert_eq!(
            detonated.smart_bomb_flash_steps_remaining,
            SMART_BOMB_FLASH_STEPS
        );
        assert_eq!(detonated.render_scene().clear_color, Color::WHITE);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
        assert_eq!(
            detonated
                .commands
                .iter()
                .filter(|command| matches!(command, GameCommand::Destroy(_)))
                .count(),
            5
        );

        let blocked_restore = driver.step(GameInput::NONE);
        let mut sounds = blocked_restore.sounds.clone();
        assert_eq!(blocked_restore.enemy_reserve, detonated.enemy_reserve);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);

        sounds.extend(collect_driver_smart_bomb_sound_sequence(&mut driver));
        assert_eq!(sounds, source_smart_bomb_sound_cues());

        let restored = step_until_driver_source_reserve_activates(&mut driver);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 5,
                ..EnemyReserveSnapshot::default()
            }
        );
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
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
    }
