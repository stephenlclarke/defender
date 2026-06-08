#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteResource {
    pub colors: Vec<Color>,
}

impl PaletteResource {
    pub fn defender_default() -> Self {
        Self {
            colors: vec![
                Color {
                    rgba: [0, 0, 0, 255],
                },
                Color::WHITE,
                Color {
                    rgba: [217, 81, 255, 255],
                },
                Color {
                    rgba: [38, 174, 0, 255],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontAtlas {
    pub glyph_size: [u32; 2],
    pub glyph_count: u16,
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self {
            glyph_size: [8, 8],
            glyph_count: 96,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRendererResources {
    pub atlas: TextureAtlas,
    pub palette: PaletteResource,
    pub font: FontAtlas,
    pub pipelines: BTreeSet<NativeRenderPipeline>,
}

impl Default for NativeRendererResources {
    fn default() -> Self {
        let pipelines = [
            NativeRenderPipeline::TemporaryRaster,
            NativeRenderPipeline::Terrain,
            NativeRenderPipeline::Starfield,
            NativeRenderPipeline::Sprites,
            NativeRenderPipeline::Projectiles,
            NativeRenderPipeline::Explosions,
            NativeRenderPipeline::HudText,
            NativeRenderPipeline::DebugOverlay,
        ]
        .into_iter()
        .collect();

        Self {
            atlas: TextureAtlas::default_sprites(),
            palette: PaletteResource::defender_default(),
            font: FontAtlas::default(),
            pipelines,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRasterUpload {
    pub surface: SurfaceSize,
    pub byte_len: usize,
    pub visual_signature: Option<u32>,
    pub non_blank: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteQuadVertex {
    pub unit_position: [f32; 2],
    pub unit_uv: [f32; 2],
}

impl SpriteQuadVertex {
    pub const FLOAT_COMPONENTS: usize = 4;
    pub const BYTE_SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        5 => Float32x2,
        6 => Float32x2,
    ];

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::BYTE_SIZE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

const SPRITE_QUAD_VERTICES: [SpriteQuadVertex; 4] = [
    SpriteQuadVertex {
        unit_position: [0.0, 0.0],
        unit_uv: [0.0, 0.0],
    },
    SpriteQuadVertex {
        unit_position: [1.0, 0.0],
        unit_uv: [1.0, 0.0],
    },
    SpriteQuadVertex {
        unit_position: [0.0, 1.0],
        unit_uv: [0.0, 1.0],
    },
    SpriteQuadVertex {
        unit_position: [1.0, 1.0],
        unit_uv: [1.0, 1.0],
    },
];

const SPRITE_QUAD_INDICES: [u16; 6] = [0, 2, 1, 2, 3, 1];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteQuadGeometry;

impl SpriteQuadGeometry {
    pub const VERTICES: [SpriteQuadVertex; 4] = SPRITE_QUAD_VERTICES;
    pub const INDICES: [u16; 6] = SPRITE_QUAD_INDICES;
    pub const VERTEX_COUNT: u32 = SPRITE_QUAD_VERTICES.len() as u32;
    pub const INDEX_COUNT: u32 = SPRITE_QUAD_INDICES.len() as u32;
    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        SpriteQuadVertex::vertex_buffer_layout()
    }

    pub fn vertices() -> &'static [SpriteQuadVertex] {
        &SPRITE_QUAD_VERTICES
    }

    pub fn indices() -> &'static [u16] {
        &SPRITE_QUAD_INDICES
    }

    pub fn vertex_upload_bytes() -> &'static [u8] {
        bytemuck::cast_slice(&SPRITE_QUAD_VERTICES)
    }

    pub fn index_upload_bytes() -> &'static [u8] {
        bytemuck::cast_slice(&SPRITE_QUAD_INDICES)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteDrawInstance {
    pub sprite: SpriteId,
    pub atlas_origin: [u32; 2],
    pub atlas_size: [u32; 2],
    pub layer: RenderLayer,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub tint: Color,
}

impl SpriteDrawInstance {
    fn from_sprite(sprite: SceneSprite, region: AtlasRegion) -> Self {
        Self {
            sprite: sprite.sprite,
            atlas_origin: region.origin,
            atlas_size: region.size,
            layer: sprite.layer,
            position: sprite.position,
            size: sprite.size,
            tint: sprite.tint,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstanceBufferRecord {
    pub scene_origin: [f32; 2],
    pub scene_size: [f32; 2],
    pub atlas_uv_origin: [f32; 2],
    pub atlas_uv_size: [f32; 2],
    pub tint: [f32; 4],
}

impl SpriteInstanceBufferRecord {
    pub const FLOAT_COMPONENTS: usize = 12;
    pub const BYTE_SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x4,
    ];

    pub fn from_instance(instance: SpriteDrawInstance, atlas_surface: SurfaceSize) -> Option<Self> {
        if atlas_surface.is_empty() {
            return None;
        }

        Some(Self {
            scene_origin: instance.position,
            scene_size: instance.size,
            atlas_uv_origin: [
                instance.atlas_origin[0] as f32 / atlas_surface.width as f32,
                instance.atlas_origin[1] as f32 / atlas_surface.height as f32,
            ],
            atlas_uv_size: [
                instance.atlas_size[0] as f32 / atlas_surface.width as f32,
                instance.atlas_size[1] as f32 / atlas_surface.height as f32,
            ],
            tint: instance.tint.to_normalized_rgba(),
        })
    }

    pub const fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::BYTE_SIZE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteDrawBatch {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub instances: Vec<SpriteDrawInstance>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstanceBuffer {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub records: Vec<SpriteInstanceBufferRecord>,
}

impl SpriteInstanceBuffer {
    fn from_batch(batch: &SpriteDrawBatch, atlas_surface: SurfaceSize) -> Option<Self> {
        let records = batch
            .instances
            .iter()
            .copied()
            .filter_map(|instance| {
                SpriteInstanceBufferRecord::from_instance(instance, atlas_surface)
            })
            .collect::<Vec<_>>();

        if records.is_empty() {
            return None;
        }

        Some(Self {
            pipeline: batch.pipeline,
            layer: batch.layer,
            records,
        })
    }

    pub fn upload_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.records)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteInstanceUpload {
    pub records: Vec<SpriteInstanceBufferRecord>,
}

impl SpriteInstanceUpload {
    fn from_instance_buffers(buffers: &[SpriteInstanceBuffer]) -> Option<Self> {
        let records = buffers
            .iter()
            .flat_map(|buffer| buffer.records.iter().copied())
            .collect::<Vec<_>>();

        if records.is_empty() {
            return None;
        }

        Some(Self { records })
    }

    pub fn instance_count(&self) -> usize {
        self.records.len()
    }

    pub fn byte_len(&self) -> wgpu::BufferAddress {
        self.upload_bytes().len() as wgpu::BufferAddress
    }

    pub fn upload_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.records)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteBufferRole {
    QuadVertices,
    QuadIndices,
    Instances,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBufferUpload {
    pub role: SpriteBufferRole,
    pub label: &'static str,
    pub usage: wgpu::BufferUsages,
    pub byte_len: wgpu::BufferAddress,
    pub bytes: Vec<u8>,
}

impl SpriteBufferUpload {
    fn quad_vertices() -> Self {
        let bytes = SpriteQuadGeometry::vertex_upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::QuadVertices,
            label: "defender.sprite.quad.vertices",
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }

    fn quad_indices() -> Self {
        let bytes = SpriteQuadGeometry::index_upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::QuadIndices,
            label: "defender.sprite.quad.indices",
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }

    fn instances(upload: &SpriteInstanceUpload) -> Self {
        let bytes = upload.upload_bytes().to_vec();
        Self {
            role: SpriteBufferRole::Instances,
            label: "defender.sprite.instances",
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            byte_len: bytes.len() as wgpu::BufferAddress,
            bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteBufferUploadPlan {
    pub quad_vertices: SpriteBufferUpload,
    pub quad_indices: SpriteBufferUpload,
    pub instances: SpriteBufferUpload,
}

impl SpriteBufferUploadPlan {
    fn from_instance_upload(upload: &SpriteInstanceUpload) -> Self {
        Self {
            quad_vertices: SpriteBufferUpload::quad_vertices(),
            quad_indices: SpriteBufferUpload::quad_indices(),
            instances: SpriteBufferUpload::instances(upload),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteVertexBufferBinding {
    pub role: SpriteBufferRole,
    pub slot: u32,
    pub byte_offset: wgpu::BufferAddress,
    pub byte_len: wgpu::BufferAddress,
}

impl SpriteVertexBufferBinding {
    pub const QUAD_VERTEX_SLOT: u32 = 0;
    pub const INSTANCE_SLOT: u32 = 1;

    fn quad_vertices(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            slot: Self::QUAD_VERTEX_SLOT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }

    fn instances(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            slot: Self::INSTANCE_SLOT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteIndexBufferBinding {
    pub role: SpriteBufferRole,
    pub index_format: wgpu::IndexFormat,
    pub byte_offset: wgpu::BufferAddress,
    pub byte_len: wgpu::BufferAddress,
}

impl SpriteIndexBufferBinding {
    fn quad_indices(upload: &SpriteBufferUpload) -> Self {
        Self {
            role: upload.role,
            index_format: SpriteQuadGeometry::INDEX_FORMAT,
            byte_offset: 0,
            byte_len: upload.byte_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassDraw {
    pub pipeline: NativeRenderPipeline,
    pub layer: RenderLayer,
    pub indices: std::ops::Range<u32>,
    pub base_vertex: i32,
    pub instances: std::ops::Range<u32>,
    pub instance_buffer_byte_offset: wgpu::BufferAddress,
    pub instance_buffer_byte_len: wgpu::BufferAddress,
}

impl SpriteRenderPassDraw {
    fn from_command(command: SpriteDrawCommand) -> Self {
        Self {
            pipeline: command.pipeline,
            layer: command.layer,
            indices: command.first_index..command.first_index + command.index_count,
            base_vertex: command.base_vertex,
            instances: command.first_instance..command.first_instance + command.instance_count,
            instance_buffer_byte_offset: command.instance_buffer_byte_offset,
            instance_buffer_byte_len: command.instance_buffer_byte_len,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteRenderPassPlan {
    pub quad_vertices: SpriteVertexBufferBinding,
    pub instances: SpriteVertexBufferBinding,
    pub indices: SpriteIndexBufferBinding,
    pub draws: Vec<SpriteRenderPassDraw>,
}

impl SpriteRenderPassPlan {
    fn from_uploads_and_commands(
        uploads: &SpriteBufferUploadPlan,
        commands: &[SpriteDrawCommand],
    ) -> Option<Self> {
        if commands.is_empty() {
            return None;
        }

        Some(Self {
            quad_vertices: SpriteVertexBufferBinding::quad_vertices(&uploads.quad_vertices),
            instances: SpriteVertexBufferBinding::instances(&uploads.instances),
            indices: SpriteIndexBufferBinding::quad_indices(&uploads.quad_indices),
            draws: commands
                .iter()
                .copied()
                .map(SpriteRenderPassDraw::from_command)
                .collect(),
        })
    }

    pub fn draw_count(&self) -> usize {
        self.draws.len()
    }

    pub fn instance_count(&self) -> u32 {
        self.draws
            .iter()
            .map(|draw| draw.instances.end - draw.instances.start)
            .sum()
    }
}
