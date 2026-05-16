//! Clean gameplay smoke runner.
//!
//! This command path exercises the domain game and native draw planning without
//! entering the windowed live presenter.

use std::collections::BTreeSet;

use anyhow::bail;

use crate::{
    Game, GameFrame, GameInput, GamePhase, NativeRenderPipeline, NativeSceneRenderer, RenderLayer,
    SceneDrawPlan, SpriteId, SurfaceSize,
};

const SMOKE_FRAMES: u32 = 24;
const REQUIRED_INPUTS: [&str; 9] = [
    "coin",
    "start_one",
    "altitude_up",
    "thrust",
    "fire",
    "reverse",
    "smart_bomb",
    "hyperspace",
    "altitude_down",
];
const REQUIRED_SPRITES: [&str; 7] = [
    "player_ship",
    "enemy_lander",
    "human",
    "player_projectile",
    "terrain_tile",
    "star",
    "score_text",
];
const REQUIRED_PIPELINES: [&str; 5] =
    ["terrain", "starfield", "sprites", "projectiles", "hud_text"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GameSmokeReport {
    pub(crate) frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_scene_signatures: usize,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_frames: u32,
    pub(crate) credited_frames: u32,
    pub(crate) playing_frames: u32,
    pub(crate) sprite_frames: u32,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) terrain_sprites: usize,
    pub(crate) starfield_sprites: usize,
    pub(crate) object_sprites: usize,
    pub(crate) projectile_sprites: usize,
    pub(crate) hud_sprites: usize,
    pub(crate) covered_sprites: Vec<String>,
    pub(crate) terrain_draw_commands: usize,
    pub(crate) starfield_draw_commands: usize,
    pub(crate) object_draw_commands: usize,
    pub(crate) projectile_draw_commands: usize,
    pub(crate) hud_draw_commands: usize,
    pub(crate) drawn_sprite_instances: usize,
    pub(crate) terrain_draw_instances: usize,
    pub(crate) starfield_draw_instances: usize,
    pub(crate) object_draw_instances: usize,
    pub(crate) projectile_draw_instances: usize,
    pub(crate) hud_draw_instances: usize,
    pub(crate) covered_pipelines: Vec<String>,
    pub(crate) wgpu_frame_commands: usize,
    pub(crate) frame_plan_begin_render_pass_commands: usize,
    pub(crate) frame_plan_ordered_sprite_only_frames: u32,
    pub(crate) frame_plan_viewport_commands: usize,
    pub(crate) sprite_render_pass_commands: usize,
    pub(crate) temporary_raster_commands: usize,
    pub(crate) frame_plan_scene_projection_upload_bytes: usize,
    pub(crate) sprite_frame_plan_encoder_commands: usize,
    pub(crate) sprite_frame_plan_draws: usize,
    pub(crate) sprite_frame_plan_instances: usize,
    pub(crate) sprite_render_pass_plan_frames: u32,
    pub(crate) sprite_render_pass_plan_draws: usize,
    pub(crate) sprite_render_pass_plan_instances: usize,
    pub(crate) sprite_resource_binding_frames: u32,
    pub(crate) sprite_pipeline_layout_frames: u32,
    pub(crate) sprite_render_pipeline_descriptor_frames: u32,
    pub(crate) sprite_render_pass_encoder_frames: u32,
    pub(crate) sprite_encoder_commands: usize,
    pub(crate) sprite_encoder_draws: usize,
    pub(crate) sprite_buffer_upload_frames: u32,
    pub(crate) sprite_quad_vertex_upload_bytes: usize,
    pub(crate) sprite_quad_index_upload_bytes: usize,
    pub(crate) sprite_buffer_instance_upload_bytes: usize,
    pub(crate) sprite_instance_upload_records: usize,
    pub(crate) sprite_instance_upload_bytes: usize,
    pub(crate) sprite_atlas_upload_bytes: usize,
    pub(crate) scene_projection_upload_bytes: usize,
    pub(crate) raster_frames: u32,
    pub(crate) missing_sprite_regions: usize,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

impl GameSmokeReport {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.frames == 0 {
            bail!("clean game smoke did not advance any frames");
        }
        if self.first_frame_size.is_none() {
            bail!("clean game smoke did not record a frame size");
        }
        if self.distinct_scene_signatures < 3 {
            bail!("clean game smoke did not produce dynamic scene signatures");
        }
        if !self.saw_attract || self.attract_frames == 0 {
            bail!("clean game smoke did not observe attract frames");
        }
        if !self.saw_credit || self.credited_frames == 0 {
            bail!("clean game smoke did not observe a credited attract frame");
        }
        if !self.saw_playing || self.playing_frames == 0 {
            bail!("clean game smoke did not observe playing frames");
        }
        if self.sprite_frames != self.frames {
            bail!("clean game smoke did not produce sprites for every frame");
        }
        if self.sprite_instances == 0 || self.sprite_draw_commands == 0 {
            bail!("clean game smoke did not produce sprite draw plans");
        }
        if self.terrain_sprites == 0 {
            bail!("clean game smoke did not produce terrain sprites");
        }
        if self.starfield_sprites == 0 {
            bail!("clean game smoke did not produce starfield sprites");
        }
        if self.object_sprites == 0 {
            bail!("clean game smoke did not produce object sprites");
        }
        if self.projectile_sprites == 0 {
            bail!("clean game smoke did not produce projectile sprites");
        }
        if self.hud_sprites == 0 {
            bail!("clean game smoke did not produce hud sprites");
        }
        for required in REQUIRED_SPRITES {
            if !self.covered_sprites.iter().any(|sprite| sprite == required) {
                bail!("clean game smoke did not cover {required} sprite");
            }
        }
        if self.terrain_draw_commands == 0 {
            bail!("clean game smoke did not produce terrain draw commands");
        }
        if self.starfield_draw_commands == 0 {
            bail!("clean game smoke did not produce starfield draw commands");
        }
        if self.object_draw_commands == 0 {
            bail!("clean game smoke did not produce object draw commands");
        }
        if self.projectile_draw_commands == 0 {
            bail!("clean game smoke did not produce projectile draw commands");
        }
        if self.hud_draw_commands == 0 {
            bail!("clean game smoke did not produce hud draw commands");
        }
        if self.drawn_sprite_instances != self.sprite_instances {
            bail!("clean game smoke drawn sprite instances did not match sprite instances");
        }
        if self.terrain_draw_instances != self.terrain_sprites {
            bail!("clean game smoke terrain draw instances did not match terrain sprites");
        }
        if self.starfield_draw_instances != self.starfield_sprites {
            bail!("clean game smoke starfield draw instances did not match starfield sprites");
        }
        if self.object_draw_instances != self.object_sprites {
            bail!("clean game smoke object draw instances did not match object sprites");
        }
        if self.projectile_draw_instances != self.projectile_sprites {
            bail!("clean game smoke projectile draw instances did not match projectile sprites");
        }
        if self.hud_draw_instances != self.hud_sprites {
            bail!("clean game smoke hud draw instances did not match hud sprites");
        }
        if self.sprite_instance_upload_records != self.drawn_sprite_instances {
            bail!(
                "clean game smoke sprite instance upload records did not match drawn sprite instances"
            );
        }
        for required in REQUIRED_PIPELINES {
            if !self
                .covered_pipelines
                .iter()
                .any(|pipeline| pipeline == required)
            {
                bail!("clean game smoke did not cover {required} pipeline");
            }
        }
        if self.wgpu_frame_commands == 0 {
            bail!("clean game smoke did not produce wgpu frame commands");
        }
        if self.frame_plan_begin_render_pass_commands != self.frames as usize {
            bail!("clean game smoke did not produce begin-pass frame commands for every frame");
        }
        if self.frame_plan_ordered_sprite_only_frames != self.frames {
            bail!(
                "clean game smoke did not produce ordered sprite-only frame commands for every frame"
            );
        }
        if self.frame_plan_viewport_commands != self.frames as usize {
            bail!("clean game smoke did not produce viewport frame commands for every frame");
        }
        if self.sprite_render_pass_commands < self.frames as usize {
            bail!("clean game smoke did not produce sprite render-pass commands for every frame");
        }
        if self.temporary_raster_commands != 0 {
            bail!("clean game smoke unexpectedly produced temporary raster frame commands");
        }
        if self.sprite_encoder_commands == 0 {
            bail!("clean game smoke did not produce sprite encoder commands");
        }
        if self.sprite_frame_plan_encoder_commands != self.sprite_encoder_commands {
            bail!(
                "clean game smoke frame-plan sprite encoder commands did not match sprite encoder commands"
            );
        }
        if self.sprite_frame_plan_draws != self.sprite_draw_commands {
            bail!("clean game smoke frame-plan sprite draws did not match sprite draw commands");
        }
        if self.sprite_frame_plan_instances != self.drawn_sprite_instances {
            bail!(
                "clean game smoke frame-plan sprite instances did not match drawn sprite instances"
            );
        }
        if self.sprite_render_pass_plan_frames != self.frames {
            bail!("clean game smoke did not produce sprite render-pass plans for every frame");
        }
        if self.sprite_render_pass_plan_draws != self.sprite_draw_commands {
            bail!("clean game smoke sprite render-pass draws did not match sprite draw commands");
        }
        if self.sprite_render_pass_plan_instances != self.drawn_sprite_instances {
            bail!(
                "clean game smoke sprite render-pass instances did not match drawn sprite instances"
            );
        }
        if self.sprite_resource_binding_frames != self.frames {
            bail!("clean game smoke did not produce sprite resource bindings for every frame");
        }
        if self.sprite_pipeline_layout_frames != self.frames {
            bail!("clean game smoke did not produce sprite pipeline layouts for every frame");
        }
        if self.sprite_render_pipeline_descriptor_frames != self.frames {
            bail!(
                "clean game smoke did not produce sprite render pipeline descriptors for every frame"
            );
        }
        if self.sprite_render_pass_encoder_frames != self.frames {
            bail!("clean game smoke did not produce sprite render-pass encoders for every frame");
        }
        if self.sprite_encoder_draws != self.sprite_draw_commands {
            bail!("clean game smoke sprite encoder draws did not match sprite draw commands");
        }
        if self.sprite_instance_upload_bytes == 0 {
            bail!("clean game smoke did not produce sprite instance upload bytes");
        }
        if self.sprite_buffer_upload_frames != self.frames {
            bail!("clean game smoke did not produce sprite buffer uploads for every frame");
        }
        if self.sprite_quad_vertex_upload_bytes == 0 {
            bail!("clean game smoke did not produce sprite quad vertex upload bytes");
        }
        if self.sprite_quad_index_upload_bytes == 0 {
            bail!("clean game smoke did not produce sprite quad index upload bytes");
        }
        if self.sprite_buffer_instance_upload_bytes == 0 {
            bail!("clean game smoke did not produce sprite buffer instance upload bytes");
        }
        if self.sprite_buffer_instance_upload_bytes != self.sprite_instance_upload_bytes {
            bail!(
                "clean game smoke sprite buffer instance bytes did not match instance upload bytes"
            );
        }
        if self.sprite_atlas_upload_bytes == 0 {
            bail!("clean game smoke did not produce sprite atlas upload bytes");
        }
        if self.scene_projection_upload_bytes == 0 {
            bail!("clean game smoke did not produce scene projection upload bytes");
        }
        if self.frame_plan_scene_projection_upload_bytes != self.scene_projection_upload_bytes {
            bail!(
                "clean game smoke frame-plan scene projection bytes did not match resource binding projection bytes"
            );
        }
        if self.raster_frames != 0 {
            bail!("clean game smoke unexpectedly produced raster payloads");
        }
        if self.missing_sprite_regions != 0 {
            bail!("clean game smoke had missing sprite atlas regions");
        }
        for required in REQUIRED_INPUTS {
            if !self.injected_inputs.iter().any(|input| input == required) {
                bail!("clean game smoke did not inject {required}");
            }
        }
        if !self.clean_exit {
            bail!("clean game smoke did not exit cleanly");
        }

        Ok(())
    }

    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "clean game smoke passed\n  frames: {}\n  first_frame_size: {}\n  distinct_scene_signatures: {}\n  saw_attract: {} (frames: {})\n  saw_credit: {} (frames: {})\n  saw_playing: {} (frames: {})\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  terrain_sprites: {}\n  starfield_sprites: {}\n  object_sprites: {}\n  projectile_sprites: {}\n  hud_sprites: {}\n  covered_sprites: {}\n  terrain_draw_commands: {}\n  starfield_draw_commands: {}\n  object_draw_commands: {}\n  projectile_draw_commands: {}\n  hud_draw_commands: {}\n  drawn_sprite_instances: {}\n  terrain_draw_instances: {}\n  starfield_draw_instances: {}\n  object_draw_instances: {}\n  projectile_draw_instances: {}\n  hud_draw_instances: {}\n  covered_pipelines: {}\n  wgpu_frame_commands: {}\n  frame_plan_begin_render_pass_commands: {}\n  frame_plan_ordered_sprite_only_frames: {}\n  frame_plan_viewport_commands: {}\n  sprite_render_pass_commands: {}\n  temporary_raster_commands: {}\n  frame_plan_scene_projection_upload_bytes: {}\n  sprite_frame_plan_encoder_commands: {}\n  sprite_frame_plan_draws: {}\n  sprite_frame_plan_instances: {}\n  sprite_render_pass_plan_frames: {}\n  sprite_render_pass_plan_draws: {}\n  sprite_render_pass_plan_instances: {}\n  sprite_resource_binding_frames: {}\n  sprite_pipeline_layout_frames: {}\n  sprite_render_pipeline_descriptor_frames: {}\n  sprite_render_pass_encoder_frames: {}\n  sprite_encoder_commands: {}\n  sprite_encoder_draws: {}\n  sprite_buffer_upload_frames: {}\n  sprite_quad_vertex_upload_bytes: {}\n  sprite_quad_index_upload_bytes: {}\n  sprite_buffer_instance_upload_bytes: {}\n  sprite_instance_upload_records: {}\n  sprite_instance_upload_bytes: {}\n  sprite_atlas_upload_bytes: {}\n  scene_projection_upload_bytes: {}\n  raster_frames: {}\n  missing_sprite_regions: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.frames,
            frame_size,
            self.distinct_scene_signatures,
            self.saw_attract,
            self.attract_frames,
            self.saw_credit,
            self.credited_frames,
            self.saw_playing,
            self.playing_frames,
            self.sprite_frames,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.terrain_sprites,
            self.starfield_sprites,
            self.object_sprites,
            self.projectile_sprites,
            self.hud_sprites,
            self.covered_sprites.join(","),
            self.terrain_draw_commands,
            self.starfield_draw_commands,
            self.object_draw_commands,
            self.projectile_draw_commands,
            self.hud_draw_commands,
            self.drawn_sprite_instances,
            self.terrain_draw_instances,
            self.starfield_draw_instances,
            self.object_draw_instances,
            self.projectile_draw_instances,
            self.hud_draw_instances,
            self.covered_pipelines.join(","),
            self.wgpu_frame_commands,
            self.frame_plan_begin_render_pass_commands,
            self.frame_plan_ordered_sprite_only_frames,
            self.frame_plan_viewport_commands,
            self.sprite_render_pass_commands,
            self.temporary_raster_commands,
            self.frame_plan_scene_projection_upload_bytes,
            self.sprite_frame_plan_encoder_commands,
            self.sprite_frame_plan_draws,
            self.sprite_frame_plan_instances,
            self.sprite_render_pass_plan_frames,
            self.sprite_render_pass_plan_draws,
            self.sprite_render_pass_plan_instances,
            self.sprite_resource_binding_frames,
            self.sprite_pipeline_layout_frames,
            self.sprite_render_pipeline_descriptor_frames,
            self.sprite_render_pass_encoder_frames,
            self.sprite_encoder_commands,
            self.sprite_encoder_draws,
            self.sprite_buffer_upload_frames,
            self.sprite_quad_vertex_upload_bytes,
            self.sprite_quad_index_upload_bytes,
            self.sprite_buffer_instance_upload_bytes,
            self.sprite_instance_upload_records,
            self.sprite_instance_upload_bytes,
            self.sprite_atlas_upload_bytes,
            self.scene_projection_upload_bytes,
            self.raster_frames,
            self.missing_sprite_regions,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }
}

pub(crate) fn run() -> anyhow::Result<()> {
    let report = smoke_report(SMOKE_FRAMES)?;

    print!("{}", report.to_text());
    Ok(())
}

pub(crate) fn smoke_report(frames: u32) -> anyhow::Result<GameSmokeReport> {
    if frames == 0 {
        bail!("clean game smoke frame count must be positive");
    }

    let mut game = Game::new();
    let renderer = NativeSceneRenderer::default();
    let mut report = GameSmokeReport {
        clean_exit: true,
        ..GameSmokeReport::default()
    };
    let mut signatures = BTreeSet::new();

    for frame_index in 0..frames {
        let input = smoke_input(frame_index);
        if let Some(label) = input.label {
            record_unique_label(&mut report.injected_inputs, label);
        }

        let frame = game.step(input.value);
        let plan = renderer.prepare(&frame.scene);
        observe_frame(&mut report, &mut signatures, &frame, &plan);
    }

    report.distinct_scene_signatures = signatures.len();
    report.validate()?;
    Ok(report)
}

fn observe_frame(
    report: &mut GameSmokeReport,
    signatures: &mut BTreeSet<u64>,
    frame: &GameFrame,
    plan: &SceneDrawPlan,
) {
    report.frames = report.frames.saturating_add(1);
    report
        .first_frame_size
        .get_or_insert(surface_tuple(frame.scene.surface));
    report.saw_attract |= frame.state.phase == GamePhase::Attract;
    report.saw_credit |= frame.state.phase == GamePhase::Attract && frame.state.credits > 0;
    report.saw_playing |= frame.state.phase == GamePhase::Playing;

    if frame.state.phase == GamePhase::Attract {
        report.attract_frames = report.attract_frames.saturating_add(1);
        if frame.state.credits > 0 {
            report.credited_frames = report.credited_frames.saturating_add(1);
        }
    }
    if frame.state.phase == GamePhase::Playing {
        report.playing_frames = report.playing_frames.saturating_add(1);
    }

    let summary = frame.scene.summary();
    if summary.sprite_count > 0 {
        report.sprite_frames = report.sprite_frames.saturating_add(1);
    }
    if summary.raster_count > 0 {
        report.raster_frames = report.raster_frames.saturating_add(1);
    }
    report.terrain_sprites = report
        .terrain_sprites
        .saturating_add(summary.layers.terrain);
    report.starfield_sprites = report
        .starfield_sprites
        .saturating_add(summary.layers.starfield);
    report.object_sprites = report.object_sprites.saturating_add(summary.layers.objects);
    report.projectile_sprites = report
        .projectile_sprites
        .saturating_add(summary.layers.projectiles);
    report.hud_sprites = report.hud_sprites.saturating_add(summary.layers.hud);
    for sprite in &frame.scene.sprites {
        if let Some(label) = required_sprite_label(sprite.sprite) {
            record_unique_label(&mut report.covered_sprites, label);
        }
    }
    report.sprite_instances = report
        .sprite_instances
        .saturating_add(plan.sprite_instances);
    report.sprite_draw_commands = report
        .sprite_draw_commands
        .saturating_add(plan.sprite_draw_commands.len());
    for command in &plan.sprite_draw_commands {
        record_draw_command(report, command.layer, command.instance_count);
        if let Some(label) = required_pipeline_label(command.pipeline) {
            record_unique_label(&mut report.covered_pipelines, label);
        }
    }
    report.wgpu_frame_commands = report
        .wgpu_frame_commands
        .saturating_add(plan.frame_plan.command_count());
    report.frame_plan_begin_render_pass_commands = report
        .frame_plan_begin_render_pass_commands
        .saturating_add(plan.frame_plan.begin_render_pass_count());
    if plan.frame_plan.has_ordered_sprite_only_commands() {
        report.frame_plan_ordered_sprite_only_frames = report
            .frame_plan_ordered_sprite_only_frames
            .saturating_add(1);
    }
    report.frame_plan_viewport_commands = report
        .frame_plan_viewport_commands
        .saturating_add(plan.frame_plan.viewport_command_count());
    report.sprite_render_pass_commands = report
        .sprite_render_pass_commands
        .saturating_add(plan.frame_plan.sprite_pass_count());
    report.temporary_raster_commands = report
        .temporary_raster_commands
        .saturating_add(plan.frame_plan.temporary_raster_count());
    report.frame_plan_scene_projection_upload_bytes = report
        .frame_plan_scene_projection_upload_bytes
        .saturating_add(buffer_address_len(
            plan.frame_plan.scene_projection_upload_byte_len(),
        ));
    report.sprite_frame_plan_encoder_commands = report
        .sprite_frame_plan_encoder_commands
        .saturating_add(plan.frame_plan.sprite_encoder_command_count());
    report.sprite_frame_plan_draws = report
        .sprite_frame_plan_draws
        .saturating_add(plan.frame_plan.sprite_draw_count());
    report.sprite_frame_plan_instances = report
        .sprite_frame_plan_instances
        .saturating_add(plan.frame_plan.sprite_instance_count());
    if let Some(render_pass) = &plan.sprite_render_pass {
        report.sprite_render_pass_plan_frames =
            report.sprite_render_pass_plan_frames.saturating_add(1);
        report.sprite_render_pass_plan_draws = report
            .sprite_render_pass_plan_draws
            .saturating_add(render_pass.draw_count());
        report.sprite_render_pass_plan_instances = report
            .sprite_render_pass_plan_instances
            .saturating_add(render_pass.instance_count() as usize);
    }
    if let Some(upload) = &plan.sprite_instance_upload {
        report.sprite_instance_upload_records = report
            .sprite_instance_upload_records
            .saturating_add(upload.instance_count());
        report.sprite_instance_upload_bytes = report
            .sprite_instance_upload_bytes
            .saturating_add(buffer_address_len(upload.byte_len()));
    }
    if let Some(uploads) = &plan.sprite_buffer_uploads {
        report.sprite_buffer_upload_frames = report.sprite_buffer_upload_frames.saturating_add(1);
        report.sprite_quad_vertex_upload_bytes = report
            .sprite_quad_vertex_upload_bytes
            .saturating_add(buffer_address_len(uploads.quad_vertices.byte_len));
        report.sprite_quad_index_upload_bytes = report
            .sprite_quad_index_upload_bytes
            .saturating_add(buffer_address_len(uploads.quad_indices.byte_len));
        report.sprite_buffer_instance_upload_bytes = report
            .sprite_buffer_instance_upload_bytes
            .saturating_add(buffer_address_len(uploads.instances.byte_len));
    }
    if let Some(bindings) = &plan.sprite_resource_bindings {
        report.sprite_resource_binding_frames =
            report.sprite_resource_binding_frames.saturating_add(1);
        report.sprite_atlas_upload_bytes = report
            .sprite_atlas_upload_bytes
            .saturating_add(bindings.atlas_upload.byte_len);
        report.scene_projection_upload_bytes = report
            .scene_projection_upload_bytes
            .saturating_add(buffer_address_len(bindings.projection_upload.byte_len));
    }
    if plan.sprite_pipeline_layout.is_some() {
        report.sprite_pipeline_layout_frames =
            report.sprite_pipeline_layout_frames.saturating_add(1);
    }
    if plan.sprite_render_pipeline_descriptor.is_some() {
        report.sprite_render_pipeline_descriptor_frames = report
            .sprite_render_pipeline_descriptor_frames
            .saturating_add(1);
    }
    if let Some(encoder) = &plan.sprite_render_pass_encoder {
        report.sprite_render_pass_encoder_frames =
            report.sprite_render_pass_encoder_frames.saturating_add(1);
        report.sprite_encoder_commands = report
            .sprite_encoder_commands
            .saturating_add(encoder.command_count());
        report.sprite_encoder_draws = report
            .sprite_encoder_draws
            .saturating_add(encoder.draw_count());
    }
    report.missing_sprite_regions = report
        .missing_sprite_regions
        .saturating_add(plan.missing_sprite_regions);
    signatures.insert(scene_signature(frame, plan));
}

fn buffer_address_len(byte_len: u64) -> usize {
    usize::try_from(byte_len).unwrap_or(usize::MAX)
}

fn required_sprite_label(sprite: SpriteId) -> Option<&'static str> {
    match sprite {
        SpriteId::PLAYER_SHIP => Some("player_ship"),
        SpriteId::ENEMY_LANDER => Some("enemy_lander"),
        SpriteId::HUMAN => Some("human"),
        SpriteId::PLAYER_PROJECTILE => Some("player_projectile"),
        SpriteId::TERRAIN_TILE => Some("terrain_tile"),
        SpriteId::STAR => Some("star"),
        SpriteId::SCORE_TEXT => Some("score_text"),
        _ => None,
    }
}

fn required_pipeline_label(pipeline: NativeRenderPipeline) -> Option<&'static str> {
    match pipeline {
        NativeRenderPipeline::Terrain => Some("terrain"),
        NativeRenderPipeline::Starfield => Some("starfield"),
        NativeRenderPipeline::Sprites => Some("sprites"),
        NativeRenderPipeline::Projectiles => Some("projectiles"),
        NativeRenderPipeline::HudText => Some("hud_text"),
        _ => None,
    }
}

fn record_draw_command(report: &mut GameSmokeReport, layer: RenderLayer, instance_count: u32) {
    let instance_count = instance_count as usize;
    match layer {
        RenderLayer::Terrain => {
            report.terrain_draw_commands = report.terrain_draw_commands.saturating_add(1);
            report.terrain_draw_instances =
                report.terrain_draw_instances.saturating_add(instance_count);
            report.drawn_sprite_instances =
                report.drawn_sprite_instances.saturating_add(instance_count);
        }
        RenderLayer::Starfield => {
            report.starfield_draw_commands = report.starfield_draw_commands.saturating_add(1);
            report.starfield_draw_instances = report
                .starfield_draw_instances
                .saturating_add(instance_count);
            report.drawn_sprite_instances =
                report.drawn_sprite_instances.saturating_add(instance_count);
        }
        RenderLayer::Objects => {
            report.object_draw_commands = report.object_draw_commands.saturating_add(1);
            report.object_draw_instances =
                report.object_draw_instances.saturating_add(instance_count);
            report.drawn_sprite_instances =
                report.drawn_sprite_instances.saturating_add(instance_count);
        }
        RenderLayer::Projectiles => {
            report.projectile_draw_commands = report.projectile_draw_commands.saturating_add(1);
            report.projectile_draw_instances = report
                .projectile_draw_instances
                .saturating_add(instance_count);
            report.drawn_sprite_instances =
                report.drawn_sprite_instances.saturating_add(instance_count);
        }
        RenderLayer::Hud => {
            report.hud_draw_commands = report.hud_draw_commands.saturating_add(1);
            report.hud_draw_instances = report.hud_draw_instances.saturating_add(instance_count);
            report.drawn_sprite_instances =
                report.drawn_sprite_instances.saturating_add(instance_count);
        }
        RenderLayer::Overlay => {}
    }
}

fn surface_tuple(surface: SurfaceSize) -> (u32, u32) {
    (surface.width, surface.height)
}

fn scene_signature(frame: &GameFrame, plan: &SceneDrawPlan) -> u64 {
    let mut signature = 0x6A09_E667_F3BC_C909_u64;
    signature = mix_signature(signature, frame.state.frame);
    signature = mix_signature(signature, phase_code(frame.state.phase));
    signature = mix_signature(signature, u64::from(frame.state.credits));
    signature = mix_signature(signature, u64::from(frame.state.wave));
    signature = mix_signature(signature, frame.scene.summary().sprite_count as u64);
    signature = mix_signature(signature, plan.sprite_instances as u64);
    signature = mix_signature(signature, plan.sprite_draw_commands.len() as u64);
    signature = mix_signature(signature, plan.pipelines.len() as u64);
    signature
}

fn mix_signature(current: u64, value: u64) -> u64 {
    current
        ^ value
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(current << 6)
            .wrapping_add(current >> 2)
}

fn phase_code(phase: GamePhase) -> u64 {
    match phase {
        GamePhase::Attract => 1,
        GamePhase::Playing => 2,
        GamePhase::GameOver => 3,
        GamePhase::HighScoreEntry => 4,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScriptedInput {
    value: GameInput,
    label: Option<&'static str>,
}

fn smoke_input(frame_index: u32) -> ScriptedInput {
    let (value, label) = match frame_index {
        1 => (
            GameInput {
                coin: true,
                ..GameInput::NONE
            },
            Some("coin"),
        ),
        3 => (
            GameInput {
                start_one: true,
                ..GameInput::NONE
            },
            Some("start_one"),
        ),
        5 => (
            GameInput {
                altitude_up: true,
                ..GameInput::NONE
            },
            Some("altitude_up"),
        ),
        7 => (
            GameInput {
                thrust: true,
                ..GameInput::NONE
            },
            Some("thrust"),
        ),
        9 => (
            GameInput {
                fire: true,
                ..GameInput::NONE
            },
            Some("fire"),
        ),
        11 => (
            GameInput {
                reverse: true,
                ..GameInput::NONE
            },
            Some("reverse"),
        ),
        13 => (
            GameInput {
                smart_bomb: true,
                ..GameInput::NONE
            },
            Some("smart_bomb"),
        ),
        15 => (
            GameInput {
                hyperspace: true,
                ..GameInput::NONE
            },
            Some("hyperspace"),
        ),
        17 => (
            GameInput {
                altitude_down: true,
                ..GameInput::NONE
            },
            Some("altitude_down"),
        ),
        _ => (GameInput::NONE, None),
    };

    ScriptedInput { value, label }
}

fn record_unique_label(labels: &mut Vec<String>, label: &str) {
    if !labels.iter().any(|existing| existing == label) {
        labels.push(String::from(label));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GameSmokeReport, record_draw_command, required_pipeline_label, required_sprite_label,
        smoke_input, smoke_report,
    };

    #[test]
    fn smoke_report_exercises_clean_game_and_native_draw_plans() {
        let report = smoke_report(24).expect("clean game smoke report");

        assert_eq!(report.frames, 24);
        assert_eq!(report.first_frame_size, Some((292, 240)));
        assert!(report.distinct_scene_signatures >= 3);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
        assert!(report.attract_frames > 0);
        assert!(report.credited_frames > 0);
        assert!(report.playing_frames > 0);
        assert_eq!(report.sprite_frames, report.frames);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.terrain_sprites > 0);
        assert!(report.starfield_sprites > 0);
        assert!(report.object_sprites > 0);
        assert!(report.projectile_sprites > 0);
        assert!(report.hud_sprites > 0);
        assert_eq!(
            report.covered_sprites,
            vec![
                "score_text",
                "star",
                "terrain_tile",
                "enemy_lander",
                "human",
                "player_ship",
                "player_projectile",
            ]
        );
        assert!(report.terrain_draw_commands > 0);
        assert!(report.starfield_draw_commands > 0);
        assert!(report.object_draw_commands > 0);
        assert!(report.projectile_draw_commands > 0);
        assert!(report.hud_draw_commands > 0);
        assert_eq!(report.drawn_sprite_instances, report.sprite_instances);
        assert_eq!(report.terrain_draw_instances, report.terrain_sprites);
        assert_eq!(report.starfield_draw_instances, report.starfield_sprites);
        assert_eq!(report.object_draw_instances, report.object_sprites);
        assert_eq!(report.projectile_draw_instances, report.projectile_sprites);
        assert_eq!(report.hud_draw_instances, report.hud_sprites);
        assert_eq!(
            report.covered_pipelines,
            vec!["hud_text", "starfield", "terrain", "sprites", "projectiles",]
        );
        assert!(report.wgpu_frame_commands > 0);
        assert_eq!(
            report.frame_plan_begin_render_pass_commands,
            report.frames as usize
        );
        assert_eq!(report.frame_plan_ordered_sprite_only_frames, report.frames);
        assert_eq!(report.frame_plan_viewport_commands, report.frames as usize);
        assert!(report.sprite_render_pass_commands >= report.frames as usize);
        assert_eq!(report.temporary_raster_commands, 0);
        assert_eq!(
            report.frame_plan_scene_projection_upload_bytes,
            report.scene_projection_upload_bytes
        );
        assert_eq!(
            report.sprite_frame_plan_encoder_commands,
            report.sprite_encoder_commands
        );
        assert_eq!(report.sprite_frame_plan_draws, report.sprite_draw_commands);
        assert_eq!(
            report.sprite_frame_plan_instances,
            report.drawn_sprite_instances
        );
        assert_eq!(report.sprite_render_pass_plan_frames, report.frames);
        assert_eq!(
            report.sprite_render_pass_plan_draws,
            report.sprite_draw_commands
        );
        assert_eq!(
            report.sprite_render_pass_plan_instances,
            report.drawn_sprite_instances
        );
        assert_eq!(report.sprite_resource_binding_frames, report.frames);
        assert_eq!(report.sprite_pipeline_layout_frames, report.frames);
        assert_eq!(
            report.sprite_render_pipeline_descriptor_frames,
            report.frames
        );
        assert_eq!(report.sprite_render_pass_encoder_frames, report.frames);
        assert!(report.sprite_encoder_commands > 0);
        assert_eq!(report.sprite_encoder_draws, report.sprite_draw_commands);
        assert_eq!(report.sprite_buffer_upload_frames, report.frames);
        assert!(report.sprite_quad_vertex_upload_bytes > 0);
        assert!(report.sprite_quad_index_upload_bytes > 0);
        assert_eq!(
            report.sprite_buffer_instance_upload_bytes,
            report.sprite_instance_upload_bytes
        );
        assert_eq!(
            report.sprite_instance_upload_records,
            report.sprite_instances
        );
        assert_eq!(
            report.sprite_instance_upload_records,
            report.drawn_sprite_instances
        );
        assert!(report.sprite_instance_upload_bytes > 0);
        assert!(report.sprite_atlas_upload_bytes > 0);
        assert!(report.scene_projection_upload_bytes > 0);
        assert_eq!(report.raster_frames, 0);
        assert_eq!(report.missing_sprite_regions, 0);
        assert_eq!(
            report.injected_inputs,
            vec![
                "coin",
                "start_one",
                "altitude_up",
                "thrust",
                "fire",
                "reverse",
                "smart_bomb",
                "hyperspace",
                "altitude_down",
            ]
        );
        assert!(report.clean_exit);
    }

    #[test]
    fn smoke_report_rejects_zero_frames() {
        let error = smoke_report(0).expect_err("zero-frame smoke should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame count must be positive"
        );
    }

    #[test]
    fn smoke_report_validates_required_play_states() {
        let mut report = valid_report();
        report.saw_playing = false;
        report.playing_frames = 0;

        let error = report
            .validate()
            .expect_err("missing playing evidence should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not observe playing frames"
        );
    }

    #[test]
    fn smoke_report_validates_frame_command_evidence() {
        let mut report = valid_report();
        report.wgpu_frame_commands = 0;

        let error = report
            .validate()
            .expect_err("missing wgpu frame command evidence should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce wgpu frame commands"
        );

        let mut report = valid_report();
        report.frame_plan_begin_render_pass_commands = report
            .frame_plan_begin_render_pass_commands
            .saturating_sub(1);

        let error = report
            .validate()
            .expect_err("missing begin-pass frame command coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce begin-pass frame commands for every frame"
        );

        let mut report = valid_report();
        report.frame_plan_ordered_sprite_only_frames = report
            .frame_plan_ordered_sprite_only_frames
            .saturating_sub(1);

        let error = report
            .validate()
            .expect_err("missing ordered sprite-only frame command coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce ordered sprite-only frame commands for every frame"
        );

        let mut report = valid_report();
        report.frame_plan_viewport_commands = report.frame_plan_viewport_commands.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("missing viewport frame command coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce viewport frame commands for every frame"
        );

        let mut report = valid_report();
        report.sprite_render_pass_commands = 2;

        let error = report
            .validate()
            .expect_err("missing sprite render-pass command coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite render-pass commands for every frame"
        );

        let mut report = valid_report();
        report.temporary_raster_commands = 1;

        let error = report
            .validate()
            .expect_err("temporary raster frame command should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke unexpectedly produced temporary raster frame commands"
        );

        let mut report = valid_report();
        report.frame_plan_scene_projection_upload_bytes = report
            .frame_plan_scene_projection_upload_bytes
            .saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched frame-plan scene projection bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame-plan scene projection bytes did not match resource binding projection bytes"
        );

        let mut report = valid_report();
        report.sprite_frame_plan_encoder_commands =
            report.sprite_frame_plan_encoder_commands.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched frame-plan sprite encoder commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame-plan sprite encoder commands did not match sprite encoder commands"
        );

        let mut report = valid_report();
        report.sprite_frame_plan_draws = report.sprite_frame_plan_draws.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched frame-plan sprite draws should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame-plan sprite draws did not match sprite draw commands"
        );

        let mut report = valid_report();
        report.sprite_frame_plan_instances = report.sprite_frame_plan_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched frame-plan sprite instances should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame-plan sprite instances did not match drawn sprite instances"
        );

        let mut report = valid_report();
        report.sprite_render_pass_plan_frames = 2;

        let error = report
            .validate()
            .expect_err("missing sprite render-pass plan frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite render-pass plans for every frame"
        );

        let mut report = valid_report();
        report.sprite_render_pass_plan_draws =
            report.sprite_render_pass_plan_draws.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched sprite render-pass draws should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke sprite render-pass draws did not match sprite draw commands"
        );

        let mut report = valid_report();
        report.sprite_render_pass_plan_instances =
            report.sprite_render_pass_plan_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched sprite render-pass instances should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke sprite render-pass instances did not match drawn sprite instances"
        );
    }

    #[test]
    fn smoke_report_validates_sprite_coverage_evidence() {
        let mut report = valid_report();
        report.terrain_sprites = 0;

        let error = report
            .validate()
            .expect_err("missing terrain sprite coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce terrain sprites"
        );

        let mut report = valid_report();
        report.starfield_sprites = 0;

        let error = report
            .validate()
            .expect_err("missing starfield sprite coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce starfield sprites"
        );

        let mut report = valid_report();
        report.object_sprites = 0;

        let error = report
            .validate()
            .expect_err("missing object sprite coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce object sprites"
        );

        let mut report = valid_report();
        report.projectile_sprites = 0;

        let error = report
            .validate()
            .expect_err("missing projectile sprite coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce projectile sprites"
        );

        let mut report = valid_report();
        report.hud_sprites = 0;

        let error = report
            .validate()
            .expect_err("missing hud sprite coverage should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce hud sprites"
        );

        let mut report = valid_report();
        report.covered_sprites.retain(|sprite| sprite != "human");

        let error = report
            .validate()
            .expect_err("missing required sprite id should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not cover human sprite"
        );
    }

    #[test]
    fn smoke_report_validates_sprite_draw_pipeline_evidence() {
        let mut report = valid_report();
        report.terrain_draw_commands = 0;

        let error = report
            .validate()
            .expect_err("missing terrain draw commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce terrain draw commands"
        );

        let mut report = valid_report();
        report.starfield_draw_commands = 0;

        let error = report
            .validate()
            .expect_err("missing starfield draw commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce starfield draw commands"
        );

        let mut report = valid_report();
        report.object_draw_commands = 0;

        let error = report
            .validate()
            .expect_err("missing object draw commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce object draw commands"
        );

        let mut report = valid_report();
        report.projectile_draw_commands = 0;

        let error = report
            .validate()
            .expect_err("missing projectile draw commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce projectile draw commands"
        );

        let mut report = valid_report();
        report.hud_draw_commands = 0;

        let error = report
            .validate()
            .expect_err("missing hud draw commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce hud draw commands"
        );

        let mut report = valid_report();
        report
            .covered_pipelines
            .retain(|pipeline| pipeline != "projectiles");

        let error = report
            .validate()
            .expect_err("missing required pipeline should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not cover projectiles pipeline"
        );
    }

    #[test]
    fn smoke_report_validates_sprite_draw_instance_evidence() {
        let mut report = valid_report();
        report.drawn_sprite_instances = report.drawn_sprite_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("missing drawn sprite instance should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke drawn sprite instances did not match sprite instances"
        );

        let mut report = valid_report();
        report.terrain_draw_instances = report.terrain_draw_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("terrain draw instance mismatch should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke terrain draw instances did not match terrain sprites"
        );

        let mut report = valid_report();
        report.starfield_draw_instances = report.starfield_draw_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("starfield draw instance mismatch should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke starfield draw instances did not match starfield sprites"
        );

        let mut report = valid_report();
        report.object_draw_instances = report.object_draw_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("object draw instance mismatch should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke object draw instances did not match object sprites"
        );

        let mut report = valid_report();
        report.projectile_draw_instances = report.projectile_draw_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("projectile draw instance mismatch should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke projectile draw instances did not match projectile sprites"
        );

        let mut report = valid_report();
        report.hud_draw_instances = report.hud_draw_instances.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("hud draw instance mismatch should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke hud draw instances did not match hud sprites"
        );
    }

    #[test]
    fn smoke_report_validates_gpu_resource_evidence() {
        let mut report = valid_report();
        report.sprite_resource_binding_frames = 2;

        let error = report
            .validate()
            .expect_err("missing resource binding frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite resource bindings for every frame"
        );

        let mut report = valid_report();
        report.sprite_pipeline_layout_frames = 2;

        let error = report
            .validate()
            .expect_err("missing pipeline layout frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite pipeline layouts for every frame"
        );

        let mut report = valid_report();
        report.sprite_render_pipeline_descriptor_frames = 2;

        let error = report
            .validate()
            .expect_err("missing render pipeline descriptor frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite render pipeline descriptors for every frame"
        );

        let mut report = valid_report();
        report.sprite_render_pass_encoder_frames = 2;

        let error = report
            .validate()
            .expect_err("missing render-pass encoder frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite render-pass encoders for every frame"
        );
    }

    #[test]
    fn smoke_report_validates_gpu_upload_evidence() {
        let mut report = valid_report();
        report.sprite_encoder_commands = 0;

        let error = report
            .validate()
            .expect_err("missing sprite encoder commands should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite encoder commands"
        );

        let mut report = valid_report();
        report.sprite_encoder_draws = 2;

        let error = report
            .validate()
            .expect_err("mismatched encoder draws should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke sprite encoder draws did not match sprite draw commands"
        );

        let mut report = valid_report();
        report.sprite_buffer_upload_frames = 2;

        let error = report
            .validate()
            .expect_err("missing sprite buffer upload frame should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite buffer uploads for every frame"
        );

        let mut report = valid_report();
        report.sprite_quad_vertex_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing quad vertex upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite quad vertex upload bytes"
        );

        let mut report = valid_report();
        report.sprite_quad_index_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing quad index upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite quad index upload bytes"
        );

        let mut report = valid_report();
        report.sprite_buffer_instance_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing buffer instance upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite buffer instance upload bytes"
        );

        let mut report = valid_report();
        report.sprite_buffer_instance_upload_bytes =
            report.sprite_buffer_instance_upload_bytes.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched buffer instance bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke sprite buffer instance bytes did not match instance upload bytes"
        );

        let mut report = valid_report();
        report.sprite_instance_upload_records =
            report.sprite_instance_upload_records.saturating_sub(1);

        let error = report
            .validate()
            .expect_err("mismatched instance upload records should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke sprite instance upload records did not match drawn sprite instances"
        );

        let mut report = valid_report();
        report.sprite_instance_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing instance upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite instance upload bytes"
        );

        let mut report = valid_report();
        report.sprite_atlas_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing atlas upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce sprite atlas upload bytes"
        );

        let mut report = valid_report();
        report.scene_projection_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing projection upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke did not produce scene projection upload bytes"
        );

        let mut report = valid_report();
        report.frame_plan_scene_projection_upload_bytes = 0;

        let error = report
            .validate()
            .expect_err("missing frame-plan projection upload bytes should fail");

        assert_eq!(
            error.to_string(),
            "clean game smoke frame-plan scene projection bytes did not match resource binding projection bytes"
        );
    }

    #[test]
    fn smoke_report_formats_current_cli_output() {
        let report = GameSmokeReport {
            frames: 2,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 2,
            saw_attract: true,
            saw_credit: true,
            saw_playing: false,
            attract_frames: 2,
            credited_frames: 1,
            playing_frames: 0,
            sprite_frames: 2,
            sprite_instances: 9,
            sprite_draw_commands: 5,
            terrain_sprites: 2,
            starfield_sprites: 2,
            object_sprites: 2,
            projectile_sprites: 1,
            hud_sprites: 2,
            covered_sprites: vec![
                String::from("star"),
                String::from("terrain_tile"),
                String::from("score_text"),
                String::from("enemy_lander"),
                String::from("human"),
                String::from("player_ship"),
                String::from("player_projectile"),
            ],
            terrain_draw_commands: 1,
            starfield_draw_commands: 1,
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            drawn_sprite_instances: 9,
            terrain_draw_instances: 2,
            starfield_draw_instances: 2,
            object_draw_instances: 2,
            projectile_draw_instances: 1,
            hud_draw_instances: 2,
            covered_pipelines: vec![
                String::from("terrain"),
                String::from("starfield"),
                String::from("sprites"),
                String::from("projectiles"),
                String::from("hud_text"),
            ],
            wgpu_frame_commands: 6,
            frame_plan_begin_render_pass_commands: 2,
            frame_plan_ordered_sprite_only_frames: 2,
            frame_plan_viewport_commands: 2,
            sprite_render_pass_commands: 2,
            temporary_raster_commands: 0,
            frame_plan_scene_projection_upload_bytes: 32,
            sprite_frame_plan_encoder_commands: 12,
            sprite_frame_plan_draws: 5,
            sprite_frame_plan_instances: 9,
            sprite_render_pass_plan_frames: 2,
            sprite_render_pass_plan_draws: 5,
            sprite_render_pass_plan_instances: 9,
            sprite_resource_binding_frames: 2,
            sprite_pipeline_layout_frames: 2,
            sprite_render_pipeline_descriptor_frames: 2,
            sprite_render_pass_encoder_frames: 2,
            sprite_encoder_commands: 12,
            sprite_encoder_draws: 5,
            sprite_buffer_upload_frames: 2,
            sprite_quad_vertex_upload_bytes: 128,
            sprite_quad_index_upload_bytes: 24,
            sprite_buffer_instance_upload_bytes: 432,
            sprite_instance_upload_records: 9,
            sprite_instance_upload_bytes: 432,
            sprite_atlas_upload_bytes: 131_072,
            scene_projection_upload_bytes: 32,
            raster_frames: 0,
            missing_sprite_regions: 0,
            injected_inputs: vec![String::from("coin")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "clean game smoke passed\n",
                "  frames: 2\n",
                "  first_frame_size: 292x240\n",
                "  distinct_scene_signatures: 2\n",
                "  saw_attract: true (frames: 2)\n",
                "  saw_credit: true (frames: 1)\n",
                "  saw_playing: false (frames: 0)\n",
                "  sprite_frames: 2\n",
                "  sprite_instances: 9\n",
                "  sprite_draw_commands: 5\n",
                "  terrain_sprites: 2\n",
                "  starfield_sprites: 2\n",
                "  object_sprites: 2\n",
                "  projectile_sprites: 1\n",
                "  hud_sprites: 2\n",
                "  covered_sprites: star,terrain_tile,score_text,enemy_lander,human,player_ship,player_projectile\n",
                "  terrain_draw_commands: 1\n",
                "  starfield_draw_commands: 1\n",
                "  object_draw_commands: 1\n",
                "  projectile_draw_commands: 1\n",
                "  hud_draw_commands: 1\n",
                "  drawn_sprite_instances: 9\n",
                "  terrain_draw_instances: 2\n",
                "  starfield_draw_instances: 2\n",
                "  object_draw_instances: 2\n",
                "  projectile_draw_instances: 1\n",
                "  hud_draw_instances: 2\n",
                "  covered_pipelines: terrain,starfield,sprites,projectiles,hud_text\n",
                "  wgpu_frame_commands: 6\n",
                "  frame_plan_begin_render_pass_commands: 2\n",
                "  frame_plan_ordered_sprite_only_frames: 2\n",
                "  frame_plan_viewport_commands: 2\n",
                "  sprite_render_pass_commands: 2\n",
                "  temporary_raster_commands: 0\n",
                "  frame_plan_scene_projection_upload_bytes: 32\n",
                "  sprite_frame_plan_encoder_commands: 12\n",
                "  sprite_frame_plan_draws: 5\n",
                "  sprite_frame_plan_instances: 9\n",
                "  sprite_render_pass_plan_frames: 2\n",
                "  sprite_render_pass_plan_draws: 5\n",
                "  sprite_render_pass_plan_instances: 9\n",
                "  sprite_resource_binding_frames: 2\n",
                "  sprite_pipeline_layout_frames: 2\n",
                "  sprite_render_pipeline_descriptor_frames: 2\n",
                "  sprite_render_pass_encoder_frames: 2\n",
                "  sprite_encoder_commands: 12\n",
                "  sprite_encoder_draws: 5\n",
                "  sprite_buffer_upload_frames: 2\n",
                "  sprite_quad_vertex_upload_bytes: 128\n",
                "  sprite_quad_index_upload_bytes: 24\n",
                "  sprite_buffer_instance_upload_bytes: 432\n",
                "  sprite_instance_upload_records: 9\n",
                "  sprite_instance_upload_bytes: 432\n",
                "  sprite_atlas_upload_bytes: 131072\n",
                "  scene_projection_upload_bytes: 32\n",
                "  raster_frames: 0\n",
                "  missing_sprite_regions: 0\n",
                "  injected_inputs: coin\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn smoke_script_uses_release_frames_between_edge_inputs() {
        assert!(smoke_input(1).value.coin);
        assert_eq!(smoke_input(2).value, crate::GameInput::NONE);
        assert!(smoke_input(3).value.start_one);
        assert_eq!(smoke_input(4).value, crate::GameInput::NONE);
    }

    #[test]
    fn smoke_sprite_labels_ignore_unknown_sprite_ids() {
        assert_eq!(required_sprite_label(crate::SpriteId(99)), None);
    }

    #[test]
    fn smoke_pipeline_labels_ignore_non_gameplay_pipelines() {
        assert_eq!(
            required_pipeline_label(crate::NativeRenderPipeline::TemporaryRaster),
            None
        );
    }

    #[test]
    fn smoke_draw_command_counter_ignores_overlay_layer() {
        let mut report = GameSmokeReport::default();

        record_draw_command(&mut report, crate::RenderLayer::Overlay, 4);

        assert_eq!(report.terrain_draw_commands, 0);
        assert_eq!(report.starfield_draw_commands, 0);
        assert_eq!(report.object_draw_commands, 0);
        assert_eq!(report.projectile_draw_commands, 0);
        assert_eq!(report.hud_draw_commands, 0);
        assert_eq!(report.drawn_sprite_instances, 0);
        assert_eq!(report.terrain_draw_instances, 0);
        assert_eq!(report.starfield_draw_instances, 0);
        assert_eq!(report.object_draw_instances, 0);
        assert_eq!(report.projectile_draw_instances, 0);
        assert_eq!(report.hud_draw_instances, 0);
    }

    fn valid_report() -> GameSmokeReport {
        GameSmokeReport {
            frames: 3,
            first_frame_size: Some((292, 240)),
            distinct_scene_signatures: 3,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_frames: 2,
            credited_frames: 1,
            playing_frames: 1,
            sprite_frames: 3,
            sprite_instances: 13,
            sprite_draw_commands: 5,
            terrain_sprites: 3,
            starfield_sprites: 3,
            object_sprites: 3,
            projectile_sprites: 1,
            hud_sprites: 3,
            covered_sprites: vec![
                String::from("star"),
                String::from("terrain_tile"),
                String::from("score_text"),
                String::from("enemy_lander"),
                String::from("human"),
                String::from("player_ship"),
                String::from("player_projectile"),
            ],
            terrain_draw_commands: 1,
            starfield_draw_commands: 1,
            object_draw_commands: 1,
            projectile_draw_commands: 1,
            hud_draw_commands: 1,
            drawn_sprite_instances: 13,
            terrain_draw_instances: 3,
            starfield_draw_instances: 3,
            object_draw_instances: 3,
            projectile_draw_instances: 1,
            hud_draw_instances: 3,
            covered_pipelines: vec![
                String::from("terrain"),
                String::from("starfield"),
                String::from("sprites"),
                String::from("projectiles"),
                String::from("hud_text"),
            ],
            wgpu_frame_commands: 9,
            frame_plan_begin_render_pass_commands: 3,
            frame_plan_ordered_sprite_only_frames: 3,
            frame_plan_viewport_commands: 3,
            sprite_render_pass_commands: 3,
            frame_plan_scene_projection_upload_bytes: 48,
            sprite_frame_plan_encoder_commands: 18,
            sprite_frame_plan_draws: 5,
            sprite_frame_plan_instances: 13,
            sprite_render_pass_plan_frames: 3,
            sprite_render_pass_plan_draws: 5,
            sprite_render_pass_plan_instances: 13,
            sprite_resource_binding_frames: 3,
            sprite_pipeline_layout_frames: 3,
            sprite_render_pipeline_descriptor_frames: 3,
            sprite_render_pass_encoder_frames: 3,
            sprite_encoder_commands: 18,
            sprite_encoder_draws: 5,
            sprite_buffer_upload_frames: 3,
            sprite_quad_vertex_upload_bytes: 192,
            sprite_quad_index_upload_bytes: 36,
            sprite_buffer_instance_upload_bytes: 624,
            sprite_instance_upload_records: 13,
            sprite_instance_upload_bytes: 624,
            sprite_atlas_upload_bytes: 196_608,
            scene_projection_upload_bytes: 48,
            clean_exit: true,
            injected_inputs: vec![
                String::from("coin"),
                String::from("start_one"),
                String::from("altitude_up"),
                String::from("thrust"),
                String::from("fire"),
                String::from("reverse"),
                String::from("smart_bomb"),
                String::from("hyperspace"),
                String::from("altitude_down"),
            ],
            ..GameSmokeReport::default()
        }
    }
}
