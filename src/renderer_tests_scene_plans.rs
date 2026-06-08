    #[test]
    fn embedded_png_decoder_rejects_truncated_or_unsupported_assets() {
        let rgba_png = encode_test_png(png::ColorType::Rgba, &[255, 0, 0, 255]);
        let truncated = truncate_first_idat_payload(rgba_png);

        assert_panic_message(
            || {
                let _ = super::decode_embedded_png_rgba("truncated-test", &truncated);
            },
            "must contain a frame",
        );

        let rgb_png = encode_test_png(png::ColorType::Rgb, &[255, 0, 0]);
        assert_panic_message(
            || {
                let _ = super::decode_embedded_png_rgba("rgb-test", &rgb_png);
            },
            "must be 8-bit RGBA",
        );
    }

    #[test]
    fn default_sprite_blitters_skip_missing_or_empty_sources() {
        let atlas_surface = SurfaceSize::new(2, 2);
        let source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 255],
        };
        let mut atlas_pixels = vec![0; atlas_surface.rgba_len().expect("atlas byte length")];
        let unchanged = atlas_pixels.clone();

        super::blit_default_region_from_source(
            &mut atlas_pixels,
            atlas_surface,
            &[],
            SpriteId::PLAYER_SHIP,
            &source,
            super::SpriteAssetSource {
                origin: [0, 0],
                size: [1, 1],
            },
        );
        assert_eq!(atlas_pixels, unchanged);

        super::blit_scaled_region(
            &mut atlas_pixels,
            atlas_surface,
            AtlasRegion {
                sprite: SpriteId::PLAYER_SHIP,
                origin: [0, 0],
                size: [0, 1],
            },
            &source,
            super::SpriteAssetSource {
                origin: [0, 0],
                size: [1, 1],
            },
        );
        assert_eq!(atlas_pixels, unchanged);

        super::copy_source_pixel(&mut atlas_pixels, atlas_surface, [0, 0], &source, [1, 0]);
        assert_eq!(atlas_pixels, unchanged);
    }

    #[test]
    fn star_blitter_skips_missing_region_or_transparent_source() {
        let atlas_surface = SurfaceSize::new(2, 2);
        let source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 255],
        };
        let mut atlas_pixels = vec![0; atlas_surface.rgba_len().expect("atlas byte length")];
        let unchanged = atlas_pixels.clone();

        super::blit_star_region(&mut atlas_pixels, atlas_surface, &[], &source);
        assert_eq!(atlas_pixels, unchanged);

        let transparent_source = super::EmbeddedSprite {
            surface: SurfaceSize::new(1, 1),
            pixels: vec![10, 20, 30, 0],
        };
        super::blit_star_region(
            &mut atlas_pixels,
            atlas_surface,
            &[AtlasRegion {
                sprite: SpriteId::STAR,
                origin: [0, 0],
                size: [1, 1],
            }],
            &transparent_source,
        );
        assert_eq!(atlas_pixels, unchanged);
    }

    fn encode_test_png(color_type: png::ColorType, pixels: &[u8]) -> Vec<u8> {
        let mut png_bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, 1, 1);
            encoder.set_color(color_type);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("write png header");
            writer.write_image_data(pixels).expect("write png data");
        }
        png_bytes
    }

    fn truncate_first_idat_payload(mut png_bytes: Vec<u8>) -> Vec<u8> {
        let mut offset = 8;
        while offset + 8 <= png_bytes.len() {
            let length = u32::from_be_bytes(
                png_bytes[offset..offset + 4]
                    .try_into()
                    .expect("png chunk length"),
            ) as usize;
            let kind = &png_bytes[offset + 4..offset + 8];
            if kind == b"IDAT" {
                assert!(length > 1);
                png_bytes.truncate(offset + 8 + length - 1);
                return png_bytes;
            }
            offset += 12 + length;
        }
        panic!("test png must contain IDAT");
    }

    fn assert_panic_message(run: impl FnOnce() + std::panic::UnwindSafe, expected: &str) {
        let panic = std::panic::catch_unwind(run).expect_err("expected panic");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&'static str>().copied())
            .expect("panic message");
        assert!(
            message.contains(expected),
            "panic message {message:?} must contain {expected:?}"
        );
    }

    fn assert_non_placeholder_region(atlas: &TextureAtlas, sprite: SpriteId) {
        let pixels = atlas_region_pixels(atlas, sprite);
        assert!(
            pixels.iter().any(|pixel| pixel[3] != 0),
            "sprite {:?} region must contain visible pixels",
            sprite
        );
        let distinct_pixels = pixels
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            distinct_pixels.len() > 1,
            "sprite {:?} region must not be one solid placeholder color",
            sprite
        );
    }

    fn assert_visible_region(atlas: &TextureAtlas, sprite: SpriteId) {
        assert!(
            atlas_region_pixels(atlas, sprite)
                .iter()
                .any(|pixel| pixel[3] != 0),
            "sprite {:?} region must contain visible pixels",
            sprite
        );
    }

    fn assert_transparent_region(atlas: &TextureAtlas, sprite: SpriteId) {
        assert!(
            atlas_region_pixels(atlas, sprite)
                .iter()
                .all(|pixel| pixel[3] == 0),
            "sprite {:?} region must stay transparent",
            sprite
        );
    }

    fn source_sprite_pixel(sprite: &EmbeddedSprite, x: u32, y: u32) -> [u8; 4] {
        let start = ((y as usize * sprite.surface.width as usize) + x as usize) * 4;
        let pixel = &sprite.pixels[start..start + 4];
        [pixel[0], pixel[1], pixel[2], pixel[3]]
    }

    fn sprite_alpha_rows(sprite: &EmbeddedSprite) -> Vec<String> {
        (0..sprite.surface.height)
            .map(|y| {
                (0..sprite.surface.width)
                    .map(|x| {
                        if source_sprite_pixel(sprite, x, y)[3] == 0 {
                            '.'
                        } else {
                            '#'
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn atlas_region_alpha_rows(atlas: &TextureAtlas, sprite: SpriteId) -> Vec<String> {
        let region = atlas.region(sprite).expect("sprite region");
        let pixels = atlas_region_pixels(atlas, sprite);
        (0..region.size[1])
            .map(|y| {
                (0..region.size[0])
                    .map(|x| {
                        let index = (y * region.size[0] + x) as usize;
                        if pixels[index][3] == 0 { '.' } else { '#' }
                    })
                    .collect()
            })
            .collect()
    }

    fn atlas_region_pixels(atlas: &TextureAtlas, sprite: SpriteId) -> Vec<[u8; 4]> {
        let region = atlas.region(sprite).expect("sprite region");
        let mut pixels = Vec::new();
        for y in region.origin[1]..region.origin[1] + region.size[1] {
            for x in region.origin[0]..region.origin[0] + region.size[0] {
                let start = ((y as usize * atlas.surface.width as usize) + x as usize) * 4;
                let pixel = &atlas.pixels()[start..start + 4];
                pixels.push([pixel[0], pixel[1], pixel[2], pixel[3]]);
            }
        }
        pixels
    }

    #[test]
    fn native_scene_renderer_builds_draw_plan_from_scene_layers() {
        let mut scene = RenderScene::empty(34, SurfaceSize::new(292, 240));
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

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.frame, 34);
        assert_eq!(
            plan.viewport,
            ViewportLayout {
                scene: SurfaceSize::new(292, 240),
                target: SurfaceSize::new(292, 240),
                origin: [0, 0],
                size: SurfaceSize::new(292, 240),
                scale: 1.0,
            }
        );
        assert_eq!(
            plan.gpu_pass,
            WgpuPassPlan {
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
                viewport: Some(WgpuViewportCommand {
                    x: 0.0,
                    y: 0.0,
                    width: 292.0,
                    height: 240.0,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }),
                scene_projection: SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)),
            }
        );
        assert_eq!(plan.sprite_instances, 2);
        assert_eq!(plan.missing_sprite_regions, 0);
        assert_eq!(
            plan.pipelines,
            vec![NativeRenderPipeline::Sprites, NativeRenderPipeline::HudText]
        );
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                hud: 1,
                ..RenderLayerCounts::default()
            }
        );
        assert_eq!(
            plan.sprite_batches,
            vec![
                SpriteDrawBatch {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    instances: vec![SpriteDrawInstance {
                        sprite: SpriteId::PLAYER_SHIP,
                        atlas_origin: [0, 0],
                        atlas_size: [16, 6],
                        layer: RenderLayer::Objects,
                        position: [128.0, 96.0],
                        size: [16.0, 8.0],
                        tint: Color::WHITE,
                    }],
                },
                SpriteDrawBatch {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    instances: vec![SpriteDrawInstance {
                        sprite: SpriteId::SCORE_TEXT,
                        atlas_origin: [0, 16],
                        atlas_size: [80, 8],
                        layer: RenderLayer::Hud,
                        position: [0.0, 0.0],
                        size: [80.0, 8.0],
                        tint: Color::WHITE,
                    }],
                },
            ]
        );
        assert_eq!(
            plan.sprite_instance_buffers,
            vec![
                SpriteInstanceBuffer {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    records: vec![SpriteInstanceBufferRecord {
                        scene_origin: [128.0, 96.0],
                        scene_size: [16.0, 8.0],
                        atlas_uv_origin: [0.0, 0.0],
                        atlas_uv_size: [0.125, 0.03125],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    }],
                },
                SpriteInstanceBuffer {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    records: vec![SpriteInstanceBufferRecord {
                        scene_origin: [0.0, 0.0],
                        scene_size: [80.0, 8.0],
                        atlas_uv_origin: [0.0, 0.083333336],
                        atlas_uv_size: [0.625, 0.041666668],
                        tint: [1.0, 1.0, 1.0, 1.0],
                    }],
                },
            ]
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan
                    .sprite_instance_buffers
                    .iter()
                    .flat_map(|buffer| buffer.records.iter().copied())
                    .collect(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![
                expected_sprite_draw_command(
                    NativeRenderPipeline::Sprites,
                    RenderLayer::Objects,
                    0,
                    1,
                ),
                expected_sprite_draw_command(NativeRenderPipeline::HudText, RenderLayer::Hud, 1, 1),
            ]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
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
            plan.sprite_render_pipeline_descriptor
                .as_ref()
                .map(|descriptor| descriptor.vertex_buffer_layouts().len()),
            Some(2)
        );
        assert_eq!(
            plan.sprite_render_pass_encoder
                .as_ref()
                .map(SpriteRenderPassEncoderPlan::command_count),
            Some(8)
        );
        assert_eq!(plan.frame_plan.command_count(), 4);
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 0);
        assert_eq!(plan.raster_upload, None);
    }

    #[test]
    fn native_scene_renderer_respects_available_resource_pipelines() {
        let mut resources = NativeRendererResources::default();
        resources
            .pipelines
            .remove(&NativeRenderPipeline::TemporaryRaster);
        let renderer = NativeSceneRenderer::new(resources);
        let scene = RenderScene::from_rgba(1, SurfaceSize::new(1, 1), vec![0, 0, 0, 255], None)
            .expect("raster scene");

        let plan = renderer.prepare(&scene);

        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert!(plan.raster_upload.is_some());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 1);
    }

    #[test]
    fn native_scene_renderer_skips_sprite_commands_for_unavailable_sprite_pipelines() {
        let mut resources = NativeRendererResources::default();
        resources.pipelines.remove(&NativeRenderPipeline::Sprites);
        let renderer = NativeSceneRenderer::new(resources);
        let mut scene = RenderScene::empty(35, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = renderer.prepare(&scene);

        assert_eq!(plan.sprite_instances, 0);
        assert_eq!(plan.sprite_batches, Vec::<SpriteDrawBatch>::new());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn native_scene_renderer_maps_all_domain_layers_to_pipelines() {
        let mut scene = RenderScene::empty(80, SurfaceSize::new(292, 240));
        for (layer, sprite) in [
            (RenderLayer::Terrain, SpriteId::TERRAIN_TILE),
            (RenderLayer::Starfield, SpriteId::STAR),
            (RenderLayer::Projectiles, SpriteId::PLAYER_PROJECTILE),
        ] {
            scene.push_sprite(SceneSprite {
                sprite,
                layer,
                position: [0.0, 0.0],
                size: [1.0, 1.0],
                tint: Color::WHITE,
            });
        }

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(
            plan.pipelines,
            vec![
                NativeRenderPipeline::Terrain,
                NativeRenderPipeline::Starfield,
                NativeRenderPipeline::Projectiles
            ]
        );
    }

    #[test]
    fn native_scene_renderer_counts_missing_sprite_atlas_regions() {
        let mut scene = RenderScene::empty(81, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId(900),
            layer: RenderLayer::Objects,
            position: [12.0, 34.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.sprite_instances, 0);
        assert_eq!(plan.missing_sprite_regions, 1);
        assert_eq!(plan.sprite_batches, Vec::<SpriteDrawBatch>::new());
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(plan.pipelines, Vec::<NativeRenderPipeline>::new());
        assert_eq!(
            plan.layer_counts,
            RenderLayerCounts {
                objects: 1,
                ..RenderLayerCounts::default()
            }
        );
    }

    #[test]
    fn native_scene_renderer_batches_multiple_sprites_by_pipeline_and_layer() {
        let mut scene = RenderScene::empty(82, SurfaceSize::new(292, 240));
        for position in [[2.0, 4.0], [10.0, 4.0]] {
            scene.push_sprite(SceneSprite {
                sprite: SpriteId::PLAYER_PROJECTILE,
                layer: RenderLayer::Projectiles,
                position,
                size: [8.0, 2.0],
                tint: Color::WHITE,
            });
        }

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(plan.sprite_instances, 2);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_batches[0].pipeline,
            NativeRenderPipeline::Projectiles
        );
        assert_eq!(plan.sprite_batches[0].layer, RenderLayer::Projectiles);
        assert_eq!(plan.sprite_batches[0].instances.len(), 2);
        assert_eq!(plan.sprite_batches[0].instances[0].atlas_origin, [0, 48]);
        assert_eq!(plan.sprite_batches[0].instances[0].atlas_size, [16, 1]);
        assert_eq!(plan.sprite_batches[0].instances[1].position, [10.0, 4.0]);
        assert_eq!(plan.sprite_instance_buffers.len(), 1);
        assert_eq!(plan.sprite_instance_buffers[0].records.len(), 2);
        assert_eq!(
            plan.sprite_instance_buffers[0].records[1].scene_origin,
            [10.0, 4.0]
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::Projectiles,
                RenderLayer::Projectiles,
                0,
                2,
            )]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
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
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
    }

    #[test]
    fn native_scene_renderer_skips_instance_buffers_when_atlas_surface_is_empty() {
        let resources = NativeRendererResources {
            atlas: TextureAtlas::new(
                SurfaceSize::new(0, 128),
                vec![AtlasRegion {
                    sprite: SpriteId::PLAYER_SHIP,
                    origin: [0, 0],
                    size: [16, 8],
                }],
            ),
            ..NativeRendererResources::default()
        };
        let mut scene = RenderScene::empty(86, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::new(resources).prepare(&scene);

        assert_eq!(plan.sprite_instances, 1);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(plan.sprite_instance_upload, None);
        assert_eq!(plan.sprite_buffer_uploads, None);
        assert_eq!(plan.sprite_draw_commands, Vec::<SpriteDrawCommand>::new());
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
    }

    #[test]
    fn native_scene_renderer_uses_target_viewport_for_sprite_and_raster_plans() {
        let mut sprite_scene = RenderScene::empty(83, SurfaceSize::new(292, 240));
        sprite_scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
        let mut pixels = vec![0; 292 * 240 * 4];
        pixels[0] = 1;
        pixels[3] = 255;
        let raster_scene =
            RenderScene::from_rgba(84, SurfaceSize::new(292, 240), pixels, Some(0xCAFE_BABE))
                .expect("raster scene");
        let target = SurfaceSize::new(640, 480);
        let expected = ViewportLayout {
            scene: SurfaceSize::new(292, 240),
            target,
            origin: [28, 0],
            size: SurfaceSize::new(584, 480),
            scale: 2.0,
        };
        let renderer = NativeSceneRenderer::default();

        let sprite_plan = renderer.prepare_for_target(&sprite_scene, target);
        let raster_plan = renderer.prepare_for_target(&raster_scene, target);

        assert_eq!(sprite_plan.viewport, expected);
        assert_eq!(raster_plan.viewport, expected);
        assert_eq!(
            sprite_plan.gpu_pass.viewport,
            Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            })
        );
        assert_eq!(sprite_plan.gpu_pass, raster_plan.gpu_pass);
        assert_eq!(sprite_plan.sprite_instances, 1);
        assert_eq!(sprite_plan.sprite_instance_buffers.len(), 1);
        assert_eq!(
            sprite_plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: sprite_plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            sprite_plan.sprite_buffer_uploads,
            sprite_plan
                .sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            sprite_plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::Sprites,
                RenderLayer::Objects,
                0,
                1,
            )]
        );
        assert_eq!(
            sprite_plan.sprite_render_pass,
            expected_sprite_render_pass(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_pipeline,
            expected_sprite_pipeline(&sprite_plan, GpuRendererSettings::default())
        );
        assert_eq!(
            sprite_plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_pipeline_layout,
            expected_sprite_pipeline_layout(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_render_pipeline_descriptor,
            expected_sprite_render_pipeline_descriptor(&sprite_plan)
        );
        assert_eq!(
            sprite_plan.sprite_render_pass_encoder,
            expected_sprite_render_pass_encoder(&sprite_plan)
        );
        assert_eq!(sprite_plan.frame_plan, expected_frame_plan(&sprite_plan));
        assert_eq!(sprite_plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(
            raster_plan.sprite_instance_buffers,
            Vec::<SpriteInstanceBuffer>::new()
        );
        assert_eq!(raster_plan.sprite_instance_upload, None);
        assert_eq!(raster_plan.sprite_buffer_uploads, None);
        assert_eq!(
            raster_plan.sprite_draw_commands,
            Vec::<SpriteDrawCommand>::new()
        );
        assert_eq!(raster_plan.sprite_render_pass, None);
        assert_eq!(raster_plan.sprite_pipeline, None);
        assert_eq!(raster_plan.sprite_resource_bindings, None);
        assert_eq!(raster_plan.sprite_pipeline_layout, None);
        assert_eq!(raster_plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(raster_plan.sprite_render_pass_encoder, None);
        assert_eq!(raster_plan.frame_plan, expected_frame_plan(&raster_plan));
        assert_eq!(raster_plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(raster_plan.frame_plan.temporary_raster_count(), 1);
        assert_eq!(
            raster_plan.raster_upload,
            Some(super::SceneRasterUpload {
                surface: SurfaceSize::new(292, 240),
                byte_len: 292 * 240 * 4,
                visual_signature: Some(0xCAFE_BABE),
                non_blank: true,
            })
        );
    }

    #[test]
    fn native_scene_renderer_omits_gpu_viewport_for_empty_target() {
        let scene = RenderScene::empty(85, SurfaceSize::new(292, 240));

        let plan =
            NativeSceneRenderer::default().prepare_for_target(&scene, SurfaceSize::new(0, 0));

        assert!(plan.viewport.is_empty());
        assert_eq!(plan.gpu_pass.viewport, None);
        assert_eq!(plan.sprite_render_pass, None);
        assert_eq!(plan.sprite_pipeline, None);
        assert_eq!(plan.sprite_resource_bindings, None);
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
        assert_eq!(
            plan.gpu_pass.scene_projection,
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240))
        );
    }

    #[test]
    fn native_scene_renderer_omits_sprite_pipeline_layout_for_empty_target() {
        let mut scene = RenderScene::empty(87, SurfaceSize::new(292, 240));
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [128.0, 96.0],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });

        let plan =
            NativeSceneRenderer::default().prepare_for_target(&scene, SurfaceSize::new(0, 0));

        assert!(plan.viewport.is_empty());
        assert_eq!(plan.gpu_pass.viewport, None);
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
        );
        assert_eq!(
            plan.sprite_resource_bindings,
            expected_sprite_resource_bindings(&plan)
        );
        assert_eq!(plan.sprite_pipeline_layout, None);
        assert_eq!(plan.sprite_render_pipeline_descriptor, None);
        assert_eq!(plan.sprite_render_pass_encoder, None);
        assert_eq!(plan.frame_plan, expected_frame_plan(&plan));
        assert_eq!(plan.frame_plan.sprite_pass_count(), 0);
    }

    #[test]
    fn native_scene_renderer_keeps_raster_upload_separate_from_sprites() {
        let mut scene = RenderScene::from_rgba(
            55,
            SurfaceSize::new(1, 1),
            vec![1, 2, 3, 255],
            Some(0xFEED_FACE),
        )
        .expect("raster scene");
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::STATUS_TEXT,
            layer: RenderLayer::Overlay,
            position: [0.0, 0.0],
            size: [8.0, 8.0],
            tint: Color::WHITE,
        });

        let plan = NativeSceneRenderer::default().prepare(&scene);

        assert_eq!(
            plan.pipelines,
            vec![
                NativeRenderPipeline::TemporaryRaster,
                NativeRenderPipeline::DebugOverlay
            ]
        );
        assert_eq!(
            plan.raster_upload,
            Some(super::SceneRasterUpload {
                surface: SurfaceSize::new(1, 1),
                byte_len: 4,
                visual_signature: Some(0xFEED_FACE),
                non_blank: true,
            })
        );
        assert_eq!(plan.sprite_instances, 1);
        assert_eq!(plan.missing_sprite_regions, 0);
        assert_eq!(plan.sprite_batches.len(), 1);
        assert_eq!(
            plan.sprite_batches[0].pipeline,
            NativeRenderPipeline::DebugOverlay
        );
        assert_eq!(
            plan.sprite_batches[0].instances[0].sprite,
            SpriteId::STATUS_TEXT
        );
        assert_eq!(
            plan.sprite_instance_upload,
            Some(SpriteInstanceUpload {
                records: plan.sprite_instance_buffers[0].records.clone(),
            })
        );
        assert_eq!(
            plan.sprite_buffer_uploads,
            plan.sprite_instance_upload
                .as_ref()
                .map(SpriteBufferUploadPlan::from_instance_upload)
        );
        assert_eq!(
            plan.sprite_draw_commands,
            vec![expected_sprite_draw_command(
                NativeRenderPipeline::DebugOverlay,
                RenderLayer::Overlay,
                0,
                1,
            )]
        );
        assert_eq!(plan.sprite_render_pass, expected_sprite_render_pass(&plan));
        assert_eq!(
            plan.sprite_pipeline,
            expected_sprite_pipeline(&plan, GpuRendererSettings::default())
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
        assert_eq!(plan.frame_plan.sprite_pass_count(), 1);
        assert_eq!(plan.frame_plan.temporary_raster_count(), 1);
    }
