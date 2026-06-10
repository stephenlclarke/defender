#[cfg(all(not(test), not(coverage)))]
const RGBA_BYTES_PER_PIXEL: usize = 4;
#[cfg(all(not(test), not(coverage)))]
const TRANSPARENT_BLACK_RGBA: [u8; RGBA_BYTES_PER_PIXEL] = [0; RGBA_BYTES_PER_PIXEL];
#[cfg(all(not(test), not(coverage)))]
const FNV1A_64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
#[cfg(all(not(test), not(coverage)))]
const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01B3;

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OffscreenWgpuSmokeReport {
    frames: u32,
    non_blank_frames: u32,
    distinct_frame_signatures: usize,
    first_frame_signature: Option<u64>,
    last_frame_signature: Option<u64>,
}

#[cfg(all(not(test), not(coverage)))]
async fn render_actor_offscreen_smoke() -> anyhow::Result<OffscreenWgpuSmokeReport> {
    let mut renderer = WgpuOffscreenRenderer::new().await?;
    let mut runtime = ActorRuntimeAdapter::new();
    let mut signatures = BTreeSet::new();
    let mut report = OffscreenWgpuSmokeReport::default();

    for frame_index in 0..crate::actor_smoke::smoke_frame_count() {
        let frame = runtime.step(crate::actor_smoke::smoke_actor_input(frame_index));
        let rendered = renderer.render_scene(&frame.scene)?;
        report.frames = report.frames.saturating_add(1);
        if rendered.non_blank {
            report.non_blank_frames = report.non_blank_frames.saturating_add(1);
        }
        report
            .first_frame_signature
            .get_or_insert(rendered.signature);
        report.last_frame_signature = Some(rendered.signature);
        signatures.insert(rendered.signature);
    }

    report.distinct_frame_signatures = signatures.len();
    Ok(report)
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuOffscreenRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: NativeSceneRenderer,
    sprite_resources: Option<SpriteGpuResources>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuOffscreenRenderer {
    const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .context("requesting clean offscreen wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("defender clean offscreen wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting clean offscreen wgpu device")?;
        let renderer = NativeSceneRenderer::with_settings(
            Default::default(),
            GpuRendererSettings {
                texture_format: Self::TEXTURE_FORMAT,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
            },
        );

        Ok(Self {
            device,
            queue,
            renderer,
            sprite_resources: None,
        })
    }

    fn render_scene(
        &mut self,
        scene: &crate::renderer::RenderScene,
    ) -> anyhow::Result<RenderedOffscreenFrame> {
        if scene.surface.is_empty() {
            anyhow::bail!("cannot render empty offscreen scene");
        }

        let plan = self.renderer.prepare(scene);
        self.update_sprite_resources(&plan)?;

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("defender.offscreen.live_smoke.texture"),
            size: wgpu::Extent3d {
                width: scene.surface.width,
                height: scene.surface.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let readback = OffscreenReadbackLayout::for_surface(scene.surface)?;
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("defender.offscreen.live_smoke.readback"),
            size: readback.buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender.offscreen.live_smoke.encoder"),
            });
        encode_scene_render_pass(&mut encoder, &view, &plan, self.sprite_resources.as_ref());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback.padded_bytes_per_row),
                    rows_per_image: Some(scene.surface.height),
                },
            },
            wgpu::Extent3d {
                width: scene.surface.width,
                height: scene.surface.height,
                depth_or_array_layers: 1,
            },
        );

        let (sender, receiver) = mpsc::channel();
        encoder.map_buffer_on_submit(
            &readback_buffer,
            wgpu::MapMode::Read,
            0..readback.buffer_size,
            move |result| {
                let _ = sender.send(result);
            },
        );
        self.queue.submit([encoder.finish()]);
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("polling clean offscreen wgpu readback")?;
        receiver
            .recv()
            .context("waiting for clean offscreen wgpu readback")?
            .context("mapping clean offscreen wgpu readback")?;

        let mapped = readback_buffer.slice(..).get_mapped_range();
        let pixels = readback.unpadded_pixels(&mapped);
        drop(mapped);
        readback_buffer.unmap();

        Ok(RenderedOffscreenFrame {
            surface: scene.surface,
            signature: rendered_rgba_signature(scene.surface, &pixels),
            non_blank: rendered_rgba_is_non_blank(&pixels),
        })
    }

    fn update_sprite_resources(&mut self, plan: &SceneDrawPlan) -> anyhow::Result<()> {
        if plan.sprite_render_pass_encoder.is_none() {
            return Ok(());
        }
        if self.sprite_resources.is_none() {
            self.sprite_resources = Some(SpriteGpuResources::new(&self.device, &self.queue, plan)?);
        }
        let Some(resources) = &mut self.sprite_resources else {
            return Ok(());
        };
        let Some(bindings) = &plan.sprite_resource_bindings else {
            return Ok(());
        };
        self.queue.write_buffer(
            &resources.projection_buffer,
            0,
            &bindings.projection_upload.bytes,
        );
        if let Some(uploads) = &plan.sprite_buffer_uploads {
            resources.update_instances(&self.device, &self.queue, &uploads.instances);
        }
        Ok(())
    }
}

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RenderedOffscreenFrame {
    surface: SurfaceSize,
    signature: u64,
    non_blank: bool,
}

#[cfg(all(not(test), not(coverage)))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OffscreenReadbackLayout {
    unpadded_bytes_per_row: u32,
    padded_bytes_per_row: u32,
    buffer_size: wgpu::BufferAddress,
    surface: SurfaceSize,
}

#[cfg(all(not(test), not(coverage)))]
impl OffscreenReadbackLayout {
    fn for_surface(surface: SurfaceSize) -> anyhow::Result<Self> {
        let unpadded_bytes_per_row = surface
            .width
            .checked_mul(RGBA_BYTES_PER_PIXEL as u32)
            .context("clean offscreen readback row byte length overflow")?;
        let padded_bytes_per_row = align_copy_bytes_per_row(unpadded_bytes_per_row);
        let buffer_size = u64::from(padded_bytes_per_row)
            .checked_mul(u64::from(surface.height))
            .context("clean offscreen readback buffer length overflow")?;

        Ok(Self {
            unpadded_bytes_per_row,
            padded_bytes_per_row,
            buffer_size,
            surface,
        })
    }

    fn unpadded_pixels(&self, mapped: &[u8]) -> Vec<u8> {
        let unpadded_bytes_per_row = self.unpadded_bytes_per_row as usize;
        let padded_bytes_per_row = self.padded_bytes_per_row as usize;
        let mut pixels = Vec::with_capacity(self.surface.rgba_len().unwrap_or_default());

        for row in 0..self.surface.height as usize {
            let row_start = row * padded_bytes_per_row;
            let row_end = row_start + unpadded_bytes_per_row;
            pixels.extend_from_slice(&mapped[row_start..row_end]);
        }

        pixels
    }
}

#[cfg(all(not(test), not(coverage)))]
fn align_copy_bytes_per_row(bytes_per_row: u32) -> u32 {
    bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
}

#[cfg(all(not(test), not(coverage)))]
fn rendered_rgba_is_non_blank(pixels: &[u8]) -> bool {
    pixels
        .chunks_exact(RGBA_BYTES_PER_PIXEL)
        .any(|pixel| pixel != TRANSPARENT_BLACK_RGBA)
}

#[cfg(all(not(test), not(coverage)))]
fn rendered_rgba_signature(surface: SurfaceSize, pixels: &[u8]) -> u64 {
    let mut signature = FNV1A_64_OFFSET_BASIS;
    signature = fnv1a_mix_u64(signature, u64::from(surface.width));
    signature = fnv1a_mix_u64(signature, u64::from(surface.height));
    for byte in pixels {
        signature ^= u64::from(*byte);
        signature = signature.wrapping_mul(FNV1A_64_PRIME);
    }
    signature
}

#[cfg(all(not(test), not(coverage)))]
fn fnv1a_mix_u64(mut signature: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        signature ^= u64::from(byte);
        signature = signature.wrapping_mul(FNV1A_64_PRIME);
    }
    signature
}

#[cfg(all(not(test), not(coverage)))]
struct SpriteGpuResources {
    pipeline: wgpu::RenderPipeline,
    projection_buffer: wgpu::Buffer,
    projection_bind_group: wgpu::BindGroup,
    atlas_bind_group: wgpu::BindGroup,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_size: wgpu::BufferAddress,
}

#[cfg(all(not(test), not(coverage)))]
impl SpriteGpuResources {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &SceneDrawPlan,
    ) -> anyhow::Result<Self> {
        let bindings = plan
            .sprite_resource_bindings
            .as_ref()
            .context("sprite plan missing resource bindings")?;
        let layout = plan
            .sprite_pipeline_layout
            .as_ref()
            .context("sprite plan missing pipeline layout")?;
        let descriptor = plan
            .sprite_render_pipeline_descriptor
            .as_ref()
            .context("sprite plan missing render pipeline descriptor")?;
        let pipeline_plan = plan
            .sprite_pipeline
            .as_ref()
            .context("sprite plan missing pipeline")?;
        let uploads = plan
            .sprite_buffer_uploads
            .as_ref()
            .context("sprite plan missing buffer uploads")?;

        let projection_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bindings.projection_layout.label),
            entries: &bindings.projection_layout.entries,
        });
        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(bindings.atlas_layout.label),
            entries: &bindings.atlas_layout.entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(layout.label),
            bind_group_layouts: &[Some(&projection_layout), Some(&atlas_layout)],
            immediate_size: layout.immediate_size,
        });

        let projection_buffer = create_buffer(device, &bindings.projection_upload);
        queue.write_buffer(&projection_buffer, 0, &bindings.projection_upload.bytes);
        let projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("defender.sprite.scene_projection.bind_group"),
            layout: &projection_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
        });

        let atlas_texture = device.create_texture(&bindings.atlas_upload.texture_descriptor());
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bindings.atlas_upload.bytes,
            bindings.atlas_upload.copy_layout(),
            bindings.atlas_upload.extent(),
        );
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(bindings.atlas_sampler.label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("defender.sprite.atlas.bind_group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(pipeline_plan.shader.shader_module_descriptor());
        let color_targets = descriptor.color_targets();
        let vertex_buffers = descriptor.vertex_buffer_layouts();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(descriptor.label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(descriptor.vertex_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_buffers,
            },
            primitive: descriptor.primitive,
            depth_stencil: None,
            multisample: descriptor.multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(descriptor.fragment_entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &color_targets,
            }),
            multiview_mask: None,
            cache: None,
        });

        let quad_vertex_buffer =
            create_buffer_from_sprite_upload(device, queue, &uploads.quad_vertices);
        let quad_index_buffer =
            create_buffer_from_sprite_upload(device, queue, &uploads.quad_indices);
        let mut resources = Self {
            pipeline,
            projection_buffer,
            projection_bind_group,
            atlas_bind_group,
            quad_vertex_buffer,
            quad_index_buffer,
            instance_buffer: None,
            instance_buffer_size: 0,
        };
        resources.update_instances(device, queue, &uploads.instances);
        Ok(resources)
    }

    fn update_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        upload: &SpriteBufferUpload,
    ) {
        if upload.byte_len == 0 {
            self.instance_buffer = None;
            self.instance_buffer_size = 0;
            return;
        }
        if self.instance_buffer_size < upload.byte_len {
            self.instance_buffer = Some(create_empty_buffer(
                device,
                upload.label,
                upload.usage,
                upload.byte_len,
            ));
            self.instance_buffer_size = upload.byte_len;
        }
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, &upload.bytes);
        }
    }
}

#[cfg(all(not(test), not(coverage)))]
fn create_buffer(
    device: &wgpu::Device,
    upload: &crate::renderer::SceneProjectionUniformUpload,
) -> wgpu::Buffer {
    create_empty_buffer(device, upload.label, upload.usage, upload.byte_len)
}

#[cfg(all(not(test), not(coverage)))]
fn create_buffer_from_sprite_upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    upload: &SpriteBufferUpload,
) -> wgpu::Buffer {
    let buffer = create_empty_buffer(device, upload.label, upload.usage, upload.byte_len);
    queue.write_buffer(&buffer, 0, &upload.bytes);
    buffer
}

#[cfg(all(not(test), not(coverage)))]
fn create_empty_buffer(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    byte_len: wgpu::BufferAddress,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: byte_len.max(1),
        usage,
        mapped_at_creation: false,
    })
}

#[cfg(all(not(test), not(coverage)))]
fn encode_scene_render_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    plan: &SceneDrawPlan,
    sprite_resources: Option<&SpriteGpuResources>,
) {
    let color_attachment = Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(plan.gpu_pass.clear_color),
            store: wgpu::StoreOp::Store,
        },
    });
    let color_attachments = [color_attachment];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("defender clean sprite render pass"),
        color_attachments: &color_attachments,
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    if let Some(viewport) = plan.gpu_pass.viewport {
        pass.set_viewport(
            viewport.x,
            viewport.y,
            viewport.width,
            viewport.height,
            viewport.min_depth,
            viewport.max_depth,
        );
    }
    if let (Some(resources), Some(encoder_plan)) =
        (sprite_resources, &plan.sprite_render_pass_encoder)
    {
        encode_sprite_commands(&mut pass, resources, encoder_plan);
    }
}

#[cfg(all(not(test), not(coverage)))]
fn encode_sprite_commands<'pass>(
    pass: &mut wgpu::RenderPass<'pass>,
    resources: &'pass SpriteGpuResources,
    encoder_plan: &'pass crate::renderer::SpriteRenderPassEncoderPlan,
) {
    for command in &encoder_plan.commands {
        match command {
            SpriteRenderPassEncoderCommand::SetPipeline { .. } => {
                pass.set_pipeline(&resources.pipeline);
            }
            SpriteRenderPassEncoderCommand::SetBindGroup {
                role, group_index, ..
            } => {
                let bind_group = match role {
                    SpriteBindGroupRole::SceneProjection => &resources.projection_bind_group,
                    SpriteBindGroupRole::SpriteAtlas => &resources.atlas_bind_group,
                };
                pass.set_bind_group(*group_index, bind_group, &[]);
            }
            SpriteRenderPassEncoderCommand::SetVertexBuffer {
                role,
                slot,
                byte_offset,
                byte_len,
            } => match role {
                SpriteBufferRole::QuadVertices => pass.set_vertex_buffer(
                    *slot,
                    resources
                        .quad_vertex_buffer
                        .slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                ),
                SpriteBufferRole::Instances => {
                    if let Some(buffer) = &resources.instance_buffer {
                        pass.set_vertex_buffer(
                            *slot,
                            buffer.slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                        );
                    }
                }
                SpriteBufferRole::QuadIndices => {}
            },
            SpriteRenderPassEncoderCommand::SetIndexBuffer {
                index_format,
                byte_offset,
                byte_len,
                ..
            } => pass.set_index_buffer(
                resources
                    .quad_index_buffer
                    .slice(*byte_offset..byte_offset.saturating_add(*byte_len)),
                *index_format,
            ),
            SpriteRenderPassEncoderCommand::DrawIndexed { draw } => {
                pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instances.clone(),
                );
            }
        }
    }
}
