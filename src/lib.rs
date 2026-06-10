//! Defender actor-runtime implementation.
//!
//! The accepted runtime is a clean Rust implementation backed by actor-owned
//! simulation, `wgpu` rendering, and synthesized audio.

pub mod actor_game;
mod actor_smoke;
pub mod audio;
mod game;
mod live_wgpu;
pub mod platform;
mod reference_assets;
pub mod renderer;
mod runtime;
mod sound_board;
pub mod systems;
pub mod typed_values;

pub use game::{
    AttractPresentationPage, AttractPresentationSnapshot, Direction, EnemyKind,
    EnemyReserveSnapshot, EnemySnapshot, GameEvent, GameEvents, GameFrame, GameInput,
    GameOverSnapshot, GamePhase, GameState, HighScoreEntrySnapshot, HighScoreSubmissionSnapshot,
    HighScoreTableEntrySnapshot, HighScoreTablesSnapshot, HumanSnapshot,
    PlayerExplosionCloudSnapshot, PlayerExplosionPieceSnapshot, PlayerSnapshot,
    PlayerStockSnapshot, ProjectileSnapshot, ScoreSnapshot, SoundEvent, TerrainBlowSnapshot,
    TerrainBlowStage, TerrainSegment, WaveProfileSnapshot, WorldSnapshot, WorldVector,
};
pub use platform::{AudioOutput, ControlProfile, RunMode, RuntimeConfig};
pub use reference_assets::MessageId;
pub use renderer::{
    AtlasRegion, Color, FontAtlas, GpuRendererSettings, NativeRenderPipeline,
    NativeRendererResources, NativeSceneRenderer, PaletteResource, RenderLayer, RenderLayerCounts,
    RenderScene, RenderSceneSummary, SceneDrawPlan, SceneProjectionUniformUpload,
    SceneProjectionUniforms, SceneRaster, SceneRasterError, SceneRasterUpload, SceneSprite,
    SpriteAtlasTextureUpload, SpriteBindGroupLayoutPlan, SpriteBindGroupRole, SpriteBufferRole,
    SpriteBufferUpload, SpriteBufferUploadPlan, SpriteDrawBatch, SpriteDrawCommand,
    SpriteDrawInstance, SpriteId, SpriteIndexBufferBinding, SpriteInstanceBuffer,
    SpriteInstanceBufferRecord, SpriteInstanceUpload, SpritePipelineLayoutBindGroup,
    SpritePipelineLayoutPlan, SpritePipelinePlan, SpriteQuadGeometry, SpriteQuadVertex,
    SpriteRenderPassDraw, SpriteRenderPassEncoderCommand, SpriteRenderPassEncoderPlan,
    SpriteRenderPassPlan, SpriteRenderPipelineDescriptorPlan, SpriteResourceBindingPlan,
    SpriteResourceBindingRole, SpriteSamplerBindingPlan, SpriteShaderPlan,
    SpriteTextureBindingPlan, SpriteVertexBufferBinding, SpriteVertexBufferLayoutPlan, SurfaceSize,
    TextureAtlas, ViewportLayout, WgpuFrameCommand, WgpuFramePlan, WgpuPassPlan,
    WgpuViewportCommand, render_scene_to_rgba,
};
pub use systems::{
    CollisionBox, CollisionSystem, EnemyMotionStep, EnemyMotionSystem, FixedStepAccumulator,
    FrameRate, HighScoreEntryStep, HighScoreEntrySystem, HighScoreInitialsState,
    HighScoreInitialsStep, OperatorActionTriggers, OperatorControlStep, OperatorControlSystem,
    PlayerActionTriggers, PlayerControlIntent, PlayerControlStep, PlayerControlSystem,
    PlayerDamageStep, PlayerDamageSystem, PlayerEnemyHit, PlayerMotionState, PlayerMotionStep,
    PlayerMotionSystem, PlayerStock, ProjectileEnemyHit, ProjectileLaunchOutcome,
    ProjectileMotionStep, ProjectileMotionSystem, ProjectileState, ProjectileSystem, ScoreStep,
    ScoreSystem, ScreenPosition, ScreenVelocity, SmartBombStep, SmartBombSystem, VerticalControl,
    WaveState, WaveStatus, WaveSystem,
};
pub use typed_values::{ScreenAddress, SoundCommand, SpriteFrameIndex, TimelineStep};

#[cfg(test)]
mod public_api_tests {
    #[test]
    fn public_actor_runtime_advances_from_attract() {
        let mut game = crate::actor_game::ActorRuntimeAdapter::new();
        let frame = game.step(crate::actor_game::GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, crate::GamePhase::Attract);
        assert!(frame.scene.summary().sprite_count > 0);
    }

    #[test]
    fn binary_entrypoint_uses_platform_runtime_boundary() {
        let main_rs = include_str!("main.rs");

        assert!(main_rs.contains("defender::platform::run()"));
    }

    #[test]
    fn retired_conversion_tree_is_removed() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

        assert!(!manifest_dir.join("src_legacy").exists());
        assert!(!manifest_dir.join("oldsrc").exists());
    }
}
