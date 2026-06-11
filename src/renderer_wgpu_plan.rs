const SPRITE_SHADER_SOURCE: &str = r#"
struct SceneProjection {
    scale: vec2<f32>,
    translate: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene_projection: SceneProjection;
@group(1) @binding(0) var sprite_atlas: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct SpriteInstance {
    @location(0) scene_origin: vec2<f32>,
    @location(1) scene_size: vec2<f32>,
    @location(2) atlas_uv_origin: vec2<f32>,
    @location(3) atlas_uv_size: vec2<f32>,
    @location(4) tint: vec4<f32>,
};

struct SpriteVertex {
    @location(5) unit_position: vec2<f32>,
    @location(6) unit_uv: vec2<f32>,
};

struct SpriteVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) atlas_uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn sprite_vs(instance: SpriteInstance, vertex: SpriteVertex) -> SpriteVertexOut {
    let scene_position = instance.scene_origin + vertex.unit_position * instance.scene_size;

    var out: SpriteVertexOut;
    out.position = vec4<f32>(
        scene_position * scene_projection.scale + scene_projection.translate,
        0.0,
        1.0,
    );
    out.atlas_uv = instance.atlas_uv_origin + vertex.unit_uv * instance.atlas_uv_size;
    out.tint = instance.tint;
    return out;
}

@fragment
fn sprite_fs(in: SpriteVertexOut) -> @location(0) vec4<f32> {
    return textureSample(sprite_atlas, sprite_sampler, in.atlas_uv) * in.tint;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteShaderPlan {
    pub label: &'static str,
    pub source: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
}

impl SpriteShaderPlan {
    pub fn shader_module_descriptor(&self) -> wgpu::ShaderModuleDescriptor<'static> {
        wgpu::ShaderModuleDescriptor {
            label: Some(self.label),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(self.source)),
        }
    }
}

impl Default for SpriteShaderPlan {
    fn default() -> Self {
        Self {
            label: "defender.sprite.shader",
            source: SPRITE_SHADER_SOURCE,
            vertex_entry: "sprite_vs",
            fragment_entry: "sprite_fs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteVertexBufferLayoutPlan {
    pub role: SpriteBufferRole,
    pub slot: u32,
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: &'static [wgpu::VertexAttribute],
}

impl SpriteVertexBufferLayoutPlan {
    fn quad_vertices() -> Self {
        let layout = SpriteQuadGeometry::vertex_buffer_layout();
        Self {
            role: SpriteBufferRole::QuadVertices,
            slot: SpriteVertexBufferBinding::QUAD_VERTEX_SLOT,
            array_stride: layout.array_stride,
            step_mode: layout.step_mode,
            attributes: layout.attributes,
        }
    }

    fn instances() -> Self {
        let layout = SpriteInstanceBufferRecord::vertex_buffer_layout();
        Self {
            role: SpriteBufferRole::Instances,
            slot: SpriteVertexBufferBinding::INSTANCE_SLOT,
            array_stride: layout.array_stride,
            step_mode: layout.step_mode,
            attributes: layout.attributes,
        }
    }

    pub const fn vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: self.attributes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpritePipelinePlan {
    pub label: &'static str,
    pub shader: SpriteShaderPlan,
    pub vertex_buffers: [SpriteVertexBufferLayoutPlan; 2],
    pub primitive: wgpu::PrimitiveState,
    pub color_target: wgpu::ColorTargetState,
    pub multisample: wgpu::MultisampleState,
}

impl SpritePipelinePlan {
    fn for_settings(settings: GpuRendererSettings) -> Self {
        Self {
            label: "defender.sprite.pipeline",
            shader: SpriteShaderPlan::default(),
            vertex_buffers: [
                SpriteVertexBufferLayoutPlan::quad_vertices(),
                SpriteVertexBufferLayoutPlan::instances(),
            ],
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..wgpu::PrimitiveState::default()
            },
            color_target: wgpu::ColorTargetState {
                format: settings.texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            },
            multisample: wgpu::MultisampleState::default(),
        }
    }

    pub fn vertex_buffer_layouts(&self) -> [wgpu::VertexBufferLayout<'static>; 2] {
        self.vertex_buffers
            .map(|buffer| buffer.vertex_buffer_layout())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteBindGroupRole {
    SceneProjection,
    SpriteAtlas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteResourceBindingRole {
    SceneProjectionUniform,
    SpriteAtlasTexture,
    SpriteAtlasSampler,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneProjectionUniformUpload {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub usage: wgpu::BufferUsages,
    pub byte_len: wgpu::BufferAddress,
    pub bytes: Vec<u8>,
}

impl SceneProjectionUniformUpload {
    fn from_projection(projection: SceneProjectionUniforms) -> Self {
        let bytes = projection.as_bytes().to_vec();
        Self {
            role: SpriteResourceBindingRole::SceneProjectionUniform,
            label: "defender.sprite.scene_projection.uniform",
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBindGroupLayoutPlan {
    pub role: SpriteBindGroupRole,
    pub label: &'static str,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl SpriteBindGroupLayoutPlan {
    pub const SCENE_PROJECTION_GROUP: u32 = 0;
    pub const SPRITE_ATLAS_GROUP: u32 = 1;
    pub const SCENE_PROJECTION_BINDING: u32 = 0;
    pub const ATLAS_TEXTURE_BINDING: u32 = 0;
    pub const ATLAS_SAMPLER_BINDING: u32 = 1;

    fn scene_projection() -> Self {
        Self {
            role: SpriteBindGroupRole::SceneProjection,
            label: "defender.sprite.scene_projection.bind_group_layout",
            entries: vec![wgpu::BindGroupLayoutEntry {
                binding: Self::SCENE_PROJECTION_BINDING,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(SceneProjectionUniforms::BYTE_SIZE),
                },
                count: None,
            }],
        }
    }

    fn sprite_atlas() -> Self {
        Self {
            role: SpriteBindGroupRole::SpriteAtlas,
            label: "defender.sprite.atlas.bind_group_layout",
            entries: vec![
                wgpu::BindGroupLayoutEntry {
                    binding: Self::ATLAS_TEXTURE_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::ATLAS_SAMPLER_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }
    }

    pub const fn group_index(&self) -> u32 {
        match self.role {
            SpriteBindGroupRole::SceneProjection => Self::SCENE_PROJECTION_GROUP,
            SpriteBindGroupRole::SpriteAtlas => Self::SPRITE_ATLAS_GROUP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteTextureBindingPlan {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
    pub multisampled: bool,
    pub surface: SurfaceSize,
}

impl SpriteTextureBindingPlan {
    fn atlas(surface: SurfaceSize) -> Self {
        Self {
            role: SpriteResourceBindingRole::SpriteAtlasTexture,
            label: "defender.sprite.atlas.texture_view",
            binding: SpriteBindGroupLayoutPlan::ATLAS_TEXTURE_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
            surface,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteSamplerBindingPlan {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub sampler_binding: wgpu::SamplerBindingType,
}

impl SpriteSamplerBindingPlan {
    fn atlas() -> Self {
        Self {
            role: SpriteResourceBindingRole::SpriteAtlasSampler,
            label: "defender.sprite.atlas.sampler",
            binding: SpriteBindGroupLayoutPlan::ATLAS_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            sampler_binding: wgpu::SamplerBindingType::Filtering,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteAtlasTextureUpload {
    pub role: SpriteResourceBindingRole,
    pub label: &'static str,
    pub usage: wgpu::TextureUsages,
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureDimension,
    pub surface: SurfaceSize,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub depth_or_array_layers: u32,
    pub bytes_per_row: u32,
    pub rows_per_image: u32,
    pub byte_len: usize,
    pub bytes: Vec<u8>,
    pub non_blank: bool,
}

impl SpriteAtlasTextureUpload {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const MIP_LEVEL_COUNT: u32 = 1;
    pub const SAMPLE_COUNT: u32 = 1;
    pub const DEPTH_OR_ARRAY_LAYERS: u32 = 1;

    fn from_atlas(atlas: &TextureAtlas) -> Option<Self> {
        if atlas.surface.is_empty() {
            return None;
        }
        if atlas.pixels.is_empty() || atlas.pixels.len() != atlas.surface.rgba_len()? {
            return None;
        }

        Some(Self {
            role: SpriteResourceBindingRole::SpriteAtlasTexture,
            label: "defender.sprite.atlas.texture",
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            format: Self::FORMAT,
            dimension: wgpu::TextureDimension::D2,
            surface: atlas.surface,
            mip_level_count: Self::MIP_LEVEL_COUNT,
            sample_count: Self::SAMPLE_COUNT,
            depth_or_array_layers: Self::DEPTH_OR_ARRAY_LAYERS,
            bytes_per_row: atlas.surface.width.checked_mul(4)?,
            rows_per_image: atlas.surface.height,
            byte_len: atlas.pixels.len(),
            bytes: atlas.pixels.clone(),
            non_blank: atlas.is_non_blank(),
        })
    }

    pub const fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.surface.width,
            height: self.surface.height,
            depth_or_array_layers: self.depth_or_array_layers,
        }
    }

    pub fn copy_layout(&self) -> wgpu::TexelCopyBufferLayout {
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(self.bytes_per_row),
            rows_per_image: Some(self.rows_per_image),
        }
    }

    pub fn texture_descriptor(&self) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(self.label),
            size: self.extent(),
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteResourceBindingPlan {
    pub atlas_upload: SpriteAtlasTextureUpload,
    pub projection_upload: SceneProjectionUniformUpload,
    pub projection_layout: SpriteBindGroupLayoutPlan,
    pub atlas_layout: SpriteBindGroupLayoutPlan,
    pub atlas_texture: SpriteTextureBindingPlan,
    pub atlas_sampler: SpriteSamplerBindingPlan,
}

impl SpriteResourceBindingPlan {
    pub const BIND_GROUP_COUNT: usize = 2;
    pub const BINDING_ENTRY_COUNT: usize = 3;

    fn from_projection_and_atlas(
        projection: SceneProjectionUniforms,
        atlas: &TextureAtlas,
    ) -> Option<Self> {
        let atlas_upload = SpriteAtlasTextureUpload::from_atlas(atlas)?;

        Some(Self {
            atlas_upload,
            projection_upload: SceneProjectionUniformUpload::from_projection(projection),
            projection_layout: SpriteBindGroupLayoutPlan::scene_projection(),
            atlas_layout: SpriteBindGroupLayoutPlan::sprite_atlas(),
            atlas_texture: SpriteTextureBindingPlan::atlas(atlas.surface),
            atlas_sampler: SpriteSamplerBindingPlan::atlas(),
        })
    }

    pub fn bind_group_count(&self) -> usize {
        Self::BIND_GROUP_COUNT
    }

    pub fn binding_entry_count(&self) -> usize {
        self.projection_layout.entries.len() + self.atlas_layout.entries.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpritePipelineLayoutBindGroup {
    pub role: SpriteBindGroupRole,
    pub group_index: u32,
    pub layout_label: &'static str,
    pub entry_count: usize,
}

impl SpritePipelineLayoutBindGroup {
    fn from_layout(layout: &SpriteBindGroupLayoutPlan) -> Self {
        Self {
            role: layout.role,
            group_index: layout.group_index(),
            layout_label: layout.label,
            entry_count: layout.entries.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpritePipelineLayoutPlan {
    pub label: &'static str,
    pub bind_groups: Vec<SpritePipelineLayoutBindGroup>,
    pub immediate_size: u32,
}

impl SpritePipelineLayoutPlan {
    pub const BIND_GROUP_COUNT: usize = SpriteResourceBindingPlan::BIND_GROUP_COUNT;
    pub const BINDING_ENTRY_COUNT: usize = SpriteResourceBindingPlan::BINDING_ENTRY_COUNT;
    pub const IMMEDIATE_SIZE: u32 = 0;

    fn from_resource_bindings(bindings: &SpriteResourceBindingPlan) -> Self {
        Self {
            label: "defender.sprite.pipeline_layout",
            bind_groups: vec![
                SpritePipelineLayoutBindGroup::from_layout(&bindings.projection_layout),
                SpritePipelineLayoutBindGroup::from_layout(&bindings.atlas_layout),
            ],
            immediate_size: Self::IMMEDIATE_SIZE,
        }
    }

    pub fn bind_group_count(&self) -> usize {
        self.bind_groups.len()
    }

    pub fn binding_entry_count(&self) -> usize {
        self.bind_groups
            .iter()
            .map(|bind_group| bind_group.entry_count)
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteRenderPipelineDescriptorPlan {
    pub label: &'static str,
    pub layout_label: &'static str,
    pub layout_bind_group_count: usize,
    pub immediate_size: u32,
    pub shader_label: &'static str,
    pub vertex_entry: &'static str,
    pub fragment_entry: &'static str,
    pub vertex_buffers: [SpriteVertexBufferLayoutPlan; 2],
    pub primitive: wgpu::PrimitiveState,
    pub color_target: wgpu::ColorTargetState,
    pub multisample: wgpu::MultisampleState,
}

impl SpriteRenderPipelineDescriptorPlan {
    pub const LAYOUT_BIND_GROUP_COUNT: usize = SpritePipelineLayoutPlan::BIND_GROUP_COUNT;
    pub const VERTEX_BUFFER_COUNT: usize = 2;
    pub const COLOR_TARGET_COUNT: usize = 1;

    fn from_pipeline_and_layout(
        pipeline: &SpritePipelinePlan,
        layout: &SpritePipelineLayoutPlan,
    ) -> Self {
        Self {
            label: pipeline.label,
            layout_label: layout.label,
            layout_bind_group_count: layout.bind_group_count(),
            immediate_size: layout.immediate_size,
            shader_label: pipeline.shader.label,
            vertex_entry: pipeline.shader.vertex_entry,
            fragment_entry: pipeline.shader.fragment_entry,
            vertex_buffers: pipeline.vertex_buffers,
            primitive: pipeline.primitive,
            color_target: pipeline.color_target.clone(),
            multisample: pipeline.multisample,
        }
    }

    pub fn vertex_buffer_layouts(&self) -> [wgpu::VertexBufferLayout<'static>; 2] {
        self.vertex_buffers
            .map(|buffer| buffer.vertex_buffer_layout())
    }

    pub fn layout_bind_group_count(&self) -> usize {
        self.layout_bind_group_count
    }

    pub fn vertex_buffer_count(&self) -> usize {
        self.vertex_buffers.len()
    }

    pub fn color_target_count(&self) -> usize {
        Self::COLOR_TARGET_COUNT
    }

    pub fn color_targets(&self) -> [Option<wgpu::ColorTargetState>; 1] {
        [Some(self.color_target.clone())]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteRenderPassEncoderCommand {
    SetPipeline {
        label: &'static str,
    },
    SetBindGroup {
        role: SpriteBindGroupRole,
        group_index: u32,
        layout_label: &'static str,
    },
    SetVertexBuffer {
        role: SpriteBufferRole,
        slot: u32,
        byte_offset: wgpu::BufferAddress,
        byte_len: wgpu::BufferAddress,
    },
    SetIndexBuffer {
        role: SpriteBufferRole,
        index_format: wgpu::IndexFormat,
        byte_offset: wgpu::BufferAddress,
        byte_len: wgpu::BufferAddress,
    },
    DrawIndexed {
        draw: SpriteRenderPassDraw,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassEncoderPlan {
    pub label: &'static str,
    pub commands: Vec<SpriteRenderPassEncoderCommand>,
}

impl SpriteRenderPassEncoderPlan {
    pub const SET_PIPELINE_COMMAND_COUNT: usize = 1;
    pub const SET_BIND_GROUP_COMMAND_COUNT: usize = SpritePipelineLayoutPlan::BIND_GROUP_COUNT;
    pub const SET_VERTEX_BUFFER_COMMAND_COUNT: usize =
        SpriteRenderPipelineDescriptorPlan::VERTEX_BUFFER_COUNT;
    pub const SET_INDEX_BUFFER_COMMAND_COUNT: usize = 1;

    fn from_render_pass_layout_and_descriptor(
        render_pass: &SpriteRenderPassPlan,
        layout: &SpritePipelineLayoutPlan,
        descriptor: &SpriteRenderPipelineDescriptorPlan,
    ) -> Self {
        let mut commands =
            Vec::with_capacity(1 + layout.bind_group_count() + 3 + render_pass.draws.len());
        commands.push(SpriteRenderPassEncoderCommand::SetPipeline {
            label: descriptor.label,
        });
        commands.extend(layout.bind_groups.iter().map(|bind_group| {
            SpriteRenderPassEncoderCommand::SetBindGroup {
                role: bind_group.role,
                group_index: bind_group.group_index,
                layout_label: bind_group.layout_label,
            }
        }));
        commands.push(SpriteRenderPassEncoderCommand::SetVertexBuffer {
            role: render_pass.quad_vertices.role,
            slot: render_pass.quad_vertices.slot,
            byte_offset: render_pass.quad_vertices.byte_offset,
            byte_len: render_pass.quad_vertices.byte_len,
        });
        commands.push(SpriteRenderPassEncoderCommand::SetVertexBuffer {
            role: render_pass.instances.role,
            slot: render_pass.instances.slot,
            byte_offset: render_pass.instances.byte_offset,
            byte_len: render_pass.instances.byte_len,
        });
        commands.push(SpriteRenderPassEncoderCommand::SetIndexBuffer {
            role: render_pass.indices.role,
            index_format: render_pass.indices.index_format,
            byte_offset: render_pass.indices.byte_offset,
            byte_len: render_pass.indices.byte_len,
        });
        commands.extend(
            render_pass
                .draws
                .iter()
                .cloned()
                .map(|draw| SpriteRenderPassEncoderCommand::DrawIndexed { draw }),
        );

        Self {
            label: "defender.sprite.render_pass.encoder",
            commands,
        }
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn set_pipeline_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, SpriteRenderPassEncoderCommand::SetPipeline { .. }))
            .count()
    }

    pub fn set_bind_group_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(command, SpriteRenderPassEncoderCommand::SetBindGroup { .. })
            })
            .count()
    }

    pub fn set_vertex_buffer_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    SpriteRenderPassEncoderCommand::SetVertexBuffer { .. }
                )
            })
            .count()
    }

    pub fn set_index_buffer_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    SpriteRenderPassEncoderCommand::SetIndexBuffer { .. }
                )
            })
            .count()
    }

    pub fn draw_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, SpriteRenderPassEncoderCommand::DrawIndexed { .. }))
            .count()
    }

    pub fn instance_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                SpriteRenderPassEncoderCommand::DrawIndexed { draw } => {
                    (draw.instances.end - draw.instances.start) as usize
                }
                _ => 0,
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WgpuFrameCommand {
    BeginRenderPass {
        clear_color: wgpu::Color,
    },
    SetViewport {
        viewport: WgpuViewportCommand,
    },
    UploadSceneProjection {
        byte_len: wgpu::BufferAddress,
    },
    UploadTemporaryRaster {
        upload: SceneRasterUpload,
    },
    ExecuteSpriteRenderPass {
        encoder_label: &'static str,
        command_count: usize,
        draw_count: usize,
        instance_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WgpuFramePlan {
    pub label: &'static str,
    pub commands: Vec<WgpuFrameCommand>,
}

impl WgpuFramePlan {
    fn from_pass_raster_and_sprite_encoder(
        pass: &WgpuPassPlan,
        raster_upload: Option<SceneRasterUpload>,
        sprite_encoder: Option<&SpriteRenderPassEncoderPlan>,
    ) -> Self {
        let mut commands = Vec::new();
        commands.push(WgpuFrameCommand::BeginRenderPass {
            clear_color: pass.clear_color,
        });
        if let Some(viewport) = pass.viewport {
            commands.push(WgpuFrameCommand::SetViewport { viewport });
        }
        if let Some(projection) = pass.scene_projection {
            commands.push(WgpuFrameCommand::UploadSceneProjection {
                byte_len: projection.as_bytes().len() as wgpu::BufferAddress,
            });
        }
        if let Some(upload) = raster_upload {
            commands.push(WgpuFrameCommand::UploadTemporaryRaster { upload });
        }
        if let Some(encoder) = sprite_encoder {
            commands.push(WgpuFrameCommand::ExecuteSpriteRenderPass {
                encoder_label: encoder.label,
                command_count: encoder.command_count(),
                draw_count: encoder.draw_count(),
                instance_count: encoder.instance_count(),
            });
        }

        Self {
            label: "defender.render.commands",
            commands,
        }
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn has_ordered_sprite_only_commands(&self) -> bool {
        matches!(
            self.commands.as_slice(),
            [
                WgpuFrameCommand::BeginRenderPass { .. },
                WgpuFrameCommand::SetViewport { .. },
                WgpuFrameCommand::UploadSceneProjection { .. },
                WgpuFrameCommand::ExecuteSpriteRenderPass { .. },
            ]
        )
    }

    pub fn sprite_pass_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::ExecuteSpriteRenderPass { .. }))
            .count()
    }

    pub fn temporary_raster_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::UploadTemporaryRaster { .. }))
            .count()
    }

    pub fn begin_render_pass_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::BeginRenderPass { .. }))
            .count()
    }

    pub fn viewport_command_count(&self) -> usize {
        self.commands
            .iter()
            .filter(|command| matches!(command, WgpuFrameCommand::SetViewport { .. }))
            .count()
    }

    pub fn scene_projection_upload_byte_len(&self) -> wgpu::BufferAddress {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::UploadSceneProjection { byte_len } => *byte_len,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_encoder_command_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { command_count, .. } => *command_count,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_draw_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { draw_count, .. } => *draw_count,
                _ => 0,
            })
            .sum()
    }

    pub fn sprite_instance_count(&self) -> usize {
        self.commands
            .iter()
            .map(|command| match command {
                WgpuFrameCommand::ExecuteSpriteRenderPass { instance_count, .. } => *instance_count,
                _ => 0,
            })
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteDrawCommand {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_format: wgpu::IndexFormat,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
    pub instance_count: u32,
    pub vertex_buffer_byte_len: wgpu::BufferAddress,
    pub index_buffer_byte_len: wgpu::BufferAddress,
    pub instance_buffer_byte_offset: wgpu::BufferAddress,
    pub instance_buffer_byte_len: wgpu::BufferAddress,
}

impl SpriteDrawCommand {
    fn from_instance_buffer(buffer: &SpriteInstanceBuffer, first_instance: u32) -> Option<Self> {
        if buffer.records.is_empty() {
            return None;
        }

        let instance_count = u32::try_from(buffer.records.len()).ok()?;
        let instance_buffer_byte_offset =
            u64::from(first_instance) * SpriteInstanceBufferRecord::BYTE_SIZE;
        let instance_buffer_byte_len =
            u64::from(instance_count) * SpriteInstanceBufferRecord::BYTE_SIZE;

        Some(Self {
            pipeline: buffer.pipeline,
            layer: buffer.layer,
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
            instance_buffer_byte_offset,
            instance_buffer_byte_len,
        })
    }
}

fn sprite_draw_commands_from_instance_buffers(
    buffers: &[SpriteInstanceBuffer],
) -> Vec<SpriteDrawCommand> {
    let mut first_instance = 0;
    let mut commands = Vec::new();

    for buffer in buffers {
        let Some(command) = SpriteDrawCommand::from_instance_buffer(buffer, first_instance) else {
            continue;
        };
        commands.push(command);
        first_instance += command.instance_count;
    }

    commands
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDrawPlan {
    pub step: u64,
    pub surface: SurfaceSize,
    pub viewport: ViewportLayout,
    pub gpu_pass: WgpuPassPlan,
    pub frame_plan: WgpuFramePlan,
    pub pipelines: Vec<NativeRenderPipeline>,
    pub sprite_instances: usize,
    pub missing_sprite_regions: usize,
    pub sprite_batches: Vec<SpriteDrawBatch>,
    pub sprite_instance_buffers: Vec<SpriteInstanceBuffer>,
    pub sprite_instance_upload: Option<SpriteInstanceUpload>,
    pub sprite_buffer_uploads: Option<SpriteBufferUploadPlan>,
    pub sprite_draw_commands: Vec<SpriteDrawCommand>,
    pub sprite_render_pass: Option<SpriteRenderPassPlan>,
    pub sprite_pipeline: Option<SpritePipelinePlan>,
    pub sprite_resource_bindings: Option<SpriteResourceBindingPlan>,
    pub sprite_pipeline_layout: Option<SpritePipelineLayoutPlan>,
    pub sprite_render_pipeline_descriptor: Option<SpriteRenderPipelineDescriptorPlan>,
    pub sprite_render_pass_encoder: Option<SpriteRenderPassEncoderPlan>,
    pub layer_counts: RenderLayerCounts,
    pub raster_upload: Option<SceneRasterUpload>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeSceneRenderer {
    pub resources: NativeRendererResources,
    pub settings: GpuRendererSettings,
}

impl NativeSceneRenderer {
    pub fn new(resources: NativeRendererResources) -> Self {
        Self {
            resources,
            settings: GpuRendererSettings::default(),
        }
    }

    pub fn with_settings(
        resources: NativeRendererResources,
        settings: GpuRendererSettings,
    ) -> Self {
        Self {
            resources,
            settings,
        }
    }

    pub fn prepare(&self, scene: &RenderScene) -> SceneDrawPlan {
        self.prepare_for_target(scene, scene.surface)
    }

    pub fn prepare_for_target(&self, scene: &RenderScene, target: SurfaceSize) -> SceneDrawPlan {
        let mut requested = BTreeSet::new();
        if scene.raster.is_some() {
            requested.insert(NativeRenderPipeline::TemporaryRaster);
        }

        let mut layer_counts = RenderLayerCounts::default();
        let mut missing_sprite_regions = 0;
        let mut sprite_batches = Vec::new();
        for sprite in &scene.sprites {
            layer_counts.add(sprite.layer);
            let pipeline = pipeline_for_layer(sprite.layer);
            let Some(region) = self.resources.atlas.region(sprite.sprite) else {
                missing_sprite_regions += 1;
                continue;
            };
            if self.resources.pipelines.contains(&pipeline) {
                requested.insert(pipeline);
                push_sprite_instance(
                    &mut sprite_batches,
                    pipeline,
                    sprite.layer,
                    SpriteDrawInstance::from_sprite(*sprite, region),
                );
            }
        }

        let pipelines = requested
            .into_iter()
            .filter(|pipeline| self.resources.pipelines.contains(pipeline))
            .collect();
        let sprite_instances = sprite_batches
            .iter()
            .map(|batch: &SpriteDrawBatch| batch.instances.len())
            .sum();
        let sprite_instance_buffers = sprite_batches
            .iter()
            .filter_map(|batch| {
                SpriteInstanceBuffer::from_batch(batch, self.resources.atlas.surface)
            })
            .collect::<Vec<_>>();
        let sprite_instance_upload =
            SpriteInstanceUpload::from_instance_buffers(&sprite_instance_buffers);
        let sprite_buffer_uploads = sprite_instance_upload
            .as_ref()
            .map(SpriteBufferUploadPlan::from_instance_upload);
        let sprite_draw_commands =
            sprite_draw_commands_from_instance_buffers(&sprite_instance_buffers);
        let sprite_render_pass = sprite_buffer_uploads.as_ref().and_then(|uploads| {
            SpriteRenderPassPlan::from_uploads_and_commands(uploads, &sprite_draw_commands)
        });
        let sprite_pipeline = sprite_render_pass
            .as_ref()
            .map(|_| SpritePipelinePlan::for_settings(self.settings));
        let viewport = ViewportLayout::fit(scene.surface, target);
        let gpu_pass = WgpuPassPlan::from_scene(scene, viewport);
        let sprite_resource_bindings = sprite_pipeline.as_ref().and_then(|_| {
            gpu_pass.scene_projection.and_then(|projection| {
                SpriteResourceBindingPlan::from_projection_and_atlas(
                    projection,
                    &self.resources.atlas,
                )
            })
        });
        let sprite_pipeline_layout = match (
            sprite_pipeline.as_ref(),
            sprite_resource_bindings.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(_), Some(bindings), Some(_)) => {
                Some(SpritePipelineLayoutPlan::from_resource_bindings(bindings))
            }
            _ => None,
        };
        let sprite_render_pipeline_descriptor = match (
            sprite_render_pass.as_ref(),
            sprite_pipeline.as_ref(),
            sprite_resource_bindings.as_ref(),
            sprite_pipeline_layout.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(_), Some(pipeline), Some(_), Some(layout), Some(_)) => {
                Some(SpriteRenderPipelineDescriptorPlan::from_pipeline_and_layout(pipeline, layout))
            }
            _ => None,
        };
        let sprite_render_pass_encoder = match (
            sprite_render_pass.as_ref(),
            sprite_resource_bindings.as_ref(),
            sprite_pipeline_layout.as_ref(),
            sprite_render_pipeline_descriptor.as_ref(),
            gpu_pass.viewport,
        ) {
            (Some(render_pass), Some(_), Some(layout), Some(descriptor), Some(_)) => Some(
                SpriteRenderPassEncoderPlan::from_render_pass_layout_and_descriptor(
                    render_pass,
                    layout,
                    descriptor,
                ),
            ),
            _ => None,
        };
        let raster_upload = scene.raster.as_ref().map(|raster| SceneRasterUpload {
            surface: raster.surface,
            byte_len: raster.pixels.len(),
            visual_signature: scene.visual_signature,
            non_blank: raster.is_non_blank(),
        });
        let frame_plan = WgpuFramePlan::from_pass_raster_and_sprite_encoder(
            &gpu_pass,
            raster_upload,
            sprite_render_pass_encoder.as_ref(),
        );

        SceneDrawPlan {
            step: scene.step,
            surface: scene.surface,
            viewport,
            gpu_pass,
            frame_plan,
            pipelines,
            sprite_instances,
            missing_sprite_regions,
            sprite_batches,
            sprite_instance_buffers,
            sprite_instance_upload,
            sprite_buffer_uploads,
            sprite_draw_commands,
            sprite_render_pass,
            sprite_pipeline,
            sprite_resource_bindings,
            sprite_pipeline_layout,
            sprite_render_pipeline_descriptor,
            sprite_render_pass_encoder,
            layer_counts,
            raster_upload,
        }
    }
}

fn push_sprite_instance(
    batches: &mut Vec<SpriteDrawBatch>,
    pipeline: NativeRenderPipeline,
    layer: RenderLayer,
    instance: SpriteDrawInstance,
) {
    if let Some(batch) = batches
        .iter_mut()
        .find(|batch| batch.pipeline == pipeline && batch.layer == layer)
    {
        batch.instances.push(instance);
        return;
    }

    batches.push(SpriteDrawBatch {
        pipeline,
        layer,
        instances: vec![instance],
    });
}

fn pipeline_for_layer(layer: RenderLayer) -> NativeRenderPipeline {
    match layer {
        RenderLayer::Terrain => NativeRenderPipeline::Terrain,
        RenderLayer::Starfield => NativeRenderPipeline::Starfield,
        RenderLayer::Objects => NativeRenderPipeline::Sprites,
        RenderLayer::Projectiles => NativeRenderPipeline::Projectiles,
        RenderLayer::Hud => NativeRenderPipeline::HudText,
        RenderLayer::Overlay => NativeRenderPipeline::DebugOverlay,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuRendererSettings {
    pub texture_format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
}

impl Default for GpuRendererSettings {
    fn default() -> Self {
        Self {
            texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        }
    }
}
