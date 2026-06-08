#[cfg(test)]
mod tests {
    use super::{
        ATTRACT_DEFENDER_WORDMARK_BLOCK_COUNT, AtlasRegion, Color, GpuRendererSettings,
        NativeRenderPipeline, NativeRendererResources, NativeSceneRenderer, ObjectPicturePalette,
        PALE_YELLOW_RGBA, PICTURE_COLOR_TABLE, PURPLE_RGBA, RenderLayer, RenderLayerCounts,
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
        decode_object_picture_asset_rgba, pseudo_color_rgba, push_source_controlled_message_sprites,
        push_source_text_bytes_sprites, render_scene_with_atlas_to_rgba, source_message_text,
        source_screen_position, source_screen_position_with_offset,
    };
    use crate::renderer::{
        EmbeddedSprite, WHITE_RGBA, decode_source_attract_williams_logo_rgba,
        source_attract_williams_logo_operation_pixel_counts,
        source_attract_williams_logo_pixel_path,
    };

    include!("renderer_tests_core.rs");
    include!("renderer_tests_gpu_plans.rs");
    include!("renderer_tests_scene_plans.rs");
}
