//! Runtime-facing WGPU live launch facade.

use std::path::Path;
#[cfg(all(not(test), not(coverage)))]
use std::{
    collections::BTreeSet,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[cfg(all(not(test), not(coverage)))]
use anyhow::Context;
#[cfg(all(not(test), not(coverage)))]
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

#[cfg(any(test, all(not(test), not(coverage))))]
use crate::game::GameInput;
use crate::{audio::LiveAudioMode, game_smoke::GameSmokeReport};
#[cfg(all(not(test), not(coverage)))]
use crate::{
    audio::LiveAudioRuntime,
    game::{Game, GameFrame},
    renderer::{
        GpuRendererSettings, NativeSceneRenderer, SceneDrawPlan, SpriteBindGroupRole,
        SpriteBufferRole, SpriteBufferUpload, SpriteRenderPassEncoderCommand, SurfaceSize,
    },
    systems::{FixedStepAccumulator, FrameRate},
};

#[cfg(all(not(test), not(coverage)))]
const INITIAL_WINDOW_WIDTH: u32 = 1_024;
#[cfg(all(not(test), not(coverage)))]
const INITIAL_WINDOW_HEIGHT: u32 = 768;
#[cfg(all(not(test), not(coverage)))]
const MAX_STEPS_PER_TICK: u32 = 5;
#[cfg(all(not(test), not(coverage)))]
const EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE: u64 = 0x7269_0A81_19CA_46EE;
#[cfg(all(not(test), not(coverage)))]
const EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE: u64 = 0x1420_C241_52C8_111D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveInputProfile {
    Planetoid,
    Cabinet,
    Test,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LiveSmokeReport {
    pub(crate) frame_source: &'static str,
    pub(crate) legacy_presenter_used: bool,
    pub(crate) window_created: bool,
    pub(crate) rendered_frames: u32,
    pub(crate) first_frame_size: Option<(u32, u32)>,
    pub(crate) distinct_frame_signatures: usize,
    pub(crate) saw_non_blank_frame: bool,
    pub(crate) saw_attract: bool,
    pub(crate) saw_credit: bool,
    pub(crate) saw_playing: bool,
    pub(crate) attract_visual_frames: u32,
    pub(crate) credit_visual_frames: u32,
    pub(crate) playing_visual_frames: u32,
    pub(crate) attract_distinct_frame_signatures: usize,
    pub(crate) credit_distinct_frame_signatures: usize,
    pub(crate) playing_distinct_frame_signatures: usize,
    pub(crate) clean_game_frames: u32,
    pub(crate) sprite_frames: u32,
    pub(crate) sprite_instances: usize,
    pub(crate) sprite_draw_commands: usize,
    pub(crate) temporary_raster_frames: u32,
    pub(crate) temporary_raster_commands: usize,
    pub(crate) offscreen_wgpu_frames: u32,
    pub(crate) offscreen_non_blank_frames: u32,
    pub(crate) offscreen_distinct_frame_signatures: usize,
    pub(crate) offscreen_first_frame_signature: Option<u64>,
    pub(crate) offscreen_last_frame_signature: Option<u64>,
    pub(crate) injected_inputs: Vec<String>,
    pub(crate) clean_exit: bool,
}

impl LiveSmokeReport {
    pub(crate) fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        let offscreen_first_frame_signature = self
            .offscreen_first_frame_signature
            .map(|signature| format!("{signature:016x}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        let offscreen_last_frame_signature = self
            .offscreen_last_frame_signature
            .map(|signature| format!("{signature:016x}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "wgpu live smoke passed\n  frame_source: {}\n  legacy_presenter_used: {}\n  window_created: {}\n  rendered_frames: {}\n  first_frame_size: {}\n  distinct_frame_signatures: {}\n  saw_non_blank_frame: {}\n  saw_attract: {} (visual_frames: {}, visual_signatures: {})\n  saw_credit: {} (visual_frames: {}, visual_signatures: {})\n  saw_playing: {} (visual_frames: {}, visual_signatures: {})\n  clean_game_frames: {}\n  sprite_frames: {}\n  sprite_instances: {}\n  sprite_draw_commands: {}\n  temporary_raster_frames: {}\n  temporary_raster_commands: {}\n  offscreen_wgpu_frames: {}\n  offscreen_non_blank_frames: {}\n  offscreen_distinct_frame_signatures: {}\n  offscreen_first_frame_signature: {}\n  offscreen_last_frame_signature: {}\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.frame_source,
            self.legacy_presenter_used,
            self.window_created,
            self.rendered_frames,
            frame_size,
            self.distinct_frame_signatures,
            self.saw_non_blank_frame,
            self.saw_attract,
            self.attract_visual_frames,
            self.attract_distinct_frame_signatures,
            self.saw_credit,
            self.credit_visual_frames,
            self.credit_distinct_frame_signatures,
            self.saw_playing,
            self.playing_visual_frames,
            self.playing_distinct_frame_signatures,
            self.clean_game_frames,
            self.sprite_frames,
            self.sprite_instances,
            self.sprite_draw_commands,
            self.temporary_raster_frames,
            self.temporary_raster_commands,
            self.offscreen_wgpu_frames,
            self.offscreen_non_blank_frames,
            self.offscreen_distinct_frame_signatures,
            offscreen_first_frame_signature,
            offscreen_last_frame_signature,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }

    #[cfg(all(not(test), not(coverage)))]
    fn validate_offscreen_wgpu(&self) -> anyhow::Result<()> {
        if self.offscreen_wgpu_frames != self.rendered_frames {
            anyhow::bail!(
                "wgpu live smoke rendered {} offscreen frame(s), expected {}",
                self.offscreen_wgpu_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_non_blank_frames != self.rendered_frames {
            anyhow::bail!(
                "wgpu live smoke rendered {} nonblank offscreen frame(s), expected {}",
                self.offscreen_non_blank_frames,
                self.rendered_frames
            );
        }
        if self.offscreen_distinct_frame_signatures < 3 {
            anyhow::bail!("wgpu live smoke did not produce dynamic offscreen frame signatures");
        }
        let Some(first_frame_signature) = self.offscreen_first_frame_signature else {
            anyhow::bail!("wgpu live smoke did not record an offscreen frame signature");
        };
        if first_frame_signature != EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE {
            anyhow::bail!(
                "wgpu live smoke first offscreen frame signature {first_frame_signature:016x} did not match expected {EXPECTED_OFFSCREEN_FIRST_FRAME_SIGNATURE:016x}"
            );
        }
        let Some(last_frame_signature) = self.offscreen_last_frame_signature else {
            anyhow::bail!("wgpu live smoke did not record a final offscreen frame signature");
        };
        if last_frame_signature != EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE {
            anyhow::bail!(
                "wgpu live smoke last offscreen frame signature {last_frame_signature:016x} did not match expected {EXPECTED_OFFSCREEN_LAST_FRAME_SIGNATURE:016x}"
            );
        }
        Ok(())
    }
}

impl From<GameSmokeReport> for LiveSmokeReport {
    fn from(report: GameSmokeReport) -> Self {
        Self {
            frame_source: "clean_game",
            legacy_presenter_used: false,
            window_created: false,
            rendered_frames: report.frames,
            first_frame_size: report.first_frame_size,
            distinct_frame_signatures: report.distinct_scene_signatures,
            saw_non_blank_frame: report.sprite_frames > 0,
            saw_attract: report.saw_attract,
            saw_credit: report.saw_credit,
            saw_playing: report.saw_playing,
            attract_visual_frames: report.attract_frames,
            credit_visual_frames: report.credited_frames,
            playing_visual_frames: report.playing_frames,
            attract_distinct_frame_signatures: usize::from(report.saw_attract),
            credit_distinct_frame_signatures: usize::from(report.saw_credit),
            playing_distinct_frame_signatures: usize::from(report.saw_playing),
            clean_game_frames: report.frames,
            sprite_frames: report.sprite_frames,
            sprite_instances: report.sprite_instances,
            sprite_draw_commands: report.sprite_draw_commands,
            temporary_raster_frames: report.raster_frames,
            temporary_raster_commands: report.temporary_raster_commands,
            offscreen_wgpu_frames: 0,
            offscreen_non_blank_frames: 0,
            offscreen_distinct_frame_signatures: 0,
            offscreen_first_frame_signature: None,
            offscreen_last_frame_signature: None,
            injected_inputs: report.injected_inputs,
            clean_exit: report.clean_exit,
        }
    }
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    cmos_path: Option<&Path>,
) -> anyhow::Result<()> {
    run_clean_live(input_profile, audio_mode, cmos_path)
}

#[cfg(any(test, coverage))]
pub(crate) fn run(
    _input_profile: LiveInputProfile,
    _audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
pub(crate) fn run_smoke(
    _input_profile: LiveInputProfile,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
    let game_report = crate::game_smoke::default_smoke_report()?;
    let offscreen_report = pollster::block_on(render_offscreen_smoke())?;
    let mut report = LiveSmokeReport::from(game_report);
    report.saw_non_blank_frame = offscreen_report.non_blank_frames > 0;
    report.offscreen_wgpu_frames = offscreen_report.frames;
    report.offscreen_non_blank_frames = offscreen_report.non_blank_frames;
    report.offscreen_distinct_frame_signatures = offscreen_report.distinct_frame_signatures;
    report.offscreen_first_frame_signature = offscreen_report.first_frame_signature;
    report.offscreen_last_frame_signature = offscreen_report.last_frame_signature;
    report.validate_offscreen_wgpu()?;
    Ok(report)
}

#[cfg(any(test, coverage))]
pub(crate) fn run_smoke(
    _input_profile: LiveInputProfile,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<LiveSmokeReport> {
    crate::game_smoke::default_smoke_report().map(LiveSmokeReport::from)
}

#[cfg(all(not(test), not(coverage)))]
fn run_clean_live(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
) -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new().context("creating wgpu event loop")?;
    let mut app = CleanLiveApp::new(input_profile, LiveAudioRuntime::for_mode(audio_mode));

    event_loop
        .run_app(&mut app)
        .context("running clean wgpu live event loop")?;
    if let Some(error) = app.take_error() {
        return Err(error);
    }
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
struct CleanLiveApp {
    input_profile: LiveInputProfile,
    game: Game,
    audio: LiveAudioRuntime,
    input: LiveInputState,
    accumulator: FixedStepAccumulator,
    frame_duration: Duration,
    last_tick: Instant,
    next_wake_at: Instant,
    latest_frame: Option<GameFrame>,
    quit_requested: bool,
    window: Option<Arc<Window>>,
    presenter: Option<WgpuScenePresenter>,
    error: Option<anyhow::Error>,
}

#[cfg(all(not(test), not(coverage)))]
impl CleanLiveApp {
    fn new(input_profile: LiveInputProfile, audio: LiveAudioRuntime) -> Self {
        let now = Instant::now();
        let frame_duration = Duration::from_micros(FrameRate::CABINET.frame_duration_micros());
        let mut app = Self {
            input_profile,
            game: Game::new(),
            audio,
            input: LiveInputState::default(),
            accumulator: FixedStepAccumulator::new(FrameRate::CABINET),
            frame_duration,
            last_tick: now,
            next_wake_at: now + frame_duration,
            latest_frame: None,
            quit_requested: false,
            window: None,
            presenter: None,
            error: None,
        };
        app.step_one_frame();
        app
    }

    fn take_error(&mut self) -> Option<anyhow::Error> {
        self.error.take()
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Defender")
                        .with_inner_size(LogicalSize::new(
                            f64::from(INITIAL_WINDOW_WIDTH),
                            f64::from(INITIAL_WINDOW_HEIGHT),
                        )),
                )
                .context("creating clean wgpu window")?,
        );
        let presenter = pollster::block_on(WgpuScenePresenter::new(window.clone()))
            .context("initializing clean wgpu presenter")?;
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.last_tick = Instant::now();
        self.next_wake_at = self.last_tick + self.frame_duration;
        Ok(())
    }

    fn handle_error(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
        event_loop.exit();
    }

    fn window_matches(&self, window_id: WindowId) -> bool {
        self.window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        let Some(control) = live_control_from_winit(self.input_profile, event) else {
            return;
        };
        let pressed = event.state == ElementState::Pressed;
        if control == LiveControl::Quit && pressed {
            self.quit_requested = true;
            return;
        }
        self.input.apply(control, pressed);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let Some((width, height)) = renderable_window_size(size) else {
            return;
        };
        if let Some(presenter) = &mut self.presenter {
            presenter.resize(width, height);
        }
    }

    fn step_one_frame(&mut self) {
        let frame = self.game.step(self.input.drain_game_input());
        self.audio.submit_game_frame(&frame);
        self.latest_frame = Some(frame);
    }

    fn step_due_frames(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        self.accumulator
            .add_elapsed_micros(elapsed.as_micros().try_into().unwrap_or(u64::MAX));
        let due_steps = self.accumulator.consume_due_steps(MAX_STEPS_PER_TICK);

        for _ in 0..due_steps {
            self.step_one_frame();
        }

        let micros_until_next = FrameRate::CABINET
            .frame_duration_micros()
            .saturating_sub(self.accumulator.accumulated_micros())
            .max(1);
        self.next_wake_at = Instant::now() + Duration::from_micros(micros_until_next);
        due_steps > 0
    }

    fn draw_frame(&mut self) -> anyhow::Result<()> {
        let Some(frame) = &self.latest_frame else {
            return Ok(());
        };
        let Some(presenter) = &mut self.presenter else {
            return Ok(());
        };
        presenter.draw_scene(&frame.scene)
    }
}

#[cfg(all(not(test), not(coverage)))]
impl ApplicationHandler for CleanLiveApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize_window(event_loop) {
            self.handle_error(event_loop, error);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.window_matches(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(&event);
                if self.quit_requested {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }
                if let Err(error) = self.draw_frame() {
                    self.handle_error(event_loop, error);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.error.is_some() || self.quit_requested {
            event_loop.exit();
            return;
        }

        if self.step_due_frames()
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_wake_at));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.presenter = None;
        self.window = None;
    }
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuScenePresenter {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: NativeSceneRenderer,
    sprite_resources: Option<SpriteGpuResources>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuScenePresenter {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let (width, height) =
            renderable_window_size(size).unwrap_or((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .context("creating clean wgpu surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("requesting clean wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("defender clean wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting clean wgpu device")?;
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| {
                anyhow::anyhow!("wgpu surface is not supported by the selected adapter")
            })?;
        config.present_mode = wgpu::PresentMode::Fifo;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);
        let settings = GpuRendererSettings {
            texture_format: config.format,
            present_mode: config.present_mode,
            alpha_mode: config.alpha_mode,
        };
        let renderer = NativeSceneRenderer::with_settings(Default::default(), settings);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            sprite_resources: None,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn draw_scene(&mut self, scene: &crate::renderer::RenderScene) -> anyhow::Result<()> {
        let target = SurfaceSize::new(self.config.width, self.config.height);
        let plan = self.renderer.prepare_for_target(scene, target);
        self.update_sprite_resources(&plan)?;

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let size = self.window.inner_size();
                if let Some((width, height)) = renderable_window_size(size) {
                    self.resize(width, height);
                }
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                anyhow::bail!("wgpu surface validation error while acquiring frame")
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender clean wgpu frame encoder"),
            });
        encode_scene_render_pass(&mut encoder, &view, &plan, self.sprite_resources.as_ref());

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OffscreenWgpuSmokeReport {
    frames: u32,
    non_blank_frames: u32,
    distinct_frame_signatures: usize,
    first_frame_signature: Option<u64>,
    last_frame_signature: Option<u64>,
}

#[cfg(all(not(test), not(coverage)))]
async fn render_offscreen_smoke() -> anyhow::Result<OffscreenWgpuSmokeReport> {
    let mut renderer = WgpuOffscreenRenderer::new().await?;
    let mut game = Game::new();
    let mut signatures = BTreeSet::new();
    let mut report = OffscreenWgpuSmokeReport::default();

    for frame_index in 0..crate::game_smoke::smoke_frame_count() {
        let frame = game.step(crate::game_smoke::smoke_game_input(frame_index));
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
            .checked_mul(4)
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
    pixels.chunks_exact(4).any(|pixel| pixel != [0, 0, 0, 0])
}

#[cfg(all(not(test), not(coverage)))]
fn rendered_rgba_signature(surface: SurfaceSize, pixels: &[u8]) -> u64 {
    let mut signature = 0xCBF2_9CE4_8422_2325_u64;
    signature = fnv1a_mix_u64(signature, u64::from(surface.width));
    signature = fnv1a_mix_u64(signature, u64::from(surface.height));
    for byte in pixels {
        signature ^= u64::from(*byte);
        signature = signature.wrapping_mul(0x0000_0100_0000_01B3);
    }
    signature
}

#[cfg(all(not(test), not(coverage)))]
fn fnv1a_mix_u64(mut signature: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        signature ^= u64::from(byte);
        signature = signature.wrapping_mul(0x0000_0100_0000_01B3);
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

#[cfg(all(not(test), not(coverage)))]
fn renderable_window_size(size: PhysicalSize<u32>) -> Option<(u32, u32)> {
    if size.width == 0 || size.height == 0 {
        None
    } else {
        Some((size.width, size.height))
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveControl {
    Coin,
    StartOne,
    StartTwo,
    AltitudeUp,
    AltitudeDown,
    Reverse,
    Thrust,
    Fire,
    SmartBomb,
    Hyperspace,
    ServiceAutoUp,
    ServiceAdvance,
    HighScoreReset,
    HighScoreBackspace,
    HighScoreInitial(char),
    Quit,
}

#[cfg(any(test, all(not(test), not(coverage))))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LiveInputState {
    coin: bool,
    start_one: bool,
    start_two: bool,
    altitude_up: bool,
    altitude_down: bool,
    reverse: bool,
    thrust: bool,
    fire: bool,
    smart_bomb: bool,
    hyperspace: bool,
    service_auto_up: bool,
    service_advance: bool,
    high_score_reset: bool,
    high_score_initial: Option<char>,
    high_score_backspace: bool,
}

#[cfg(any(test, all(not(test), not(coverage))))]
impl LiveInputState {
    fn apply(&mut self, control: LiveControl, pressed: bool) {
        match control {
            LiveControl::Coin => self.coin |= pressed,
            LiveControl::StartOne => self.start_one |= pressed,
            LiveControl::StartTwo => self.start_two |= pressed,
            LiveControl::AltitudeUp => self.altitude_up = pressed,
            LiveControl::AltitudeDown => self.altitude_down = pressed,
            LiveControl::Reverse => self.reverse = pressed,
            LiveControl::Thrust => self.thrust = pressed,
            LiveControl::Fire => self.fire = pressed,
            LiveControl::SmartBomb => self.smart_bomb = pressed,
            LiveControl::Hyperspace => self.hyperspace = pressed,
            LiveControl::ServiceAutoUp => self.service_auto_up = pressed,
            LiveControl::ServiceAdvance => self.service_advance |= pressed,
            LiveControl::HighScoreReset => self.high_score_reset |= pressed,
            LiveControl::HighScoreBackspace => self.high_score_backspace |= pressed,
            LiveControl::HighScoreInitial(value) => {
                if pressed {
                    self.high_score_initial = Some(value);
                }
            }
            LiveControl::Quit => {}
        }
    }

    fn drain_game_input(&mut self) -> GameInput {
        GameInput {
            coin: take_bool(&mut self.coin),
            coin_two: false,
            coin_three: false,
            start_one: take_bool(&mut self.start_one),
            start_two: take_bool(&mut self.start_two),
            altitude_up: self.altitude_up,
            altitude_down: self.altitude_down,
            reverse: self.reverse,
            thrust: self.thrust,
            fire: self.fire,
            smart_bomb: self.smart_bomb,
            hyperspace: self.hyperspace,
            service_auto_up: self.service_auto_up,
            service_advance: take_bool(&mut self.service_advance),
            high_score_reset: take_bool(&mut self.high_score_reset),
            high_score_initial: self.high_score_initial.take(),
            high_score_backspace: take_bool(&mut self.high_score_backspace),
            tilt: false,
        }
    }
}

#[cfg(any(test, all(not(test), not(coverage))))]
fn take_bool(value: &mut bool) -> bool {
    let taken = *value;
    *value = false;
    taken
}

#[cfg(all(not(test), not(coverage)))]
fn live_control_from_winit(profile: LiveInputProfile, event: &KeyEvent) -> Option<LiveControl> {
    physical_control(profile, &event.physical_key)
        .or_else(|| logical_control(profile, &event.logical_key))
}

#[cfg(all(not(test), not(coverage)))]
fn physical_control(profile: LiveInputProfile, physical_key: &PhysicalKey) -> Option<LiveControl> {
    let PhysicalKey::Code(code) = physical_key else {
        return None;
    };

    match code {
        KeyCode::Escape => Some(LiveControl::Quit),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(LiveControl::Coin),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(LiveControl::StartOne),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(LiveControl::StartTwo),
        KeyCode::F1 => Some(LiveControl::ServiceAutoUp),
        KeyCode::F2 => Some(LiveControl::ServiceAdvance),
        KeyCode::F3 => Some(LiveControl::HighScoreReset),
        KeyCode::Backspace => Some(LiveControl::HighScoreBackspace),
        _ => gameplay_physical_control(profile, *code),
    }
}

#[cfg(all(not(test), not(coverage)))]
fn gameplay_physical_control(profile: LiveInputProfile, code: KeyCode) -> Option<LiveControl> {
    match profile {
        LiveInputProfile::Planetoid => match code {
            KeyCode::Enter | KeyCode::NumpadEnter => Some(LiveControl::Fire),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(LiveControl::Thrust),
            KeyCode::KeyA => Some(LiveControl::AltitudeUp),
            KeyCode::KeyZ => Some(LiveControl::AltitudeDown),
            KeyCode::Space => Some(LiveControl::Reverse),
            KeyCode::Tab => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
        LiveInputProfile::Cabinet | LiveInputProfile::Test => match code {
            KeyCode::KeyF => Some(LiveControl::Fire),
            KeyCode::KeyT => Some(LiveControl::Thrust),
            KeyCode::ArrowUp => Some(LiveControl::AltitudeUp),
            KeyCode::ArrowDown => Some(LiveControl::AltitudeDown),
            KeyCode::KeyR => Some(LiveControl::Reverse),
            KeyCode::KeyB => Some(LiveControl::SmartBomb),
            KeyCode::KeyH => Some(LiveControl::Hyperspace),
            _ => None,
        },
    }
}

#[cfg(all(not(test), not(coverage)))]
fn logical_control(profile: LiveInputProfile, logical_key: &Key) -> Option<LiveControl> {
    match logical_key {
        Key::Named(NamedKey::Escape) => Some(LiveControl::Quit),
        Key::Named(NamedKey::Enter) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Fire)
        }
        Key::Named(NamedKey::Tab) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::SmartBomb)
        }
        Key::Named(NamedKey::Backspace) => Some(LiveControl::HighScoreBackspace),
        Key::Named(NamedKey::ArrowUp) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeUp)
        }
        Key::Named(NamedKey::ArrowDown) => {
            (profile != LiveInputProfile::Planetoid).then_some(LiveControl::AltitudeDown)
        }
        Key::Named(NamedKey::Shift) => {
            (profile == LiveInputProfile::Planetoid).then_some(LiveControl::Thrust)
        }
        Key::Named(NamedKey::F1) => Some(LiveControl::ServiceAutoUp),
        Key::Named(NamedKey::F2) => Some(LiveControl::ServiceAdvance),
        Key::Named(NamedKey::F3) => Some(LiveControl::HighScoreReset),
        Key::Character(text) => character_control(profile, text),
        _ => None,
    }
}

#[cfg(all(not(test), not(coverage)))]
fn character_control(profile: LiveInputProfile, text: &str) -> Option<LiveControl> {
    let value = single_character(text)?;
    match value.to_ascii_lowercase() {
        '1' => Some(LiveControl::StartOne),
        '2' => Some(LiveControl::StartTwo),
        '5' => Some(LiveControl::Coin),
        'a' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeUp),
        'z' if profile == LiveInputProfile::Planetoid => Some(LiveControl::AltitudeDown),
        ' ' if profile == LiveInputProfile::Planetoid => Some(LiveControl::Reverse),
        'h' => Some(LiveControl::Hyperspace),
        'f' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Fire),
        't' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Thrust),
        'r' if profile != LiveInputProfile::Planetoid => Some(LiveControl::Reverse),
        'b' if profile != LiveInputProfile::Planetoid => Some(LiveControl::SmartBomb),
        'a'..='z' => Some(LiveControl::HighScoreInitial(value.to_ascii_uppercase())),
        _ => None,
    }
}

#[cfg(all(not(test), not(coverage)))]
fn single_character(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let value = chars.next()?;
    chars.next().is_none().then_some(value)
}

#[cfg(test)]
mod tests {
    use crate::GameInput;

    use super::{LiveInputState, LiveSmokeReport, run_smoke};

    #[test]
    fn live_smoke_report_formats_current_cli_output() {
        let report = LiveSmokeReport {
            frame_source: "clean_game",
            legacy_presenter_used: false,
            window_created: false,
            rendered_frames: 3,
            first_frame_size: Some((640, 480)),
            distinct_frame_signatures: 2,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 1,
            credit_visual_frames: 1,
            playing_visual_frames: 1,
            attract_distinct_frame_signatures: 1,
            credit_distinct_frame_signatures: 1,
            playing_distinct_frame_signatures: 1,
            clean_game_frames: 3,
            sprite_frames: 3,
            sprite_instances: 12,
            sprite_draw_commands: 4,
            temporary_raster_frames: 0,
            temporary_raster_commands: 0,
            offscreen_wgpu_frames: 3,
            offscreen_non_blank_frames: 3,
            offscreen_distinct_frame_signatures: 2,
            offscreen_first_frame_signature: Some(0x1234_ABCD),
            offscreen_last_frame_signature: Some(0xABCD_1234),
            injected_inputs: vec![String::from("coin"), String::from("start_one")],
            clean_exit: true,
        };

        assert_eq!(
            report.to_text(),
            concat!(
                "wgpu live smoke passed\n",
                "  frame_source: clean_game\n",
                "  legacy_presenter_used: false\n",
                "  window_created: false\n",
                "  rendered_frames: 3\n",
                "  first_frame_size: 640x480\n",
                "  distinct_frame_signatures: 2\n",
                "  saw_non_blank_frame: true\n",
                "  saw_attract: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_credit: true (visual_frames: 1, visual_signatures: 1)\n",
                "  saw_playing: true (visual_frames: 1, visual_signatures: 1)\n",
                "  clean_game_frames: 3\n",
                "  sprite_frames: 3\n",
                "  sprite_instances: 12\n",
                "  sprite_draw_commands: 4\n",
                "  temporary_raster_frames: 0\n",
                "  temporary_raster_commands: 0\n",
                "  offscreen_wgpu_frames: 3\n",
                "  offscreen_non_blank_frames: 3\n",
                "  offscreen_distinct_frame_signatures: 2\n",
                "  offscreen_first_frame_signature: 000000001234abcd\n",
                "  offscreen_last_frame_signature: 00000000abcd1234\n",
                "  injected_inputs: coin,start_one\n",
                "  clean_exit: true\n",
            )
        );
    }

    #[test]
    fn live_smoke_uses_clean_game_frame_source() {
        let report = run_smoke(super::LiveInputProfile::Test, None).expect("clean live smoke");

        assert_eq!(report.frame_source, "clean_game");
        assert!(!report.legacy_presenter_used);
        assert!(!report.window_created);
        assert_eq!(report.clean_game_frames, report.rendered_frames);
        assert_eq!(report.temporary_raster_frames, 0);
        assert_eq!(report.temporary_raster_commands, 0);
        assert!(report.sprite_frames > 0);
        assert!(report.sprite_instances > 0);
        assert!(report.sprite_draw_commands > 0);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
    }

    #[test]
    fn live_input_state_emits_edge_pulses_and_held_gameplay_controls() {
        let mut input = LiveInputState::default();
        input.apply(super::LiveControl::Coin, true);
        input.apply(super::LiveControl::StartOne, true);
        input.apply(super::LiveControl::StartTwo, true);
        input.apply(super::LiveControl::Thrust, true);
        input.apply(super::LiveControl::AltitudeUp, true);
        input.apply(super::LiveControl::AltitudeDown, true);
        input.apply(super::LiveControl::Reverse, true);
        input.apply(super::LiveControl::Fire, true);
        input.apply(super::LiveControl::SmartBomb, true);
        input.apply(super::LiveControl::Hyperspace, true);
        input.apply(super::LiveControl::ServiceAutoUp, true);
        input.apply(super::LiveControl::ServiceAdvance, true);
        input.apply(super::LiveControl::HighScoreReset, true);
        input.apply(super::LiveControl::HighScoreInitial('A'), true);
        input.apply(super::LiveControl::HighScoreBackspace, true);
        input.apply(super::LiveControl::Quit, true);

        assert_eq!(
            input.drain_game_input(),
            GameInput {
                coin: true,
                start_one: true,
                start_two: true,
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                service_advance: true,
                high_score_reset: true,
                high_score_initial: Some('A'),
                high_score_backspace: true,
                ..GameInput::NONE
            }
        );
        assert_eq!(
            input.drain_game_input(),
            GameInput {
                thrust: true,
                altitude_up: true,
                altitude_down: true,
                reverse: true,
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                service_auto_up: true,
                ..GameInput::NONE
            }
        );

        input.apply(super::LiveControl::Thrust, false);
        input.apply(super::LiveControl::AltitudeUp, false);
        input.apply(super::LiveControl::AltitudeDown, false);
        input.apply(super::LiveControl::Reverse, false);
        input.apply(super::LiveControl::Fire, false);
        input.apply(super::LiveControl::SmartBomb, false);
        input.apply(super::LiveControl::Hyperspace, false);
        input.apply(super::LiveControl::ServiceAutoUp, false);
        assert_eq!(input.drain_game_input(), GameInput::NONE);
    }
}
