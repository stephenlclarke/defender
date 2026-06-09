    #[test]
    fn embedded_actor_behavior_script_matches_arcade_profile_defaults() {
        let parsed = ActorBehaviorScript::parse_text(ACTOR_BEHAVIOR_SCRIPT)
            .expect("embedded actor behavior script should parse");

        assert_eq!(ActorBehaviorScript::default().manifest(), parsed.manifest());
        assert_eq!(
            ActorBehaviorScript::from_arcade_profile().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            parsed.manifest().default_profile,
            ActorBehaviorProfile::arcade_default()
        );
        assert_eq!(parsed.manifest().default_profile.player_speed, 1);
        assert!(parsed.manifest().kind_profiles.is_empty());
        assert!(parsed.manifest().actor_profiles.is_empty());
    }

    #[test]
    fn default_driver_exposes_embedded_actor_script_manifests() {
        let driver = ActorGameDriver::new();
        let manifest = driver.script_manifest();

        assert_eq!(
            manifest.attract_script,
            AttractScript::parse_text(ACTOR_ATTRACT_SCRIPT)
                .expect("embedded attract script should parse")
                .manifest()
        );
        assert_eq!(
            manifest.behavior_script,
            ActorBehaviorScript::parse_text(ACTOR_BEHAVIOR_SCRIPT)
                .expect("embedded behavior script should parse")
                .manifest()
        );
        assert_eq!(
            manifest.wave_script,
            ActorWaveScript::parse_text(ACTOR_WAVE_SCRIPT)
                .expect("embedded wave script should parse")
                .manifest()
        );
    }

    #[test]
    fn driver_script_bundle_parses_and_installs_custom_scripts() {
        let scripts = ActorDriverScripts::parse_texts(
            "text 1 forever 12 20 CUSTOM DRIVER\n",
            "\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 5\n",
            "\
            name bundled custom waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        )
        .expect("driver script bundle should parse");
        let expected_attract = scripts.attract_script.manifest();
        let expected_wave = scripts.wave_script.manifest();
        let mut driver = ActorGameDriver::with_scripts(scripts);

        let attract = driver.step(GameInput::NONE);
        assert!(attract.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("CUSTOM DRIVER") && draw.position == Point::new(12, 20)
        }));

        let attract_manifest = driver.script_manifest();
        assert_eq!(attract_manifest.attract_script, expected_attract);
        assert_eq!(attract_manifest.wave_script, expected_wave);
        assert!(
            !attract_manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let attract_wave_lander = attract_manifest
            .current_wave_profile
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave profile should inherit bundled lander behavior");
        assert_eq!(attract_wave_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(attract_wave_lander.lander_drift_speed, 5);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing = step_until_driver_player_start_completes(&mut driver, 1);
        let playing_manifest = driver.script_manifest();

        assert_eq!(playing.phase, Phase::Playing);
        assert!(
            !playing_manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let playing_lander = playing_manifest
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("playing behavior should come from bundled wave profile");
        assert_eq!(playing_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(playing_lander.lander_drift_speed, 5);
    }

    #[test]
    fn sectioned_driver_script_parses_and_installs_custom_scripts() {
        let scripts = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [behavior]\n\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 4\n\
            [wave]\n\
            name sectioned waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n",
        )
        .expect("sectioned driver script should parse");
        let expected_attract = scripts.attract_script.manifest();
        let expected_wave = scripts.wave_script.manifest();
        let mut driver = ActorGameDriver::with_scripts(scripts);

        let attract = driver.step(GameInput::NONE);
        assert!(attract.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("SECTIONED DRIVER") && draw.position == Point::new(12, 20)
        }));
        assert_eq!(driver.script_manifest().attract_script, expected_attract);
        assert_eq!(driver.script_manifest().wave_script, expected_wave);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing = step_until_driver_player_start_completes(&mut driver, 1);
        let manifest = driver.script_manifest();

        assert_eq!(playing.phase, Phase::Playing);
        assert!(
            !manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        let lander = manifest
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("sectioned wave should inherit bundled behavior");
        assert_eq!(lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(lander.lander_drift_speed, 4);
    }

    #[test]
    fn sectioned_driver_script_parses_to_manifest_and_runtime_adapter() {
        let scripts = "\
            [attract]\n\
            text 1 forever 12 20 RUNTIME SCRIPT\n\
            [behavior]\n\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 3\n\
            [wave]\n\
            name runtime script waves\n\
            wave 1\n\
            lander 80 214\n\
            human 100 214\n"
            .parse::<ActorDriverScripts>()
            .expect("sectioned driver script should parse via FromStr");
        let manifest = scripts.manifest();

        assert_eq!(manifest.attract_script.events.len(), 1);
        assert!(
            !manifest
                .behavior_script
                .default_profile
                .player_takes_enemy_collision_damage
        );
        assert_eq!(manifest.wave_script.name, "runtime script waves");
        let wave_lander = manifest.wave_script.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave should inherit bundled lander behavior");
        assert_eq!(wave_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(wave_lander.lander_drift_speed, 3);

        let mut runtime = ActorRuntimeAdapter::with_scripts(scripts);
        let frame = runtime.step(GameInput::NONE);

        assert_eq!(
            runtime.driver().script_manifest().wave_script,
            manifest.wave_script
        );
        assert!(frame.report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("RUNTIME SCRIPT") && draw.position == Point::new(12, 20)
        }));
        assert_eq!(frame.state.phase, GamePhase::Attract);
    }

    #[test]
    fn driver_script_bundle_reports_sectioned_parse_errors() {
        let error = ActorDriverScripts::parse_texts(
            "text 1 forever 12 20 CUSTOM DRIVER\n",
            "kind lander lander_mode drift\n",
            "\
            name broken wave\n\
            lander 80 214\n",
        )
        .expect_err("wave script should reject spawn before wave");

        assert_eq!(error.section, ActorDriverScriptSection::Wave);
        assert_eq!(error.line, 2);
        assert!(
            error
                .to_string()
                .contains("actor driver wave script line 2")
        );
        assert!(error.message.contains("wave action must appear"));
    }

    #[test]
    fn sectioned_driver_script_preserves_source_line_errors() {
        let error = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [behavior]\n\
            kind lander lander_mode drift\n\
            [wave]\n\
            name broken sectioned waves\n\
            lander 80 214\n",
        )
        .expect_err("sectioned wave script should reject spawn before wave");

        assert_eq!(error.section, ActorDriverScriptSection::Wave);
        assert_eq!(error.line, 7);
        assert!(
            error
                .to_string()
                .contains("actor driver wave script line 7")
        );
        assert!(error.message.contains("wave action must appear"));

        let error = ActorDriverScripts::parse_text(
            "\
            [attract]\n\
            text 1 forever 12 20 SECTIONED DRIVER\n\
            [driver]\n\
            noop\n",
        )
        .expect_err("unknown section should fail before parsing content");
        assert_eq!(error.section, ActorDriverScriptSection::Driver);
        assert_eq!(error.line, 3);
        assert!(
            error
                .message
                .contains("unknown driver script section `driver`")
        );
    }

    #[test]
    fn wave_script_text_parser_can_inherit_custom_base_behavior() {
        let behavior_script = ActorBehaviorScript::parse_text(
            "\
            default player_takes_enemy_collision_damage false\n\
            kind lander lander_mode drift\n\
            kind lander lander_drift_speed 6\n",
        )
        .expect("base behavior script should parse");
        let wave_script = ActorWaveScript::parse_text_with_base_behavior(
            "\
            name inherited wave behavior\n\
            arcade_wave 1 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n\
            wave 2\n\
            lander 80 214\n",
            &behavior_script,
        )
        .expect("wave script should inherit base behavior");
        let manifest = wave_script.manifest();

        assert_eq!(manifest.waves.len(), 2);
        for profile in &manifest.waves {
            assert!(
                !profile
                    .behavior_script
                    .default_profile
                    .player_takes_enemy_collision_damage
            );
        }
        let lander_runtime = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("arcade wave should keep arcade lander behavior");
        assert_eq!(
            lander_runtime.lander_fire_period_steps,
            ArcadeWaveProfile::for_wave(1)
                .lander_behavior()
                .lander_fire_period_steps
        );
        let clean_lander = manifest.waves[1]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("clean wave should inherit base kind behavior");
        assert_eq!(clean_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(clean_lander.lander_drift_speed, 6);
    }

    #[test]
    fn driver_script_manifest_exports_current_wave_and_spawns() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 6,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::new(
            "manifest-test",
            vec![ActorWaveProfile::with_family_spawns(
                1,
                ActorBehaviorScript::default()
                    .with_kind_behavior(ActorKind::Lander, lander_behavior),
                vec![ActorLanderSpawn::new(Point::new(80, 96))],
                vec![ActorBomberSpawn::new(Point::new(120, 80))],
                vec![ActorPodSpawn::new(Point::new(160, 88))],
                vec![ActorHumanSpawn::new(
                    Point::new(32, HUMAN_GROUND_Y),
                    HumanMode::Grounded,
                )],
            )],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        let attract_manifest = driver.script_manifest();
        assert_eq!(attract_manifest.phase, Phase::Attract);
        assert!(matches!(
            attract_manifest.attract_script.events[0].action,
            AttractScriptActionManifest::WilliamsLogo { .. }
        ));
        assert!(
            attract_manifest
                .attract_script
                .events
                .iter()
                .any(|event| matches!(
                    event.action,
                    AttractScriptActionManifest::DefenderWordmark { .. }
                ))
        );
        assert_eq!(attract_manifest.wave_script.name, "manifest-test");
        assert!(attract_manifest.wave_script.behavior_presets.is_empty());
        assert!(
            attract_manifest
                .wave_script
                .spawn_behavior_presets
                .is_empty()
        );
        assert_eq!(attract_manifest.current_wave_profile.arcade_wave, None);
        assert_eq!(
            attract_manifest.current_wave_profile.lander_spawns[0].position,
            Point::new(80, 96)
        );
        assert_eq!(
            attract_manifest
                .current_wave_profile
                .bomber_spawns
                .first()
                .map(|spawn| spawn.position),
            Some(Point::new(120, 80))
        );
        assert_eq!(
            attract_manifest
                .behavior_script
                .kind_profile(ActorKind::Lander),
            None
        );

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let playing_manifest = driver.script_manifest();

        assert_eq!(playing_manifest.phase, Phase::Playing);
        assert_eq!(playing_manifest.wave, 1);
        assert_eq!(
            playing_manifest
                .behavior_script
                .kind_profile(ActorKind::Lander),
            Some(lander_behavior)
        );
        assert_eq!(
            playing_manifest
                .current_wave_profile
                .behavior_script
                .kind_profile(ActorKind::Lander),
            Some(lander_behavior)
        );
    }

    #[test]
    fn step_report_manifest_carries_effective_xyzzy_behavior_override() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();

        let report = driver.step(GameInput {
            xyzzy: XyzzyMode {
                active: true,
                invincible: true,
                ..XyzzyMode::INACTIVE
            },
            ..GameInput::NONE
        });

        assert!(
            !report
                .behavior_script
                .behavior_for(player, ActorKind::Player)
                .player_takes_enemy_collision_damage
        );
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .behavior_for(player, ActorKind::Player)
                .player_takes_enemy_collision_damage
        );
    }

    #[test]
    fn wave_script_applies_behavior_when_play_starts() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::single_wave(
            "opening-chasers",
            ActorBehaviorScript::default().with_kind_behavior(ActorKind::Lander, lander_behavior),
            vec![Point::new(80, HUMAN_GROUND_Y)],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.wave, 1);
        assert_eq!(driver.wave(), 1);
        assert_eq!(driver.wave_script_name(), "opening-chasers");

        step_until_driver_player_start_completes(&mut driver, 1);
        let chasing = driver.step(GameInput::NONE);
        let lander = chasing
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave script should spawn a lander");
        assert_eq!(lander.position, Point::new(74, HUMAN_GROUND_Y - 5));
    }

    #[test]
    fn wave_script_can_configure_initial_human_spawns() {
        let wave_script = ActorWaveScript::new(
            "single-human-opening",
            vec![ActorWaveProfile::with_spawns(
                1,
                ActorBehaviorScript::default(),
                vec![ActorLanderSpawn::new(Point::new(220, 80))],
                vec![ActorHumanSpawn::new(
                    Point::new(32, HUMAN_GROUND_Y),
                    HumanMode::Grounded,
                )],
            )],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let settled = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Human), 1);
        assert!(settled.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Human
                && snapshot.position == Point::new(32, HUMAN_GROUND_Y)
                && snapshot.human_runtime.is_none()
        }));
    }

    #[test]
    fn wave_script_text_parser_builds_sorted_profiles_and_spawns() {
        let wave_script = ActorWaveScript::parse_text(
            "\
            name parsed progression\n\
            wave 2\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 5\n\
            lander 100 100\n\
            bomber 120 80\n\
            pod 160 88\n\
            mutant 140 90\n\
            swarmer 150 96\n\
            baiter 170 104\n\
            reserve_full 2 1 1 3 4\n\
            human 32 214 grounded\n\
            wave 1\n\
            behavior kind lander lander_mode chase_player\n\
            behavior kind lander lander_seek_speed 6\n\
            enemy_reserve 3 0 0 2\n\
            spawn_behavior lander 0 lander_seek_speed 8\n\
            lander 80 96\n\
            human 40 214 falling -1\n",
        )
        .expect("wave script text should parse");

        let manifest = wave_script.manifest();

        assert_eq!(manifest.name, "parsed progression");
        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            manifest.waves[0].lander_spawns[0].position,
            Point::new(80, 96)
        );
        assert_eq!(
            manifest.waves[0].human_spawns[0].mode,
            HumanMode::Falling { velocity: -1 }
        );
        assert_eq!(
            manifest.waves[0].enemy_reserve,
            EnemyReserveSnapshot {
                landers: 3,
                swarmers: 2,
                ..EnemyReserveSnapshot::default()
            }
        );
        let wave_one_lander = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave 1 lander profile should parse");
        assert_eq!(wave_one_lander.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(wave_one_lander.lander_seek_speed, 6);
        assert_eq!(manifest.waves[0].spawn_behavior_profiles.len(), 1);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0].kind,
            ActorKind::Lander
        );
        assert_eq!(manifest.waves[0].spawn_behavior_profiles[0].spawn_index, 0);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0]
                .profile
                .lander_seek_speed,
            8
        );

        assert_eq!(
            manifest.waves[1].bomber_spawns[0].position,
            Point::new(120, 80)
        );
        assert_eq!(
            manifest.waves[1].pod_spawns[0].position,
            Point::new(160, 88)
        );
        assert_eq!(
            manifest.waves[1].mutant_spawns[0].position,
            Point::new(140, 90)
        );
        assert_eq!(
            manifest.waves[1].swarmer_spawns[0].position,
            Point::new(150, 96)
        );
        assert_eq!(
            manifest.waves[1].baiter_spawns[0].position,
            Point::new(170, 104)
        );
        assert_eq!(
            manifest.waves[1].enemy_reserve,
            EnemyReserveSnapshot {
                landers: 2,
                bombers: 1,
                pods: 1,
                mutants: 3,
                swarmers: 4,
            }
        );
        let wave_two_lander = manifest.waves[1]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("wave 2 lander profile should parse");
        assert_eq!(wave_two_lander.lander_mode, LanderBehaviorMode::Drift);
        assert_eq!(wave_two_lander.lander_drift_speed, 5);
    }

    #[test]
    fn parsed_wave_script_drives_wave_spawns_and_next_wave_behavior() {
        let wave_script = "\
            name parsed waves\n\
            wave 1\n\
            behavior kind lander lander_mode chase_player\n\
            behavior kind lander lander_seek_speed 5\n\
            lander 80 214\n\
            human 100 214\n\
            wave 2\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 5\n\
            lander 100 100\n"
            .parse::<ActorWaveScript>()
            .expect("wave script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);
        let chasing = driver.step(GameInput::NONE);
        let lander = chasing
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave 1 should spawn a lander");
        assert_eq!(lander.position, Point::new(74, 209));

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert!(
            !pressed
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        let cleared = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            next_wave
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        let lander = next_wave
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("wave 2 should spawn a lander");
        assert_eq!(lander.position, Point::new(95, 100));
    }

    #[test]
    fn parsed_wave_script_drives_custom_hostile_family_spawns() {
        let wave_script = "\
            name custom hostiles\n\
            wave 1\n\
            behavior kind mutant mutant_mode drift\n\
            behavior kind swarmer swarmer_mode drift\n\
            behavior kind swarmer swarmer_seek_speed 1\n\
            behavior kind baiter baiter_mode drift\n\
            behavior kind baiter baiter_seek_speed 1\n\
            mutant 80 72\n\
            swarmer 120 84\n\
            baiter 160 96\n\
            human 40 214\n"
            .parse::<ActorWaveScript>()
            .expect("custom hostile wave script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 0);
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Swarmer), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Baiter), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 1);
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Mutant && snapshot.position == Point::new(79, 72)
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Swarmer && snapshot.position == Point::new(119, 84)
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Baiter && snapshot.position == Point::new(159, 96)
        }));
    }

    #[test]
    fn parsed_arcade_wave_overrides_drive_source_shaped_custom_wave() {
        let wave_script = concat!(
            "name custom source shape\n",
            "arcade_wave 1 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
            "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
            "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
        )
        .parse::<ActorWaveScript>()
        .expect("arcade wave overrides should parse");
        let manifest = wave_script.manifest();
        let profile = &manifest.waves[0];
        let source = profile
            .arcade_wave
            .expect("arcade_wave override should preserve source metadata");

        assert_eq!(source.wave_size, 5);
        assert_eq!(source.landers, 1);
        assert_eq!(source.bombers, 1);
        assert_eq!(source.pods, 1);
        assert_eq!(source.mutants, 1);
        assert_eq!(source.swarmers, 1);
        assert_eq!(source.swarmer_x_velocity, 64);
        assert_eq!(source.swarmer_shot_time, 11);
        assert_eq!(source.baiter_delay, 24);
        assert_eq!(source.mutant_x_velocity, 48);
        assert_eq!(source.mutant_random_y, 2);
        assert_eq!(source.mutant_shot_time, 12);
        assert_eq!(profile.lander_spawns.len(), 1);
        assert_eq!(profile.bomber_spawns.len(), 1);
        assert_eq!(profile.pod_spawns.len(), 1);
        assert_eq!(profile.mutant_spawns.len(), 1);
        assert_eq!(profile.swarmer_spawns.len(), 1);
        assert_eq!(profile.enemy_reserve, EnemyReserveSnapshot::default());
        assert!(profile.mutant_spawns[0].source.is_some());
        assert!(profile.swarmer_spawns[0].source.is_some());

        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);

        assert_eq!(driver.snapshot_count(ActorKind::Lander), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Bomber), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Pod), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Mutant), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Swarmer), 1);
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Mutant && snapshot.mutant_runtime.is_some()
        }));
        assert!(report.snapshots.iter().any(|snapshot| {
            snapshot.kind == ActorKind::Swarmer && snapshot.swarmer_runtime.is_some()
        }));
        assert_eq!(report.arcade_wave.wave_size, 5);
        assert_eq!(report.arcade_wave.mutant_x_velocity, 48);
        assert_eq!(report.arcade_wave.swarmer_shot_time, 11);
        let state_profile = report.game_state().wave_profile;
        assert_eq!(state_profile.wave_size, 5);
        assert_eq!(state_profile.landers, 1);
        assert_eq!(state_profile.bombers, 1);
        assert_eq!(state_profile.pods, 1);
        assert_eq!(state_profile.mutants, 1);
        assert_eq!(state_profile.swarmers, 1);
        assert_eq!(state_profile.swarmer_x_velocity, 64);
        assert_eq!(state_profile.swarmer_shot_time, 11);
        assert_eq!(state_profile.baiter_delay, 24);
        assert_eq!(state_profile.mutant_x_velocity, 48);
        assert_eq!(state_profile.mutant_random_y, 2);
        assert_eq!(state_profile.mutant_shot_time, 12);
        assert_eq!(
            state_profile.wave_time,
            WaveProfileSnapshot::for_wave(1).wave_time
        );
        assert_eq!(
            driver
                .script_manifest()
                .current_wave_profile
                .arcade_wave
                .expect("current wave manifest should expose source override")
                .mutants,
            1
        );
    }

    #[test]
    fn parsed_arcade_wave_range_overrides_apply_to_each_expanded_profile() {
        let wave_script = concat!(
            "name ranged source shape\n",
            "arcade_waves 1 2 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1 ",
            "swarmer_x_velocity 64 swarmer_shot_time 11 baiter_time 24 ",
            "mutant_x_velocity 48 mutant_random_y 2 mutant_shot_time 12\n",
        )
        .parse::<ActorWaveScript>()
        .expect("arcade wave range overrides should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for profile in &manifest.waves {
            let source = profile
                .arcade_wave
                .expect("range override should preserve source metadata");
            assert_eq!(source.wave_size, 5);
            assert_eq!(source.landers, 1);
            assert_eq!(source.bombers, 1);
            assert_eq!(source.pods, 1);
            assert_eq!(source.mutants, 1);
            assert_eq!(source.swarmers, 1);
            assert_eq!(source.swarmer_x_velocity, 64);
            assert_eq!(source.swarmer_shot_time, 11);
            assert_eq!(source.baiter_delay, 24);
            assert_eq!(source.mutant_x_velocity, 48);
            assert_eq!(source.mutant_random_y, 2);
            assert_eq!(source.mutant_shot_time, 12);
            assert_eq!(profile.lander_spawns.len(), 1);
            assert_eq!(profile.bomber_spawns.len(), 1);
            assert_eq!(profile.pod_spawns.len(), 1);
            assert_eq!(profile.mutant_spawns.len(), 1);
            assert_eq!(profile.swarmer_spawns.len(), 1);
            assert_eq!(profile.enemy_reserve, EnemyReserveSnapshot::default());
            assert!(profile.mutant_spawns[0].source.is_some());
            assert!(profile.swarmer_spawns[0].source.is_some());
        }

        let second = wave_script.profile_for_wave(2);
        assert_eq!(
            second
                .arcade_wave
                .expect("wave 2 should use the effective range override")
                .mutants,
            1
        );
        assert_eq!(second.mutant_spawns.len(), 1);
        assert_eq!(second.swarmer_spawns.len(), 1);
    }

    #[test]
    fn parsed_wave_script_applies_behavior_ranges_to_existing_profiles() {
        let wave_script = concat!(
            "name ranged behavior\n",
            "arcade_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "behavior_waves 1 2 kind lander lander_mode chase_player\n",
            "behavior_waves 1 2 kind lander lander_seek_speed 7\n",
            "spawn_behavior_waves 1 2 lander 0 lander_seek_speed 9\n",
        )
        .parse::<ActorWaveScript>()
        .expect("range behavior script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        for profile in &manifest.waves {
            let lander_behavior = profile
                .behavior_script
                .kind_profile(ActorKind::Lander)
                .expect("range behavior should install lander kind profile");
            assert_eq!(lander_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(lander_behavior.lander_seek_speed, 7);
            assert_eq!(profile.spawn_behavior_profiles.len(), 1);
            assert_eq!(profile.spawn_behavior_profiles[0].kind, ActorKind::Lander);
            assert_eq!(profile.spawn_behavior_profiles[0].spawn_index, 0);
            assert_eq!(
                profile.spawn_behavior_profiles[0].profile.lander_seek_speed,
                9
            );
        }
    }

    #[test]
    fn parsed_wave_script_applies_named_behavior_presets_to_current_and_range_profiles() {
        let wave_script = concat!(
            "name preset behavior\n",
            "behavior_preset hard_lander kind lander lander_mode chase_player\n",
            "behavior_preset hard_lander kind lander lander_seek_speed 7\n",
            "arcade_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_behavior_waves 1 2 hard_lander\n",
            "wave 3\n",
            "use_behavior hard_lander\n",
            "lander 80 214\n",
        )
        .parse::<ActorWaveScript>()
        .expect("behavior preset wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        for profile in manifest.waves.iter().take(2) {
            let lander_runtime = ArcadeWaveProfile::for_wave(profile.wave).lander_behavior();
            let lander_behavior = profile
                .behavior_script
                .kind_profile(ActorKind::Lander)
                .expect("range preset should install lander kind profile");
            assert_eq!(lander_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(lander_behavior.lander_seek_speed, 7);
            assert_eq!(
                lander_behavior.lander_fire_period_steps,
                lander_runtime.lander_fire_period_steps
            );
        }
        let clean_wave_lander = manifest.waves[2]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("current-wave preset should install lander kind profile");
        assert_eq!(
            clean_wave_lander.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(clean_wave_lander.lander_seek_speed, 7);
        assert_eq!(manifest.waves[2].arcade_wave, None);
    }

    #[test]
    fn parsed_wave_script_manifest_exposes_reusable_behavior_presets() {
        let wave_script = concat!(
            "name preset manifest\n",
            "behavior_preset Hard-Lander kind lander lander_mode chase_player\n",
            "behavior_preset Hard-Lander kind lander lander_seek_speed 7\n",
            "spawn_behavior_preset Fast-Slot lander_mode chase_player\n",
            "spawn_behavior_preset Fast-Slot lander_seek_speed 9\n",
            "arcade_wave 1 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_behavior hard_lander\n",
            "use_spawn_behavior lander 0 fast_slot\n",
        )
        .parse::<ActorWaveScript>()
        .expect("preset manifest wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest.behavior_presets,
            [ActorWaveBehaviorPresetManifest {
                name: "hard_lander".to_string(),
                updates: vec![
                    "kind lander lander_mode chase_player".to_string(),
                    "kind lander lander_seek_speed 7".to_string(),
                ],
            }]
        );
        assert_eq!(
            manifest.spawn_behavior_presets,
            [ActorWaveSpawnBehaviorPresetManifest {
                name: "fast_slot".to_string(),
                updates: vec![
                    ActorWaveSpawnBehaviorPresetUpdateManifest {
                        field: "lander_mode".to_string(),
                        values: vec!["chase_player".to_string()],
                    },
                    ActorWaveSpawnBehaviorPresetUpdateManifest {
                        field: "lander_seek_speed".to_string(),
                        values: vec!["9".to_string()],
                    },
                ],
            }]
        );
        let lander_behavior = manifest.waves[0]
            .behavior_script
            .kind_profile(ActorKind::Lander)
            .expect("behavior preset should still apply to wave profile");
        assert_eq!(lander_behavior.lander_seek_speed, 7);
        assert_eq!(manifest.waves[0].spawn_behavior_profiles.len(), 1);
        assert_eq!(
            manifest.waves[0].spawn_behavior_profiles[0]
                .profile
                .lander_seek_speed,
            9
        );
    }

    #[test]
    fn parsed_wave_script_reports_missing_behavior_range_profiles() {
        let error = "\
            name missing behavior range\n\
            arcade_wave 1\n\
            behavior_waves 1 2 kind lander lander_seek_speed 7\n"
            .parse::<ActorWaveScript>()
            .expect_err("range behavior should require existing profiles");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "wave range references undefined wave `2`");
    }

    #[test]
    fn parsed_wave_script_reports_unknown_behavior_presets() {
        let error = "\
            name missing preset\n\
            arcade_wave 1\n\
            use_behavior missing\n"
            .parse::<ActorWaveScript>()
            .expect_err("preset use should require a definition");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "unknown behavior preset `missing`");
    }

    #[test]
    fn parsed_wave_script_applies_spawn_behavior_presets_to_current_and_range_profiles() {
        let wave_script = concat!(
            "name spawn preset behavior\n",
            "spawn_behavior_preset fast_slot lander_mode chase_player\n",
            "spawn_behavior_preset fast_slot lander_seek_speed 9\n",
            "arcade_waves 1 2 wave_size 5 landers 2 bombers 0 pods 0 mutants 0 swarmers 0\n",
            "use_spawn_behavior_waves 1 2 lander 0 fast_slot\n",
            "wave 3\n",
            "behavior kind lander lander_mode drift\n",
            "behavior kind lander lander_drift_speed 4\n",
            "use_spawn_behavior lander 1 fast_slot\n",
            "lander 80 214\n",
            "lander 120 214\n",
        )
        .parse::<ActorWaveScript>()
        .expect("spawn behavior preset wave script should parse");
        let manifest = wave_script.manifest();

        assert_eq!(
            manifest
                .waves
                .iter()
                .map(|profile| profile.wave)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        for profile in manifest.waves.iter().take(2) {
            let lander_runtime = ArcadeWaveProfile::for_wave(profile.wave).lander_behavior();
            assert_eq!(profile.spawn_behavior_profiles.len(), 1);
            assert_eq!(profile.spawn_behavior_profiles[0].kind, ActorKind::Lander);
            assert_eq!(profile.spawn_behavior_profiles[0].spawn_index, 0);
            let spawn_profile = profile.spawn_behavior_profiles[0].profile;
            assert_eq!(spawn_profile.lander_mode, LanderBehaviorMode::ChasePlayer);
            assert_eq!(spawn_profile.lander_seek_speed, 9);
            assert_eq!(
                spawn_profile.lander_fire_period_steps,
                lander_runtime.lander_fire_period_steps
            );
        }

        let clean_spawn = manifest.waves[2].spawn_behavior_profiles[0];
        assert_eq!(clean_spawn.kind, ActorKind::Lander);
        assert_eq!(clean_spawn.spawn_index, 1);
        assert_eq!(
            clean_spawn.profile.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(clean_spawn.profile.lander_seek_speed, 9);
        assert_eq!(clean_spawn.profile.lander_drift_speed, 4);
    }

    #[test]
    fn parsed_wave_script_applies_spawn_behavior_preset_after_actor_allocation() {
        let wave_script = "\
            name spawn preset allocation\n\
            spawn_behavior_preset fast_slot lander_mode chase_player\n\
            spawn_behavior_preset fast_slot lander_seek_speed 5\n\
            wave 1\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 1\n\
            use_spawn_behavior lander 1 fast_slot\n\
            lander 80 214\n\
            lander 120 214\n\
            human 100 214\n"
            .parse::<ActorWaveScript>()
            .expect("spawn behavior preset script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);
        let landers = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();

        assert_eq!(landers.len(), 2);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(landers[0].id)
                .is_none()
        );
        let second_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(landers[1].id)
            .expect("preset spawn index should receive actor-id behavior");
        assert_eq!(second_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(second_behavior.lander_seek_speed, 5);
        assert_eq!(landers[0].position, Point::new(79, 214));
        assert!(landers[1].position.x < 120);
    }

    #[test]
    fn parsed_wave_script_reports_unknown_spawn_behavior_presets() {
        let error = "\
            name missing spawn preset\n\
            arcade_wave 1\n\
            use_spawn_behavior lander 0 missing\n"
            .parse::<ActorWaveScript>()
            .expect_err("spawn preset use should require a definition");

        assert_eq!(error.line, 3);
        assert_eq!(error.message, "unknown spawn behavior preset `missing`");
    }

    #[test]
    fn parsed_wave_script_applies_spawn_index_behavior_after_actor_allocation() {
        let wave_script = "\
            name spawn behavior\n\
            wave 1\n\
            behavior kind lander lander_mode drift\n\
            behavior kind lander lander_drift_speed 1\n\
            lander 80 214\n\
            lander 120 214\n\
            human 100 214\n\
            spawn_behavior lander 0 lander_mode chase_player\n\
            spawn_behavior lander 0 lander_seek_speed 5\n"
            .parse::<ActorWaveScript>()
            .expect("spawn behavior script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);

        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        let report = step_until_driver_player_start_completes(&mut driver, 1);
        let landers = report
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();

        assert_eq!(landers.len(), 2);
        let first_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(landers[0].id)
            .expect("first spawn should receive actor-id behavior");
        assert_eq!(first_behavior.lander_mode, LanderBehaviorMode::ChasePlayer);
        assert_eq!(first_behavior.lander_seek_speed, 5);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(landers[1].id)
                .is_none()
        );
        assert!(landers[0].position.x < 80);
        assert_eq!(landers[1].position, Point::new(119, 214));
    }

    #[test]
    fn parsed_wave_script_applies_spawn_index_behavior_to_reserve_allocations() {
        let wave_script = "\
            name reserve spawn behavior\n\
            arcade_wave 2\n\
            spawn_behavior lander 3 lander_mode chase_player\n\
            spawn_behavior lander 3 lander_seek_speed 5\n"
            .parse::<ActorWaveScript>()
            .expect("reserve spawn behavior script should parse");
        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.phase = Phase::Playing;
        driver.wave = 2;
        driver.arcade_rng = PLAYFIELD_START_RNG;
        driver.apply_wave_profile();
        driver.spawn_player();
        driver.spawn_wave_hostiles();
        driver.spawn_initial_humans();
        driver.arm_first_wave_early_lander_reserve_delay();

        let initial = driver.step(GameInput::NONE);
        let mut initial_landers = initial
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        initial_landers.sort_by_key(|snapshot| snapshot.id);
        assert_eq!(initial_landers.len(), 3);
        assert!(
            driver
                .script_manifest()
                .behavior_script
                .actor_profile(initial_landers[0].id)
                .is_none()
        );
        assert_eq!(
            initial.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 17,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );

        destroy_wave_hostiles(&mut driver, &initial);
        let restored = driver.step(GameInput::NONE);
        let mut reserve_landers = restored
            .snapshots
            .iter()
            .filter(|snapshot| snapshot.kind == ActorKind::Lander)
            .collect::<Vec<_>>();
        reserve_landers.sort_by_key(|snapshot| snapshot.id);
        let reserve_lander = reserve_landers
            .first()
            .expect("reserve should spawn replacement landers");

        assert!(
            restored
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::Spawn(SpawnRequest::Lander { .. })))
        );
        assert_eq!(reserve_landers.len(), MAX_ACTIVE_WAVE_ENEMIES);
        assert_eq!(
            restored.enemy_reserve,
            EnemyReserveSnapshot {
                landers: 12,
                bombers: 2,
                pods: 0,
                ..EnemyReserveSnapshot::default()
            }
        );
        let reserve_behavior = driver
            .script_manifest()
            .behavior_script
            .actor_profile(reserve_lander.id)
            .expect("reserve spawn index should receive actor-id behavior");
        assert_eq!(
            reserve_behavior.lander_mode,
            LanderBehaviorMode::ChasePlayer
        );
        assert_eq!(reserve_behavior.lander_seek_speed, 5);
    }

    #[test]
    fn wave_script_text_parser_reports_line_errors() {
        let error = ActorWaveScript::parse_text("lander 80 96\n")
            .expect_err("spawn before wave should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("wave action must appear"));

        let error = ActorWaveScript::parse_text("wave 1\nwave 1\n")
            .expect_err("duplicate wave should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("duplicate wave"));

        let error = ActorWaveScript::parse_text("wave 1\nbehavior kind lander no_such_field 1\n")
            .expect_err("bad behavior update should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("unknown behavior field"));

        let error =
            ActorWaveScript::parse_text("behavior_preset hard kind lander no_such_field 1\n")
                .expect_err("bad behavior preset update should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));

        let error = ActorWaveScript::parse_text("spawn_behavior_preset hard no_such_field 1\n")
            .expect_err("bad spawn behavior preset update should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown behavior field"));
    }

    #[test]
    fn cleared_wave_advances_to_next_behavior_script() {
        let wave_two_lander = ActorBehaviorProfile {
            lander_drift_speed: 5,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::Drift,
            ..ActorBehaviorProfile::default()
        };
        let wave_script = ActorWaveScript::new(
            "two-wave-test",
            vec![
                ActorWaveProfile::new(1, ActorBehaviorScript::default(), vec![Point::new(180, 88)]),
                ActorWaveProfile::new(
                    2,
                    ActorBehaviorScript::default()
                        .with_kind_behavior(ActorKind::Lander, wave_two_lander),
                    vec![Point::new(100, 100)],
                ),
            ],
        );
        let mut driver = ActorGameDriver::with_wave_script(wave_script);
        driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        step_until_driver_player_start_completes(&mut driver, 1);

        let pressed = driver.step(GameInput {
            smart_bomb: true,
            ..GameInput::NONE
        });
        assert!(
            !pressed
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );
        let cleared = step_until_driver_smart_bomb_detonates(&mut driver);
        assert_eq!(cleared.wave, 1);
        assert!(
            cleared
                .commands
                .contains(&GameCommand::WaveCleared { next_wave: 2 })
        );

        let next_wave = step_until_wave_started(&mut driver, 2);
        assert_eq!(next_wave.wave, 2);
        assert!(
            next_wave
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 2 })
        );
        let lander = next_wave
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Lander)
            .expect("next wave should spawn a scripted lander");
        assert_eq!(lander.position, Point::new(95, 100));
    }

    #[test]
    fn behavior_script_can_choose_lander_targeting_mode() {
        let lander_behavior = ActorBehaviorProfile {
            lander_seek_speed: 4,
            lander_fire_period_steps: u64::MAX,
            lander_mode: LanderBehaviorMode::ChasePlayer,
            ..ActorBehaviorProfile::default()
        };
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.set_kind_behavior(ActorKind::Lander, lander_behavior);
        driver.spawn_player();
        let lander_id = driver.spawn_lander_for_test(Point::new(80, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let chasing = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&chasing, lander_id).position.x, 75);
    }

    #[test]
    fn actor_specific_behavior_override_changes_one_actor() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let fast_lander = driver.spawn_lander_for_test(Point::new(80, 100));
        let normal_lander = driver.spawn_lander_for_test(Point::new(100, 100));
        let fast_behavior = ActorBehaviorProfile {
            lander_drift_speed: 4,
            lander_fire_period_steps: u64::MAX,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(fast_lander, fast_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(snapshot_for(&report, fast_lander).position.x, 76);
        assert_eq!(snapshot_for(&report, normal_lander).position.x, 99);
    }

    #[test]
    fn script_can_tune_player_and_laser_behavior() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        let player = driver.spawn_player();
        let player_behavior = ActorBehaviorProfile {
            player_speed: 5,
            player_laser_cooldown_steps: 1,
            ..ActorBehaviorProfile::default()
        };
        driver.set_actor_behavior(player, player_behavior);

        let moved = driver.step(GameInput {
            thrust: true,
            fire: true,
            ..GameInput::NONE
        });
        assert_eq!(snapshot_for(&moved, player).position.x, 47);

        let laser_behavior = ActorBehaviorProfile {
            laser_speed: 2,
            laser_lifetime_steps: 4,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Laser, laser_behavior);

        let laser_step = driver.step(GameInput::NONE);
        let laser = laser_step
            .snapshots
            .iter()
            .find(|snapshot| snapshot.kind == ActorKind::Laser)
            .expect("configured laser should stay alive");
        assert_eq!(laser.position.x, 61);
    }

    #[test]
    fn scripted_player_damage_override_matches_xyzzy_invincibility() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_lander_for_test(Point::new(42, 120));
        let player_behavior = ActorBehaviorProfile {
            player_takes_enemy_collision_damage: false,
            ..ActorBehaviorProfile::default()
        };
        driver.set_kind_behavior(ActorKind::Player, player_behavior);

        let report = driver.step(GameInput::NONE);

        assert_eq!(report.phase, Phase::Playing);
        assert_ne!(report.lives, 0);
    }

    #[test]
    fn lander_picks_up_and_carries_a_grounded_human() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_lander_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.spawn_human_for_test(Point::new(100, HUMAN_GROUND_Y));
        driver.step(GameInput::NONE);

        let pickup = driver.step(GameInput::NONE);
        assert!(pickup.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::AttachHuman {
                    lander: _,
                    human: _,
                    position: Point {
                        x: 100,
                        y: HUMAN_GROUND_Y
                    },
                }
            )
        }));
        assert!(pickup.sounds.contains(&SoundCue::LanderPickup));

        let carried = driver.step(GameInput::NONE);
        assert!(carried.sounds.contains(&SoundCue::HumanPulled));
        assert!(
            carried
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanCarried)
        );
    }

    #[test]
    fn carried_human_falls_when_carrier_disappears() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_carried_human_for_test(Point::new(100, 120), ActorId::new(99));

        let released = driver.step(GameInput::NONE);

        assert!(released.sounds.contains(&SoundCue::HumanReleased));
        assert_eq!(
            released.sound_events(&mut ActorSoundEventBridge::new()),
            [SoundEvent::UnmappedSoundCommand {
                command: ASTRONAUT_SHORT_CATCH_SOUND_COMMAND,
            }]
        );
        assert!(
            released
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::HumanFalling)
        );
    }

    #[test]
    fn falling_human_rescue_awards_500_points_and_score_popup() {
        let mut driver = ActorGameDriver::new();
        driver.phase = Phase::Playing;
        driver.spawn_player();
        driver.spawn_falling_human_for_test(Point::new(42, 120), 0);
        driver.step(GameInput::NONE);

        let rescued = driver.step(GameInput::NONE);

        assert_eq!(rescued.score, HUMAN_RESCUE_SCORE);
        assert!(rescued.sounds.contains(&SoundCue::HumanRescued));
        assert!(
            rescued
                .commands
                .contains(&GameCommand::AddScore(HUMAN_RESCUE_SCORE))
        );
        assert!(rescued.commands.iter().any(|command| {
            matches!(
                command,
                GameCommand::Spawn(SpawnRequest::ScorePopup {
                    points: HUMAN_RESCUE_SCORE,
                    ..
                })
            )
        }));
    }
