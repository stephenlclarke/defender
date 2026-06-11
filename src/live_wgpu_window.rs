#[cfg(all(not(test), not(coverage)))]
fn run_actor_live_app(
    input_profile: LiveInputProfile,
    audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
    actor_script_path: Option<&Path>,
) -> anyhow::Result<()> {
    let event_loop =
        winit::event_loop::EventLoop::new().context("creating actor wgpu event loop")?;
    let runtime = actor_live_runtime_from_script_path(actor_script_path)?;
    let mut app = ActorLiveApp::new(
        input_profile,
        LiveAudioRuntime::for_mode(audio_mode),
        runtime,
    );

    event_loop
        .run_app(&mut app)
        .context("running actor wgpu live event loop")?;
    if let Some(error) = app.take_error() {
        return Err(error);
    }
    Ok(())
}

#[cfg(all(not(test), not(coverage)))]
struct ActorLiveApp {
    input_profile: LiveInputProfile,
    runtime: ActorRuntimeAdapter,
    audio: LiveAudioRuntime,
    input: LiveInputState,
    accumulator: FixedStepAccumulator,
    step_duration: Duration,
    last_tick: Instant,
    next_wake_at: Instant,
    latest_step_snapshot: Option<GameStepSnapshot>,
    quit_requested: bool,
    window: Option<Arc<Window>>,
    presenter: Option<WgpuScenePresenter>,
    error: Option<anyhow::Error>,
}

#[cfg(all(not(test), not(coverage)))]
impl ActorLiveApp {
    fn new(
        input_profile: LiveInputProfile,
        audio: LiveAudioRuntime,
        runtime: ActorRuntimeAdapter,
    ) -> Self {
        let now = Instant::now();
        let step_duration = Duration::from_micros(StepRate::CABINET.step_duration_micros());
        let mut app = Self {
            input_profile,
            runtime,
            audio,
            input: LiveInputState::default(),
            accumulator: FixedStepAccumulator::new(StepRate::CABINET),
            step_duration,
            last_tick: now,
            next_wake_at: now + step_duration,
            latest_step_snapshot: None,
            quit_requested: false,
            window: None,
            presenter: None,
            error: None,
        };
        app.step_once();
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
                        .with_title("Defender Actor Runtime")
                        .with_inner_size(LogicalSize::new(
                            f64::from(INITIAL_WINDOW_WIDTH),
                            f64::from(INITIAL_WINDOW_HEIGHT),
                        )),
                )
                .context("creating actor wgpu window")?,
        );
        let presenter = pollster::block_on(WgpuScenePresenter::new(window.clone()))
            .context("initializing actor wgpu presenter")?;
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.last_tick = Instant::now();
        self.next_wake_at = self.last_tick + self.step_duration;
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
        let control = live_control_from_winit(self.input_profile, event);
        self.input.observe_key_event_for_xyzzy(event, control);
        let Some(control) = control else {
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

    fn step_once(&mut self) {
        let input = self.input.drain_game_input();
        let xyzzy = self.input.drain_xyzzy_mode();
        let actor_step = self.runtime.step_clean_input(input, xyzzy);
        let snapshot = actor_step.game_step_snapshot();
        self.audio.submit_game_step(&snapshot);
        self.latest_step_snapshot = Some(snapshot);
    }

    fn step_due_updates(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        self.accumulator
            .add_elapsed_micros(elapsed.as_micros().try_into().unwrap_or(u64::MAX));
        let due_steps = self.accumulator.consume_due_steps(MAX_STEPS_PER_TICK);

        for _ in 0..due_steps {
            self.step_once();
        }

        let micros_until_next = StepRate::CABINET
            .step_duration_micros()
            .saturating_sub(self.accumulator.accumulated_micros())
            .max(1);
        self.next_wake_at = Instant::now() + Duration::from_micros(micros_until_next);
        due_steps > 0
    }

    fn draw_scene(&mut self) -> anyhow::Result<()> {
        let Some(snapshot) = &self.latest_step_snapshot else {
            return Ok(());
        };
        let Some(presenter) = &mut self.presenter else {
            return Ok(());
        };
        presenter.draw_scene(&snapshot.scene)
    }
}

#[cfg(all(not(test), not(coverage)))]
impl ApplicationHandler for ActorLiveApp {
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
                if let Err(error) = self.draw_scene() {
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

        if self.step_due_updates()
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
                anyhow::bail!("wgpu surface validation error while acquiring swapchain texture")
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("defender clean wgpu render encoder"),
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
