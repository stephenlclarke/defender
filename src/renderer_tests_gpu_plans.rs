    #[test]
    fn render_scene_collects_sprites_in_order() {
        let mut scene = RenderScene::empty(7, SurfaceSize::new(320, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(3),
            layer: RenderLayer::Objects,
            position: [12.0, 24.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        assert_eq!(scene.frame, 7);
        assert_eq!(scene.sprites[0].sprite, SpriteId(3));
    }

    #[test]
    fn render_scene_summary_counts_layers_and_visual_signature() {
        let mut scene = RenderScene::empty(12, SurfaceSize::new(292, 240));
        scene.visual_signature = Some(0xCAFE_BABE);
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::SCORE_TEXT,
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [80.0, 8.0],
            tint: Color::WHITE,
        });

        let summary = scene.summary();

        assert_eq!(summary.visual_signature, Some(0xCAFE_BABE));
        assert_eq!(summary.raster_count, 0);
        assert_eq!(summary.sprite_count, 2);
        assert_eq!(
            summary.layers,
            RenderLayerCounts {
                objects: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn scene_raster_validates_rgba_payload_length() {
        let surface = SurfaceSize::new(2, 2);
        let pixels = vec![0; 15];

        assert_eq!(
            SceneRaster::from_rgba(surface, pixels).expect_err("invalid length"),
            SceneRasterError::PixelBufferLength {
                expected: 16,
                actual: 15
            }
        );
    }

    #[test]
    fn scene_raster_reports_oversized_surfaces_and_formats_errors() {
        let error = SceneRaster::from_rgba(SurfaceSize::new(u32::MAX, u32::MAX), Vec::new())
            .expect_err("oversized surface");

        assert_eq!(
            error,
            SceneRasterError::PixelBufferTooLarge {
                surface: SurfaceSize::new(u32::MAX, u32::MAX)
            }
        );
        assert_eq!(
            error.to_string(),
            "rgba buffer is too large for 4294967295x4294967295 surface"
        );
        assert_eq!(
            SceneRasterError::PixelBufferLength {
                expected: 8,
                actual: 4
            }
            .to_string(),
            "rgba buffer length mismatch: expected 8 bytes, got 4"
        );
    }

    #[test]
    fn render_scene_from_raster_keeps_cpu_raster_payload() {
        let scene = RenderScene::from_rgba(
            21,
            SurfaceSize::new(2, 1),
            vec![0, 0, 0, 255, 10, 20, 30, 255],
            Some(0x1234_5678),
        )
        .expect("raster scene");

        let raster = scene.raster().expect("scene raster");
        assert_eq!(raster.surface, SurfaceSize::new(2, 1));
        assert!(raster.is_non_blank());
        assert_eq!(scene.summary().raster_count, 1);
        assert_eq!(scene.summary().visual_signature, Some(0x1234_5678));
    }

    #[test]
    fn render_scene_can_replace_raster_payload_and_move_pixels_out() {
        let mut scene = RenderScene::empty(22, SurfaceSize::new(1, 1));
        let raster =
            SceneRaster::from_rgba(SurfaceSize::new(2, 1), vec![1, 2, 3, 255, 4, 5, 6, 255])
                .expect("replacement raster");

        scene.set_raster(raster.clone());

        assert_eq!(scene.surface, SurfaceSize::new(2, 1));
        assert_eq!(scene.summary().raster_count, 1);
        assert_eq!(raster.into_pixels(), vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn render_scene_to_rgba_draws_atlas_sprites_without_gpu() {
        let atlas = TextureAtlas::with_rgba(
            SurfaceSize::new(1, 1),
            vec![AtlasRegion {
                sprite: SpriteId(0xAA),
                origin: [0, 0],
                size: [1, 1],
            }],
            vec![120, 200, 40, 255],
        )
        .expect("atlas");
        let mut scene = RenderScene::empty(23, SurfaceSize::new(2, 2));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(0xAA),
            layer: RenderLayer::Objects,
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            tint: Color::WHITE,
        });

        let raster = render_scene_with_atlas_to_rgba(&scene, SurfaceSize::new(2, 2), &atlas)
            .expect("rasterized scene");

        assert_eq!(&raster.pixels()[0..4], [120, 200, 40, 255]);
        assert_eq!(&raster.pixels()[4..8], [0, 0, 0, 255]);
    }

    #[test]
    fn render_scene_to_rgba_respects_layer_order_for_readme_media() {
        let atlas = TextureAtlas::with_rgba(
            SurfaceSize::new(2, 1),
            vec![
                AtlasRegion {
                    sprite: SpriteId(1),
                    origin: [0, 0],
                    size: [1, 1],
                },
                AtlasRegion {
                    sprite: SpriteId(2),
                    origin: [1, 0],
                    size: [1, 1],
                },
            ],
            vec![255, 0, 0, 255, 0, 255, 0, 255],
        )
        .expect("atlas");
        let mut scene = RenderScene::empty(24, SurfaceSize::new(1, 1));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(2),
            layer: RenderLayer::Terrain,
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            tint: Color::WHITE,
        });
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(1),
            layer: RenderLayer::Hud,
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            tint: Color::WHITE,
        });

        let raster = render_scene_with_atlas_to_rgba(&scene, SurfaceSize::new(1, 1), &atlas)
            .expect("rasterized scene");

        assert_eq!(raster.pixels(), [255, 0, 0, 255]);
    }

    #[test]
    fn render_scene_summary_counts_every_layer() {
        let mut scene = RenderScene::empty(13, SurfaceSize::new(292, 240));
        for (index, layer) in [
            RenderLayer::Terrain,
            RenderLayer::Starfield,
            RenderLayer::Objects,
            RenderLayer::Projectiles,
            RenderLayer::Hud,
            RenderLayer::Overlay,
        ]
        .into_iter()
        .enumerate()
        {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId(index as u16),
                layer,
                position: [index as f32, 0.0],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }

        assert_eq!(
            scene.summary().layers,
            RenderLayerCounts {
                terrain: 1,
                starfield: 1,
                objects: 1,
                projectiles: 1,
                hud: 1,
                overlay: 1,
            }
        );
    }

    #[test]
    fn renderer_settings_are_wgpu_native() {
        let settings = GpuRendererSettings::default();

        assert_eq!(settings.texture_format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(settings.present_mode, wgpu::PresentMode::AutoVsync);
    }

    #[test]
    fn native_scene_renderer_uses_settings_for_sprite_pipeline_plan() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };
        let renderer =
            NativeSceneRenderer::with_settings(NativeRendererResources::default(), settings);
        let mut scene = RenderScene::empty(31, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = renderer.prepare(&scene);

        assert_eq!(
            plan.sprite_pipeline,
            Some(SpritePipelinePlan::for_settings(settings))
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(
            plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&plan)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&plan)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&plan)
        );
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(
            plan.sprite_pipeline_layout
                .as_ref()
                .map(SpritePipelineLayoutPlan::bind_group_count),
            Some(2)
        );
        assert_eq!(
            plan.sprite_pipeline
                .as_ref()
                .map(|pipeline| pipeline.color_target.format),
            Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        );
        assert_eq!(
            plan.sprite_render_pipeline_descriptor
                .as_ref()
                .map(|descriptor| descriptor.color_target.format),
            Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder
                .as_ref()
                .map(SpriteRenderPassEncoderPlan::draw_count),
            Some(1)
        );
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
    }

    #[test]
    fn texture_atlas_owns_sprite_regions() {
        let atlas = TextureAtlas::new(
            SurfaceSize::new(16, 16),
            vec![AtlasRegion {
                sprite: SpriteId(42),
                origin: [1, 2],
                size: [3, 4],
            }],
        );

        assert!(atlas.contains(SpriteId(42)));
        assert!(!atlas.contains(SpriteId::PLAYER_SHIP));
        assert_eq!(atlas.pixels().len(), 16 * 16 * 4);
        assert!(!atlas.is_non_blank());
        assert_eq!(
            atlas.region(SpriteId(42)),
            Some(AtlasRegion {
                sprite: SpriteId(42),
                origin: [1, 2],
                size: [3, 4],
            })
        );
        assert_eq!(atlas.region(SpriteId::PLAYER_SHIP), None);
        let default_atlas = TextureAtlas::default_sprites();
        assert_eq!(default_atlas.pixels().len(), 128 * 192 * 4);
        assert!(default_atlas.is_non_blank());
        assert!(default_atlas.contains(SpriteId::STATUS_TEXT));
        assert!(default_atlas.contains(SpriteId::PLAYER_PROJECTILE));
        assert!(default_atlas.contains(SpriteId::TERRAIN_TILE));
        assert!(default_atlas.contains(SpriteId::TERRAIN_TILE_ALT));
        assert!(default_atlas.contains(SpriteId::STAR));
        assert!(default_atlas.contains(SpriteId::ENEMY_LANDER));
        assert!(default_atlas.contains(SpriteId::HUMAN));
        assert!(default_atlas.contains(SpriteId::ENEMY_MUTANT));
        assert!(default_atlas.contains(SpriteId::ENEMY_BAITER));
        assert!(default_atlas.contains(SpriteId::ENEMY_BOMBER));
        assert!(default_atlas.contains(SpriteId::ENEMY_POD));
        assert!(default_atlas.contains(SpriteId::ENEMY_SWARMER));
        assert!(default_atlas.contains(SpriteId::ENEMY_BOMB));
        assert!(default_atlas.contains(SpriteId::BOMB_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::SWARMER_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::SCORE_POPUP_250));
        assert!(default_atlas.contains(SpriteId::SCORE_POPUP_500));
        assert!(default_atlas.contains(SpriteId::PLAYER_LIFE_STOCK));
        assert!(default_atlas.contains(SpriteId::SMART_BOMB_STOCK));
        assert!(default_atlas.contains(SpriteId::ASTRONAUT_EXPLOSION));
        assert!(default_atlas.contains(SpriteId::NULL_OBJECT));
        assert!(default_atlas.contains(SpriteId::TERRAIN_EXPLOSION));
        for sprite in SpriteId::SCORE_DIGITS {
            assert!(default_atlas.contains(sprite));
        }
        assert!(default_atlas.contains(SpriteId::HALL_OF_FAME_UNDERLINE_WORD));
        assert!(default_atlas.contains(SpriteId::HALL_OF_FAME_DEFENDER_LOGO));
        assert!(default_atlas.contains(SpriteId::ATTRACT_COPYRIGHT_STRIP));
        assert!(default_atlas.contains(SpriteId::TOP_DISPLAY_BORDER_WORD));
        assert!(default_atlas.contains(SpriteId::SCANNER_OBJECT_BLIP));
        assert!(default_atlas.contains(SpriteId::SCANNER_PLAYER_BLIP));
        assert!(default_atlas.contains(SpriteId::PLAYER_EXPLOSION_PIXEL));
        assert_eq!(
            TextureAtlas::with_rgba(SurfaceSize::new(2, 2), Vec::new(), vec![0; 15]),
            Err(SceneRasterError::PixelBufferLength {
                expected: 16,
                actual: 15,
            })
        );
        assert_eq!(
            TextureAtlas::with_rgba(SurfaceSize::new(u32::MAX, u32::MAX), Vec::new(), Vec::new()),
            Err(SceneRasterError::PixelBufferTooLarge {
                surface: SurfaceSize::new(u32::MAX, u32::MAX),
            })
        );
    }

    #[test]
    fn default_sprite_atlas_uses_embedded_runtime_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_non_placeholder_region(&atlas, SpriteId::PLAYER_SHIP);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_TEXT);
        assert_non_placeholder_region(&atlas, SpriteId::STATUS_TEXT);
        assert_visible_region(&atlas, SpriteId::PLAYER_PROJECTILE);
        assert_non_placeholder_region(&atlas, SpriteId::TERRAIN_TILE);
        assert_non_placeholder_region(&atlas, SpriteId::TERRAIN_TILE_ALT);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_LANDER);
        assert_non_placeholder_region(&atlas, SpriteId::HUMAN);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_MUTANT);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BAITER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BOMBER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_POD);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_SWARMER);
        assert_non_placeholder_region(&atlas, SpriteId::ENEMY_BOMB);
        assert_non_placeholder_region(&atlas, SpriteId::BOMB_EXPLOSION);
        assert_non_placeholder_region(&atlas, SpriteId::SWARMER_EXPLOSION);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_POPUP_250);
        assert_non_placeholder_region(&atlas, SpriteId::SCORE_POPUP_500);
        assert_non_placeholder_region(&atlas, SpriteId::PLAYER_LIFE_STOCK);
        assert_non_placeholder_region(&atlas, SpriteId::SMART_BOMB_STOCK);
        assert_visible_region(&atlas, SpriteId::STAR);
    }

    #[test]
    fn default_sprite_atlas_decodes_arcade_terrain_word_patterns() {
        let atlas = TextureAtlas::default_sprites();
        let terrain_7007 = atlas_region_pixels(&atlas, SpriteId::TERRAIN_TILE);
        let terrain_0770 = atlas_region_pixels(&atlas, SpriteId::TERRAIN_TILE_ALT);

        assert_eq!(
            atlas.region(SpriteId::TERRAIN_TILE).expect("terrain").size,
            [2, 2]
        );
        assert_eq!(
            atlas
                .region(SpriteId::TERRAIN_TILE_ALT)
                .expect("terrain alt")
                .size,
            [2, 2]
        );
        assert_eq!(
            atlas_region_alpha_rows(&atlas, SpriteId::TERRAIN_TILE),
            vec!["#.", ".#"]
        );
        assert_eq!(
            atlas_region_alpha_rows(&atlas, SpriteId::TERRAIN_TILE_ALT),
            vec![".#", "#."]
        );
        assert_eq!(terrain_7007[0], pseudo_color_rgba(PICTURE_COLOR_TABLE[7]));
        assert_eq!(terrain_0770[1], pseudo_color_rgba(PICTURE_COLOR_TABLE[7]));
    }

    #[test]
    fn default_sprite_atlas_regions_match_object_bitmap_sizes() {
        let atlas = TextureAtlas::default_sprites();

        for (sprite, size) in [
            (SpriteId::PLAYER_SHIP, [16, 6]),
            (SpriteId::PLAYER_SHIP_LEFT, [16, 6]),
            (SpriteId::PLAYER_PROJECTILE, [16, 1]),
            (SpriteId::ENEMY_LANDER, [10, 8]),
            (SpriteId::HUMAN, [4, 8]),
            (SpriteId::ENEMY_MUTANT, [10, 8]),
            (SpriteId::ENEMY_BAITER, [12, 4]),
            (SpriteId::ENEMY_BOMBER, [8, 8]),
            (SpriteId::ENEMY_POD, [8, 8]),
            (SpriteId::ENEMY_SWARMER, [6, 4]),
            (SpriteId::ENEMY_BOMB, [4, 3]),
            (SpriteId::BOMB_EXPLOSION, [8, 8]),
            (SpriteId::SWARMER_EXPLOSION, [8, 8]),
            (SpriteId::SCORE_POPUP_250, [12, 6]),
            (SpriteId::SCORE_POPUP_500, [12, 6]),
            (SpriteId::PLAYER_LIFE_STOCK, [10, 4]),
            (SpriteId::SMART_BOMB_STOCK, [6, 3]),
            (SpriteId::ASTRONAUT_EXPLOSION, [8, 8]),
            (SpriteId::TERRAIN_EXPLOSION, [16, 6]),
        ] {
            assert_eq!(atlas.region(sprite).expect("atlas region").size, size);
        }
    }

    #[test]
    fn object_bitmaps_decode_arcade_bytes_and_palettes() {
        let ship = decode_object_picture_asset_rgba(
            ObjectBitmapId::PlayerShipRightPrimary,
            6,
            8,
            ObjectPicturePalette::ship(),
        );
        let ship_left = decode_object_picture_asset_rgba(
            ObjectBitmapId::PlayerShipLeftPrimary,
            6,
            8,
            ObjectPicturePalette::ship(),
        );
        let shot = decode_object_picture_asset_rgba(
            ObjectBitmapId::PlayerLaser,
            1,
            8,
            ObjectPicturePalette::player_shot(),
        );
        let human = decode_object_picture_asset_rgba(
            ObjectBitmapId::HumanStandingPrimary,
            8,
            2,
            ObjectPicturePalette::white(),
        );

        assert_eq!(ship.surface, SurfaceSize::new(16, 6));
        assert_eq!(
            sprite_alpha_rows(&ship),
            vec![
                "..##............",
                ".####...........",
                "######..........",
                ".###########....",
                "###############.",
                "..######........",
            ]
        );
        assert_eq!(ship_left.surface, SurfaceSize::new(16, 6));
        assert_eq!(
            sprite_alpha_rows(&ship_left),
            vec![
                "...........##...",
                "..........####..",
                ".........######.",
                "...###########..",
                "###############.",
                ".......######...",
            ]
        );
        assert_ne!(sprite_alpha_rows(&ship), sprite_alpha_rows(&ship_left));
        assert!(
            ship.pixels
                .chunks_exact(4)
                .any(|pixel| pixel == PURPLE_RGBA.as_slice())
        );
        assert_eq!(shot.surface, SurfaceSize::new(16, 1));
        assert!(
            shot.pixels
                .chunks_exact(4)
                .all(|pixel| pixel == PALE_YELLOW_RGBA.as_slice())
        );
        assert_eq!(human.surface, SurfaceSize::new(4, 8));
        assert_eq!(
            sprite_alpha_rows(&human),
            vec![
                "##..", "##..", "###.", "###.", "###.", ".#..", ".#..", ".#.."
            ]
        );
        assert!(human.pixels.chunks_exact(4).any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn default_sprite_atlas_uses_object_picture_grid_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_visible_region(&atlas, SpriteId::ASTRONAUT_EXPLOSION);
        assert_transparent_region(&atlas, SpriteId::NULL_OBJECT);
        assert_visible_region(&atlas, SpriteId::TERRAIN_EXPLOSION);
    }

    #[test]
    fn default_sprite_atlas_uses_score_digit_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(SpriteId::score_digit(0), Some(SpriteId::SCORE_DIGIT_0));
        assert_eq!(SpriteId::score_digit(9), Some(SpriteId::SCORE_DIGIT_9));
        assert_eq!(SpriteId::score_digit(10), None);
        for (index, sprite) in SpriteId::SCORE_DIGITS.iter().enumerate() {
            assert_eq!(
                atlas.region(*sprite),
                Some(AtlasRegion {
                    sprite: *sprite,
                    origin: [u32::try_from(index).expect("digit index fits") * 8, 112],
                    size: [6, 8],
                })
            );
            assert_visible_region(&atlas, *sprite);
        }
    }

    #[test]
    fn default_sprite_atlas_decodes_score_digits_in_byte_column_order() {
        let atlas = TextureAtlas::default_sprites();
        let pixels = atlas_region_pixels(&atlas, SpriteId::SCORE_DIGIT_0);
        let is_visible = |x: usize, y: usize| pixels[y * 6 + x][3] != 0;

        assert_eq!(pixels[1], WHITE_RGBA);
        assert_eq!(
            (0..6).map(|x| is_visible(x, 0)).collect::<Vec<_>>(),
            vec![false, true, true, true, true, true]
        );
        assert_eq!(
            (0..6).map(|x| is_visible(x, 1)).collect::<Vec<_>>(),
            vec![false, true, false, false, true, true]
        );
        assert_eq!(
            (0..6).map(|x| is_visible(x, 6)).collect::<Vec<_>>(),
            vec![false, true, true, true, true, true]
        );
        assert!((0..6).all(|x| !is_visible(x, 7)));
    }

    #[test]
    fn default_sprite_atlas_uses_high_score_underline_word_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::HALL_OF_FAME_UNDERLINE_WORD),
            Some(AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_UNDERLINE_WORD,
                origin: [80, 112],
                size: [2, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::HALL_OF_FAME_UNDERLINE_WORD);
    }

    #[test]
    fn default_sprite_atlas_uses_hall_of_fame_defender_logo_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::HALL_OF_FAME_DEFENDER_LOGO),
            Some(AtlasRegion {
                sprite: SpriteId::HALL_OF_FAME_DEFENDER_LOGO,
                origin: [0, 128],
                size: [120, 24],
            })
        );
        assert_visible_region(&atlas, SpriteId::HALL_OF_FAME_DEFENDER_LOGO);
    }

    #[test]
    fn default_sprite_atlas_uses_defender_wordmark_block_regions() {
        let atlas = TextureAtlas::default_sprites();
        let first = SpriteId::attract_defender_wordmark_block(0).expect("first block sprite");
        let center = SpriteId::attract_defender_wordmark_block(7).expect("center block sprite");
        let last = SpriteId::attract_defender_wordmark_block(
            ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT.saturating_sub(1),
        )
        .expect("last block sprite");

        assert_eq!(
            atlas.region(first),
            Some(AtlasRegion {
                sprite: first,
                origin: [0, 128],
                size: [8, 24],
            })
        );
        assert_eq!(
            atlas.region(center),
            Some(AtlasRegion {
                sprite: center,
                origin: [56, 128],
                size: [8, 24],
            })
        );
        assert_eq!(
            atlas.region(last),
            Some(AtlasRegion {
                sprite: last,
                origin: [112, 128],
                size: [8, 24],
            })
        );
        assert_visible_region(&atlas, first);
        assert_visible_region(&atlas, center);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_copyright_strip_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_COPYRIGHT_STRIP),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_COPYRIGHT_STRIP,
                origin: [0, 152],
                size: [80, 8],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_COPYRIGHT_STRIP);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_williams_logo_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_WILLIAMS_LOGO),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO,
                origin: [0, 160],
                size: [92, 19],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_WILLIAMS_LOGO);
    }

    #[test]
    fn default_sprite_atlas_uses_top_display_border_word_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::TOP_DISPLAY_BORDER_WORD),
            Some(AtlasRegion {
                sprite: SpriteId::TOP_DISPLAY_BORDER_WORD,
                origin: [96, 160],
                size: [2, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::TOP_DISPLAY_BORDER_WORD);
    }

    #[test]
    fn default_sprite_atlas_uses_scanner_blip_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::SCANNER_OBJECT_BLIP),
            Some(AtlasRegion {
                sprite: SpriteId::SCANNER_OBJECT_BLIP,
                origin: [100, 160],
                size: [2, 2],
            })
        );
        assert_eq!(
            atlas.region(SpriteId::SCANNER_PLAYER_BLIP),
            Some(AtlasRegion {
                sprite: SpriteId::SCANNER_PLAYER_BLIP,
                origin: [104, 160],
                size: [3, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::SCANNER_OBJECT_BLIP);
        assert_visible_region(&atlas, SpriteId::SCANNER_PLAYER_BLIP);
    }

    #[test]
    fn default_sprite_atlas_uses_player_explosion_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::PLAYER_EXPLOSION_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::PLAYER_EXPLOSION_PIXEL,
                origin: [108, 160],
                size: [4, 2],
            })
        );
        assert_visible_region(&atlas, SpriteId::PLAYER_EXPLOSION_PIXEL);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_williams_logo_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL,
                origin: [112, 160],
                size: [1, 1],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_WILLIAMS_LOGO_PIXEL);
    }

    #[test]
    fn default_sprite_atlas_uses_attract_scanner_terrain_pixel_region() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(
            atlas.region(SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL),
            Some(AtlasRegion {
                sprite: SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL,
                origin: [114, 160],
                size: [1, 1],
            })
        );
        assert_visible_region(&atlas, SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL);
    }

    #[test]
    fn attract_williams_logo_decodes_table_pixels() {
        let sprite = decode_attract_williams_logo_rgba();
        let path = attract_williams_logo_pixel_path();
        let operation_counts = attract_williams_logo_operation_pixel_counts();

        assert_eq!(sprite.surface, SurfaceSize::new(92, 19));
        assert_eq!(path.len(), 660);
        assert!(!operation_counts.is_empty());
        assert_eq!(operation_counts.last().copied(), Some(path.len()));
        assert!(
            operation_counts
                .windows(2)
                .all(|window| window[0] <= window[1])
        );
        assert!(
            operation_counts
                .get(2)
                .is_some_and(|count| *count < path.len())
        );
        assert_eq!(path.first().copied(), Some([8, 4]));
        assert!(path.contains(&[8, 4]));
        assert!(path.contains(&[89, 9]));
        assert_eq!(
            sprite
                .pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[3] != 0)
                .count(),
            660
        );
        assert_eq!(embedded_sprite_pixel(&sprite, 8, 4), WHITE_RGBA);
        assert_eq!(embedded_sprite_pixel(&sprite, 89, 9), WHITE_RGBA);
        assert_eq!(embedded_sprite_pixel(&sprite, 0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn default_sprite_atlas_uses_message_glyph_regions() {
        let atlas = TextureAtlas::default_sprites();

        assert_eq!(message_text(MessageId::PlayerOne), "PLAYER ONE");
        assert_eq!(message_text(MessageId::PlayerTwo), "PLAYER TWO");
        assert_eq!(message_text(MessageId::GameOver), "GAME OVER");
        assert_eq!(screen_position_from_address(0x3C78), [120.0, 120.0]);
        assert_eq!(
            screen_position_from_address_with_offset(0x1458, 0, 0x0A),
            [40.0, 98.0]
        );
        assert_eq!(
            SpriteId::message_glyph('P'),
            Some(SpriteId::MESSAGE_GLYPH_P)
        );
        assert_eq!(
            SpriteId::message_glyph('W'),
            Some(SpriteId::MESSAGE_GLYPH_W)
        );
        assert_eq!(SpriteId::message_glyph('a'), None);
        assert_eq!(SpriteId::message_glyph_size('P'), Some([6, 8]));
        assert_eq!(SpriteId::message_glyph_size('W'), Some([8, 8]));

        for sprite in SpriteId::MESSAGE_GLYPHS {
            assert!(atlas.contains(sprite));
        }
        assert_transparent_region(&atlas, SpriteId::MESSAGE_GLYPH_SPACE);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_P);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_G);
        assert_visible_region(&atlas, SpriteId::MESSAGE_GLYPH_W);
    }

    #[test]
    fn message_text_bytes_render_mixed_score_digits_and_message_glyphs() {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));

        push_message_text_bytes_sprites(
            &mut scene,
            b" 2A",
            screen_position_from_address(0x2B86),
            RenderLayer::Overlay,
        );

        assert_eq!(scene.sprites.len(), 2);
        assert_eq!(scene.sprites[0].sprite, SpriteId::SCORE_DIGIT_2);
        assert_eq!(scene.sprites[0].position, [90.0, 134.0]);
        assert_eq!(scene.sprites[0].size, [6.0, 8.0]);
        assert_eq!(scene.sprites[1].sprite, SpriteId::MESSAGE_GLYPH_A);
        assert_eq!(scene.sprites[1].position, [98.0, 134.0]);
        assert_eq!(scene.sprites[1].size, [6.0, 8.0]);
        assert!(
            scene
                .sprites
                .iter()
                .all(|sprite| sprite.layer == RenderLayer::Overlay)
        );
    }

    #[test]
    fn arcade_controlled_message_sprites_apply_cursor_controls() {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));
        let text = message_text(MessageId::WilliamsElectronics);

        push_arcade_controlled_message_sprites(&mut scene, text, 0x3258, RenderLayer::Overlay);

        assert_eq!(scene.sprites.len(), 23);
        assert!(scene.sprites.iter().all(|sprite| {
            sprite.layer == RenderLayer::Overlay
                && SpriteId::MESSAGE_GLYPHS.contains(&sprite.sprite)
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_E
                && sprite.position == [100.0, 88.0]
                && sprite.size == [6.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_I
                && sprite.position == [190.0, 88.0]
                && sprite.size == [4.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_PERIOD
                && sprite.position == [212.0, 88.0]
                && sprite.size == [2.0, 8.0]
        }));
        assert!(scene.sprites.iter().any(|sprite| {
            sprite.sprite == SpriteId::MESSAGE_GLYPH_P
                && sprite.position == [124.0, 108.0]
                && sprite.size == [6.0, 8.0]
        }));
    }

    #[test]
    fn legacy_object_bitmap_labels_map_reclassified_clean_sprite_assets() {
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("PLAPIC"),
            Some(SpriteId::PLAYER_SHIP)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("PLBPIC"),
            Some(SpriteId::PLAYER_SHIP_LEFT)
        );
        for label in ["LNDP1", "LNDP2", "LNDP3"] {
            assert_eq!(
                SpriteId::for_legacy_object_bitmap_label(label),
                Some(SpriteId::ENEMY_LANDER)
            );
        }
        for label in ["ASTP1", "ASTP2", "ASTP3", "ASTP4"] {
            assert_eq!(
                SpriteId::for_legacy_object_bitmap_label(label),
                Some(SpriteId::HUMAN)
            );
        }
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("LASP1"),
            Some(SpriteId::PLAYER_PROJECTILE)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("SCZP1"),
            Some(SpriteId::ENEMY_MUTANT)
        );
        for label in ["UFOP1", "UFOP2", "UFOP3"] {
            assert_eq!(
                SpriteId::for_legacy_object_bitmap_label(label),
                Some(SpriteId::ENEMY_BAITER)
            );
        }
        for label in ["TIEP1", "TIEP2", "TIEP3", "TIEP4"] {
            assert_eq!(
                SpriteId::for_legacy_object_bitmap_label(label),
                Some(SpriteId::ENEMY_BOMBER)
            );
        }
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("PRBP1"),
            Some(SpriteId::ENEMY_POD)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("SWPIC1"),
            Some(SpriteId::ENEMY_SWARMER)
        );
        for label in ["BMBP1", "BMBP2"] {
            assert_eq!(
                SpriteId::for_legacy_object_bitmap_label(label),
                Some(SpriteId::ENEMY_BOMB)
            );
        }
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("BXPIC"),
            Some(SpriteId::BOMB_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("SWXP1"),
            Some(SpriteId::SWARMER_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("C25P1"),
            Some(SpriteId::SCORE_POPUP_250)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("C5P1"),
            Some(SpriteId::SCORE_POPUP_500)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("PLAMIN"),
            Some(SpriteId::PLAYER_LIFE_STOCK)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("SBPIC"),
            Some(SpriteId::SMART_BOMB_STOCK)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("ASXP1"),
            Some(SpriteId::ASTRONAUT_EXPLOSION)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("NULOB"),
            Some(SpriteId::NULL_OBJECT)
        );
        assert_eq!(
            SpriteId::for_legacy_object_bitmap_label("TEREX"),
            Some(SpriteId::TERRAIN_EXPLOSION)
        );
        assert_eq!(SpriteId::for_legacy_object_bitmap_label("UNKNOWN"), None);
    }
