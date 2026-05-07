//! Windowed live runner using wgpu.
#![cfg_attr(any(test, coverage), allow(dead_code))]

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

use crate::{
    cmos_storage::{CmosStorage, FileCmosStorage},
    input::{
        CabinetInput, InputEvent, InputEventKind, InputKey, InputMapper, InputProfile, PolledInput,
        XyzzyOverlay,
    },
    live::{
        LiveCoreClock, live_frame_inputs, render_live_machine_frame, save_live_cmos,
        step_live_core_frames,
    },
    machine::{ArcadeMachine, CompatibilityState},
    video::{RenderedImage, Renderer},
};

const INITIAL_WINDOW_WIDTH: u32 = 1_024;
const INITIAL_WINDOW_HEIGHT: u32 = 768;
const FRAME_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

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
pub fn run_wgpu_live(input_profile: InputProfile, cmos_path: Option<&Path>) -> Result<()> {
    let cmos_storage = cmos_path.map(FileCmosStorage::new);
    let storage = cmos_storage
        .as_ref()
        .map(|storage| storage as &dyn CmosStorage);
    let machine = crate::live::live_machine_from_cmos_storage(storage)?;
    let event_loop = winit::event_loop::EventLoop::new().context("creating wgpu event loop")?;
    let mut app = WgpuLiveApp::new(input_profile, cmos_storage, machine);

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

#[cfg(any(test, coverage))]
pub fn run_wgpu_live(_input_profile: InputProfile, _cmos_path: Option<&Path>) -> Result<()> {
    Ok(())
}

struct WgpuLiveApp {
    input_mapper: InputMapper,
    xyzzy: XyzzyOverlay,
    cmos_storage: Option<FileCmosStorage>,
    machine: ArcadeMachine,
    renderer: Renderer,
    core_clock: LiveCoreClock,
    pending_cabinet_input: CabinetInput,
    pending_typed_chars: Vec<char>,
    polled_input: PolledInput,
    window: Option<Arc<Window>>,
    presenter: Option<WgpuPresenter>,
    error: Option<anyhow::Error>,
}

impl WgpuLiveApp {
    fn new(
        input_profile: InputProfile,
        cmos_storage: Option<FileCmosStorage>,
        machine: ArcadeMachine,
    ) -> Self {
        Self {
            input_mapper: InputMapper::new(input_profile),
            xyzzy: XyzzyOverlay::default(),
            cmos_storage,
            machine,
            renderer: Renderer::with_size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT),
            core_clock: LiveCoreClock::new(Instant::now()),
            pending_cabinet_input: CabinetInput::NONE,
            pending_typed_chars: Vec::new(),
            polled_input: PolledInput::default(),
            window: None,
            presenter: None,
            error: None,
        }
    }

    fn save_cmos(&self) -> Result<()> {
        save_live_cmos(
            self.cmos_storage
                .as_ref()
                .map(|storage| storage as &dyn CmosStorage),
            &self.machine,
        )
    }

    fn take_error(&mut self) -> Option<anyhow::Error> {
        self.error.take()
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
                            f64::from(INITIAL_WINDOW_WIDTH),
                            f64::from(INITIAL_WINDOW_HEIGHT),
                        )),
                )
                .context("creating wgpu window")?,
        );
        let size = renderable_window_size(window.inner_size())
            .unwrap_or((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT));
        self.renderer = Renderer::with_size(size.0, size.1);
        self.presenter = Some(
            pollster::block_on(WgpuPresenter::new(window.clone()))
                .context("initializing wgpu presenter")?,
        );
        self.window = Some(window);
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

    fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        if let Some(input_event) = input_event_from_winit(&key_event) {
            self.input_mapper
                .handle_input_event(input_event, &mut self.polled_input);
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        let Some((width, height)) = renderable_window_size(size) else {
            return;
        };
        self.renderer = Renderer::with_size(width, height);
        if let Some(presenter) = &mut self.presenter {
            presenter.resize(width, height);
        }
    }

    fn advance_core(&mut self) {
        if self.polled_input.quit_requested {
            return;
        }

        self.xyzzy
            .handle_typed_chars(&self.polled_input.typed_chars);
        self.pending_typed_chars
            .extend(self.polled_input.typed_chars.iter().copied());
        self.machine.set_compatibility(CompatibilityState {
            xyzzy_active: self.xyzzy.active(),
            xyzzy_invincible: self.xyzzy.invincible(),
            xyzzy_auto_fire: self.xyzzy.auto_fire(),
        });

        let core_steps = self.core_clock.steps_due(Instant::now());
        let held_input = self.input_mapper.held_cabinet_input();
        if let Some(frame_inputs) = live_frame_inputs(
            &mut self.pending_cabinet_input,
            self.polled_input.cabinet,
            held_input,
            core_steps,
        ) {
            step_live_core_frames(
                &mut self.machine,
                frame_inputs.first,
                frame_inputs.catch_up,
                &self.pending_typed_chars,
                core_steps,
            );
            self.pending_typed_chars.clear();
        }

        self.polled_input = PolledInput::default();
    }

    fn draw_frame(&mut self) -> Result<()> {
        let image = render_live_machine_frame(&mut self.renderer, &mut self.machine)
            .context("rendering live machine frame")?;
        let Some(presenter) = &mut self.presenter else {
            return Ok(());
        };
        presenter
            .draw_frame(image)
            .context("drawing wgpu graphics frame")
    }
}

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
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
                if self.polled_input.quit_requested {
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
        if self.error.is_some() {
            event_loop.exit();
            return;
        }
        if self.polled_input.quit_requested {
            event_loop.exit();
            return;
        }

        self.advance_core();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        let sleep = self.core_clock.sleep_until_next_step(Instant::now());
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + sleep));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.presenter = None;
        self.window = None;
    }
}

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

struct FrameTexture {
    size: (u32, u32),
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

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
    use winit::{
        dpi::PhysicalSize,
        event::ElementState,
        keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    };

    use crate::{
        input::{InputEventKind, InputKey},
        wgpu_presenter::{
            input_event_kind_from_winit, input_key_from_winit, renderable_window_size,
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
}
