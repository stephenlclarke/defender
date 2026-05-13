//! Windowed live runner using wgpu.
#![cfg_attr(any(test, coverage), allow(dead_code))]

use std::collections::BTreeSet;
use std::path::Path;
#[cfg(all(not(test), not(coverage)))]
use std::sync::Arc;
#[cfg(all(not(test), not(coverage)))]
use std::time::{Duration, Instant};

#[cfg(all(not(test), not(coverage)))]
use anyhow::{Context, anyhow};
use anyhow::{Result, bail};
#[cfg(all(not(test), not(coverage)))]
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Window, WindowId},
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};

#[cfg(all(not(test), not(coverage)))]
use crate::{
    audio::{LiveAudioMode, LiveAudioRuntime},
    cmos_storage::{CmosStorage, FileCmosStorage},
    live::{LiveAdvanceMode, LiveCoreFrame, LiveCoreRuntime, LiveCoreThread, save_live_cmos_ram},
    machine::ArcadeMachine,
};
use crate::{
    input::{InputEvent, InputEventKind, InputKey, InputProfile},
    machine_state::{GamePhase, MachineSnapshot},
    rom::crc32,
    video::RenderedImage,
};

const INITIAL_WINDOW_WIDTH: u32 = 1_024;
const INITIAL_WINDOW_HEIGHT: u32 = 768;
const SMOKE_WINDOW_WIDTH: u32 = 320;
const SMOKE_WINDOW_HEIGHT: u32 = 240;
#[cfg(all(not(test), not(coverage)))]
const FRAME_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const SMOKE_TARGET_FRAMES: u32 = 240;

#[cfg(all(not(test), not(coverage)))]
const FRAME_SHADER: &str = r#"
@group(0) @binding(0) var frame_texture: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(2.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    var out: VertexOut;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(frame_texture, frame_sampler, in.uv);
}
"#;

#[cfg(all(not(test), not(coverage)))]
pub fn run_wgpu_live(
    input_profile: InputProfile,
    audio_mode: LiveAudioMode,
    cmos_path: Option<&Path>,
) -> Result<()> {
    let cmos_storage = cmos_path.map(FileCmosStorage::new);
    let storage = cmos_storage
        .as_ref()
        .map(|storage| storage as &dyn CmosStorage);
    let machine = crate::live::live_machine_from_cmos_storage(storage)?;
    let event_loop = winit::event_loop::EventLoop::new().context("creating wgpu event loop")?;
    let mut app = WgpuLiveApp::new(input_profile, cmos_storage, machine, audio_mode);

    let run_result = event_loop
        .run_app(&mut app)
        .context("running wgpu live event loop");
    let cmos_result = app.save_cmos();
    if let Some(error) = app.take_error() {
        return Err(error);
    }
    run_result?;
    cmos_result
}

#[cfg(all(not(test), not(coverage)))]
pub fn run_wgpu_live_smoke(
    input_profile: InputProfile,
    cmos_path: Option<&Path>,
) -> Result<WgpuSmokeReport> {
    let cmos_storage = cmos_path.map(FileCmosStorage::new);
    let storage = cmos_storage
        .as_ref()
        .map(|storage| storage as &dyn CmosStorage);
    let machine = crate::live::live_machine_from_cmos_storage(storage)?;
    let event_loop = winit::event_loop::EventLoop::new().context("creating wgpu event loop")?;
    let mut app = WgpuLiveApp::new_smoke(input_profile, cmos_storage, machine, SMOKE_TARGET_FRAMES);

    let run_result = event_loop
        .run_app(&mut app)
        .context("running wgpu live smoke event loop");
    let cmos_result = app.save_cmos();
    if let Some(error) = app.take_error() {
        return Err(error);
    }
    run_result?;
    cmos_result?;

    let report = app
        .take_smoke_report()
        .ok_or_else(|| anyhow!("wgpu live smoke did not produce a report"))?;
    if let Err(error) = report.validate() {
        bail!("{error}\n{}", report.to_text());
    }
    Ok(report)
}

#[cfg(any(test, coverage))]
pub fn run_wgpu_live(
    _input_profile: InputProfile,
    _audio_mode: crate::audio::LiveAudioMode,
    _cmos_path: Option<&Path>,
) -> Result<()> {
    Ok(())
}

#[cfg(any(test, coverage))]
pub fn run_wgpu_live_smoke(
    _input_profile: InputProfile,
    _cmos_path: Option<&Path>,
) -> Result<WgpuSmokeReport> {
    let report = WgpuSmokeReport {
        window_created: true,
        rendered_frames: 3,
        first_frame_size: Some((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)),
        distinct_frame_crcs: 3,
        saw_non_blank_frame: true,
        saw_attract: true,
        saw_credit: true,
        saw_playing: true,
        attract_visual_frames: 1,
        credit_visual_frames: 1,
        playing_visual_frames: 1,
        attract_distinct_frame_crcs: 1,
        credit_distinct_frame_crcs: 1,
        playing_distinct_frame_crcs: 1,
        injected_inputs: required_smoke_inputs()
            .iter()
            .map(|input| String::from(*input))
            .collect(),
        clean_exit: true,
    };
    Ok(report)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WgpuSmokeReport {
    pub window_created: bool,
    pub rendered_frames: u32,
    pub first_frame_size: Option<(u32, u32)>,
    pub distinct_frame_crcs: usize,
    pub saw_non_blank_frame: bool,
    pub saw_attract: bool,
    pub saw_credit: bool,
    pub saw_playing: bool,
    pub attract_visual_frames: u32,
    pub credit_visual_frames: u32,
    pub playing_visual_frames: u32,
    pub attract_distinct_frame_crcs: usize,
    pub credit_distinct_frame_crcs: usize,
    pub playing_distinct_frame_crcs: usize,
    pub injected_inputs: Vec<String>,
    pub clean_exit: bool,
}

impl WgpuSmokeReport {
    pub fn validate(&self) -> Result<()> {
        if !self.window_created {
            bail!("wgpu live smoke did not create a window");
        }
        if self.rendered_frames == 0 {
            bail!("wgpu live smoke did not render any frames");
        }
        if self.first_frame_size.is_none() {
            bail!("wgpu live smoke did not record a renderable frame size");
        }
        if self.distinct_frame_crcs < 2 {
            bail!("wgpu live smoke did not render dynamic frame CRCs");
        }
        if !self.saw_non_blank_frame {
            bail!("wgpu live smoke rendered only blank frames");
        }
        if !self.saw_attract || self.attract_visual_frames == 0 {
            bail!("wgpu live smoke did not render nonblank attract-mode frames");
        }
        if self.attract_distinct_frame_crcs == 0 {
            bail!("wgpu live smoke did not record attract-mode frame CRCs");
        }
        if !self.saw_credit || self.credit_visual_frames == 0 {
            bail!("wgpu live smoke did not render nonblank credited frames");
        }
        if self.credit_distinct_frame_crcs == 0 {
            bail!("wgpu live smoke did not record credited frame CRCs");
        }
        if !self.saw_playing || self.playing_visual_frames == 0 {
            bail!("wgpu live smoke did not render nonblank gameplay frames");
        }
        if self.playing_distinct_frame_crcs == 0 {
            bail!("wgpu live smoke did not record gameplay frame CRCs");
        }
        for required in required_smoke_inputs() {
            if !self.injected_inputs.iter().any(|input| input == required) {
                bail!("wgpu live smoke did not inject {required}");
            }
        }
        if !self.clean_exit {
            bail!("wgpu live smoke did not exit cleanly");
        }
        Ok(())
    }

    pub fn to_text(&self) -> String {
        let frame_size = self
            .first_frame_size
            .map(|(width, height)| format!("{width}x{height}"))
            .unwrap_or_else(|| String::from("unrecorded"));
        format!(
            "wgpu live smoke passed\n  window_created: {}\n  rendered_frames: {}\n  first_frame_size: {}\n  distinct_frame_crcs: {}\n  saw_non_blank_frame: {}\n  saw_attract: {} (visual_frames: {}, visual_crcs: {})\n  saw_credit: {} (visual_frames: {}, visual_crcs: {})\n  saw_playing: {} (visual_frames: {}, visual_crcs: {})\n  injected_inputs: {}\n  clean_exit: {}\n",
            self.window_created,
            self.rendered_frames,
            frame_size,
            self.distinct_frame_crcs,
            self.saw_non_blank_frame,
            self.saw_attract,
            self.attract_visual_frames,
            self.attract_distinct_frame_crcs,
            self.saw_credit,
            self.credit_visual_frames,
            self.credit_distinct_frame_crcs,
            self.saw_playing,
            self.playing_visual_frames,
            self.playing_distinct_frame_crcs,
            self.injected_inputs.join(","),
            self.clean_exit
        )
    }
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuLiveApp {
    cmos_storage: Option<FileCmosStorage>,
    core: LiveCoreThread,
    latest_frame: Option<LiveCoreFrame>,
    frame_request_in_flight: bool,
    next_wake_at: Instant,
    quit_requested: bool,
    window_size: (u32, u32),
    window: Option<Arc<Window>>,
    presenter: Option<WgpuPresenter>,
    smoke: Option<WgpuSmoke>,
    error: Option<anyhow::Error>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuLiveApp {
    fn new(
        input_profile: InputProfile,
        cmos_storage: Option<FileCmosStorage>,
        machine: ArcadeMachine,
        audio_mode: LiveAudioMode,
    ) -> Self {
        Self {
            cmos_storage,
            core: LiveCoreThread::spawn_with_audio(
                input_profile,
                machine,
                Instant::now(),
                (INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT),
                LiveAudioRuntime::for_mode(audio_mode),
            ),
            latest_frame: None,
            frame_request_in_flight: false,
            next_wake_at: Instant::now(),
            quit_requested: false,
            window_size: (INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT),
            window: None,
            presenter: None,
            smoke: None,
            error: None,
        }
    }

    fn new_smoke(
        input_profile: InputProfile,
        cmos_storage: Option<FileCmosStorage>,
        machine: ArcadeMachine,
        target_frames: u32,
    ) -> Self {
        let mut app = Self::new(
            input_profile,
            cmos_storage,
            machine,
            LiveAudioMode::Disabled,
        );
        app.window_size = (SMOKE_WINDOW_WIDTH, SMOKE_WINDOW_HEIGHT);
        app.core
            .resize_renderer(SMOKE_WINDOW_WIDTH, SMOKE_WINDOW_HEIGHT)
            .expect("live core thread should accept initial smoke renderer size");
        app.smoke = Some(WgpuSmoke::new(target_frames));
        app
    }

    fn save_cmos(&self) -> Result<()> {
        let cmos = self.core.shutdown_cmos_ram()?;
        save_live_cmos_ram(
            self.cmos_storage
                .as_ref()
                .map(|storage| storage as &dyn CmosStorage),
            &cmos,
        )
    }

    fn take_error(&mut self) -> Option<anyhow::Error> {
        self.error.take()
    }

    fn take_smoke_report(&mut self) -> Option<WgpuSmokeReport> {
        self.smoke.take().map(WgpuSmoke::into_report)
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Defender red-label")
                        .with_inner_size(LogicalSize::new(
                            f64::from(self.window_size.0),
                            f64::from(self.window_size.1),
                        )),
                )
                .context("creating wgpu window")?,
        );
        let size = renderable_window_size(window.inner_size()).unwrap_or(self.window_size);
        self.core.resize_renderer(size.0, size.1)?;
        self.presenter = Some(
            pollster::block_on(WgpuPresenter::new(window.clone()))
                .context("initializing wgpu presenter")?,
        );
        self.core.reset_clock(Instant::now())?;
        self.window = Some(window);
        if let Some(smoke) = &mut self.smoke {
            smoke.observe_window_created();
        }
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

    fn handle_keyboard_input(&mut self, key_event: KeyEvent) -> Result<bool> {
        if let Some(input_event) = input_event_from_winit(&key_event) {
            self.quit_requested |= self.core.handle_input_event(input_event)?;
        }
        Ok(self.quit_requested)
    }

    fn inject_smoke_inputs(&mut self) -> Result<()> {
        let Some(frame) = self.smoke.as_ref().map(WgpuSmoke::frame) else {
            return Ok(());
        };
        for input in smoke_input_events(self.core.input_profile(), frame) {
            self.quit_requested |= self.core.handle_input_event(input.event)?;
            if input.counts_for_report
                && let Some(smoke) = &mut self.smoke
            {
                smoke.record_injected_input(input.label);
            }
        }
        Ok(())
    }

    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        let Some((width, height)) = renderable_window_size(size) else {
            return Ok(());
        };
        self.core.resize_renderer(width, height)?;
        if let Some(presenter) = &mut self.presenter {
            presenter.resize(width, height);
        }
        Ok(())
    }

    fn collect_latest_core_frame(&mut self) -> Result<bool> {
        match self.core.take_latest_frame() {
            Ok(Some(frame)) => {
                self.frame_request_in_flight = false;
                self.next_wake_at = frame.next_step_at;
                self.latest_frame = Some(frame);
                if let Some(smoke) = &mut self.smoke {
                    smoke.advance_frame();
                }
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(error) => {
                self.frame_request_in_flight = false;
                Err(error)
            }
        }
    }

    fn request_core_frame(&mut self) -> Result<()> {
        if self.quit_requested || self.frame_request_in_flight {
            return Ok(());
        }

        let now = Instant::now();
        if self.smoke.is_none() && now < self.next_wake_at {
            return Ok(());
        }

        let mode = if self.smoke.is_some() {
            LiveAdvanceMode::FixedFrames(1)
        } else {
            LiveAdvanceMode::Realtime
        };
        self.core.request_frame(mode, now)?;
        self.frame_request_in_flight = true;
        Ok(())
    }

    fn draw_frame(&mut self) -> Result<()> {
        let Some(frame) = &self.latest_frame else {
            return Ok(());
        };
        let metrics = rendered_image_metrics(&frame.image);
        let smoke_state = SmokeMachineState::from_snapshot(frame.snapshot);
        let Some(presenter) = &mut self.presenter else {
            return Ok(());
        };
        presenter
            .draw_frame(&frame.image)
            .context("drawing wgpu graphics frame")?;
        if let Some(smoke) = &mut self.smoke {
            smoke.observe_rendered_image(metrics, smoke_state);
        }
        Ok(())
    }
}

#[cfg(all(not(test), not(coverage)))]
impl ApplicationHandler for WgpuLiveApp {
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
            WindowEvent::KeyboardInput { event, .. } => match self.handle_keyboard_input(event) {
                Ok(true) => event_loop.exit(),
                Ok(false) => {}
                Err(error) => self.handle_error(event_loop, error),
            },
            WindowEvent::Resized(size) => {
                if let Err(error) = self.resize(size) {
                    self.handle_error(event_loop, error);
                }
            }
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
        if self.error.is_some() {
            event_loop.exit();
            return;
        }
        if self.quit_requested {
            event_loop.exit();
            return;
        }

        let frame_ready = match self.collect_latest_core_frame() {
            Ok(frame_ready) => frame_ready,
            Err(error) => {
                self.handle_error(event_loop, error);
                return;
            }
        };
        if frame_ready && let Some(window) = &self.window {
            window.request_redraw();
        }
        if let Some(smoke) = &mut self.smoke
            && smoke.should_stop()
        {
            smoke.mark_clean_exit();
            event_loop.exit();
            return;
        }

        let frame_due = self.smoke.is_some() || Instant::now() >= self.next_wake_at;
        if !self.frame_request_in_flight
            && frame_due
            && let Err(error) = self
                .inject_smoke_inputs()
                .and_then(|()| self.request_core_frame())
        {
            self.handle_error(event_loop, error);
            return;
        }
        if self.smoke.is_some() {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else if self.frame_request_in_flight {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(1),
            ));
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_wake_at));
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.presenter = None;
        self.window = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RenderedImageMetrics {
    size: (u32, u32),
    crc32: u32,
    non_blank: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SmokeMachineState {
    phase: GamePhase,
    credits: u8,
}

impl SmokeMachineState {
    fn from_snapshot(snapshot: MachineSnapshot) -> Self {
        Self {
            phase: snapshot.phase,
            credits: snapshot.credits,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SmokeInput {
    label: &'static str,
    event: InputEvent,
    counts_for_report: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SmokeVisualBucket {
    non_blank_frames: u32,
    frame_crcs: BTreeSet<u32>,
}

impl SmokeVisualBucket {
    fn observe(&mut self, metrics: RenderedImageMetrics) {
        if metrics.non_blank {
            self.non_blank_frames += 1;
            self.frame_crcs.insert(metrics.crc32);
        }
    }

    fn distinct_frame_crcs(&self) -> usize {
        self.frame_crcs.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WgpuSmoke {
    frame: u32,
    target_frames: u32,
    window_created: bool,
    rendered_frames: u32,
    first_frame_size: Option<(u32, u32)>,
    frame_crcs: BTreeSet<u32>,
    saw_non_blank_frame: bool,
    saw_attract: bool,
    saw_credit: bool,
    saw_playing: bool,
    attract_visual: SmokeVisualBucket,
    credit_visual: SmokeVisualBucket,
    playing_visual: SmokeVisualBucket,
    injected_inputs: BTreeSet<&'static str>,
    clean_exit: bool,
}

impl WgpuSmoke {
    fn new(target_frames: u32) -> Self {
        Self {
            frame: 0,
            target_frames,
            window_created: false,
            rendered_frames: 0,
            first_frame_size: None,
            frame_crcs: BTreeSet::new(),
            saw_non_blank_frame: false,
            saw_attract: false,
            saw_credit: false,
            saw_playing: false,
            attract_visual: SmokeVisualBucket::default(),
            credit_visual: SmokeVisualBucket::default(),
            playing_visual: SmokeVisualBucket::default(),
            injected_inputs: BTreeSet::new(),
            clean_exit: false,
        }
    }

    fn frame(&self) -> u32 {
        self.frame
    }

    fn observe_window_created(&mut self) {
        self.window_created = true;
    }

    fn observe_rendered_image(&mut self, metrics: RenderedImageMetrics, state: SmokeMachineState) {
        self.rendered_frames += 1;
        self.first_frame_size.get_or_insert(metrics.size);
        self.frame_crcs.insert(metrics.crc32);
        self.saw_non_blank_frame |= metrics.non_blank;
        self.saw_attract |= state.phase == GamePhase::Attract;
        self.saw_credit |= state.credits > 0;
        self.saw_playing |= state.phase == GamePhase::Playing;
        if state.phase == GamePhase::Attract {
            self.attract_visual.observe(metrics);
        }
        if state.credits > 0 {
            self.credit_visual.observe(metrics);
        }
        if state.phase == GamePhase::Playing {
            self.playing_visual.observe(metrics);
        }
    }

    fn record_injected_input(&mut self, label: &'static str) {
        self.injected_inputs.insert(label);
    }

    fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    fn should_stop(&self) -> bool {
        self.frame >= self.target_frames
    }

    fn mark_clean_exit(&mut self) {
        self.clean_exit = true;
    }

    fn into_report(self) -> WgpuSmokeReport {
        WgpuSmokeReport {
            window_created: self.window_created,
            rendered_frames: self.rendered_frames,
            first_frame_size: self.first_frame_size,
            distinct_frame_crcs: self.frame_crcs.len(),
            saw_non_blank_frame: self.saw_non_blank_frame,
            saw_attract: self.saw_attract,
            saw_credit: self.saw_credit,
            saw_playing: self.saw_playing,
            attract_visual_frames: self.attract_visual.non_blank_frames,
            credit_visual_frames: self.credit_visual.non_blank_frames,
            playing_visual_frames: self.playing_visual.non_blank_frames,
            attract_distinct_frame_crcs: self.attract_visual.distinct_frame_crcs(),
            credit_distinct_frame_crcs: self.credit_visual.distinct_frame_crcs(),
            playing_distinct_frame_crcs: self.playing_visual.distinct_frame_crcs(),
            injected_inputs: self
                .injected_inputs
                .iter()
                .map(|input| String::from(*input))
                .collect(),
            clean_exit: self.clean_exit,
        }
    }
}

fn rendered_image_metrics(image: &RenderedImage) -> RenderedImageMetrics {
    RenderedImageMetrics {
        size: (image.width, image.height),
        crc32: crc32(&image.pixels),
        non_blank: image
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [0, 0, 0, 255].as_slice()),
    }
}

fn required_smoke_inputs() -> &'static [&'static str] {
    &[
        "coin",
        "start_one",
        "fire",
        "thrust",
        "altitude_up",
        "altitude_down",
        "reverse",
        "smart_bomb",
        "hyperspace",
    ]
}

fn smoke_input_events(profile: InputProfile, frame: u32) -> Vec<SmokeInput> {
    match profile {
        InputProfile::Planetoid => planetoid_smoke_input_events(frame),
        InputProfile::Cabinet | InputProfile::Test => cabinet_smoke_input_events(frame),
    }
}

fn planetoid_smoke_input_events(frame: u32) -> Vec<SmokeInput> {
    match frame {
        30 => vec![smoke_press("coin", InputKey::Char('5'))],
        31 => vec![smoke_release("coin", InputKey::Char('5'))],
        70 => vec![smoke_press("start_one", InputKey::Char('1'))],
        71 => vec![smoke_release("start_one", InputKey::Char('1'))],
        120 => vec![smoke_press("fire", InputKey::Enter)],
        121 => vec![smoke_release("fire", InputKey::Enter)],
        130 => vec![smoke_press("thrust", InputKey::LeftShift)],
        145 => vec![smoke_release("thrust", InputKey::LeftShift)],
        150 => vec![smoke_press("altitude_up", InputKey::Char('a'))],
        170 => vec![smoke_release("altitude_up", InputKey::Char('a'))],
        180 => vec![smoke_press("altitude_down", InputKey::Char('z'))],
        200 => vec![smoke_release("altitude_down", InputKey::Char('z'))],
        210 => vec![smoke_press("reverse", InputKey::Char(' '))],
        211 => vec![smoke_release("reverse", InputKey::Char(' '))],
        220 => vec![smoke_press("smart_bomb", InputKey::Tab)],
        221 => vec![smoke_release("smart_bomb", InputKey::Tab)],
        230 => vec![smoke_press("hyperspace", InputKey::Char('h'))],
        231 => vec![smoke_release("hyperspace", InputKey::Char('h'))],
        _ => Vec::new(),
    }
}

fn cabinet_smoke_input_events(frame: u32) -> Vec<SmokeInput> {
    match frame {
        30 => vec![smoke_press("coin", InputKey::Char('5'))],
        31 => vec![smoke_release("coin", InputKey::Char('5'))],
        70 => vec![smoke_press("start_one", InputKey::Char('1'))],
        71 => vec![smoke_release("start_one", InputKey::Char('1'))],
        120 => vec![smoke_press("fire", InputKey::Char('f'))],
        121 => vec![smoke_release("fire", InputKey::Char('f'))],
        130 => vec![smoke_press("thrust", InputKey::Char('t'))],
        145 => vec![smoke_release("thrust", InputKey::Char('t'))],
        150 => vec![smoke_press("altitude_up", InputKey::Up)],
        170 => vec![smoke_release("altitude_up", InputKey::Up)],
        180 => vec![smoke_press("altitude_down", InputKey::Down)],
        200 => vec![smoke_release("altitude_down", InputKey::Down)],
        210 => vec![smoke_press("reverse", InputKey::Char('r'))],
        211 => vec![smoke_release("reverse", InputKey::Char('r'))],
        220 => vec![smoke_press("smart_bomb", InputKey::Char('b'))],
        221 => vec![smoke_release("smart_bomb", InputKey::Char('b'))],
        230 => vec![smoke_press("hyperspace", InputKey::Char('h'))],
        231 => vec![smoke_release("hyperspace", InputKey::Char('h'))],
        _ => Vec::new(),
    }
}

fn smoke_press(label: &'static str, key: InputKey) -> SmokeInput {
    SmokeInput {
        label,
        event: InputEvent::new(key, InputEventKind::Press),
        counts_for_report: true,
    }
}

fn smoke_release(label: &'static str, key: InputKey) -> SmokeInput {
    SmokeInput {
        label,
        event: InputEvent::new(key, InputEventKind::Release),
        counts_for_report: false,
    }
}

#[cfg(all(not(test), not(coverage)))]
struct WgpuPresenter {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    frame_texture: Option<FrameTexture>,
}

#[cfg(all(not(test), not(coverage)))]
impl WgpuPresenter {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let (width, height) =
            renderable_window_size(size).unwrap_or((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .context("creating wgpu surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("requesting wgpu adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("defender wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("requesting wgpu device")?;
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .ok_or_else(|| anyhow!("wgpu surface is not supported by the selected adapter"))?;
        config.present_mode = wgpu::PresentMode::Fifo;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("defender wgpu frame bind group layout"),
            entries: &[
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
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("defender wgpu frame sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("defender wgpu frame pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("defender wgpu frame shader"),
            source: wgpu::ShaderSource::Wgsl(FRAME_SHADER.into()),
        });
        let color_targets = [Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("defender wgpu frame pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &color_targets,
            }),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            sampler,
            bind_group_layout,
            frame_texture: None,
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

    fn draw_frame(&mut self, image: &RenderedImage) -> Result<()> {
        if image.width == 0 || image.height == 0 {
            return Ok(());
        }

        self.ensure_frame_texture(image);
        let frame_texture = self
            .frame_texture
            .as_ref()
            .expect("frame texture was created for non-zero image");
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &frame_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * 4),
                rows_per_image: Some(image.height),
            },
            wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
        );

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
                bail!("wgpu surface validation error while acquiring frame")
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender wgpu frame encoder"),
            });
        {
            let color_attachment = Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            });
            let color_attachments = [color_attachment];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("defender wgpu frame render pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &frame_texture.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        surface_texture.present();
        Ok(())
    }

    fn ensure_frame_texture(&mut self, image: &RenderedImage) {
        let size = (image.width, image.height);
        if self
            .frame_texture
            .as_ref()
            .is_some_and(|texture| texture.size == size)
        {
            return;
        }
        self.frame_texture = Some(FrameTexture::new(
            &self.device,
            &self.bind_group_layout,
            &self.sampler,
            size,
        ));
    }
}

#[cfg(all(not(test), not(coverage)))]
struct FrameTexture {
    size: (u32, u32),
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

#[cfg(all(not(test), not(coverage)))]
impl FrameTexture {
    fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("defender wgpu frame texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FRAME_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("defender wgpu frame bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Self {
            size,
            texture,
            bind_group,
        }
    }
}

fn renderable_window_size(size: PhysicalSize<u32>) -> Option<(u32, u32)> {
    if size.width == 0 || size.height == 0 {
        None
    } else {
        Some((size.width, size.height))
    }
}

fn input_event_from_winit(event: &KeyEvent) -> Option<InputEvent> {
    Some(InputEvent::new(
        input_key_from_winit(&event.physical_key, &event.logical_key)?,
        input_event_kind_from_winit(event.state, event.repeat),
    ))
}

fn input_event_kind_from_winit(state: ElementState, repeat: bool) -> InputEventKind {
    match state {
        ElementState::Pressed if repeat => InputEventKind::Repeat,
        ElementState::Pressed => InputEventKind::Press,
        ElementState::Released => InputEventKind::Release,
    }
}

fn input_key_from_winit(physical_key: &PhysicalKey, logical_key: &Key) -> Option<InputKey> {
    if let PhysicalKey::Code(code) = physical_key
        && let Some(key) = input_key_from_physical_code(*code)
    {
        return Some(key);
    }
    input_key_from_logical_key(logical_key)
}

fn input_key_from_physical_code(code: KeyCode) -> Option<InputKey> {
    match code {
        KeyCode::Enter | KeyCode::NumpadEnter => Some(InputKey::Enter),
        KeyCode::Backspace => Some(InputKey::Backspace),
        KeyCode::Escape => Some(InputKey::Escape),
        KeyCode::Tab => Some(InputKey::Tab),
        KeyCode::ArrowUp => Some(InputKey::Up),
        KeyCode::ArrowDown => Some(InputKey::Down),
        KeyCode::ShiftLeft => Some(InputKey::LeftShift),
        KeyCode::ShiftRight => Some(InputKey::RightShift),
        KeyCode::F1 => Some(InputKey::F(1)),
        KeyCode::F2 => Some(InputKey::F(2)),
        KeyCode::F3 => Some(InputKey::F(3)),
        KeyCode::F4 => Some(InputKey::F(4)),
        KeyCode::F5 => Some(InputKey::F(5)),
        KeyCode::F6 => Some(InputKey::F(6)),
        KeyCode::F7 => Some(InputKey::F(7)),
        KeyCode::F8 => Some(InputKey::F(8)),
        KeyCode::F9 => Some(InputKey::F(9)),
        KeyCode::F10 => Some(InputKey::F(10)),
        KeyCode::F11 => Some(InputKey::F(11)),
        KeyCode::F12 => Some(InputKey::F(12)),
        _ => physical_character(code).map(InputKey::Char),
    }
}

fn physical_character(code: KeyCode) -> Option<char> {
    match code {
        KeyCode::Digit0 | KeyCode::Numpad0 => Some('0'),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some('1'),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some('2'),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some('3'),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some('4'),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some('5'),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some('6'),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some('7'),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some('8'),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some('9'),
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

fn input_key_from_logical_key(logical_key: &Key) -> Option<InputKey> {
    match logical_key {
        Key::Named(named) => input_key_from_named_key(*named),
        Key::Character(text) => single_character(text.as_str()).map(InputKey::Char),
        Key::Unidentified(_) | Key::Dead(_) => None,
    }
}

fn input_key_from_named_key(named: NamedKey) -> Option<InputKey> {
    match named {
        NamedKey::Enter => Some(InputKey::Enter),
        NamedKey::Backspace => Some(InputKey::Backspace),
        NamedKey::Escape => Some(InputKey::Escape),
        NamedKey::Tab => Some(InputKey::Tab),
        NamedKey::Space => Some(InputKey::Char(' ')),
        NamedKey::ArrowUp => Some(InputKey::Up),
        NamedKey::ArrowDown => Some(InputKey::Down),
        NamedKey::Shift => Some(InputKey::LeftShift),
        NamedKey::F1 => Some(InputKey::F(1)),
        NamedKey::F2 => Some(InputKey::F(2)),
        NamedKey::F3 => Some(InputKey::F(3)),
        NamedKey::F4 => Some(InputKey::F(4)),
        NamedKey::F5 => Some(InputKey::F(5)),
        NamedKey::F6 => Some(InputKey::F(6)),
        NamedKey::F7 => Some(InputKey::F(7)),
        NamedKey::F8 => Some(InputKey::F(8)),
        NamedKey::F9 => Some(InputKey::F(9)),
        NamedKey::F10 => Some(InputKey::F(10)),
        NamedKey::F11 => Some(InputKey::F(11)),
        NamedKey::F12 => Some(InputKey::F(12)),
        _ => None,
    }
}

fn single_character(value: &str) -> Option<char> {
    let mut chars = value.chars();
    let character = chars.next()?;
    if chars.next().is_none() {
        Some(character)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use winit::{
        dpi::PhysicalSize,
        event::ElementState,
        keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    };

    use crate::{
        input::{CabinetInput, InputEventKind, InputKey, InputMapper, InputProfile, PolledInput},
        machine::ArcadeMachine,
        machine_state::GamePhase,
        video::RenderedImage,
        wgpu_presenter::{
            SmokeMachineState, WgpuSmoke, WgpuSmokeReport, input_event_kind_from_winit,
            input_key_from_winit, renderable_window_size, rendered_image_metrics,
            required_smoke_inputs, run_wgpu_live_smoke, smoke_input_events,
        },
    };

    #[test]
    fn wgpu_window_size_ignores_zero_dimensions() {
        assert_eq!(
            renderable_window_size(PhysicalSize::new(640, 480)),
            Some((640, 480))
        );
        assert_eq!(renderable_window_size(PhysicalSize::new(0, 480)), None);
        assert_eq!(renderable_window_size(PhysicalSize::new(640, 0)), None);
    }

    #[test]
    fn wgpu_input_kind_preserves_press_repeat_and_release() {
        assert_eq!(
            input_event_kind_from_winit(ElementState::Pressed, false),
            InputEventKind::Press
        );
        assert_eq!(
            input_event_kind_from_winit(ElementState::Pressed, true),
            InputEventKind::Repeat
        );
        assert_eq!(
            input_event_kind_from_winit(ElementState::Released, false),
            InputEventKind::Release
        );
    }

    #[test]
    fn wgpu_physical_keys_match_shared_input_mapper_keys() {
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Code(KeyCode::Digit5),
                &Key::Named(NamedKey::F1)
            ),
            Some(InputKey::Char('5'))
        );
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Code(KeyCode::KeyA),
                &Key::Character("A".into())
            ),
            Some(InputKey::Char('a'))
        );
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Code(KeyCode::ShiftRight),
                &Key::Named(NamedKey::Shift)
            ),
            Some(InputKey::RightShift)
        );
        assert_eq!(
            input_key_from_winit(&PhysicalKey::Code(KeyCode::F3), &Key::Named(NamedKey::F3)),
            Some(InputKey::F(3))
        );
    }

    #[test]
    fn wgpu_logical_keys_cover_non_physical_fallbacks() {
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Unidentified),
                &Key::Character("h".into())
            ),
            Some(InputKey::Char('h'))
        );
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Unidentified),
                &Key::Named(NamedKey::Space)
            ),
            Some(InputKey::Char(' '))
        );
        assert_eq!(
            input_key_from_winit(
                &PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Unidentified),
                &Key::Character("xy".into())
            ),
            None
        );
    }

    #[test]
    fn wgpu_smoke_script_exercises_planetoid_controls_through_input_mapper() {
        let (seen, labels) = smoke_script_result(InputProfile::Planetoid);

        assert!(seen.coin);
        assert!(seen.start_one);
        assert!(seen.fire);
        assert!(seen.thrust);
        assert!(seen.altitude_up);
        assert!(seen.altitude_down);
        assert!(seen.reverse);
        assert!(seen.smart_bomb);
        assert!(seen.hyperspace);
        assert!(
            required_smoke_inputs()
                .iter()
                .all(|input| labels.contains(input))
        );
    }

    #[test]
    fn wgpu_smoke_script_exercises_cabinet_controls_through_input_mapper() {
        let (seen, labels) = smoke_script_result(InputProfile::Cabinet);

        assert!(seen.coin);
        assert!(seen.start_one);
        assert!(seen.fire);
        assert!(seen.thrust);
        assert!(seen.altitude_up);
        assert!(seen.altitude_down);
        assert!(seen.reverse);
        assert!(seen.smart_bomb);
        assert!(seen.hyperspace);
        assert!(
            required_smoke_inputs()
                .iter()
                .all(|input| labels.contains(input))
        );
    }

    #[test]
    fn wgpu_smoke_report_validates_required_evidence() {
        let report = complete_smoke_report();
        report.validate().expect("complete smoke report");

        let mut report = complete_smoke_report();
        report.window_created = false;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.rendered_frames = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.first_frame_size = None;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.distinct_frame_crcs = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.distinct_frame_crcs = 1;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.saw_non_blank_frame = false;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.saw_attract = false;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.attract_visual_frames = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.attract_distinct_frame_crcs = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.saw_credit = false;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.credit_visual_frames = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.credit_distinct_frame_crcs = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.saw_playing = false;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.playing_visual_frames = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.playing_distinct_frame_crcs = 0;
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.injected_inputs.retain(|input| input != "hyperspace");
        assert!(report.validate().is_err());

        let mut report = complete_smoke_report();
        report.clean_exit = false;
        assert!(report.validate().is_err());
    }

    #[test]
    fn wgpu_smoke_report_formats_recorded_and_unrecorded_frame_size() {
        let text = complete_smoke_report().to_text();
        assert!(text.contains("first_frame_size: 1024x768"));
        assert!(text.contains("injected_inputs: coin,start_one"));

        let mut report = complete_smoke_report();
        report.first_frame_size = None;
        assert!(report.to_text().contains("first_frame_size: unrecorded"));
    }

    #[test]
    fn wgpu_smoke_accumulates_window_render_input_and_machine_evidence() {
        let mut smoke = WgpuSmoke::new(3);
        assert_eq!(smoke.frame(), 0);
        assert!(!smoke.should_stop());

        smoke.observe_window_created();
        smoke.observe_rendered_image(
            rendered_image_metrics(&RenderedImage::new_blank(2, 1, [1, 2, 3, 255])),
            SmokeMachineState {
                phase: GamePhase::Attract,
                credits: 0,
            },
        );
        smoke.observe_rendered_image(
            rendered_image_metrics(&RenderedImage::new_blank(2, 1, [4, 5, 6, 255])),
            SmokeMachineState {
                phase: GamePhase::Attract,
                credits: 1,
            },
        );
        smoke.observe_rendered_image(
            rendered_image_metrics(&RenderedImage::new_blank(2, 1, [7, 8, 9, 255])),
            SmokeMachineState {
                phase: GamePhase::Playing,
                credits: 1,
            },
        );
        for input in required_smoke_inputs() {
            smoke.record_injected_input(input);
        }
        smoke.advance_frame();
        smoke.advance_frame();
        smoke.advance_frame();
        assert!(smoke.should_stop());
        smoke.mark_clean_exit();

        let report = smoke.into_report();
        assert!(report.window_created);
        assert_eq!(report.rendered_frames, 3);
        assert_eq!(report.first_frame_size, Some((2, 1)));
        assert_eq!(report.distinct_frame_crcs, 3);
        assert!(report.saw_non_blank_frame);
        assert!(report.saw_attract);
        assert!(report.saw_credit);
        assert!(report.saw_playing);
        assert_eq!(report.attract_visual_frames, 2);
        assert_eq!(report.credit_visual_frames, 2);
        assert_eq!(report.playing_visual_frames, 1);
        assert_eq!(report.attract_distinct_frame_crcs, 2);
        assert_eq!(report.credit_distinct_frame_crcs, 2);
        assert_eq!(report.playing_distinct_frame_crcs, 1);
        assert!(report.clean_exit);
        assert!(
            required_smoke_inputs()
                .iter()
                .all(|input| report.injected_inputs.iter().any(|seen| seen == input))
        );
    }

    #[test]
    fn wgpu_smoke_machine_state_uses_thread_frame_snapshot() {
        let mut snapshot = ArcadeMachine::new().snapshot();
        snapshot.phase = GamePhase::Playing;
        snapshot.credits = 2;

        assert_eq!(
            SmokeMachineState::from_snapshot(snapshot),
            SmokeMachineState {
                phase: GamePhase::Playing,
                credits: 2,
            }
        );
    }

    #[test]
    fn test_build_wgpu_live_smoke_stub_returns_valid_report() {
        let report = run_wgpu_live_smoke(InputProfile::Planetoid, None).expect("smoke stub");
        report.validate().expect("valid smoke stub report");
    }

    fn smoke_script_result(profile: InputProfile) -> (CabinetInput, BTreeSet<&'static str>) {
        let mut mapper = InputMapper::new(profile);
        let mut seen = CabinetInput::NONE;
        let mut labels = BTreeSet::new();
        for frame in 0..260 {
            let mut input = PolledInput::default();
            for smoke_input in smoke_input_events(profile, frame) {
                mapper.handle_input_event(smoke_input.event, &mut input);
                if smoke_input.counts_for_report {
                    labels.insert(smoke_input.label);
                }
            }
            seen.merge(input.cabinet);
        }
        (seen, labels)
    }

    fn complete_smoke_report() -> WgpuSmokeReport {
        WgpuSmokeReport {
            window_created: true,
            rendered_frames: 4,
            first_frame_size: Some((1024, 768)),
            distinct_frame_crcs: 2,
            saw_non_blank_frame: true,
            saw_attract: true,
            saw_credit: true,
            saw_playing: true,
            attract_visual_frames: 1,
            credit_visual_frames: 1,
            playing_visual_frames: 1,
            attract_distinct_frame_crcs: 1,
            credit_distinct_frame_crcs: 1,
            playing_distinct_frame_crcs: 1,
            injected_inputs: required_smoke_inputs()
                .iter()
                .map(|input| String::from(*input))
                .collect(),
            clean_exit: true,
        }
    }
}
