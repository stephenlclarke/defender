#[cfg(test)]
mod tests {
    use crate::arcade_assets::{MessageId, ObjectBitmapId, message_text};
    use super::{
        ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT, AtlasRegion, Color, GpuRendererSettings,
        NativeRenderPipeline, NativeRendererResources, NativeSceneRenderer, ObjectBitmapPalette,
        PALE_YELLOW_RGBA, OBJECT_BITMAP_COLOR_TABLE, PURPLE_RGBA, RenderLayer, RenderLayerCounts,
        RenderScene, SceneDrawPlan, SceneProjectionUniformUpload, SceneProjectionUniforms,
        SceneRaster, SceneRasterError, SceneRasterUpload, SceneSprite, SpriteAtlasTextureUpload,
        SpriteBindGroupLayoutPlan, SpriteBindGroupRole, SpriteBufferRole, SpriteBufferUpload,
        SpriteBufferUploadPlan, SpriteDrawBatch, SpriteDrawCommand, SpriteDrawInstance, SpriteId,
        SpriteIndexBufferBinding, SpriteInstanceBuffer, SpriteInstanceBufferRecord,
        SpriteInstanceUpload, SpritePipelineLayoutBindGroup, SpritePipelineLayoutPlan,
        SpritePipelinePlan, SpriteQuadGeometry, SpriteQuadVertex, SpriteRenderPassDraw,
        SpriteRenderPassEncoderCommand, SpriteRenderPassEncoderPlan, SpriteRenderPassPlan,
        SpriteRenderPipelineDescriptorPlan, SpriteResourceBindingPlan, SpriteResourceBindingRole,
        SpriteSamplerBindingPlan, SpriteShaderPlan, SpriteTextureBindingPlan,
        SpriteVertexBufferBinding, SpriteVertexBufferLayoutPlan, SurfaceSize, TextureAtlas,
        ViewportLayout, WgpuFrameCommand, WgpuFramePlan, WgpuPassPlan, WgpuViewportCommand,
        decode_object_bitmap_asset_rgba, pseudo_color_rgba, push_arcade_controlled_message_sprites,
        push_message_text_bytes_sprites, render_scene_with_atlas_to_rgba, screen_position_from_cell,
        screen_position_from_cell_with_offset,
    };
    use crate::renderer::{
        EmbeddedSprite, WHITE_RGBA, attract_williams_logo_operation_pixel_counts,
        attract_williams_logo_pixel_path, decode_attract_williams_logo_rgba,
    };

    include!("renderer_tests_core.rs");
    include!("renderer_tests_gpu_plans.rs");
    include!("renderer_tests_scene_plans.rs");
}
