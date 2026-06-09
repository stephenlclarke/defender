    #[test]
    fn attract_actor_accepts_credit_and_start_commands() {
        let mut driver = ActorGameDriver::new();

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.phase, Phase::Attract);
        assert_eq!(credited.credits, 1);
        assert!(credited.sounds.contains(&SoundCue::Credit));
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::WilliamsLogo
                    && matches!(draw.effect, VisualEffect::WilliamsReveal { .. }))
        );
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(credits_label_text()))
        );
        assert!(
            credited
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("01"))
        );

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert_eq!(started.credits, 0);
        assert!(!started.sounds.contains(&SoundCue::Start));
        assert_eq!(
            started.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
                player: 1,
            })
        );
        assert_no_arcade_message(&started, MessageId::PlayerOne, PLAYER_START_PROMPT_SCREEN_ADDRESS);
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.phase, Phase::Playing);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
        assert_eq!(
            start_sound.player_start,
            Some(PlayerStartReport {
                delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS - 1,
                player: 1,
            })
        );
        assert_eq!(driver.snapshot_count(ActorKind::Player), 0);

        let settled = step_until_driver_player_start_completes(&mut driver, 1);
        assert_eq!(settled.phase, Phase::Playing);
        assert!(
            settled
                .commands
                .contains(&GameCommand::AdvanceWave { wave: 1 })
        );
        assert_eq!(driver.snapshot_count(ActorKind::Player), 1);
        assert_eq!(driver.snapshot_count(ActorKind::Lander), 5);
        assert_eq!(driver.snapshot_count(ActorKind::Human), 10);
    }

    #[test]
    fn attract_title_uses_williams_animation_and_defender_coalescence() {
        let mut driver = ActorGameDriver::new();

        let williams = driver.step(GameInput::NONE);
        assert!(williams.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::WilliamsLogo
                && matches!(
                    draw.effect,
                    VisualEffect::WilliamsReveal {
                        stroke_step: 1,
                        color_frame: 0,
                    }
                )
        }));
        let williams_scene = ActorRenderSceneBridge::new().render_scene_for_report(&williams);
        assert!(williams_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL
                && sprite.tint == VISUAL_STATE.attract_williams_logo_tint_for_frame(0)
        }));
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("HIGH SCORES"))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("1. 010000"))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some(credits_label_text()))
        );
        assert!(
            williams
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("00"))
        );
        let presents_text =
            actor_message_text(PRESENTS_MESSAGE);
        assert!(
            !williams
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(presents_text))
        );
        assert!(
            !williams
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
        );

        let mut presents = None;
        for _ in 1..ATTRACT_PRESENTS_START_STEP {
            presents = Some(driver.step(GameInput::NONE));
        }
        let presents = presents.expect("presents page should be reached");
        assert!(presents.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(presents_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
                        visual_offset: Point { x: 0, y: 0 },
                    }
                )
        }));
        let presents_scene = ActorRenderSceneBridge::new().render_scene_for_report(&presents);
        assert!(presents_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.layer == RenderLayer::Overlay
        }));

        let mut coalescing = None;
        for _ in ATTRACT_PRESENTS_START_STEP..DEFENDER_WORDMARK_START_STEP {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderCoalescence)
            {
                coalescing = Some(step);
                break;
            }
        }
        let coalescing = coalescing.expect("wordmark should enter coalescence");
        assert!(coalescing.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0,
                    }
                )
        }));

        let mut settled = None;
        for _ in 0..(DEFENDER_WORDMARK_SLOTS * DEFENDER_WORDMARK_ROW_PAIRS + 1) {
            let step = driver.step(GameInput::NONE);
            if step
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderWordmark)
            {
                settled = Some(step);
                break;
            }
        }
        let settled = settled.expect("wordmark should settle before hall-of-fame");
        assert!(settled.step < ATTRACT_HALL_OF_FAME_START_STEP);

        let mut hall = settled;
        while hall.step < ATTRACT_HALL_OF_FAME_START_STEP {
            hall = driver.step(GameInput::NONE);
        }
        assert_eq!(hall.step, ATTRACT_HALL_OF_FAME_START_STEP);
        assert!(
            hall.draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("HIGH SCORES"))
        );
        assert!(
            hall.draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some("1. 010000"))
        );
        assert!(
            hall.draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(credits_label_text()))
        );
        assert!(
            hall.draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("00"))
        );
        let hall_title_text =
            actor_message_text(ATTRACT_HALL_TITLE_MESSAGE);
        assert!(hall.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(hall_title_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: 0x3854,
                        visual_offset: ATTRACT_HALL_TABLE_VISUAL_OFFSET,
                    }
                )
        }));
        let hall_greatest_text =
            actor_message_text(ATTRACT_HALL_GREATEST_MESSAGE);
        assert_eq!(
            hall.draws
                .iter()
                .filter(|draw| draw.text.as_deref() == Some(hall_greatest_text))
                .count(),
            2
        );
        assert!(hall.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderLogo
                && draw.position == ATTRACT_HALL_DEFENDER_LOGO_POSITION
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(37, 128) && draw.text.as_deref() == Some("1")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(47, 128) && draw.text.as_deref() == Some("DRJ")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(75, 128) && draw.text.as_deref() == Some(" 21270")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(167, 128) && draw.text.as_deref() == Some("1")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(177, 128) && draw.text.as_deref() == Some("DRJ")
        }));
        assert!(hall.draws.iter().any(|draw| {
            draw.position == Point::new(205, 198) && draw.text.as_deref() == Some("  6010")
        }));
        let hall_scene = ActorRenderSceneBridge::new().render_scene_for_report(&hall);
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_H
                && sprite.position == [101.0, 78.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_D
                && sprite.position == [47.0, 128.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_DIGIT_2
                && sprite.position == [79.0, 128.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(hall_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position
                    == [
                        f32::from(ATTRACT_HALL_DEFENDER_LOGO_POSITION.x),
                        f32::from(ATTRACT_HALL_DEFENDER_LOGO_POSITION.y),
                    ]
                && sprite.layer == RenderLayer::Overlay
        }));

        let mut scoring = hall;
        while scoring.step < ATTRACT_SCORING_SEQUENCE_START_STEP {
            scoring = driver.step(GameInput::NONE);
        }
        assert_eq!(scoring.step, ATTRACT_SCORING_SEQUENCE_START_STEP);
        assert!(
            scoring
                .draws
                .iter()
                .all(|draw| draw.text.as_deref() != Some(hall_title_text))
        );
        assert!(
            scoring
                .draws
                .iter()
                .all(|draw| draw.sprite != SpriteKey::DefenderLogo)
        );
        let scan_text = actor_message_text(MessageId::ScannerInstruction);
        assert!(scoring.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(scan_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: 0x4330,
                        visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));
        for (message, _) in ATTRACT_INSTRUCTION_TEXT_LINES.iter().skip(1) {
            let text = actor_message_text(*message);
            assert!(
                !scoring
                    .draws
                    .iter()
                    .any(|draw| draw.text.as_deref() == Some(text)),
                "{message:?} should wait for the score-card reveal cadence"
            );
        }
        assert!(scoring.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::Text
                && matches!(
                    draw.effect,
                    VisualEffect::AttractScoringSurface { scoring_tick: 0 }
                )
        }));
        let scoring_scene = ActorRenderSceneBridge::new().render_scene_for_report(&scoring);
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_S
                && sprite.position == [123.0, 41.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(!scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L
                && sprite.position == [45.0, 105.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert_eq!(
            scoring_scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD)
                .count(),
            TOP_DISPLAY_BORDER_SEGMENTS.len()
        );
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TOP_DISPLAY_BORDER_WORD
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [-11.0, 33.0]
                && sprite.size == [312.0, 2.0]
                && sprite.tint == ATTRACT_SCORING_SCANNER_BORDER_TINT
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL
                && sprite.layer == RenderLayer::Hud
                && sprite.position == [85.0, 30.0]
                && sprite.size == ATTRACT_SCORING_SCANNER_TERRAIN_PIXEL_SIZE
                && sprite.tint == ATTRACT_SCORING_SCANNER_TERRAIN_TINT
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::TERRAIN_TILE
                && sprite.layer == RenderLayer::Terrain
                && sprite.tint == arcade_wave_landscape_tint(1)
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_SHIP
                && sprite.layer == RenderLayer::Objects
                && sprite.position
                    == actor_attract_scoring_scene_position(
                        ATTRACT_SCORING_PLAYER_X16,
                        ATTRACT_SCORING_PLAYER_Y16,
                    )
                && sprite.size == PLAYER_SHIP_SCENE_SIZE
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HUMAN
                && sprite.layer == RenderLayer::Objects
                && sprite.position
                    == actor_attract_scoring_scene_position(
                        ATTRACT_SCORING_HUMAN_X16,
                        ATTRACT_SCORING_HUMAN_Y16,
                    )
                && sprite.size == HUMAN_SCENE_SIZE
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.position
                    == actor_attract_scoring_scene_position(
                        ATTRACT_SCORING_LANDER_X16,
                        ATTRACT_SCORING_LANDER_Y16,
                    )
                && sprite.size == LANDER_SCENE_SIZE
        }));
        assert!(scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCANNER_PLAYER_BLIP
                && sprite.layer == RenderLayer::Hud
                && (sprite.position[0] - 98.6).abs() < 0.01
                && (sprite.position[1] - 3.0).abs() < 0.01
                && sprite.size == ATTRACT_SCORING_PLAYER_SCANNER_SIZE
                && sprite.tint == williams_color_byte_tint(0x99)
        }));
        assert_eq!(
            scoring_scene
                .sprites
                .iter()
                .filter(|sprite| sprite.sprite == SpriteId::SCANNER_OBJECT_BLIP)
                .count(),
            2
        );
        let mut rescue_score = scoring;
        let rescue_score_display_step =
            actor_attract_scoring_display_step_for_stage(ActorAttractScoringStage::RescueScore, 0);
        let rescue_score_tick =
            actor_attract_scoring_tick_for_display_step(rescue_score_display_step);
        for _ in 0..rescue_score_tick {
            rescue_score = driver.step(GameInput::NONE);
        }
        let rescue_score_scene =
            ActorRenderSceneBridge::new().render_scene_for_report(&rescue_score);
        let score_position = actor_attract_scoring_scene_position(
            ATTRACT_SCORING_SCORE_500_X16,
            ATTRACT_SCORING_SCORE_500_Y16,
        );
        assert!(!rescue_score_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_500 && sprite.layer == RenderLayer::Objects
        }));
        let score_pixels = score_popup_500_pixels(&rescue_score_scene, score_position);
        assert!(
            score_pixels.len() > 20,
            "rescued-human 500 bonus should render from coloured source pixels"
        );
        for tint in SCORE_POPUP_500_COLOR_CYCLE {
            assert!(
                score_pixels.iter().any(|sprite| sprite.tint == tint),
                "rescued-human 500 bonus should include {tint:?}"
            );
        }
        let mut next_score_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        push_attract_scoring_demo_scene(
            &mut next_score_scene,
            actor_attract_scoring_tick_for_display_step(rescue_score_display_step + 5),
        );
        let next_score_pixels = score_popup_500_pixels(&next_score_scene, score_position);
        assert_ne!(
            score_pixels
                .iter()
                .map(|sprite| (sprite.position, sprite.tint))
                .collect::<Vec<_>>(),
            next_score_pixels
                .iter()
                .map(|sprite| (sprite.position, sprite.tint))
                .collect::<Vec<_>>(),
            "rescued-human 500 bonus should colour-cycle between attract steps"
        );
        let lander_label_start = actor_attract_scoring_instruction_text_start_step(1);
        let mut lander_label = rescue_score;
        while lander_label.step < lander_label_start {
            lander_label = driver.step(GameInput::NONE);
        }
        assert_eq!(lander_label.step, lander_label_start);
        let lander_text = actor_message_text(MessageId::LanderInstruction);
        let mutant_text = actor_message_text(MessageId::MutantInstruction);
        assert!(lander_label.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(lander_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: 0x1C70,
                        visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));
        assert!(
            !lander_label
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(mutant_text))
        );
        let lander_label_scene =
            ActorRenderSceneBridge::new().render_scene_for_report(&lander_label);
        assert!(lander_label_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_L
                && sprite.position == [45.0, 105.0]
                && sprite.layer == RenderLayer::Overlay
        }));

        let last_label_start = actor_attract_scoring_instruction_text_start_step(
            ATTRACT_INSTRUCTION_TEXT_LINES.len() - 1,
        );
        let mut last_label = lander_label;
        while last_label.step < last_label_start {
            last_label = driver.step(GameInput::NONE);
        }
        assert_eq!(last_label.step, last_label_start);
        for (message, screen_address) in ATTRACT_INSTRUCTION_TEXT_LINES {
            let text = actor_message_text(message);
            assert!(last_label.draws.iter().any(|draw| {
                draw.text.as_deref() == Some(text)
                    && matches!(
                        draw.effect,
                        VisualEffect::ArcadeMessage {
                            top_left_screen_address,
                            visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
                        } if top_left_screen_address == screen_address
                    )
            }));
        }
        assert!(!scoring_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::HALL_OF_FAME_DEFENDER_LOGO
                && sprite.position
                    == [
                        f32::from(ATTRACT_HALL_DEFENDER_LOGO_POSITION.x),
                        f32::from(ATTRACT_HALL_DEFENDER_LOGO_POSITION.y),
                    ]
        }));
    }

    fn score_popup_500_pixels(scene: &RenderScene, position: [f32; 2]) -> Vec<&SceneSprite> {
        scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                    && sprite.layer == RenderLayer::Objects
                    && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                    && sprite.position[0] >= position[0]
                    && sprite.position[0] < position[0] + SCORE_POPUP_SCENE_SIZE[0]
                    && sprite.position[1] >= position[1]
                    && sprite.position[1] < position[1] + SCORE_POPUP_SCENE_SIZE[1]
            })
            .collect()
    }

    fn assert_attract_scoring_laser_stops_at_target_front(
        scene: &RenderScene,
        enemy: ActorAttractScoringEnemyKind,
        target_position: [f32; 2],
    ) {
        let expected_sprite = actor_attract_scoring_enemy_sprite(enemy);
        let target_size = actor_attract_scoring_enemy_size(enemy);
        assert!(
            scene.sprites.iter().any(|sprite| {
                sprite.sprite == expected_sprite
                    && sprite.layer == RenderLayer::Objects
                    && sprite.position == target_position
                    && sprite.size == target_size
            }),
            "scoring laser test must use the visible source target"
        );
        let target_front_edge = target_position[0];
        let target_center_y =
            actor_attract_scoring_laser_enemy_anchor(enemy, target_position)[1].round();
        let projectiles = scene
            .sprites
            .iter()
            .filter(|sprite| {
                sprite.sprite == SpriteId::PLAYER_PROJECTILE
                    && sprite.layer == RenderLayer::Projectiles
            })
            .collect::<Vec<_>>();
        let laser_right_edge = projectiles
            .iter()
            .map(|sprite| sprite.position[0] + sprite.size[0] - 1.0)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            laser_right_edge >= target_front_edge - LASER_BYTE_PIXELS as f32,
            "attract scoring laser should visibly reach the {enemy:?} front edge before explosion: laser={laser_right_edge}, target={target_front_edge}"
        );
        assert!(
            laser_right_edge <= target_front_edge,
            "attract scoring laser should stop at the {enemy:?} front instead of penetrating through it: laser={laser_right_edge}, target={target_front_edge}"
        );
        let ship_position = scene
            .sprites
            .iter()
            .find(|sprite| {
                sprite.sprite == SpriteId::PLAYER_SHIP && sprite.layer == RenderLayer::Objects
            })
            .expect("scoring laser test must render the player ship")
            .position;
        let ship_anchor = actor_attract_scoring_laser_ship_anchor(ship_position);
        let leftmost = projectiles
            .iter()
            .min_by(|left, right| left.position[0].total_cmp(&right.position[0]))
            .expect("scoring laser should render projectile pixels");
        assert!(
            leftmost.position[0] <= ship_anchor[0] + LASER_BYTE_PIXELS as f32,
            "attract scoring laser should visibly start at the player ship cannon"
        );
        assert!(
            (leftmost.position[1] - ship_anchor[1].round()).abs() <= 1.0,
            "attract scoring laser should originate from the player ship cannon row"
        );
        assert!(
            projectiles
                .iter()
                .all(|sprite| (sprite.position[1] - ship_anchor[1].round()).abs() <= 1.0),
            "attract scoring laser should stay horizontal on the player ship cannon row"
        );
        assert!(
            (target_center_y - ship_anchor[1].round()).abs() <= 1.0,
            "attract scoring target should be on the player ship cannon row before the laser fires"
        );
        let rightmost = projectiles
            .iter()
            .max_by(|left, right| left.position[0].total_cmp(&right.position[0]))
            .expect("scoring laser should render projectile pixels");
        assert!(
            (rightmost.position[1] - ship_anchor[1].round()).abs() <= 1.0,
            "attract scoring laser should end on the player ship cannon row"
        );
    }

    #[test]
    fn actor_attract_scoring_surface_projects_laser_fragments_and_legend_transfer() {
        let mut laser_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let rescue_laser_step =
            actor_attract_scoring_display_step_for_stage(ActorAttractScoringStage::RescueLaser, 2);
        push_attract_scoring_demo_scene(
            &mut laser_scene,
            actor_attract_scoring_tick_for_display_step(rescue_laser_step),
        );
        assert!(laser_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                && sprite.tint == LASER_TIP_TINT
        }));
        assert!(laser_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_PROJECTILE
                && sprite.layer == RenderLayer::Projectiles
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                && sprite.tint == LASER_BODY_TINT
        }));
        let laser_target = laser_scene
            .sprites
            .iter()
            .find(|sprite| sprite.sprite == SpriteId::ENEMY_LANDER)
            .expect("rescue laser should still render the target lander");
        assert_attract_scoring_laser_stops_at_target_front(
            &laser_scene,
            ActorAttractScoringEnemyKind::Lander,
            laser_target.position,
        );
        assert_eq!(
            actor_attract_scoring_laser_enemy_anchor(
                ActorAttractScoringEnemyKind::Lander,
                [20.0, 40.0]
            ),
            [20.0, 44.0]
        );
        assert_eq!(
            actor_attract_scoring_laser_enemy_anchor(
                ActorAttractScoringEnemyKind::Swarmer,
                [20.0, 40.0]
            ),
            [20.0, 42.0]
        );

        for (legend_index, entry) in ACTOR_ATTRACT_SCORING_LEGEND.iter().enumerate() {
            let mut legend_laser_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
            let legend_laser_step = actor_attract_scoring_display_step_for_stage(
                ActorAttractScoringStage::LegendLaser(legend_index),
                2,
            );
            push_attract_scoring_demo_scene(
                &mut legend_laser_scene,
                actor_attract_scoring_tick_for_display_step(legend_laser_step),
            );
            assert_attract_scoring_laser_stops_at_target_front(
                &legend_laser_scene,
                entry.enemy,
                actor_attract_scoring_scene_position(
                    ATTRACT_SCORING_LEGEND_ORIGIN_X16,
                    actor_attract_scoring_legend_enemy_y16(
                        entry.enemy,
                        actor_attract_scoring_legend_player_position().1,
                    ),
                ),
            );
        }

        let mut explosion_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let rescue_fall_step =
            actor_attract_scoring_display_step_for_stage(ActorAttractScoringStage::RescueFall, 5);
        push_attract_scoring_demo_scene(
            &mut explosion_scene,
            actor_attract_scoring_tick_for_display_step(rescue_fall_step),
        );
        assert!(explosion_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                && sprite.layer == RenderLayer::Objects
                && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
        }));
        assert!(!explosion_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER && sprite.layer == RenderLayer::Objects
        }));

        let mut transfer_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let legend_transfer_step = actor_attract_scoring_display_step_for_stage(
            ActorAttractScoringStage::LegendTransfer(0),
            0,
        );
        push_attract_scoring_demo_scene(
            &mut transfer_scene,
            actor_attract_scoring_tick_for_display_step(legend_transfer_step),
        );
        assert!(transfer_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::SCORE_POPUP_250
                && sprite.layer == RenderLayer::Objects
                && sprite.position == actor_attract_scoring_scene_position(0x07A0, 0x5900)
                && sprite.size == SCORE_POPUP_SCENE_SIZE
        }));
        assert!(
            transfer_scene
                .sprites
                .iter()
                .filter(|sprite| {
                    sprite.sprite == SpriteId::PLAYER_EXPLOSION_PIXEL
                        && sprite.layer == RenderLayer::Objects
                        && sprite.size == PLAYER_EXPLOSION_PIXEL_SCENE_SIZE
                })
                .count()
                > 8
        );

        let mut reveal_scene = RenderScene::empty(0, ACTOR_RENDER_SURFACE);
        let legend_reveal_step = actor_attract_scoring_display_step_for_stage(
            ActorAttractScoringStage::LegendReveal(0),
            0,
        );
        push_attract_scoring_demo_scene(
            &mut reveal_scene,
            actor_attract_scoring_tick_for_display_step(legend_reveal_step),
        );
        assert!(reveal_scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::ENEMY_LANDER
                && sprite.layer == RenderLayer::Objects
                && sprite.position == actor_attract_scoring_scene_position(0x07A0, 0x5900)
                && sprite.size == LANDER_SCENE_SIZE
        }));
    }

    #[test]
    fn embedded_actor_attract_script_matches_arcade_constructor_fallback() {
        let parsed = AttractScript::parse_text(ACTOR_ATTRACT_SCRIPT)
            .expect("embedded actor attract script should parse");

        assert_eq!(parsed.manifest().cycle_steps, Some(ATTRACT_CYCLE_STEPS));
        assert_eq!(
            AttractScript::arcade_title().manifest(),
            parsed.manifest()
        );
        assert_eq!(
            AttractScript::arcade_title_from_events().manifest(),
            parsed.manifest()
        );
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::WilliamsLogo { .. }
        ) && event.start_after_steps == 1
            && event.duration_steps == Some(ATTRACT_WILLIAMS_LOGO_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::ArcadeMessage {
                ref message,
                top_left_screen_address: ATTRACT_PRESENTS_ELECTRONICS_SCREEN,
                visual_offset: Point { x: 0, y: 0 },
            } if message == "WilliamsElectronics"
        ) && event.start_after_steps
            == ATTRACT_PRESENTS_START_STEP
            && event.duration_steps == Some(ATTRACT_PRESENTS_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::DefenderWordmark { .. }
        ) && event.start_after_steps
            == ATTRACT_DEFENDER_WORDMARK_START_STEP
            && event.duration_steps == Some(ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS)));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::HallScores {
                todays_top_left_screen_address: ATTRACT_HALL_TODAYS_TABLE_SCREEN,
                all_time_top_left_screen_address: ATTRACT_HALL_ALL_TIME_TABLE_SCREEN,
                visual_offset: ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            } if event.start_after_steps == ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps == Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Credits {
                label_position: ATTRACT_CREDIT_LABEL_POSITION,
                count_position: ATTRACT_CREDIT_COUNT_POSITION,
                minimum_credits: 1,
            } if event.start_after_steps == 1
                && event.duration_steps
                    == Some(ATTRACT_HALL_OF_FAME_START_STEP.saturating_sub(1))
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Credits {
                label_position: ATTRACT_CREDIT_LABEL_POSITION,
                count_position: ATTRACT_CREDIT_COUNT_POSITION,
                minimum_credits: 0,
            } if event.start_after_steps == ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps.is_none()
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::ArcadeMessage {
                ref message,
                top_left_screen_address: 0x3854,
                visual_offset: ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            } if event.start_after_steps == ATTRACT_HALL_OF_FAME_START_STEP
                && message == "HallTitle"
                && event.duration_steps == Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert_eq!(
            parsed
                .manifest()
                .events
                .iter()
                .filter(|event| matches!(
                    event.action,
                    AttractScriptActionManifest::ArcadeMessage {
                        ref message,
                        top_left_screen_address: 0x1E72 | 0x5F72,
                        visual_offset: ATTRACT_HALL_TABLE_VISUAL_OFFSET,
                    } if event.start_after_steps == ATTRACT_HALL_OF_FAME_START_STEP
                        && message == "HallGreatest"
                        && event.duration_steps == Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS)
                ))
                .count(),
            2
        );
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::Sprite {
                sprite: SpriteKey::DefenderLogo,
                position: ATTRACT_HALL_DEFENDER_LOGO_POSITION,
            } if event.start_after_steps == ATTRACT_HALL_OF_FAME_START_STEP
                && event.duration_steps == Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS)
        )));
        assert!(parsed.manifest().events.iter().any(|event| matches!(
            event.action,
            AttractScriptActionManifest::ScoringSurface
                if event.start_after_steps == ATTRACT_SCORING_SEQUENCE_START_STEP
                    && event.duration_steps.is_none()
        )));
        for (line_index, (message, screen_address)) in
            ATTRACT_INSTRUCTION_TEXT_LINES.iter().copied().enumerate()
        {
            assert!(parsed.manifest().events.iter().any(|event| matches!(
                event.action,
                AttractScriptActionManifest::ArcadeMessage {
                    message: ref event_message,
                    top_left_screen_address,
                    visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
                } if event.start_after_steps
                    == actor_attract_scoring_instruction_text_start_step(line_index)
                    && event.duration_steps.is_none()
                    && event_message == &format!("{message:?}")
                    && top_left_screen_address == screen_address
            )));
        }
    }

    #[test]
    fn actor_attract_scoring_instruction_labels_follow_arcade_reveal_cadence() {
        assert_eq!(
            ATTRACT_INSTRUCTION_TEXT_LINES
                .iter()
                .enumerate()
                .map(|(index, _)| actor_attract_scoring_instruction_text_start_step(index))
                .collect::<Vec<_>>(),
            vec![1200, 2028, 2214, 2394, 2574, 2760, 2940]
        );

        let parsed = AttractScript::parse_text(ACTOR_ATTRACT_SCRIPT)
            .expect("embedded actor attract script should parse");
        for (line_index, (message, _)) in ATTRACT_INSTRUCTION_TEXT_LINES.iter().copied().enumerate()
        {
            assert!(parsed.manifest().events.iter().any(|event| matches!(
                event.action,
                AttractScriptActionManifest::ArcadeMessage {
                    message: ref event_message,
                    ..
                } if event_message == &format!("{message:?}")
                    && event.start_after_steps
                        == actor_attract_scoring_instruction_text_start_step(line_index)
                    && event.duration_steps.is_none()
            )));
        }
    }

    #[test]
    fn default_actor_attract_script_loops_after_arcade_scoring_cycle() {
        let script = AttractScript::arcade_title();
        let high_scores = HighScoreTable::default().entries;

        assert_eq!(script.manifest().cycle_steps, Some(ATTRACT_CYCLE_STEPS));
        assert_eq!(ATTRACT_CYCLE_STEPS, 3479);

        let final_scoring_draws =
            script.draws_for(ActorId::new(99), ATTRACT_CYCLE_STEPS - 1, &high_scores, 0);
        assert!(
            final_scoring_draws
                .iter()
                .any(|draw| { matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) })
        );
        assert!(
            !final_scoring_draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::WilliamsLogo)
        );

        let wrapped_draws =
            script.draws_for(ActorId::new(99), ATTRACT_CYCLE_STEPS, &high_scores, 0);
        assert!(wrapped_draws.iter().any(|draw| {
            draw.sprite == SpriteKey::WilliamsLogo
                && matches!(
                    draw.effect,
                    VisualEffect::WilliamsReveal { stroke_step: 1, .. }
                )
        }));
        assert!(
            !wrapped_draws
                .iter()
                .any(|draw| { matches!(draw.effect, VisualEffect::AttractScoringSurface { .. }) })
        );
        let scan_text = actor_message_text(MessageId::ScannerInstruction);
        assert!(
            !wrapped_draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some(scan_text))
        );
    }

    #[test]
    fn actor_scanner_mini_terrain_records_match_arcade_reference_slice() {
        let records = scanner_mini_terrain_records();

        assert_eq!(records.len(), SCANNER_TERRAIN_RECORDS);
        assert_eq!(
            &records[..8],
            &[
                ScannerMiniTerrainRecord {
                    screen_address: 0x3025,
                    word: 0x7700,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x3124,
                    word: 0x7700,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x3222,
                    word: 0x0770,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x3320,
                    word: 0x0770,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x341E,
                    word: 0x0770,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x351C,
                    word: 0x0770,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x361D,
                    word: 0x7007,
                },
                ScannerMiniTerrainRecord {
                    screen_address: 0x371F,
                    word: 0x7007,
                },
            ]
        );
        assert_eq!(
            &main_terrain_record_bytes()[..9],
            &[0x25, 0x70, 0x07, 0x26, 0x77, 0x00, 0x26, 0x07, 0x70]
        );
    }

    #[test]
    fn custom_driver_can_script_its_own_attract_screen() {
        let script = AttractScript::new(vec![
            AttractScriptEvent::text(2, Some(5), Point::new(12, 20), "CUSTOM ATTRACT"),
            AttractScriptEvent::sprite(3, None, SpriteKey::DefenderLogo, Point::new(40, 44)),
            AttractScriptEvent::defender_wordmark(4, None, Point::new(70, 80)),
        ]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let first = driver.step(GameInput::NONE);
        assert!(!first.draws.iter().any(|draw| {
            draw.text.as_deref() == Some("CUSTOM ATTRACT")
                || draw.sprite == SpriteKey::DefenderLogo
                || draw.sprite == SpriteKey::DefenderCoalescence
        }));

        let second = driver.step(GameInput::NONE);
        assert!(
            second
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("CUSTOM ATTRACT"))
        );
        assert!(
            !second
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let third = driver.step(GameInput::NONE);
        assert!(
            third
                .draws
                .iter()
                .any(|draw| draw.sprite == SpriteKey::DefenderLogo)
        );

        let fourth = driver.step(GameInput::NONE);
        assert!(fourth.draws.iter().any(|draw| {
            draw.sprite == SpriteKey::DefenderCoalescence
                && matches!(
                    draw.effect,
                    VisualEffect::DefenderCoalescence {
                        slot: 0,
                        row_pair: 0
                    }
                )
        }));
    }

    #[test]
    fn attract_script_manifest_exposes_custom_driver_events() {
        let script = AttractScript::new(vec![
            AttractScriptEvent::defender_wordmark(9, Some(12), Point::new(70, 80)),
            AttractScriptEvent::text(2, Some(5), Point::new(12, 20), "CUSTOM ATTRACT"),
            AttractScriptEvent::williams_logo(5, None, Point::new(18, 44)),
        ]);
        let mut driver = ActorGameDriver::with_attract_script(script.clone());

        let manifest = driver.script_manifest();

        assert_eq!(manifest.attract_script.cycle_steps, None);
        assert_eq!(manifest.attract_script, script.manifest());
        assert_eq!(
            manifest
                .attract_script
                .events
                .iter()
                .map(|event| event.start_after_steps)
                .collect::<Vec<_>>(),
            vec![2, 5, 9]
        );
        assert_eq!(
            manifest.attract_script.events[0].action,
            AttractScriptActionManifest::Text {
                position: Point::new(12, 20),
                value: "CUSTOM ATTRACT".to_string(),
            }
        );
        assert_eq!(
            manifest.attract_script.events[1].action,
            AttractScriptActionManifest::WilliamsLogo {
                position: Point::new(18, 44),
                reveal_steps: WILLIAMS_REVEAL_STEPS,
                color_period: WILLIAMS_COLOR_PERIOD,
            }
        );
        assert_eq!(
            manifest.attract_script.events[2].action,
            AttractScriptActionManifest::DefenderWordmark {
                position: Point::new(70, 80),
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            }
        );

        driver.step(GameInput::NONE);
        assert_eq!(driver.script_manifest().attract_script, script.manifest());
    }

    #[test]
    fn attract_script_text_parser_builds_sorted_event_manifest() {
        let script = AttractScript::parse_text(
            "\
            # Custom attract script\n\
            cycle 12\n\
            defender_wordmark 9 12 70 80\n\
            text 2 5 12 20 CUSTOM ATTRACT\n\
            high_scores 4 forever 80 100 9 3\n\
            hall_scores 4 forever 0x1886 0x5986 -11 -6\n\
            scoring_surface 4 forever\n\
            credits 4 forever 12 228 82 228\n\
            credits_nonzero 4 8 14 226 84 226\n\
            sprite 6 forever defender_logo 40 44\n\
            williams_logo 5 - 18 44\n",
        )
        .expect("custom attract script text should parse");

        let manifest = script.manifest();

        assert_eq!(manifest.cycle_steps, Some(12));
        assert_eq!(
            manifest
                .events
                .iter()
                .map(|event| event.start_after_steps)
                .collect::<Vec<_>>(),
            vec![2, 4, 4, 4, 4, 4, 5, 6, 9]
        );
        assert_eq!(
            manifest.events[0].action,
            AttractScriptActionManifest::Text {
                position: Point::new(12, 20),
                value: "CUSTOM ATTRACT".to_string(),
            }
        );
        assert_eq!(
            manifest.events[1].action,
            AttractScriptActionManifest::HighScores {
                position: Point::new(80, 100),
                row_height: 9,
                rows: 3,
            }
        );
        assert_eq!(
            manifest.events[2].action,
            AttractScriptActionManifest::HallScores {
                todays_top_left_screen_address: ATTRACT_HALL_TODAYS_TABLE_SCREEN,
                all_time_top_left_screen_address: ATTRACT_HALL_ALL_TIME_TABLE_SCREEN,
                visual_offset: ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            }
        );
        assert_eq!(
            manifest.events[3].action,
            AttractScriptActionManifest::ScoringSurface
        );
        assert_eq!(
            manifest.events[4].action,
            AttractScriptActionManifest::Credits {
                label_position: Point::new(12, 228),
                count_position: Point::new(82, 228),
                minimum_credits: 0,
            }
        );
        assert_eq!(
            manifest.events[5].action,
            AttractScriptActionManifest::Credits {
                label_position: Point::new(14, 226),
                count_position: Point::new(84, 226),
                minimum_credits: 1,
            }
        );
        assert_eq!(manifest.events[5].duration_steps, Some(8));
        assert_eq!(
            manifest.events[7].action,
            AttractScriptActionManifest::Sprite {
                sprite: SpriteKey::DefenderLogo,
                position: Point::new(40, 44),
            }
        );
        assert_eq!(
            manifest.events[8].action,
            AttractScriptActionManifest::DefenderWordmark {
                position: Point::new(70, 80),
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            }
        );
    }

    #[test]
    fn custom_attract_scripts_only_loop_when_cycle_is_declared() {
        let high_scores = HighScoreTable::default().entries;
        let unlooped = AttractScript::parse_text("text 2 forever 12 20 UNBOUNDED")
            .expect("custom unlooped script should parse");
        let unlooped_draws = unlooped.draws_for(ActorId::new(1), 12, &high_scores, 0);
        assert!(
            unlooped_draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("UNBOUNDED"))
        );

        let looped = AttractScript::parse_text(
            "\
            cycle 5\n\
            text 2 forever 12 20 LOOPED\n",
        )
        .expect("custom looped script should parse");
        let wrapped_to_first_step = looped.draws_for(ActorId::new(1), 5, &high_scores, 0);
        assert!(
            wrapped_to_first_step
                .iter()
                .all(|draw| draw.text.as_deref() != Some("LOOPED"))
        );
        let wrapped_to_second_step = looped.draws_for(ActorId::new(1), 7, &high_scores, 0);
        assert!(
            wrapped_to_second_step
                .iter()
                .any(|draw| draw.text.as_deref() == Some("LOOPED"))
        );
    }

    #[test]
    fn parsed_attract_script_drives_draws_and_preserves_start_controls() {
        let script = "\
            text 1 forever 10 10 PRESS START\n\
            sprite 2 forever defender_logo 40 44\n"
            .parse::<AttractScript>()
            .expect("script text should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);

        let first = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(first.credits, 1);
        assert!(
            first
                .draws
                .iter()
                .any(|draw| draw.text.as_deref() == Some("PRESS START"))
        );

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert!(!started.sounds.contains(&SoundCue::Start));
        assert!(
            started
                .commands
                .iter()
                .any(|command| matches!(command, GameCommand::StartOnePlayer))
        );
        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
    }

    #[test]
    fn parsed_attract_script_draws_prompt_high_score_rows() {
        let script = "high_scores 1 forever 20 40 8 3"
            .parse::<AttractScript>()
            .expect("high-score script action should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);
        driver.high_scores.record(12_000);

        let report = driver.step(GameInput::NONE);

        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 40) && draw.text.as_deref() == Some("1. 012000")
        }));
        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 48) && draw.text.as_deref() == Some("2. 010000")
        }));
        assert!(report.draws.iter().any(|draw| {
            draw.position == Point::new(20, 56) && draw.text.as_deref() == Some("3. 007500")
        }));
    }

    #[test]
    fn parsed_attract_script_draws_prompt_credit_count() {
        let script = "credits 1 forever 12 228 82 228"
            .parse::<AttractScript>()
            .expect("credit script action should parse");
        let mut driver = ActorGameDriver::with_attract_script(script);
        let credits_label = actor_message_text(CREDITS_MESSAGE);

        let first = driver.step(GameInput::NONE);
        assert!(first.draws.iter().any(|draw| {
            draw.position == Point::new(12, 228)
                && draw.text.as_deref() == Some(credits_label)
        }));
        assert!(first.draws.iter().any(|draw| {
            draw.position == Point::new(82, 228) && draw.text.as_deref() == Some("00")
        }));

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.credits, 1);
        assert!(credited.draws.iter().any(|draw| {
            draw.position == Point::new(82, 228) && draw.text.as_deref() == Some("01")
        }));
    }

    #[test]
    fn parsed_attract_script_draws_arcade_message_with_controls() {
        let script = "message 1 forever ELECV 0x3258"
            .parse::<AttractScript>()
            .expect("arcade message script action should parse");
        assert_eq!(
            script.manifest().events[0].action,
            AttractScriptActionManifest::ArcadeMessage {
                message: "WilliamsElectronics".to_string(),
                top_left_screen_address: 0x3258,
                visual_offset: Point::new(0, 0),
            }
        );
        let mut driver = ActorGameDriver::with_attract_script(script);
        let message_text = actor_message_text(PRESENTS_MESSAGE);

        let report = driver.step(GameInput::NONE);
        assert!(report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(message_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: 0x3258,
                        visual_offset: Point { x: 0, y: 0 },
                    }
                )
        }));

        let scene = ActorRenderSceneBridge::new().render_scene_for_report(&report);
        assert_eq!(scene.sprites.len(), 23);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.layer == RenderLayer::Overlay
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [124.0, 108.0]
                && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn parsed_attract_script_draws_arcade_message_with_visual_offset() {
        let script = "message 1 forever ELECV 0x3258 -11 -7"
            .parse::<AttractScript>()
            .expect("arcade message script action should parse with offset");
        assert_eq!(
            script.manifest().events[0].action,
            AttractScriptActionManifest::ArcadeMessage {
                message: "WilliamsElectronics".to_string(),
                top_left_screen_address: 0x3258,
                visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
            }
        );
        let mut driver = ActorGameDriver::with_attract_script(script);
        let message_text = actor_message_text(PRESENTS_MESSAGE);

        let report = driver.step(GameInput::NONE);
        assert!(report.draws.iter().any(|draw| {
            draw.text.as_deref() == Some(message_text)
                && matches!(
                    draw.effect,
                    VisualEffect::ArcadeMessage {
                        top_left_screen_address: 0x3258,
                        visual_offset: ATTRACT_SCORING_VISUAL_OFFSET,
                    }
                )
        }));

        let scene = ActorRenderSceneBridge::new().render_scene_for_report(&report);
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [89.0, 81.0]
                && sprite.layer == RenderLayer::Overlay
        }));
    }

    #[test]
    fn attract_script_text_parser_reports_line_errors() {
        let error = AttractScript::parse_text("text 1 forever 10\n")
            .expect_err("missing text y coordinate should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("missing y"));

        let error = AttractScript::parse_text("sprite 1 forever no_such_sprite 1 2\n")
            .expect_err("unknown sprite key should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown sprite key"));

        let error = AttractScript::parse_text("message 1 forever NO_SUCH_MESSAGE 0x3258\n")
            .expect_err("unknown message key should fail");
        assert_eq!(error.line, 1);
        assert!(error.to_string().contains("unknown message key"));

        let error =
            AttractScript::parse_text("cycle 0\n").expect_err("zero cycle length should fail");
        assert_eq!(error.line, 1);
        assert!(
            error
                .to_string()
                .contains("cycle steps must be greater than zero")
        );

        let error = AttractScript::parse_text("cycle 12\ncycle 13\n")
            .expect_err("duplicate cycle directive should fail");
        assert_eq!(error.line, 2);
        assert!(error.to_string().contains("duplicate cycle directive"));
    }

    #[test]
    fn custom_attract_script_keeps_coin_and_start_controls() {
        let script = AttractScript::new(vec![AttractScriptEvent::text(
            1,
            None,
            Point::new(10, 10),
            "PRESS START",
        )]);
        let mut driver = ActorGameDriver::with_attract_script(script);

        let credited = driver.step(GameInput {
            coin: true,
            ..GameInput::NONE
        });
        assert_eq!(credited.credits, 1);

        let started = driver.step(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        assert_eq!(started.phase, Phase::Playing);
        assert!(!started.sounds.contains(&SoundCue::Start));
        let start_sound = driver.step(GameInput::NONE);
        assert_eq!(start_sound.sounds, [SoundCue::Start]);
    }
