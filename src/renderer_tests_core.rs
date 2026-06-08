    #[test]
    fn surface_size_reports_empty_edges() {
        assert!(SurfaceSize::new(0, 240).is_empty());
        assert!(SurfaceSize::new(320, 0).is_empty());
        assert!(!SurfaceSize::new(320, 240).is_empty());
    }

    #[test]
    fn viewport_layout_preserves_scene_aspect_and_centers() {
        let scene = SurfaceSize::new(292, 240);

        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(640, 480)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(640, 480),
                origin: [28, 0],
                size: SurfaceSize::new(584, 480),
                scale: 2.0,
            }
        );
        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(800, 600)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(800, 600),
                origin: [35, 0],
                size: SurfaceSize::new(730, 600),
                scale: 2.5,
            }
        );
        assert_eq!(
            ViewportLayout::fit(scene, SurfaceSize::new(320, 240)),
            ViewportLayout {
                scene,
                target: SurfaceSize::new(320, 240),
                origin: [14, 0],
                size: SurfaceSize::new(292, 240),
                scale: 1.0,
            }
        );
    }

    #[test]
    fn viewport_layout_reports_empty_scene_or_target() {
        let empty_target =
            ViewportLayout::fit(SurfaceSize::new(292, 240), SurfaceSize::new(0, 480));
        let empty_scene = ViewportLayout::fit(SurfaceSize::new(0, 240), SurfaceSize::new(640, 480));

        assert_eq!(
            empty_target,
            ViewportLayout {
                scene: SurfaceSize::new(292, 240),
                target: SurfaceSize::new(0, 480),
                origin: [0, 0],
                size: SurfaceSize::new(0, 0),
                scale: 0.0,
            }
        );
        assert!(empty_target.is_empty());
        assert_eq!(
            empty_scene,
            ViewportLayout {
                scene: SurfaceSize::new(0, 240),
                target: SurfaceSize::new(640, 480),
                origin: [0, 0],
                size: SurfaceSize::new(0, 0),
                scale: 0.0,
            }
        );
        assert!(empty_scene.is_empty());
    }

    #[test]
    fn color_normalizes_to_wgpu_clear_color() {
        let color = Color {
            rgba: [128, 64, 255, 0],
        };

        assert_eq!(
            color.to_wgpu(),
            wgpu::Color {
                r: 128.0 / 255.0,
                g: 64.0 / 255.0,
                b: 1.0,
                a: 0.0,
            }
        );
        assert_eq!(
            color.to_normalized_rgba(),
            [128.0 / 255.0, 64.0 / 255.0, 1.0, 0.0]
        );
    }

    #[test]
    fn sprite_instance_buffer_record_normalizes_atlas_and_tint() {
        let record = SpriteInstanceBufferRecord::from_instance(
            SpriteDrawInstance {
                sprite: SpriteId::PLAYER_SHIP,
                atlas_origin: [16, 32],
                atlas_size: [8, 16],
                layer: RenderLayer::Objects,
                position: [12.0, 34.0],
                size: [16.0, 8.0],
                tint: Color {
                    rgba: [255, 128, 64, 32],
                },
            },
            SurfaceSize::new(128, 64),
        )
        .expect("instance buffer record");

        assert_eq!(record.scene_origin, [12.0, 34.0]);
        assert_eq!(record.scene_size, [16.0, 8.0]);
        assert_eq!(record.atlas_uv_origin, [0.125, 0.5]);
        assert_eq!(record.atlas_uv_size, [0.0625, 0.25]);
        assert_eq!(
            record.tint,
            [1.0, 128.0 / 255.0, 64.0 / 255.0, 32.0 / 255.0]
        );
        assert_eq!(
            SpriteInstanceBufferRecord::from_instance(
                SpriteDrawInstance {
                    sprite: SpriteId::PLAYER_SHIP,
                    atlas_origin: [0, 0],
                    atlas_size: [16, 8],
                    layer: RenderLayer::Objects,
                    position: [0.0, 0.0],
                    size: [16.0, 8.0],
                    tint: Color::WHITE,
                },
                SurfaceSize::new(0, 64),
            ),
            None
        );
    }

    #[test]
    fn sprite_quad_vertex_declares_stable_gpu_layout() {
        let layout = SpriteQuadGeometry::vertex_buffer_layout();
        let expected_attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 6,
            },
        ];

        assert_eq!(SpriteQuadVertex::FLOAT_COMPONENTS, 4);
        assert_eq!(SpriteQuadVertex::BYTE_SIZE, 16);
        assert_eq!(std::mem::size_of::<SpriteQuadVertex>(), 16);
        assert_eq!(
            std::mem::align_of::<SpriteQuadVertex>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(layout.array_stride, 16);
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(layout.attributes, expected_attributes);
        assert!(SpriteQuadVertex::VERTEX_ATTRIBUTES.iter().all(|quad| {
            !SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES
                .iter()
                .any(|instance| instance.shader_location == quad.shader_location)
        }));
    }

    #[test]
    fn sprite_quad_geometry_exposes_upload_bytes_without_repacking() {
        assert_eq!(SpriteQuadGeometry::VERTEX_COUNT, 4);
        assert_eq!(SpriteQuadGeometry::INDEX_COUNT, 6);
        assert_eq!(SpriteQuadGeometry::INDEX_FORMAT, wgpu::IndexFormat::Uint16);
        assert_eq!(
            SpriteQuadGeometry::vertices(),
            SpriteQuadGeometry::VERTICES.as_slice()
        );
        assert_eq!(
            SpriteQuadGeometry::indices(),
            SpriteQuadGeometry::INDICES.as_slice()
        );
        assert_eq!(
            SpriteQuadGeometry::vertex_upload_bytes().len(),
            SpriteQuadGeometry::vertices().len() * SpriteQuadVertex::BYTE_SIZE as usize
        );
        assert_eq!(
            SpriteQuadGeometry::index_upload_bytes().len(),
            std::mem::size_of_val(SpriteQuadGeometry::indices())
        );
        assert_eq!(
            SpriteQuadGeometry::vertex_upload_bytes(),
            bytemuck::cast_slice::<SpriteQuadVertex, u8>(SpriteQuadGeometry::vertices())
        );
        assert_eq!(
            SpriteQuadGeometry::index_upload_bytes(),
            bytemuck::cast_slice::<u16, u8>(SpriteQuadGeometry::indices())
        );
        assert_eq!(
            SpriteQuadGeometry::vertices()[0].as_bytes(),
            &SpriteQuadGeometry::vertex_upload_bytes()[..SpriteQuadVertex::BYTE_SIZE as usize]
        );
        assert_eq!(
            bytemuck::cast_slice::<SpriteQuadVertex, f32>(SpriteQuadGeometry::vertices()),
            &[
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ]
        );
        assert_eq!(SpriteQuadGeometry::indices(), &[0, 2, 1, 2, 3, 1]);
    }

    #[test]
    fn sprite_quad_indices_are_front_facing_after_scene_projection() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(1, 1)).expect("projection");
        let clip_vertices = SpriteQuadGeometry::vertices()
            .iter()
            .map(|vertex| projection.project_point(vertex.unit_position))
            .collect::<Vec<_>>();

        for triangle in SpriteQuadGeometry::indices().chunks_exact(3) {
            let points = [
                clip_vertices[triangle[0] as usize],
                clip_vertices[triangle[1] as usize],
                clip_vertices[triangle[2] as usize],
            ];

            assert!(
                triangle_signed_area(points) > 0.0,
                "quad triangle {triangle:?} was not counter-clockwise in clip space"
            );
        }
    }

    #[test]
    fn sprite_instance_buffer_record_declares_stable_gpu_layout() {
        let layout = SpriteInstanceBufferRecord::vertex_buffer_layout();
        let expected_attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 4,
            },
        ];

        assert_eq!(SpriteInstanceBufferRecord::FLOAT_COMPONENTS, 12);
        assert_eq!(SpriteInstanceBufferRecord::BYTE_SIZE, 48);
        assert_eq!(std::mem::size_of::<SpriteInstanceBufferRecord>(), 48);
        assert_eq!(
            std::mem::align_of::<SpriteInstanceBufferRecord>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(layout.array_stride, 48);
        assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(layout.attributes, expected_attributes);
    }

    #[test]
    fn sprite_instance_buffer_exposes_upload_bytes_without_repacking() {
        let records = vec![
            SpriteInstanceBufferRecord {
                scene_origin: [1.0, 2.0],
                scene_size: [3.0, 4.0],
                atlas_uv_origin: [0.125, 0.25],
                atlas_uv_size: [0.5, 0.75],
                tint: [1.0, 0.5, 0.25, 0.125],
            },
            SpriteInstanceBufferRecord {
                scene_origin: [5.0, 6.0],
                scene_size: [7.0, 8.0],
                atlas_uv_origin: [0.0, 0.5],
                atlas_uv_size: [0.25, 0.125],
                tint: [0.25, 0.5, 0.75, 1.0],
            },
        ];
        let buffer = SpriteInstanceBuffer {
            pipeline: NativeRenderPipeline::Sprites,
            layer: RenderLayer::Objects,
            records,
        };

        assert_eq!(
            buffer.upload_bytes().len(),
            buffer.records.len() * SpriteInstanceBufferRecord::BYTE_SIZE as usize
        );
        assert_eq!(
            buffer.upload_bytes(),
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, u8>(&buffer.records)
        );
        assert_eq!(
            buffer.records[0].as_bytes(),
            &buffer.upload_bytes()[..SpriteInstanceBufferRecord::BYTE_SIZE as usize]
        );
        assert_eq!(
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, f32>(&buffer.records),
            &[
                1.0, 2.0, 3.0, 4.0, 0.125, 0.25, 0.5, 0.75, 1.0, 0.5, 0.25, 0.125, 5.0, 6.0, 7.0,
                8.0, 0.0, 0.5, 0.25, 0.125, 0.25, 0.5, 0.75, 1.0,
            ]
        );
    }

    #[test]
    fn sprite_instance_upload_flattens_buffers_without_repacking() {
        let first =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let empty = test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0);
        let second =
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1);

        let upload =
            SpriteInstanceUpload::from_instance_buffers(&[first.clone(), empty, second.clone()])
                .expect("sprite instance upload");
        let expected_records = first
            .records
            .iter()
            .chain(&second.records)
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(upload.instance_count(), 3);
        assert_eq!(upload.records, expected_records);
        assert_eq!(upload.byte_len(), 3 * SpriteInstanceBufferRecord::BYTE_SIZE);
        assert_eq!(
            upload.upload_bytes(),
            bytemuck::cast_slice::<SpriteInstanceBufferRecord, u8>(&upload.records)
        );
        assert_eq!(SpriteInstanceUpload::from_instance_buffers(&[]), None);
        assert_eq!(
            SpriteInstanceUpload::from_instance_buffers(&[test_sprite_instance_buffer(
                NativeRenderPipeline::DebugOverlay,
                RenderLayer::Overlay,
                0,
            )]),
            None
        );
    }

    #[test]
    fn sprite_buffer_upload_plan_describes_wgpu_buffers_and_bytes() {
        let buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let upload =
            SpriteInstanceUpload::from_instance_buffers(&[buffer]).expect("instance upload");

        let plan = SpriteBufferUploadPlan::from_instance_upload(&upload);

        assert_eq!(
            plan.quad_vertices,
            SpriteBufferUpload {
                role: SpriteBufferRole::QuadVertices,
                label: "defender.sprite.quad.vertices",
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                byte_len: SpriteQuadGeometry::vertex_upload_bytes().len() as wgpu::BufferAddress,
                bytes: SpriteQuadGeometry::vertex_upload_bytes().to_vec(),
            }
        );
        assert_eq!(
            plan.quad_indices,
            SpriteBufferUpload {
                role: SpriteBufferRole::QuadIndices,
                label: "defender.sprite.quad.indices",
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                byte_len: SpriteQuadGeometry::index_upload_bytes().len() as wgpu::BufferAddress,
                bytes: SpriteQuadGeometry::index_upload_bytes().to_vec(),
            }
        );
        assert_eq!(
            plan.instances,
            SpriteBufferUpload {
                role: SpriteBufferRole::Instances,
                label: "defender.sprite.instances",
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                byte_len: upload.byte_len(),
                bytes: upload.upload_bytes().to_vec(),
            }
        );
    }

    #[test]
    fn sprite_render_pass_plan_describes_bindings_and_indexed_draws() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
        ];
        let upload = SpriteInstanceUpload::from_instance_buffers(&buffers).expect("upload");
        let uploads = SpriteBufferUploadPlan::from_instance_upload(&upload);
        let commands = super::sprite_draw_commands_from_instance_buffers(&buffers);

        let plan = SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &commands)
            .expect("sprite render pass plan");

        assert_eq!(
            plan.quad_vertices,
            SpriteVertexBufferBinding {
                role: SpriteBufferRole::QuadVertices,
                slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                byte_offset: 0,
                byte_len: uploads.quad_vertices.byte_len,
            }
        );
        assert_eq!(
            plan.instances,
            SpriteVertexBufferBinding {
                role: SpriteBufferRole::Instances,
                slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                byte_offset: 0,
                byte_len: uploads.instances.byte_len,
            }
        );
        assert_eq!(
            plan.indices,
            SpriteIndexBufferBinding {
                role: SpriteBufferRole::QuadIndices,
                index_format: wgpu::IndexFormat::Uint16,
                byte_offset: 0,
                byte_len: uploads.quad_indices.byte_len,
            }
        );
        assert_eq!(plan.draw_count(), 2);
        assert_eq!(plan.instance_count(), 3);
        assert_eq!(
            plan.draws,
            vec![
                SpriteRenderPassDraw {
                    pipeline: NativeRenderPipeline::Sprites,
                    layer: RenderLayer::Objects,
                    indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                    base_vertex: 0,
                    instances: 0..2,
                    instance_buffer_byte_offset: 0,
                    instance_buffer_byte_len: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                },
                SpriteRenderPassDraw {
                    pipeline: NativeRenderPipeline::HudText,
                    layer: RenderLayer::Hud,
                    indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                    base_vertex: 0,
                    instances: 2..3,
                    instance_buffer_byte_offset: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                    instance_buffer_byte_len: SpriteInstanceBufferRecord::BYTE_SIZE,
                },
            ]
        );
        assert_eq!(
            SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &[]),
            None
        );
    }

    #[test]
    fn sprite_render_pass_encoder_plan_orders_wgpu_commands() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
        ];
        let upload = SpriteInstanceUpload::from_instance_buffers(&buffers).expect("upload");
        let uploads = SpriteBufferUploadPlan::from_instance_upload(&upload);
        let draw_commands = super::sprite_draw_commands_from_instance_buffers(&buffers);
        let render_pass = SpriteRenderPassPlan::from_uploads_and_commands(&uploads, &draw_commands)
            .expect("sprite render pass plan");
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);
        let pipeline = SpritePipelinePlan::for_settings(GpuRendererSettings::default());
        let descriptor =
            SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(&pipeline, &layout);

        let plan = SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
            &render_pass,
            &layout,
            &descriptor,
        );

        assert_eq!(plan.label, "defender.sprite.render_pass.encoder");
        assert_eq!(plan.command_count(), 8);
        assert_eq!(
            plan.set_pipeline_command_count(),
            SpriteRenderPassEncoderPlan::SET_PIPELINE_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_bind_group_command_count(),
            SpriteRenderPassEncoderPlan::SET_BIND_GROUP_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_vertex_buffer_command_count(),
            SpriteRenderPassEncoderPlan::SET_VERTEX_BUFFER_COMMAND_COUNT
        );
        assert_eq!(
            plan.set_index_buffer_command_count(),
            SpriteRenderPassEncoderPlan::SET_INDEX_BUFFER_COMMAND_COUNT
        );
        assert_eq!(plan.draw_count(), 2);
        assert_eq!(plan.instance_count(), 3);
        assert_eq!(
            plan.commands,
            vec![
                SpriteRenderPassEncoderCommand::SetPipeline {
                    label: "defender.sprite.pipeline",
                },
                SpriteRenderPassEncoderCommand::SetBindGroup {
                    role: SpriteBindGroupRole::SceneProjection,
                    group_index: 0,
                    layout_label: "defender.sprite.scene_projection.bind_group_layout",
                },
                SpriteRenderPassEncoderCommand::SetBindGroup {
                    role: SpriteBindGroupRole::SpriteAtlas,
                    group_index: 1,
                    layout_label: "defender.sprite.atlas.bind_group_layout",
                },
                SpriteRenderPassEncoderCommand::SetVertexBuffer {
                    role: SpriteBufferRole::QuadVertices,
                    slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                    byte_offset: 0,
                    byte_len: SpriteQuadGeometry::vertex_upload_bytes().len()
                        as wgpu::BufferAddress,
                },
                SpriteRenderPassEncoderCommand::SetVertexBuffer {
                    role: SpriteBufferRole::Instances,
                    slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                    byte_offset: 0,
                    byte_len: 3 * SpriteInstanceBufferRecord::BYTE_SIZE,
                },
                SpriteRenderPassEncoderCommand::SetIndexBuffer {
                    role: SpriteBufferRole::QuadIndices,
                    index_format: wgpu::IndexFormat::Uint16,
                    byte_offset: 0,
                    byte_len: SpriteQuadGeometry::index_upload_bytes().len() as wgpu::BufferAddress,
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: render_pass.draws[0].clone(),
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: render_pass.draws[1].clone(),
                },
            ]
        );
    }

    #[test]
    fn wgpu_frame_plan_orders_pass_raster_and_sprite_commands() {
        let pass = WgpuPassPlan {
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.1,
                b: 0.2,
                a: 1.0,
            },
            viewport: Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            }),
            scene_projection: SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)),
        };
        let raster_upload = SceneRasterUpload {
            surface: SurfaceSize::new(292, 240),
            byte_len: 292 * 240 * 4,
            visual_signature: Some(0xCAFE_BABE),
            non_blank: true,
        };
        let sprite_encoder = SpriteRenderPassEncoderPlan {
            label: "defender.sprite.render_pass.encoder",
            commands: vec![
                SpriteRenderPassEncoderCommand::SetPipeline {
                    label: "defender.sprite.pipeline",
                },
                SpriteRenderPassEncoderCommand::DrawIndexed {
                    draw: SpriteRenderPassDraw {
                        pipeline: NativeRenderPipeline::Sprites,
                        layer: RenderLayer::Objects,
                        indices: 0..SpriteQuadGeometry::INDEX_COUNT,
                        base_vertex: 0,
                        instances: 0..2,
                        instance_buffer_byte_offset: 0,
                        instance_buffer_byte_len: 2 * SpriteInstanceBufferRecord::BYTE_SIZE,
                    },
                },
            ],
        };

        let plan = WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &pass,
            Some(raster_upload),
            Some(&sprite_encoder),
        );

        assert_eq!(plan.label, "defender.frame.commands");
        assert_eq!(plan.command_count(), 5);
        assert!(!plan.has_ordered_sprite_only_commands());
        assert_eq!(plan.temporary_raster_count(), 1);
        assert_eq!(plan.sprite_pass_count(), 1);
        assert_eq!(plan.begin_render_pass_count(), 1);
        assert_eq!(plan.viewport_command_count(), 1);
        assert_eq!(
            plan.scene_projection_upload_byte_len(),
            SceneProjectionUniforms::BYTE_SIZE
        );
        assert_eq!(plan.sprite_encoder_command_count(), 2);
        assert_eq!(plan.sprite_draw_count(), 1);
        assert_eq!(plan.sprite_instance_count(), 2);
        assert_eq!(
            plan.commands,
            vec![
                WgpuFrameCommand::BeginRenderPass {
                    clear_color: pass.clear_color,
                },
                WgpuFrameCommand::SetViewport {
                    viewport: pass.viewport.expect("viewport"),
                },
                WgpuFrameCommand::UploadSceneProjection {
                    byte_len: SceneProjectionUniforms::BYTE_SIZE,
                },
                WgpuFrameCommand::UploadTemporaryRaster {
                    upload: raster_upload,
                },
                WgpuFrameCommand::ExecuteSpriteRenderPass {
                    encoder_label: "defender.sprite.render_pass.encoder",
                    command_count: 2,
                    draw_count: 1,
                    instance_count: 2,
                },
            ]
        );

        let sprite_only_plan =
            WgpuFramePlan::from_pass_raster_and_sprite_encoder(&pass, None, Some(&sprite_encoder));

        assert_eq!(sprite_only_plan.command_count(), 4);
        assert!(sprite_only_plan.has_ordered_sprite_only_commands());
    }

    #[test]
    fn sprite_shader_plan_exposes_wgsl_descriptor_and_entries() {
        let shader = SpriteShaderPlan::default();

        assert_eq!(shader.label, "defender.sprite.shader");
        assert_eq!(shader.vertex_entry, "sprite_vs");
        assert_eq!(shader.fragment_entry, "sprite_fs");
        assert!(shader.source.contains("@vertex"));
        assert!(shader.source.contains("@fragment"));
        assert!(shader.source.contains("textureSample(sprite_atlas"));
        assert!(shader.source.contains("@location(0) scene_origin"));
        assert!(shader.source.contains("@location(6) unit_uv"));

        let descriptor = shader.shader_module_descriptor();
        assert_eq!(descriptor.label, Some("defender.sprite.shader"));
        match descriptor.source {
            wgpu::ShaderSource::Wgsl(source) => assert_eq!(source.as_ref(), shader.source),
            _ => panic!("sprite shader descriptor must use WGSL"),
        }
    }

    #[test]
    fn sprite_pipeline_plan_describes_wgpu_state_and_vertex_layouts() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };

        let plan = SpritePipelinePlan::for_settings(settings);

        assert_eq!(plan.label, "defender.sprite.pipeline");
        assert_eq!(plan.shader, SpriteShaderPlan::default());
        assert_eq!(
            plan.vertex_buffers,
            [
                SpriteVertexBufferLayoutPlan {
                    role: SpriteBufferRole::QuadVertices,
                    slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
                    array_stride: SpriteQuadVertex::BYTE_SIZE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &SpriteQuadVertex::VERTEX_ATTRIBUTES,
                },
                SpriteVertexBufferLayoutPlan {
                    role: SpriteBufferRole::Instances,
                    slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
                    array_stride: SpriteInstanceBufferRecord::BYTE_SIZE,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES,
                },
            ]
        );

        let layouts = plan.vertex_buffer_layouts();
        assert_eq!(layouts[0].array_stride, SpriteQuadVertex::BYTE_SIZE);
        assert_eq!(layouts[0].step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(layouts[0].attributes, SpriteQuadVertex::VERTEX_ATTRIBUTES);
        assert_eq!(
            layouts[1].array_stride,
            SpriteInstanceBufferRecord::BYTE_SIZE
        );
        assert_eq!(layouts[1].step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(
            layouts[1].attributes,
            SpriteInstanceBufferRecord::VERTEX_ATTRIBUTES
        );
        assert_eq!(
            plan.primitive.topology,
            wgpu::PrimitiveTopology::TriangleList
        );
        assert_eq!(plan.primitive.front_face, wgpu::FrontFace::Ccw);
        assert_eq!(plan.primitive.cull_mode, None);
        assert_eq!(
            plan.color_target,
            wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }
        );
        assert_eq!(plan.multisample, wgpu::MultisampleState::default());
    }

    #[test]
    fn sprite_atlas_texture_upload_describes_wgpu_texture_copy() {
        let atlas = TextureAtlas::default_sprites();
        let upload = SpriteAtlasTextureUpload::from_atlas(&atlas).expect("atlas upload");

        assert_eq!(
            upload,
            SpriteAtlasTextureUpload {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture",
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                dimension: wgpu::TextureDimension::D2,
                surface: SurfaceSize::new(128, 192),
                mip_level_count: 1,
                sample_count: 1,
                depth_or_array_layers: 1,
                bytes_per_row: 128 * 4,
                rows_per_image: 192,
                byte_len: 128 * 192 * 4,
                bytes: atlas.pixels().to_vec(),
                non_blank: true,
            }
        );
        assert_eq!(
            upload.extent(),
            wgpu::Extent3d {
                width: 128,
                height: 192,
                depth_or_array_layers: 1,
            }
        );
        let copy_layout = upload.copy_layout();
        assert_eq!(copy_layout.offset, 0);
        assert_eq!(copy_layout.bytes_per_row, Some(128 * 4));
        assert_eq!(copy_layout.rows_per_image, Some(192));
        let descriptor = upload.texture_descriptor();
        assert_eq!(descriptor.label, Some("defender.sprite.atlas.texture"));
        assert_eq!(descriptor.size, upload.extent());
        assert_eq!(descriptor.mip_level_count, 1);
        assert_eq!(descriptor.sample_count, 1);
        assert_eq!(descriptor.dimension, wgpu::TextureDimension::D2);
        assert_eq!(descriptor.format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(
            descriptor.usage,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        );
        assert_eq!(descriptor.view_formats, &[]);

        assert_eq!(
            SpriteAtlasTextureUpload::from_atlas(&TextureAtlas::new(
                SurfaceSize::new(0, 128),
                Vec::new()
            )),
            None
        );
        let missing_pixels = TextureAtlas {
            surface: SurfaceSize::new(2, 2),
            regions: Vec::new(),
            pixels: Vec::new(),
        };
        assert_eq!(SpriteAtlasTextureUpload::from_atlas(&missing_pixels), None);
    }

    #[test]
    fn sprite_resource_binding_plan_describes_uniform_and_atlas_bindings() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::with_rgba(
            SurfaceSize::new(128, 64),
            Vec::new(),
            vec![0x80; 128 * 64 * 4],
        )
        .expect("atlas");

        let plan = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");

        assert_eq!(
            plan.atlas_upload,
            SpriteAtlasTextureUpload {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture",
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                dimension: wgpu::TextureDimension::D2,
                surface: SurfaceSize::new(128, 64),
                mip_level_count: 1,
                sample_count: 1,
                depth_or_array_layers: 1,
                bytes_per_row: 128 * 4,
                rows_per_image: 64,
                byte_len: 128 * 64 * 4,
                bytes: atlas.pixels().to_vec(),
                non_blank: true,
            }
        );
        assert_eq!(
            plan.projection_upload,
            SceneProjectionUniformUpload {
                role: SpriteResourceBindingRole::SceneProjectionUniform,
                label: "defender.sprite.scene_projection.uniform",
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                byte_len: SceneProjectionUniforms::BYTE_SIZE,
                bytes: projection.as_bytes().to_vec(),
            }
        );
        assert_eq!(plan.projection_layout.group_index(), 0);
        assert_eq!(
            plan.bind_group_count(),
            SpriteResourceBindingPlan::BIND_GROUP_COUNT
        );
        assert_eq!(
            plan.binding_entry_count(),
            SpriteResourceBindingPlan::BINDING_ENTRY_COUNT
        );
        assert_eq!(
            plan.projection_layout,
            SpriteBindGroupLayoutPlan {
                role: SpriteBindGroupRole::SceneProjection,
                label: "defender.sprite.scene_projection.bind_group_layout",
                entries: vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(SceneProjectionUniforms::BYTE_SIZE),
                    },
                    count: None,
                }],
            }
        );
        assert_eq!(plan.atlas_layout.group_index(), 1);
        assert_eq!(
            plan.atlas_layout,
            SpriteBindGroupLayoutPlan {
                role: SpriteBindGroupRole::SpriteAtlas,
                label: "defender.sprite.atlas.bind_group_layout",
                entries: vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }
        );
        assert_eq!(
            plan.atlas_texture,
            SpriteTextureBindingPlan {
                role: SpriteResourceBindingRole::SpriteAtlasTexture,
                label: "defender.sprite.atlas.texture_view",
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
                surface: SurfaceSize::new(128, 64),
            }
        );
        assert_eq!(
            plan.atlas_sampler,
            SpriteSamplerBindingPlan {
                role: SpriteResourceBindingRole::SpriteAtlasSampler,
                label: "defender.sprite.atlas.sampler",
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                sampler_binding: wgpu::SamplerBindingType::Filtering,
            }
        );
        assert_eq!(
            SpriteResourceBindingPlan::from_projection_and_atlas(
                projection,
                &TextureAtlas::new(SurfaceSize::new(0, 64), Vec::new())
            ),
            None
        );
    }

    #[test]
    fn sprite_pipeline_layout_plan_orders_resource_bind_groups() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");

        let plan = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        assert_eq!(plan.label, "defender.sprite.pipeline_layout");
        assert_eq!(plan.immediate_size, 0);
        assert_eq!(
            plan.bind_group_count(),
            SpritePipelineLayoutPlan::BIND_GROUP_COUNT
        );
        assert_eq!(
            plan.binding_entry_count(),
            SpritePipelineLayoutPlan::BINDING_ENTRY_COUNT
        );
        assert_eq!(
            plan.bind_groups,
            vec![
                SpritePipelineLayoutBindGroup {
                    role: SpriteBindGroupRole::SceneProjection,
                    group_index: 0,
                    layout_label: "defender.sprite.scene_projection.bind_group_layout",
                    entry_count: 1,
                },
                SpritePipelineLayoutBindGroup {
                    role: SpriteBindGroupRole::SpriteAtlas,
                    group_index: 1,
                    layout_label: "defender.sprite.atlas.bind_group_layout",
                    entry_count: 2,
                },
            ]
        );
    }

    #[test]
    fn sprite_render_pipeline_descriptor_plan_combines_pipeline_and_layout() {
        let settings = GpuRendererSettings {
            texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            ..GpuRendererSettings::default()
        };
        let pipeline = SpritePipelinePlan::for_settings(settings);
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        let descriptor =
            SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(&pipeline, &layout);

        assert_eq!(descriptor.label, "defender.sprite.pipeline");
        assert_eq!(descriptor.layout_label, "defender.sprite.pipeline_layout");
        assert_eq!(
            descriptor.layout_bind_group_count(),
            SpriteRenderPipelineDescriptorPlan::LAYOUT_BIND_GROUP_COUNT
        );
        assert_eq!(
            descriptor.vertex_buffer_count(),
            SpriteRenderPipelineDescriptorPlan::VERTEX_BUFFER_COUNT
        );
        assert_eq!(
            descriptor.color_target_count(),
            SpriteRenderPipelineDescriptorPlan::COLOR_TARGET_COUNT
        );
        assert_eq!(descriptor.immediate_size, 0);
        assert_eq!(descriptor.shader_label, "defender.sprite.shader");
        assert_eq!(descriptor.vertex_entry, "sprite_vs");
        assert_eq!(descriptor.fragment_entry, "sprite_fs");
        assert_eq!(descriptor.vertex_buffers, pipeline.vertex_buffers);
        assert_eq!(descriptor.primitive, pipeline.primitive);
        assert_eq!(descriptor.color_target, pipeline.color_target);
        assert_eq!(descriptor.multisample, pipeline.multisample);
        assert_eq!(
            descriptor.vertex_buffer_layouts(),
            pipeline.vertex_buffer_layouts()
        );
        assert_eq!(
            descriptor.color_targets(),
            [Some(pipeline.color_target.clone())]
        );
    }

    #[test]
    fn sprite_shader_bindings_match_resource_binding_plan() {
        let shader = SpriteShaderPlan::default();
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");
        let atlas = TextureAtlas::default_sprites();
        let bindings = SpriteResourceBindingPlan::from_projection_and_atlas(projection, &atlas)
            .expect("sprite resource binding plan");
        let pipeline_layout = SpritePipelineLayoutPlan::from_resource_bindings(&bindings);

        assert!(shader.source.contains("@group(0) @binding(0)"));
        assert_eq!(bindings.projection_layout.group_index(), 0);
        assert_eq!(bindings.projection_layout.entries[0].binding, 0);
        assert_eq!(pipeline_layout.bind_groups[0].group_index, 0);
        assert!(shader.source.contains("@group(1) @binding(0)"));
        assert!(shader.source.contains("@group(1) @binding(1)"));
        assert_eq!(bindings.atlas_layout.group_index(), 1);
        assert_eq!(bindings.atlas_texture.binding, 0);
        assert_eq!(bindings.atlas_sampler.binding, 1);
        assert_eq!(pipeline_layout.bind_groups[1].group_index, 1);
    }

    #[test]
    fn sprite_draw_command_uses_quad_geometry_and_instance_buffer_metadata() {
        let buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2);
        let empty_buffer =
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0);

        assert_eq!(
            SpriteDrawCommand::from_instance_buffer(&buffer, 7),
            Some(expected_sprite_draw_command(
                NativeRenderPipeline::Sprites,
                RenderLayer::Objects,
                7,
                2,
            ))
        );
        assert_eq!(
            SpriteDrawCommand::from_instance_buffer(&empty_buffer, 7),
            None
        );
    }

    #[test]
    fn sprite_draw_commands_track_cumulative_instance_ranges() {
        let buffers = vec![
            test_sprite_instance_buffer(NativeRenderPipeline::Sprites, RenderLayer::Objects, 2),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 0),
            test_sprite_instance_buffer(NativeRenderPipeline::HudText, RenderLayer::Hud, 1),
            test_sprite_instance_buffer(
                NativeRenderPipeline::Projectiles,
                RenderLayer::Projectiles,
                3,
            ),
        ];

        assert_eq!(
            super::sprite_draw_commands_from_instance_buffers(&buffers),
            vec![
                expected_sprite_draw_command(
                    NativeRenderPipeline::Sprites,
                    RenderLayer::Objects,
                    0,
                    2,
                ),
                expected_sprite_draw_command(NativeRenderPipeline::HudText, RenderLayer::Hud, 2, 1),
                expected_sprite_draw_command(
                    NativeRenderPipeline::Projectiles,
                    RenderLayer::Projectiles,
                    3,
                    3,
                ),
            ]
        );
    }

    #[test]
    fn wgpu_viewport_command_matches_non_empty_layout() {
        let layout = ViewportLayout::fit(SurfaceSize::new(292, 240), SurfaceSize::new(640, 480));

        assert_eq!(
            WgpuViewportCommand::from_layout(layout),
            Some(WgpuViewportCommand {
                x: 28.0,
                y: 0.0,
                width: 584.0,
                height: 480.0,
                min_depth: 0.0,
                max_depth: 1.0,
            })
        );
        assert_eq!(
            WgpuViewportCommand::from_layout(ViewportLayout::fit(
                SurfaceSize::new(292, 240),
                SurfaceSize::new(0, 480)
            )),
            None
        );
    }

    #[test]
    fn scene_projection_uniforms_map_scene_points_to_clip_space() {
        let projection =
            SceneProjectionUniforms::for_surface(SurfaceSize::new(292, 240)).expect("projection");

        assert_eq!(SceneProjectionUniforms::FLOAT_COMPONENTS, 4);
        assert_eq!(SceneProjectionUniforms::BYTE_SIZE, 16);
        assert_eq!(std::mem::size_of::<SceneProjectionUniforms>(), 16);
        assert_eq!(
            std::mem::align_of::<SceneProjectionUniforms>(),
            std::mem::align_of::<f32>()
        );
        assert_eq!(projection.scale, [2.0 / 292.0, -2.0 / 240.0]);
        assert_eq!(projection.translate, [-1.0, 1.0]);
        assert_eq!(projection.project_point([0.0, 0.0]), [-1.0, 1.0]);
        assert_clip_point_near(projection.project_point([146.0, 120.0]), [0.0, 0.0]);
        assert_clip_point_near(projection.project_point([292.0, 240.0]), [1.0, -1.0]);
        assert_eq!(
            bytemuck::cast_slice::<SceneProjectionUniforms, f32>(&[projection]),
            &[2.0 / 292.0, -2.0 / 240.0, -1.0, 1.0]
        );
        assert_eq!(projection.as_bytes(), bytemuck::bytes_of(&projection));
        assert_eq!(
            SceneProjectionUniforms::for_surface(SurfaceSize::new(0, 240)),
            None
        );
    }

    fn assert_clip_point_near(actual: [f32; 2], expected: [f32; 2]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= f32::EPSILON,
                "clip point component {actual} was not near {expected}"
            );
        }
    }

    fn triangle_signed_area(points: [[f32; 2]; 3]) -> f32 {
        let [a, b, c] = points;
        ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5
    }

    fn test_sprite_instance_buffer_record(scene_origin: [f32; 2]) -> SpriteInstanceBufferRecord {
        SpriteInstanceBufferRecord {
            scene_origin,
            scene_size: [8.0, 4.0],
            atlas_uv_origin: [0.125, 0.25],
            atlas_uv_size: [0.5, 0.75],
            tint: [1.0, 0.5, 0.25, 1.0],
        }
    }

    fn test_sprite_instance_buffer(
        pipeline: NativeRenderPipeline,
        layer: RenderLayer,
        record_count: usize,
    ) -> SpriteInstanceBuffer {
        SpriteInstanceBuffer {
            pipeline,
            layer,
            records: (0..record_count)
                .map(|index| test_sprite_instance_buffer_record([index as f32, index as f32]))
                .collect(),
        }
    }

    fn expected_sprite_draw_command(
        pipeline: NativeRenderPipeline,
        layer: RenderLayer,
        first_instance: u32,
        instance_count: u32,
    ) -> SpriteDrawCommand {
        SpriteDrawCommand {
            pipeline,
            layer,
            vertex_count: SpriteQuadGeometry::VERTEX_COUNT,
            index_count: SpriteQuadGeometry::INDEX_COUNT,
            index_format: SpriteQuadGeometry::INDEX_FORMAT,
            first_index: 0,
            base_vertex: 0,
            first_instance,
            instance_count,
            vertex_buffer_byte_len: SpriteQuadGeometry::vertex_upload_bytes().len()
                as wgpu::BufferAddress,
            index_buffer_byte_len: SpriteQuadGeometry::index_upload_bytes().len()
                as wgpu::BufferAddress,
            instance_buffer_byte_offset: u64::from(first_instance)
                * SpriteInstanceBufferRecord::BYTE_SIZE,
            instance_buffer_byte_len: u64::from(instance_count)
                * SpriteInstanceBufferRecord::BYTE_SIZE,
        }
    }

    fn expected_sprite_render_pass(plan: &SceneDrawPlan) -> Option<SpriteRenderPassPlan> {
        plan.sprite_buffer_uploads.as_ref().and_then(|uploads| {
            SpriteRenderPassPlan::from_uploads_and_commands(uploads, &plan.sprite_draw_commands)
        })
    }

    fn expected_sprite_pipeline(
        plan: &SceneDrawPlan,
        settings: GpuRendererSettings,
    ) -> Option<SpritePipelinePlan> {
        plan.sprite_render_pass
            .as_ref()
            .map(|_| SpritePipelinePlan::for_settings(settings))
    }

    fn expected_sprite_resource_bindings(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteResourceBindingPlan> {
        plan.sprite_pipeline.as_ref().and_then(|_| {
            plan.gpu_pass.scene_projection.and_then(|projection| {
                SpriteResourceBindingPlan::from_projection_and_atlas(
                    projection,
                    &TextureAtlas::default_sprites(),
                )
            })
        })
    }

    fn expected_sprite_pipeline_layout(plan: &SceneDrawPlan) -> Option<SpritePipelineLayoutPlan> {
        match (
            plan.sprite_pipeline.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(_), Some(bindings), Some(_)) => {
                Some(SpritePipelineLayoutPlan::from_resource_bindings(bindings))
            }
            _ => None,
        }
    }

    fn expected_sprite_render_pipeline_descriptor(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteRenderPipelineDescriptorPlan> {
        match (
            plan.sprite_render_pass.as_ref(),
            plan.sprite_pipeline.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.sprite_pipeline_layout.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(_), Some(pipeline), Some(_), Some(layout), Some(_)) => {
                Some(SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(pipeline, layout))
            }
            _ => None,
        }
    }

    fn expected_sprite_render_pass_encoder(
        plan: &SceneDrawPlan,
    ) -> Option<SpriteRenderPassEncoderPlan> {
        match (
            plan.sprite_render_pass.as_ref(),
            plan.sprite_resource_bindings.as_ref(),
            plan.sprite_pipeline_layout.as_ref(),
            plan.sprite_render_pipeline_descriptor.as_ref(),
            plan.gpu_pass.viewport,
        ) {
            (Some(render_pass), Some(_), Some(layout), Some(descriptor), Some(_)) => Some(
                SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
                    render_pass,
                    layout,
                    descriptor,
                ),
            ),
            _ => None,
        }
    }

    fn expected_frame_plan(plan: &SceneDrawPlan) -> WgpuFramePlan {
        WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &plan.gpu_pass,
            plan.raster_upload,
            plan.sprite_render_pass_encoder.as_ref(),
        )
    }
