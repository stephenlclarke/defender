//! Live terminal runner for the new core.
#![cfg_attr(coverage, allow(dead_code, unused_imports))]

use std::any::Any;
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};

use crate::{
    audio::{LiveAudioMode, LiveAudioRuntime},
    board::CmosRam,
    cmos_storage::CmosStorage,
    input::{CabinetInput, InputEvent, InputMapper, InputProfile, PolledInput, XyzzyOverlay},
    machine::{ArcadeMachine, FRAME_RATE_MILLIHZ},
    machine_state::{CompatibilityState, MachineSnapshot},
    renderer::{RenderScene, SurfaceSize},
    rom::crc32,
    video::{RenderedImage, Renderer},
};

pub(crate) const FRAME_DURATION: Duration =
    Duration::from_micros(cabinet_frame_duration_micros(FRAME_RATE_MILLIHZ));

pub(crate) const fn cabinet_frame_duration_micros(frame_rate_millihz: u32) -> u64 {
    let rate = frame_rate_millihz as u64;
    (1_000_000_000 + (rate / 2)) / rate
}

pub(crate) struct LiveCoreClock {
    next_step: Instant,
}

impl LiveCoreClock {
    pub(crate) fn new(now: Instant) -> Self {
        Self { next_step: now }
    }

    pub(crate) fn steps_due(&mut self, now: Instant) -> u32 {
        let mut steps = 0;
        while now >= self.next_step {
            steps += 1;
            self.next_step += FRAME_DURATION;
        }
        steps
    }

    #[cfg(test)]
    pub(crate) fn sleep_until_next_step(&self, now: Instant) -> Duration {
        self.next_step.saturating_duration_since(now)
    }

    pub(crate) fn next_step_at(&self) -> Instant {
        self.next_step
    }
}

pub(crate) struct LiveCoreDriver {
    input_mapper: InputMapper,
    xyzzy: XyzzyOverlay,
    machine: ArcadeMachine,
    core_clock: LiveCoreClock,
    audio: LiveAudioRuntime,
    pending_cabinet_input: CabinetInput,
    pending_typed_chars: Vec<char>,
}

#[cfg_attr(test, allow(dead_code))]
impl LiveCoreDriver {
    #[cfg(test)]
    pub(crate) fn new(input_profile: InputProfile, machine: ArcadeMachine, now: Instant) -> Self {
        Self::new_with_audio(input_profile, machine, now, LiveAudioRuntime::disabled())
    }

    pub(crate) fn new_with_audio(
        input_profile: InputProfile,
        machine: ArcadeMachine,
        now: Instant,
        audio: LiveAudioRuntime,
    ) -> Self {
        Self {
            input_mapper: InputMapper::new(input_profile),
            xyzzy: XyzzyOverlay::default(),
            machine,
            core_clock: LiveCoreClock::new(now),
            audio,
            pending_cabinet_input: CabinetInput::NONE,
            pending_typed_chars: Vec::new(),
        }
    }

    pub(crate) fn machine(&self) -> &ArcadeMachine {
        &self.machine
    }

    pub(crate) fn machine_mut(&mut self) -> &mut ArcadeMachine {
        &mut self.machine
    }

    pub(crate) fn handle_input_event(&mut self, input_event: InputEvent, input: &mut PolledInput) {
        self.input_mapper.handle_input_event(input_event, input);
    }

    pub(crate) fn reset_clock(&mut self, now: Instant) {
        self.core_clock = LiveCoreClock::new(now);
        self.audio.flush();
    }

    pub(crate) fn next_step_at(&self) -> Instant {
        self.core_clock.next_step_at()
    }

    pub(crate) fn advance_realtime(&mut self, input: &PolledInput, now: Instant) -> u32 {
        let core_steps = self.core_clock.steps_due(now);
        self.advance_fixed_frames(input, core_steps);
        core_steps
    }

    pub(crate) fn advance_fixed_frames(&mut self, input: &PolledInput, frames: u32) {
        if input.quit_requested {
            return;
        }

        self.xyzzy.handle_typed_chars(&input.typed_chars);
        self.pending_typed_chars
            .extend(input.typed_chars.iter().copied());
        self.machine.set_compatibility(CompatibilityState {
            xyzzy_active: self.xyzzy.active(),
            xyzzy_invincible: self.xyzzy.invincible(),
            xyzzy_auto_fire: self.xyzzy.auto_fire(),
        });

        let held_input = self.input_mapper.held_cabinet_input();
        if let Some(frame_inputs) = live_frame_inputs(
            &mut self.pending_cabinet_input,
            input.cabinet,
            held_input,
            frames,
        ) {
            step_live_core_frames(
                &mut self.machine,
                frame_inputs.first,
                frame_inputs.catch_up,
                &self.pending_typed_chars,
                frames,
                &self.audio,
            );
            self.pending_typed_chars.clear();
        }
    }

    #[cfg(test)]
    pub(crate) fn shutdown_audio(&mut self) {
        self.audio.shutdown();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveAdvanceMode {
    Realtime,
    FixedFrames(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveCoreFrame {
    pub(crate) image: RenderedImage,
    pub(crate) scene: RenderScene,
    pub(crate) snapshot: MachineSnapshot,
    pub(crate) next_step_at: Instant,
}

#[derive(Clone, Default)]
struct LiveCoreFrameMailbox {
    latest: Arc<Mutex<Option<std::result::Result<LiveCoreFrame, LiveCoreError>>>>,
}

impl LiveCoreFrameMailbox {
    fn post(
        &self,
        frame: std::result::Result<LiveCoreFrame, LiveCoreError>,
    ) -> std::result::Result<(), LiveCoreError> {
        *self
            .latest
            .lock()
            .map_err(|_| LiveCoreError::mailbox_unavailable(LiveCoreCommandName::RequestFrame))? =
            Some(frame);
        Ok(())
    }

    fn take_latest(&self) -> Result<Option<LiveCoreFrame>> {
        let Some(frame) = self
            .latest
            .lock()
            .map_err(|_| LiveCoreError::mailbox_unavailable(LiveCoreCommandName::RequestFrame))?
            .take()
        else {
            return Ok(None);
        };

        frame.map(Some).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiveCoreCommandName {
    InputEvent,
    ResetClock,
    ResizeRenderer,
    #[cfg(test)]
    Advance,
    RequestFrame,
    #[cfg(test)]
    CmosRam,
    Shutdown,
    #[cfg(test)]
    FailNextRenderForTest,
    #[cfg(test)]
    PanicForTest,
}

impl std::fmt::Display for LiveCoreCommandName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::InputEvent => "input_event",
            Self::ResetClock => "reset_clock",
            Self::ResizeRenderer => "resize_renderer",
            #[cfg(test)]
            Self::Advance => "advance",
            Self::RequestFrame => "request_frame",
            #[cfg(test)]
            Self::CmosRam => "cmos_ram",
            Self::Shutdown => "shutdown",
            #[cfg(test)]
            Self::FailNextRenderForTest => "fail_next_render_for_test",
            #[cfg(test)]
            Self::PanicForTest => "panic_for_test",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LiveCoreWorkerError {
    Render(String),
}

impl LiveCoreWorkerError {
    fn render(error: impl Into<String>) -> Self {
        Self::Render(error.into())
    }
}

impl std::fmt::Display for LiveCoreWorkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Render(error) => write!(formatter, "render failed: {error}"),
        }
    }
}

impl std::error::Error for LiveCoreWorkerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LiveCoreErrorKind {
    CommandSend,
    WorkerTerminated,
    WorkerPanic(String),
    WorkerFailed(LiveCoreWorkerError),
    MailboxUnavailable,
    WorkerStateUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveCoreError {
    command: LiveCoreCommandName,
    kind: LiveCoreErrorKind,
}

impl LiveCoreError {
    fn command_send(command: LiveCoreCommandName) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::CommandSend,
        }
    }

    fn worker_terminated(command: LiveCoreCommandName) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::WorkerTerminated,
        }
    }

    fn worker_panic(command: LiveCoreCommandName, message: String) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::WorkerPanic(message),
        }
    }

    fn worker_failed(command: LiveCoreCommandName, error: LiveCoreWorkerError) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::WorkerFailed(error),
        }
    }

    fn mailbox_unavailable(command: LiveCoreCommandName) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::MailboxUnavailable,
        }
    }

    fn worker_state_unavailable(command: LiveCoreCommandName) -> Self {
        Self {
            command,
            kind: LiveCoreErrorKind::WorkerStateUnavailable,
        }
    }

    #[cfg(test)]
    fn command(&self) -> LiveCoreCommandName {
        self.command
    }

    #[cfg(test)]
    fn kind(&self) -> &LiveCoreErrorKind {
        &self.kind
    }
}

impl std::fmt::Display for LiveCoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "live core command {} failed: ", self.command)?;
        match &self.kind {
            LiveCoreErrorKind::CommandSend => formatter.write_str("command channel closed"),
            LiveCoreErrorKind::WorkerTerminated => {
                formatter.write_str("worker terminated before replying")
            }
            LiveCoreErrorKind::WorkerPanic(message) => {
                write!(formatter, "worker panicked: {message}")
            }
            LiveCoreErrorKind::WorkerFailed(error) => write!(formatter, "{error}"),
            LiveCoreErrorKind::MailboxUnavailable => {
                formatter.write_str("frame mailbox is unavailable")
            }
            LiveCoreErrorKind::WorkerStateUnavailable => {
                formatter.write_str("worker state is unavailable")
            }
        }
    }
}

impl std::error::Error for LiveCoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            LiveCoreErrorKind::WorkerFailed(error) => Some(error),
            _ => None,
        }
    }
}

pub(crate) trait LiveCoreRuntime {
    fn input_profile(&self) -> InputProfile;
    fn handle_input_event(&self, input_event: InputEvent) -> Result<bool>;
    fn reset_clock(&self, now: Instant) -> Result<()>;
    fn resize_renderer(&self, width: u32, height: u32) -> Result<()>;
    #[cfg(test)]
    fn advance(&self, mode: LiveAdvanceMode, now: Instant) -> Result<LiveCoreFrame>;
    fn request_frame(&self, mode: LiveAdvanceMode, now: Instant) -> Result<()>;
    fn take_latest_frame(&self) -> Result<Option<LiveCoreFrame>>;
    #[cfg(test)]
    fn cmos_ram(&self) -> Result<CmosRam>;
    fn shutdown_cmos_ram(&self) -> Result<CmosRam>;
}

pub(crate) struct LiveCoreThread {
    input_profile: InputProfile,
    commands: Sender<LiveCoreCommand>,
    frames: LiveCoreFrameMailbox,
    worker: Mutex<Option<JoinHandle<()>>>,
}

type LiveCoreResponse<T> = std::result::Result<T, LiveCoreWorkerError>;

enum LiveCoreCommand {
    InputEvent {
        input_event: InputEvent,
        response: Sender<bool>,
    },
    ResetClock {
        now: Instant,
    },
    ResizeRenderer {
        width: u32,
        height: u32,
    },
    #[cfg(test)]
    Advance {
        mode: LiveAdvanceMode,
        now: Instant,
        response: Sender<LiveCoreResponse<LiveCoreFrame>>,
    },
    RequestFrame {
        mode: LiveAdvanceMode,
        now: Instant,
    },
    #[cfg(test)]
    CmosRam {
        response: Sender<CmosRam>,
    },
    Shutdown {
        response: Option<Sender<CmosRam>>,
    },
    #[cfg(test)]
    FailNextRenderForTest {
        message: String,
    },
    #[cfg(test)]
    PanicForTest,
}

impl LiveCoreThread {
    #[cfg(test)]
    pub(crate) fn spawn(
        input_profile: InputProfile,
        machine: ArcadeMachine,
        now: Instant,
        renderer_size: (u32, u32),
    ) -> Self {
        Self::spawn_with_audio(
            input_profile,
            machine,
            now,
            renderer_size,
            LiveAudioRuntime::disabled(),
        )
    }

    pub(crate) fn spawn_with_audio(
        input_profile: InputProfile,
        machine: ArcadeMachine,
        now: Instant,
        renderer_size: (u32, u32),
        audio: LiveAudioRuntime,
    ) -> Self {
        let (commands, receiver) = mpsc::channel();
        let frames = LiveCoreFrameMailbox::default();
        let worker_frames = frames.clone();
        let worker = thread::spawn(move || {
            run_live_core_thread(
                input_profile,
                machine,
                now,
                renderer_size,
                audio,
                receiver,
                worker_frames,
            )
        });
        Self {
            input_profile,
            commands,
            frames,
            worker: Mutex::new(Some(worker)),
        }
    }

    fn request<T>(
        &self,
        command_name: LiveCoreCommandName,
        command: impl FnOnce(Sender<T>) -> LiveCoreCommand,
    ) -> Result<T> {
        let (response, receiver) = mpsc::channel();
        self.send_command(command_name, command(response))?;
        receiver
            .recv()
            .map_err(|_| self.worker_stopped_error(command_name).into())
    }

    #[cfg(test)]
    fn request_result<T>(
        &self,
        command_name: LiveCoreCommandName,
        command: impl FnOnce(Sender<LiveCoreResponse<T>>) -> LiveCoreCommand,
    ) -> Result<T> {
        self.request(command_name, command).and_then(|result| {
            result.map_err(|error| LiveCoreError::worker_failed(command_name, error).into())
        })
    }

    fn send_command(
        &self,
        command_name: LiveCoreCommandName,
        command: LiveCoreCommand,
    ) -> std::result::Result<(), LiveCoreError> {
        self.commands
            .send(command)
            .map_err(|_| self.command_send_error(command_name))
    }

    fn send(&self, command_name: LiveCoreCommandName, command: LiveCoreCommand) -> Result<()> {
        self.send_command(command_name, command).map_err(Into::into)
    }

    fn command_send_error(&self, command: LiveCoreCommandName) -> LiveCoreError {
        self.join_worker(command)
            .err()
            .unwrap_or_else(|| LiveCoreError::command_send(command))
    }

    fn worker_stopped_error(&self, command: LiveCoreCommandName) -> LiveCoreError {
        self.join_worker(command)
            .err()
            .unwrap_or_else(|| LiveCoreError::worker_terminated(command))
    }

    fn worker_is_finished(&self, command: LiveCoreCommandName) -> Result<bool> {
        let worker = self
            .worker
            .lock()
            .map_err(|_| LiveCoreError::worker_state_unavailable(command))?;
        Ok(match worker.as_ref() {
            Some(worker) => worker.is_finished(),
            None => true,
        })
    }

    fn join_worker(&self, command: LiveCoreCommandName) -> std::result::Result<(), LiveCoreError> {
        let Some(worker) = self
            .worker
            .lock()
            .map_err(|_| LiveCoreError::worker_state_unavailable(command))?
            .take()
        else {
            return Ok(());
        };

        worker
            .join()
            .map_err(|panic| LiveCoreError::worker_panic(command, panic_message(panic)))
    }

    #[cfg(test)]
    fn fail_next_render_for_test(&self, message: impl Into<String>) -> Result<()> {
        self.send(
            LiveCoreCommandName::FailNextRenderForTest,
            LiveCoreCommand::FailNextRenderForTest {
                message: message.into(),
            },
        )
    }

    #[cfg(test)]
    fn panic_worker_for_test(&self) -> Result<()> {
        self.send(
            LiveCoreCommandName::PanicForTest,
            LiveCoreCommand::PanicForTest,
        )
    }
}

impl LiveCoreRuntime for LiveCoreThread {
    fn input_profile(&self) -> InputProfile {
        self.input_profile
    }

    fn handle_input_event(&self, input_event: InputEvent) -> Result<bool> {
        self.request(LiveCoreCommandName::InputEvent, |response| {
            LiveCoreCommand::InputEvent {
                input_event,
                response,
            }
        })
    }

    fn reset_clock(&self, now: Instant) -> Result<()> {
        self.send(
            LiveCoreCommandName::ResetClock,
            LiveCoreCommand::ResetClock { now },
        )
    }

    fn resize_renderer(&self, width: u32, height: u32) -> Result<()> {
        self.send(
            LiveCoreCommandName::ResizeRenderer,
            LiveCoreCommand::ResizeRenderer { width, height },
        )
    }

    #[cfg(test)]
    fn advance(&self, mode: LiveAdvanceMode, now: Instant) -> Result<LiveCoreFrame> {
        self.request_result(LiveCoreCommandName::Advance, |response| {
            LiveCoreCommand::Advance {
                mode,
                now,
                response,
            }
        })
    }

    fn request_frame(&self, mode: LiveAdvanceMode, now: Instant) -> Result<()> {
        self.send(
            LiveCoreCommandName::RequestFrame,
            LiveCoreCommand::RequestFrame { mode, now },
        )
    }

    fn take_latest_frame(&self) -> Result<Option<LiveCoreFrame>> {
        if let Some(frame) = self.frames.take_latest()? {
            return Ok(Some(frame));
        }

        if self.worker_is_finished(LiveCoreCommandName::RequestFrame)? {
            return Err(self
                .worker_stopped_error(LiveCoreCommandName::RequestFrame)
                .into());
        }

        Ok(None)
    }

    #[cfg(test)]
    fn cmos_ram(&self) -> Result<CmosRam> {
        self.request(LiveCoreCommandName::CmosRam, |response| {
            LiveCoreCommand::CmosRam { response }
        })
    }

    fn shutdown_cmos_ram(&self) -> Result<CmosRam> {
        let cmos = self.request(LiveCoreCommandName::Shutdown, |response| {
            LiveCoreCommand::Shutdown {
                response: Some(response),
            }
        })?;
        self.join_worker(LiveCoreCommandName::Shutdown)?;
        Ok(cmos)
    }
}

impl Drop for LiveCoreThread {
    fn drop(&mut self) {
        let _ = self.send_command(
            LiveCoreCommandName::Shutdown,
            LiveCoreCommand::Shutdown { response: None },
        );
        let _ = self.join_worker(LiveCoreCommandName::Shutdown);
    }
}

fn run_live_core_thread(
    input_profile: InputProfile,
    machine: ArcadeMachine,
    now: Instant,
    renderer_size: (u32, u32),
    audio: LiveAudioRuntime,
    receiver: mpsc::Receiver<LiveCoreCommand>,
    frames: LiveCoreFrameMailbox,
) {
    let mut core = LiveCoreDriver::new_with_audio(input_profile, machine, now, audio);
    let mut renderer = Renderer::with_size(renderer_size.0, renderer_size.1);
    let mut polled_input = PolledInput::default();
    let mut next_render_error = None;

    while let Ok(command) = receiver.recv() {
        match command {
            LiveCoreCommand::InputEvent {
                input_event,
                response,
            } => {
                core.handle_input_event(input_event, &mut polled_input);
                let _ = response.send(polled_input.quit_requested);
            }
            LiveCoreCommand::ResetClock { now } => core.reset_clock(now),
            LiveCoreCommand::ResizeRenderer { width, height } => {
                renderer = Renderer::with_size(width, height);
            }
            #[cfg(test)]
            LiveCoreCommand::Advance {
                mode,
                now,
                response,
            } => {
                let result = advance_and_render_live_core_response(
                    &mut core,
                    &mut renderer,
                    &mut polled_input,
                    mode,
                    now,
                    &mut next_render_error,
                );
                let _ = response.send(result);
            }
            LiveCoreCommand::RequestFrame { mode, now } => {
                let result = advance_and_render_live_core_response(
                    &mut core,
                    &mut renderer,
                    &mut polled_input,
                    mode,
                    now,
                    &mut next_render_error,
                )
                .map_err(|error| {
                    LiveCoreError::worker_failed(LiveCoreCommandName::RequestFrame, error)
                });
                let _ = frames.post(result);
            }
            #[cfg(test)]
            LiveCoreCommand::CmosRam { response } => {
                let _ = response.send(*core.machine().red_label_cmos_ram());
            }
            LiveCoreCommand::Shutdown { response } => {
                if let Some(response) = response {
                    let _ = response.send(*core.machine().red_label_cmos_ram());
                }
                break;
            }
            #[cfg(test)]
            LiveCoreCommand::FailNextRenderForTest { message } => {
                next_render_error = Some(message);
            }
            #[cfg(test)]
            LiveCoreCommand::PanicForTest => {
                panic!("live core worker panic requested by test");
            }
        }
    }
}

fn advance_and_render_live_core_response(
    core: &mut LiveCoreDriver,
    renderer: &mut Renderer,
    polled_input: &mut PolledInput,
    mode: LiveAdvanceMode,
    now: Instant,
    next_render_error: &mut Option<String>,
) -> LiveCoreResponse<LiveCoreFrame> {
    if let Some(error) = next_render_error.take() {
        return Err(LiveCoreWorkerError::render(error));
    }

    advance_and_render_live_core(core, renderer, polled_input, mode, now)
        .map_err(|error| LiveCoreWorkerError::render(error.to_string()))
}

fn advance_and_render_live_core(
    core: &mut LiveCoreDriver,
    renderer: &mut Renderer,
    polled_input: &mut PolledInput,
    mode: LiveAdvanceMode,
    now: Instant,
) -> Result<LiveCoreFrame> {
    if !polled_input.quit_requested {
        match mode {
            LiveAdvanceMode::Realtime => {
                core.advance_realtime(polled_input, now);
            }
            LiveAdvanceMode::FixedFrames(frames) => {
                core.advance_fixed_frames(polled_input, frames);
            }
        }
    }

    *polled_input = PolledInput::default();
    let image = render_live_machine_frame(renderer, core.machine_mut())?.clone();
    let snapshot = core.machine().snapshot();
    let scene = live_render_scene(snapshot.frame, &image)?;
    Ok(LiveCoreFrame {
        image,
        scene,
        snapshot,
        next_step_at: core.next_step_at(),
    })
}

fn panic_message(panic: Box<dyn Any + Send + 'static>) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        return String::from(*message);
    }
    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }
    String::from("unknown panic payload")
}

#[cfg(all(not(test), not(coverage)))]
pub fn run_live(
    input_profile: InputProfile,
    audio_mode: LiveAudioMode,
    cmos_path: Option<&Path>,
) -> Result<()> {
    crate::wgpu_presenter::run_wgpu_live(input_profile, audio_mode, cmos_path)
}

#[cfg(any(test, coverage))]
pub fn run_live(
    _input_profile: InputProfile,
    _audio_mode: LiveAudioMode,
    _cmos_path: Option<&Path>,
) -> Result<()> {
    Ok(())
}

pub(crate) fn render_live_machine_frame<'a>(
    renderer: &'a mut Renderer,
    machine: &mut ArcadeMachine,
) -> Result<&'a RenderedImage> {
    machine
        .red_label_copy_color_mapping_to_palette_ram()
        .map_err(|error| anyhow!("copying red-label color mapping to palette RAM: {error}"))?;
    let native_frame = machine
        .red_label_visible_rgba_image()
        .context("red-label visible frame is unavailable")?;
    Ok(render_live_frame(renderer, native_frame))
}

fn render_live_frame(renderer: &mut Renderer, native_frame: RenderedImage) -> &RenderedImage {
    renderer.render_cabinet_frame(&native_frame)
}

fn live_render_scene(frame: u64, image: &RenderedImage) -> Result<RenderScene> {
    RenderScene::from_rgba(
        frame,
        SurfaceSize::new(image.width, image.height),
        image.pixels.clone(),
        Some(crc32(&image.pixels)),
    )
    .context("building live render scene")
}

pub(crate) fn step_live_core_frames(
    machine: &mut ArcadeMachine,
    first_input: CabinetInput,
    catch_up_input: CabinetInput,
    typed_chars: &[char],
    frames: u32,
    audio: &LiveAudioRuntime,
) {
    if frames == 0 {
        return;
    }

    let output = machine.step_with_typed_chars(first_input, typed_chars);
    audio.submit_frame_output(&output);
    for _ in 1..frames {
        let output = machine.step(catch_up_input);
        audio.submit_frame_output(&output);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LiveFrameInputs {
    pub(crate) first: CabinetInput,
    pub(crate) catch_up: CabinetInput,
}

pub(crate) fn live_frame_inputs(
    pending_pulses: &mut CabinetInput,
    polled_pulses: CabinetInput,
    held_input: CabinetInput,
    frames: u32,
) -> Option<LiveFrameInputs> {
    pending_pulses.merge(polled_pulses);
    if frames == 0 {
        return None;
    }

    let mut first = *pending_pulses;
    first.merge(held_input);
    *pending_pulses = CabinetInput::NONE;

    Some(LiveFrameInputs {
        first,
        catch_up: held_input,
    })
}

pub(crate) fn live_machine_from_cmos_storage(
    storage: Option<&dyn CmosStorage>,
) -> Result<ArcadeMachine> {
    let Some(storage) = storage else {
        return Ok(ArcadeMachine::new());
    };

    let Some(cmos) = storage.load_cmos().context("loading persisted CMOS RAM")? else {
        return Ok(ArcadeMachine::new());
    };

    ArcadeMachine::try_new_with_cmos(cmos)
        .map_err(|error| anyhow!("loading persisted CMOS RAM into arcade core: {error}"))
}

pub(crate) fn save_live_cmos_ram(storage: Option<&dyn CmosStorage>, cmos: &CmosRam) -> Result<()> {
    let Some(storage) = storage else {
        return Ok(());
    };

    storage.save_cmos(cmos).context("saving persisted CMOS RAM")
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::io;
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::audio::{LiveAudioBackend, LiveAudioCommandBatch, LiveAudioRuntime};
    use crate::board::{
        CMOS_RAM_SIZE, CmosRam, RED_LABEL_CRHSTD_CELL_OFFSET, cmos_sram_write_byte,
    };
    use crate::cmos_storage::CmosStorage;
    use crate::input::{
        CabinetInput, InputEvent, InputEventKind, InputKey, InputProfile, PolledInput,
    };
    use crate::machine::{ArcadeMachine, FRAME_RATE_MILLIHZ, VISIBLE_HEIGHT, VISIBLE_WIDTH};
    use crate::machine_state::{GamePhase, MachineEvent, RedLabelSoundBoardSnapshot};
    use crate::rom::crc32;
    use crate::sound::{SoundCommand, SoundCommandLatch};
    use crate::video::{RenderedImage, Renderer, defender_visible_byte_offset};

    use super::{
        FRAME_DURATION, LiveAdvanceMode, LiveCoreClock, LiveCoreCommandName, LiveCoreDriver,
        LiveCoreError, LiveCoreErrorKind, LiveCoreFrame, LiveCoreFrameMailbox, LiveCoreRuntime,
        LiveCoreThread, LiveCoreWorkerError, cabinet_frame_duration_micros, live_frame_inputs,
        live_machine_from_cmos_storage, live_render_scene, panic_message, render_live_frame,
        render_live_machine_frame, save_live_cmos_ram, step_live_core_frames,
    };

    #[derive(Default)]
    struct MemoryCmosStorage {
        cmos: RefCell<Option<CmosRam>>,
    }

    impl CmosStorage for MemoryCmosStorage {
        fn load_cmos(&self) -> io::Result<Option<CmosRam>> {
            Ok(*self.cmos.borrow())
        }

        fn save_cmos(&self, cmos: &CmosRam) -> io::Result<()> {
            *self.cmos.borrow_mut() = Some(*cmos);
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingAudioBackend {
        batches: Arc<Mutex<Vec<LiveAudioCommandBatch>>>,
    }

    impl LiveAudioBackend for RecordingAudioBackend {
        fn handle_command_batch(&mut self, batch: LiveAudioCommandBatch) {
            self.batches
                .lock()
                .expect("audio recording lock")
                .push(batch);
        }
    }

    #[test]
    fn render_live_frame_uses_native_cabinet_frame_even_when_blank() {
        let mut renderer = Renderer::with_size(32, 24);
        let blank_native = RenderedImage::new_blank(2, 1, [0, 0, 0, 255]);

        let image = render_live_frame(&mut renderer, blank_native);

        assert_eq!((image.width, image.height), (32, 24));
        assert!(
            image
                .pixels
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 0, 255])
        );
    }

    #[test]
    fn render_live_frame_uses_visible_native_frames_and_machine_wrapper() {
        let mut machine = ArcadeMachine::new();
        let visible = RenderedImage {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 0, 95, 255, 255],
        };
        let mut renderer = Renderer::with_size(32, 16);

        let native = render_live_frame(&mut renderer, visible);
        assert!(
            native
                .pixels
                .chunks_exact(4)
                .any(|pixel| pixel == [0, 95, 255, 255].as_slice())
        );

        let source_frame =
            render_live_machine_frame(&mut renderer, &mut machine).expect("render machine");
        assert_eq!((source_frame.width, source_frame.height), (32, 16));
    }

    #[test]
    fn live_render_scene_wraps_current_frame_as_temporary_raster_payload() {
        let image = RenderedImage::new_blank(2, 1, [9, 8, 7, 255]);

        let scene = live_render_scene(17, &image).expect("live render scene");

        assert_eq!(scene.frame, 17);
        assert_eq!(scene.summary().raster_count, 1);
        assert_eq!(scene.summary().visual_hash, Some(crc32(&image.pixels)));
        assert_eq!(
            scene.raster().expect("scene raster").pixels(),
            image.pixels.as_slice()
        );
    }

    #[test]
    fn render_live_machine_frame_applies_source_color_mapping_before_scaling() {
        let mut machine = ArcadeMachine::new();
        let visible_offset =
            defender_visible_byte_offset(0, 0).expect("visible origin maps into video RAM");
        machine.red_label_write_ram_byte_for_test(visible_offset as u16, 0xAB);
        machine.red_label_write_ram_byte_for_test(0xA026 + 0x0A, 0xD6);
        machine.red_label_write_ram_byte_for_test(0xA026 + 0x0B, 0x29);
        let mut renderer = Renderer::with_size(292, 240);

        let image = render_live_machine_frame(&mut renderer, &mut machine).expect("render machine");

        assert_eq!(machine.red_label_palette_ram()[0x0A], 0xD6);
        assert_eq!(machine.red_label_palette_ram()[0x0B], 0x29);
        assert_eq!(&image.pixels[0..4], &[217, 81, 255, 255]);
        assert_eq!(&image.pixels[4..8], &[38, 174, 0, 255]);
    }

    #[test]
    fn render_live_machine_frame_survives_williams_handoff_and_remains_playable() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut render_crcs = BTreeSet::new();
        let mut color_cycle_render_crcs = BTreeSet::new();
        let mut color_cycle_video_crcs = BTreeSet::new();
        let mut saw_non_blank_frame = false;
        let mut saw_post_williams_non_blank_frame = false;
        let mut post_non_blank_blank_frames = Vec::new();

        for frame in 0..1_220 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(
                output.snapshot.phase,
                GamePhase::Attract,
                "unexpected phase while rendering idle live attract frame {frame}"
            );
            if frame < 700 && frame % 20 != 0 {
                continue;
            }
            let (render_crc, non_blank) = {
                let image =
                    render_live_machine_frame(&mut renderer, &mut machine).expect("render frame");
                (
                    crc32(&image.pixels),
                    image
                        .pixels
                        .chunks_exact(4)
                        .any(|pixel| pixel != [0, 0, 0, 255].as_slice()),
                )
            };
            render_crcs.insert(render_crc);
            if saw_non_blank_frame && !non_blank {
                post_non_blank_blank_frames.push(frame);
            }
            saw_non_blank_frame |= non_blank;
            if frame >= 420 {
                saw_post_williams_non_blank_frame |= non_blank;
            }
            if (900..=1_040).contains(&frame) {
                color_cycle_render_crcs.insert(render_crc);
                color_cycle_video_crcs.insert(
                    machine
                        .red_label_visible_video_crc32()
                        .expect("video CRC remains available during Williams color cycle"),
                );
                assert!(
                    non_blank,
                    "Williams color-cycle frame {frame} should not render blank"
                );
            }
        }

        assert!(saw_non_blank_frame);
        assert!(saw_post_williams_non_blank_frame);
        assert!(
            post_non_blank_blank_frames.is_empty(),
            "render path returned blank frames after startup became visible: {post_non_blank_blank_frames:?}"
        );
        assert!(
            color_cycle_render_crcs.len() >= 2,
            "expected Williams color cycle to change rendered palette CRCs"
        );
        assert_eq!(
            color_cycle_video_crcs.len(),
            1,
            "Williams color cycle should not blank or rewrite visible video RAM"
        );
        assert!(
            render_crcs.len() > 8,
            "expected animated rendered frames through Williams handoff, got {} distinct CRCs",
            render_crcs.len()
        );

        let _ = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });
        let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render coin press");
        let mut credit_added = false;
        for _ in 0..32 {
            let output = machine.step(CabinetInput::NONE);
            let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render credit");
            credit_added |= output
                .events()
                .any(|event| event == MachineEvent::CreditAdded);
            if credit_added {
                break;
            }
        }
        assert!(credit_added);
        assert!(machine.snapshot().credits > 0);

        let mut game_started = false;
        for _ in 0..16 {
            let output = machine.step(CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            });
            let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render start");
            game_started |= output
                .events()
                .any(|event| event == MachineEvent::GameStarted);
            if game_started {
                break;
            }
        }
        assert!(game_started);
        assert_eq!(machine.snapshot().phase, GamePhase::Playing);
    }

    #[test]
    fn live_credited_start_renders_terrain_and_enemy_objects() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);

        start_live_one_player_game(&mut machine, &mut renderer);

        let mut saw_terrain = false;
        let mut saw_rendered_terrain = false;
        let mut saw_enemy_object = false;
        let mut saw_post_reverse_rendered_terrain = false;
        for frame in 0..1_200 {
            assert_live_target_list_valid(&machine);
            let input = if matches!(frame, 360 | 720) {
                CabinetInput {
                    reverse: true,
                    ..CabinetInput::NONE
                }
            } else {
                CabinetInput::NONE
            };
            machine.step(input);
            assert_live_target_list_valid(&machine);
            let image =
                render_live_machine_frame(&mut renderer, &mut machine).expect("render gameplay");
            saw_terrain |= native_nonzero_pixels_in_band(&machine, 180..240) >= 100;
            let rendered_terrain = rendered_nonblack_pixels_in_band(image, 180..240) >= 100;
            saw_rendered_terrain |= rendered_terrain;
            saw_post_reverse_rendered_terrain |= frame > 720 && rendered_terrain;
            saw_enemy_object |= live_visible_enemy_object_count(&machine) > 0
                || live_visible_enemy_appearance_count(&machine) > 0;

            // Keep running after the first good world frame. The release
            // build was able to show player/HUD/world briefly and then crash
            // from later process/object corruption.
        }

        assert!(
            saw_terrain,
            "credited live start did not render terrain/ground pixels"
        );
        assert!(
            saw_rendered_terrain,
            "credited live start did not present terrain/ground pixels in the rendered cabinet frame"
        );
        assert!(
            saw_enemy_object,
            "credited live start did not render visible active enemy/object pixels"
        );
        assert!(
            saw_post_reverse_rendered_terrain,
            "credited live reverse path did not preserve rendered terrain/ground pixels"
        );
    }

    #[test]
    fn rendered_live_gameplay_fire_draws_beam_not_single_bolt() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);

        start_live_one_player_game(&mut machine, &mut renderer);
        let mut longest_beam = 0;
        let mut longest_live_beam = 0;
        for frame in 0..1_200 {
            let input = if frame % 8 == 0 {
                CabinetInput {
                    fire: true,
                    ..CabinetInput::NONE
                }
            } else {
                CabinetInput::NONE
            };
            machine.step(input);
            let image =
                render_live_machine_frame(&mut renderer, &mut machine).expect("render fire");
            longest_beam =
                longest_beam.max(rendered_max_nonblack_horizontal_streak(image, 60..170));
            longest_live_beam = longest_live_beam.max(live_visible_laser_beam_byte_count(&machine));
            if longest_live_beam >= 12 {
                break;
            }
        }

        assert!(
            longest_live_beam >= 12,
            "gameplay fire rendered as a short bolt instead of an arcade-style beam; longest presented beam was {longest_live_beam} video bytes and longest rendered streak was {longest_beam} pixels"
        );
    }

    #[test]
    fn rendered_live_attract_visibly_advances_after_title_page() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut title_render_crc = None;
        let mut saw_post_title_render = false;

        for tick in 1..=6_000 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            if tick == 900 || tick > 900 && tick % 30 == 0 {
                let image =
                    render_live_machine_frame(&mut renderer, &mut machine).expect("render attract");
                let render_crc = crc32(&image.pixels);
                if tick == 900 {
                    title_render_crc = Some(render_crc);
                }
                if let Some(title_render_crc) = title_render_crc {
                    saw_post_title_render |= tick > 900 && render_crc != title_render_crc;
                }
            }
            if saw_post_title_render {
                return;
            }
        }

        assert!(
            saw_post_title_render,
            "rendered live attract did not visibly leave the title page"
        );
    }

    #[test]
    fn rendered_live_williams_defender_wordmark_does_not_blink_after_coalescing() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut saw_coalesced_wordmark = false;
        let mut saw_full_wordmark = false;
        let mut blanked_after_coalescing = Vec::new();

        for frame in 1..=520 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            let _ = render_live_machine_frame(&mut renderer, &mut machine)
                .expect("render Williams title");
            let wordmark_bytes = live_defender_wordmark_byte_count(&machine);
            saw_coalesced_wordmark |= wordmark_bytes >= 500;
            saw_full_wordmark |= wordmark_bytes >= 1_000;
            if saw_coalesced_wordmark && wordmark_bytes < 250 {
                blanked_after_coalescing.push((frame, wordmark_bytes));
            }
        }

        assert!(
            saw_coalesced_wordmark,
            "Williams title did not produce a coalescing DEFENDER wordmark"
        );
        assert!(
            saw_full_wordmark,
            "Williams title did not settle on the full DEFENDER wordmark"
        );
        assert!(
            blanked_after_coalescing.is_empty(),
            "Williams DEFENDER wordmark blinked after coalescing: {blanked_after_coalescing:?}"
        );
    }

    #[test]
    fn rendered_live_williams_defender_wordmark_clears_dot_bands_after_coalescing() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut saw_coalesced_wordmark = false;
        let mut saw_precoalescence_dot_bands = false;
        let mut stray_dot_bands = Vec::new();

        for frame in 1..=520 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            let _ = render_live_machine_frame(&mut renderer, &mut machine)
                .expect("render Williams title");

            let wordmark_bytes = live_defender_wordmark_byte_count(&machine);
            let noisy_rows: Vec<_> =
                native_nonzero_rows_in_band(&machine, 190..usize::from(VISIBLE_HEIGHT))
                    .into_iter()
                    .filter(|(_, count)| *count >= 20)
                    .collect();
            if wordmark_bytes >= 500 {
                saw_coalesced_wordmark = true;
                if !noisy_rows.is_empty() {
                    stray_dot_bands.push((frame, noisy_rows));
                }
            } else if !noisy_rows.is_empty() {
                saw_precoalescence_dot_bands = true;
            }
        }

        assert!(
            saw_precoalescence_dot_bands,
            "Williams title did not exercise the MAME-observed dot-band coalescence phase"
        );
        assert!(
            saw_coalesced_wordmark,
            "Williams title did not produce a coalescing DEFENDER wordmark"
        );
        assert!(
            stray_dot_bands.is_empty(),
            "Williams DEFENDER wordmark left post-coalescence dot bands: {stray_dot_bands:?}"
        );
    }

    #[test]
    fn rendered_live_attract_action_scene_has_objects_without_vertical_trails() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut saw_action_scene = false;
        let mut saw_visible_enemy_sprites = false;
        let mut worst_vertical_streak = 0;
        let mut worst_diagonal_streak = 0;

        for tick in 1..=5_900 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            if tick < 4_000 || tick % 30 != 0 {
                continue;
            }

            let image =
                render_live_machine_frame(&mut renderer, &mut machine).expect("render attract");
            let has_ship = live_object_screen_from_pointer(&machine, 0xA18B).is_some();
            let has_astronaut = live_object_screen_from_pointer(&machine, 0xA189).is_some();
            let has_terrain =
                rendered_nonblack_pixels_in_band(image, 180..usize::from(VISIBLE_HEIGHT)) >= 100;
            let vertical_streak = rendered_max_nonblack_vertical_streak(image, 40..220);
            let diagonal_streak = rendered_max_nonblack_diagonal_streak(image, 40..180);
            worst_vertical_streak = worst_vertical_streak.max(vertical_streak);
            worst_diagonal_streak = worst_diagonal_streak.max(diagonal_streak);
            saw_visible_enemy_sprites |=
                live_visible_attract_instruction_enemy_sprite_count(&machine) >= 3;
            if has_ship && has_astronaut && has_terrain {
                saw_action_scene = true;
                assert!(
                    vertical_streak <= 16,
                    "attract action scene retained a vertical object trail of {vertical_streak} pixels"
                );
            }
        }

        assert!(
            saw_action_scene,
            "rendered live attract did not show the instruction-scene ship, astronaut, and terrain"
        );
        assert!(
            saw_visible_enemy_sprites,
            "rendered live attract kept enemy state on the scanner/HUD without producing main-screen enemy sprites"
        );
        assert!(
            worst_vertical_streak <= 16,
            "rendered live attract retained vertical object trails; worst streak was {worst_vertical_streak} pixels"
        );
        assert!(
            worst_diagonal_streak <= 24,
            "rendered live attract retained diagonal ship/laser trails; worst streak was {worst_diagonal_streak} pixels"
        );
    }

    #[test]
    fn frame_duration_tracks_cabinet_refresh_not_old_ninety_ms_tick() {
        assert_eq!(
            FRAME_DURATION.as_micros(),
            u128::from(cabinet_frame_duration_micros(FRAME_RATE_MILLIHZ))
        );
        assert_eq!(FRAME_DURATION.as_micros(), 16_639);
    }

    fn start_live_one_player_game(machine: &mut ArcadeMachine, renderer: &mut Renderer) {
        let _ = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });
        let _ = render_live_machine_frame(renderer, machine).expect("render coin press");
        let mut credit_added = false;
        for _ in 0..32 {
            let output = machine.step(CabinetInput::NONE);
            let _ = render_live_machine_frame(renderer, machine).expect("render credit");
            credit_added |= output
                .events()
                .any(|event| event == MachineEvent::CreditAdded);
            if credit_added {
                break;
            }
        }
        assert!(credit_added);

        let mut game_started = false;
        for _ in 0..16 {
            let output = machine.step(CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            });
            let _ = render_live_machine_frame(renderer, machine).expect("render start");
            game_started |= output
                .events()
                .any(|event| event == MachineEvent::GameStarted);
            if game_started {
                break;
            }
        }
        assert!(game_started);
        assert_eq!(machine.snapshot().phase, GamePhase::Playing);
    }

    fn native_nonzero_pixels_in_band(
        machine: &ArcadeMachine,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let nibbles = machine
            .red_label_visible_pixel_nibbles()
            .expect("native visible pixel nibbles");
        let width = usize::from(VISIBLE_WIDTH);
        y_range
            .flat_map(|y| {
                let row = y * width;
                nibbles[row..row + width].iter()
            })
            .filter(|nibble| **nibble != 0)
            .count()
    }

    fn native_nonzero_rows_in_band(
        machine: &ArcadeMachine,
        y_range: std::ops::Range<usize>,
    ) -> Vec<(usize, usize)> {
        let nibbles = machine
            .red_label_visible_pixel_nibbles()
            .expect("native visible pixel nibbles");
        let width = usize::from(VISIBLE_WIDTH);
        y_range
            .map(|y| {
                let row = y * width;
                let count = nibbles[row..row + width]
                    .iter()
                    .filter(|nibble| **nibble != 0)
                    .count();
                (y, count)
            })
            .filter(|(_, count)| *count != 0)
            .collect()
    }

    fn rendered_nonblack_pixels_in_band(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        y_range
            .flat_map(|y| {
                let row = y * width * 4;
                image.pixels[row..row + width * 4].chunks_exact(4)
            })
            .filter(|pixel| *pixel != [0, 0, 0, 255])
            .count()
    }

    fn rendered_max_nonblack_vertical_streak(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        let mut max_streak = 0;
        for x in 0..width {
            let mut streak = 0;
            for y in y_range.clone() {
                let offset = (y * width + x) * 4;
                if &image.pixels[offset..offset + 4] != [0, 0, 0, 255].as_slice() {
                    streak += 1;
                    max_streak = max_streak.max(streak);
                } else {
                    streak = 0;
                }
            }
        }
        max_streak
    }

    fn rendered_max_nonblack_horizontal_streak(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        let mut max_streak = 0;
        for y in y_range {
            let mut streak = 0;
            for x in 0..width {
                let offset = (y * width + x) * 4;
                if &image.pixels[offset..offset + 4] != [0, 0, 0, 255].as_slice() {
                    streak += 1;
                    max_streak = max_streak.max(streak);
                } else {
                    streak = 0;
                }
            }
        }
        max_streak
    }

    fn rendered_max_nonblack_diagonal_streak(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        let height = image.height as usize;
        let mut max_streak = 0;
        for y in y_range.clone() {
            for x in 0..width {
                max_streak = max_streak.max(rendered_diagonal_streak_from(image, x, y, 1, 1));
                max_streak = max_streak.max(rendered_diagonal_streak_from(image, x, y, -1, 1));
            }
        }
        max_streak.min(height)
    }

    fn rendered_diagonal_streak_from(
        image: &RenderedImage,
        mut x: usize,
        mut y: usize,
        x_step: isize,
        y_step: isize,
    ) -> usize {
        let width = image.width as usize;
        let height = image.height as usize;
        let mut streak = 0;
        loop {
            let offset = (y * width + x) * 4;
            if &image.pixels[offset..offset + 4] == [0, 0, 0, 255].as_slice() {
                return streak;
            }
            streak += 1;
            let Some(next_x) = x.checked_add_signed(x_step) else {
                return streak;
            };
            let Some(next_y) = y.checked_add_signed(y_step) else {
                return streak;
            };
            if next_x >= width || next_y >= height {
                return streak;
            }
            x = next_x;
            y = next_y;
        }
    }

    fn live_object_screen_from_pointer(
        machine: &ArcadeMachine,
        pointer_address: u16,
    ) -> Option<u16> {
        let object_address = read_word(machine, pointer_address)?;
        if !live_object_address_is_valid(object_address) {
            return None;
        }
        let screen_address = read_word(machine, object_address + 0x04)?;
        (screen_address != 0).then_some(screen_address)
    }

    fn live_visible_enemy_object_count(machine: &ArcadeMachine) -> usize {
        let mut count = 0;
        let mut object_address = read_word(machine, 0xA065).expect("active object head");
        for _ in 0..95 {
            if object_address == 0 {
                break;
            }
            if !live_object_address_is_valid(object_address) {
                break;
            }

            let next_object = read_word(machine, object_address).expect("active object link word");
            let picture = read_word(machine, object_address + 0x02).expect("object OPICT");
            let screen = read_word(machine, object_address + 0x04).expect("object screen");
            let collision_vector =
                read_word(machine, object_address + 0x08).expect("object OCVECT");
            if screen != 0 && picture != 0xF8EC && live_enemy_collision_vector(collision_vector) {
                count += 1;
            }
            object_address = next_object;
        }
        count
    }

    fn live_visible_attract_instruction_enemy_sprite_count(machine: &ArcadeMachine) -> usize {
        const ATTRACT_ENEMY_PICTURES: [u16; 6] = [0xF985, 0xF8CE, 0xF9A3, 0xF929, 0xF8F7, 0xF97B];
        let mut count = 0;
        let mut object_address = read_word(machine, 0xA065).expect("active object head");
        for _ in 0..95 {
            if object_address == 0 {
                break;
            }
            if !live_object_address_is_valid(object_address) {
                break;
            }

            let next_object = read_word(machine, object_address).expect("active object link word");
            let picture = read_word(machine, object_address + 0x02).expect("object OPICT");
            let screen = read_word(machine, object_address + 0x04).expect("object screen");
            if screen != 0
                && ATTRACT_ENEMY_PICTURES.contains(&picture)
                && live_screen_region_has_nonzero_bytes(machine, screen, 16, 16)
            {
                count += 1;
            }
            object_address = next_object;
        }
        count
    }

    fn live_visible_laser_beam_byte_count(machine: &ArcadeMachine) -> usize {
        const ACTIVE_PROCESS_HEAD: u16 = 0xA05F;
        const PROCESS_TABLE_BASE: u16 = 0xAAC5;
        const PROCESS_ENTRY_SIZE: u16 = 0x0F;
        const PROCESS_ENTRIES: usize = 75;
        const LASR0: u16 = 0xE5CB;
        const LASL0: u16 = 0xE638;

        let mut longest_beam = 0;
        let mut process_address = read_word(machine, ACTIVE_PROCESS_HEAD).expect("ACTIVE process");
        for _ in 0..PROCESS_ENTRIES {
            if process_address == 0 {
                return longest_beam;
            }
            if !(PROCESS_TABLE_BASE..PROCESS_TABLE_BASE + PROCESS_ENTRY_SIZE * 75)
                .contains(&process_address)
                || !(process_address - PROCESS_TABLE_BASE).is_multiple_of(PROCESS_ENTRY_SIZE)
            {
                return longest_beam;
            }

            let next_process = read_word(machine, process_address).expect("process PLINK");
            let routine_address =
                read_word(machine, process_address + 0x02).expect("process PADDR");
            let direction = match routine_address {
                LASR0 => Some(LiveLaserDirection::Right),
                LASL0 => Some(LiveLaserDirection::Left),
                _ => None,
            };
            if let Some(direction) = direction {
                let tip = read_word(machine, process_address + 0x07).expect("process PD");
                let tail = read_word(machine, process_address + 0x0B).expect("process PD4");
                longest_beam = longest_beam.max(live_laser_beam_span_byte_count(
                    machine, direction, tail, tip,
                ));
            }
            process_address = next_process;
        }

        longest_beam
    }

    #[derive(Clone, Copy)]
    enum LiveLaserDirection {
        Right,
        Left,
    }

    fn live_laser_beam_span_byte_count(
        machine: &ArcadeMachine,
        direction: LiveLaserDirection,
        tail: u16,
        tip: u16,
    ) -> usize {
        let mut address = tail;
        let mut beam_bytes = 0;
        for _ in 0..=u8::MAX {
            if read_byte(machine, address).is_some_and(|byte| byte == 0x99) {
                beam_bytes += 1;
            }
            let next = match direction {
                LiveLaserDirection::Right => address.wrapping_add(0x0100),
                LiveLaserDirection::Left => address.wrapping_sub(0x0100),
            };
            let keep_counting = match direction {
                LiveLaserDirection::Right => next <= tip,
                LiveLaserDirection::Left => next >= tip,
            };
            if !keep_counting {
                return beam_bytes;
            }
            address = next;
        }

        beam_bytes
    }

    fn live_screen_region_has_nonzero_bytes(
        machine: &ArcadeMachine,
        upper_left: u16,
        width: u8,
        height: u8,
    ) -> bool {
        for column in 0..width {
            let column_address = upper_left.wrapping_add(u16::from(column) << 8);
            for row in 0..height {
                let address = column_address.wrapping_add(u16::from(row));
                if read_byte(machine, address).is_some_and(|byte| byte != 0) {
                    return true;
                }
            }
        }
        false
    }

    fn live_defender_wordmark_byte_count(machine: &ArcadeMachine) -> usize {
        let mut count = 0;
        for column in 0..0x3Cu16 {
            let column_address = 0x3090u16.wrapping_add(column << 8);
            for row in 0..0x18u16 {
                let address = column_address.wrapping_add(row);
                if read_byte(machine, address).is_some_and(|byte| byte != 0) {
                    count += 1;
                }
            }
        }
        count
    }

    fn assert_live_target_list_valid(machine: &ArcadeMachine) {
        let target_pointer = read_word(machine, 0xA09B).expect("TPTR");
        if target_pointer == 0 {
            return;
        }
        assert!(
            (0xA11A..0xA142).contains(&target_pointer)
                && (target_pointer - 0xA11A).is_multiple_of(2),
            "live target cursor drifted outside TLIST: 0x{target_pointer:04X}"
        );
        for slot_address in (0xA11A..0xA142).step_by(2) {
            let object_address = read_word(machine, slot_address).expect("TLIST slot");
            assert!(
                object_address == 0 || live_object_address_is_valid(object_address),
                "live TLIST slot 0x{slot_address:04X} points outside object table: 0x{object_address:04X}"
            );
        }
    }

    fn live_visible_enemy_appearance_count(machine: &ArcadeMachine) -> usize {
        let mut count = 0;
        for slot in 0..16 {
            let slot_address = 0x9C00 + slot * 0x40;
            let size = read_word(machine, slot_address).expect("appearance RSIZE");
            if size == 0 {
                continue;
            }
            let object_address =
                read_word(machine, slot_address + 0x0A).expect("appearance OBJPTR");
            if !live_object_address_is_valid(object_address) {
                continue;
            }
            let collision_vector =
                read_word(machine, object_address + 0x08).expect("appearance object OCVECT");
            if live_enemy_collision_vector(collision_vector) {
                count += 1;
            }
        }
        count
    }

    fn live_object_address_is_valid(object_address: u16) -> bool {
        (0xA23C..0xA23C + 95 * 0x17).contains(&object_address)
            && (object_address - 0xA23C).is_multiple_of(0x17)
    }

    fn live_enemy_collision_vector(collision_vector: u16) -> bool {
        matches!(collision_vector, 0xEB2B | 0xEBE9 | 0xEF6D | 0xF20B)
    }

    fn read_word(machine: &ArcadeMachine, address: u16) -> Option<u16> {
        let bytes = machine.red_label_ram_range(address..address + 2)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_byte(machine: &ArcadeMachine, address: u16) -> Option<u8> {
        machine
            .red_label_ram_range(address..address.wrapping_add(1))
            .and_then(|bytes| bytes.first().copied())
    }

    #[test]
    fn core_clock_reports_due_frames_independent_of_draw_calls() {
        let start = Instant::now();
        let mut clock = LiveCoreClock::new(start);

        assert_eq!(clock.steps_due(start), 1);
        assert_eq!(clock.sleep_until_next_step(start), FRAME_DURATION);
        assert_eq!(clock.steps_due(start + (FRAME_DURATION / 2)), 0);
        assert_eq!(
            clock.sleep_until_next_step(start + (FRAME_DURATION / 2)),
            FRAME_DURATION / 2
        );

        let stalled_until = start + FRAME_DURATION + FRAME_DURATION + FRAME_DURATION;
        assert_eq!(clock.steps_due(stalled_until), 3);
        assert_eq!(clock.sleep_until_next_step(stalled_until), FRAME_DURATION);
    }

    #[test]
    fn live_core_steps_catch_up_without_replaying_typed_chars() {
        let mut machine = ArcadeMachine::new();
        let audio = LiveAudioRuntime::disabled();
        step_live_core_frames(
            &mut machine,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &['z'],
            0,
            &audio,
        );
        assert_eq!(machine.snapshot().frame, 0);

        let mut snapshot = machine.snapshot();
        snapshot.phase = GamePhase::HighScoreEntry;
        machine.restore(snapshot);
        machine
            .red_label_begin_live_high_score_entry(50_000)
            .expect("high score table should be valid");

        step_live_core_frames(
            &mut machine,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &['a'],
            3,
            &audio,
        );

        let snapshot = machine.snapshot();
        assert_eq!(snapshot.frame, 3);
        assert_eq!(
            snapshot
                .high_score_entry
                .expect("entry still active")
                .initials,
            [b'A', b' ', b' ']
        );
    }

    #[test]
    fn live_core_driver_advances_machine_overlay_and_held_input() {
        let mut driver =
            LiveCoreDriver::new(InputProfile::Test, ArcadeMachine::new(), Instant::now());
        let mut input = PolledInput::default();
        driver.handle_input_event(
            InputEvent::new(InputKey::Char('t'), InputEventKind::Press),
            &mut input,
        );
        input.typed_chars.extend(['x', 'y', 'z', 'z', 'y', 'f']);

        driver.advance_fixed_frames(&input, 2);

        let snapshot = driver.machine().snapshot();
        assert_eq!(snapshot.frame, 2);
        assert_eq!(
            snapshot.last_input_bits,
            CabinetInput {
                thrust: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
        assert!(snapshot.xyzzy_active);
        assert!(snapshot.xyzzy_auto_fire);
    }

    #[test]
    fn live_core_driver_buffers_pulses_until_realtime_step_is_due() {
        let start = Instant::now();
        let mut driver = LiveCoreDriver::new(InputProfile::Test, ArcadeMachine::new(), start);
        let pulse = PolledInput {
            cabinet: CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            },
            ..PolledInput::default()
        };

        assert_eq!(
            driver.advance_realtime(&pulse, start - Duration::from_millis(1)),
            0
        );
        assert_eq!(driver.machine().snapshot().frame, 0);
        assert_eq!(driver.advance_realtime(&PolledInput::default(), start), 1);

        let snapshot = driver.machine().snapshot();
        assert_eq!(snapshot.frame, 1);
        assert_eq!(
            snapshot.last_input_bits,
            CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
    }

    #[test]
    fn live_core_driver_feeds_sound_commands_to_audio_runtime() {
        let start = Instant::now();
        let backend = RecordingAudioBackend::default();
        let recorded = backend.batches.clone();
        let audio = LiveAudioRuntime::spawn_with_capacity(backend, 8);
        let mut driver = LiveCoreDriver::new_with_audio(
            InputProfile::Test,
            ArcadeMachine::new_cold_boot_trace(),
            start,
            audio,
        );

        driver.advance_fixed_frames(&PolledInput::default(), 731);
        driver.shutdown_audio();

        let recorded = recorded.lock().expect("recorded audio batches");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].frame, 731);
        assert_eq!(
            recorded[0].commands().collect::<Vec<_>>(),
            vec![SoundCommand::from_main_board_pia_port_b(0xC0)]
        );
    }

    #[test]
    fn live_core_thread_advances_renders_and_reports_overlay_snapshot() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        assert_eq!(runtime.input_profile(), InputProfile::Test);
        runtime.reset_clock(start).expect("reset core clock");
        assert!(
            !runtime
                .handle_input_event(InputEvent::new(InputKey::Char('t'), InputEventKind::Press))
                .expect("input event")
        );
        for character in ['x', 'y', 'z', 'z', 'y', 'f'] {
            runtime
                .handle_input_event(InputEvent::new(
                    InputKey::Char(character),
                    InputEventKind::Press,
                ))
                .expect("typed input");
        }

        let frame = runtime
            .advance(LiveAdvanceMode::FixedFrames(2), start)
            .expect("advance core thread");

        assert_eq!((frame.image.width, frame.image.height), (64, 48));
        assert_eq!(frame.snapshot.frame, 2);
        assert_eq!(
            frame.snapshot.last_input_bits,
            CabinetInput {
                thrust: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
        assert!(frame.snapshot.xyzzy_active);
        assert!(frame.snapshot.xyzzy_auto_fire);
    }

    #[test]
    fn live_core_thread_buffers_pulses_until_realtime_step_is_due() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime
            .handle_input_event(InputEvent::new(InputKey::Char('5'), InputEventKind::Press))
            .expect("coin input");

        let early_frame = runtime
            .advance(LiveAdvanceMode::Realtime, start - Duration::from_millis(1))
            .expect("early frame");
        assert_eq!(early_frame.snapshot.frame, 0);

        let due_frame = runtime
            .advance(LiveAdvanceMode::Realtime, start)
            .expect("due frame");
        assert_eq!(due_frame.snapshot.frame, 1);
        assert_eq!(
            due_frame.snapshot.last_input_bits,
            CabinetInput {
                coin: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
    }

    #[test]
    fn live_core_thread_resizes_renderer_and_returns_cmos_snapshot() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime.resize_renderer(80, 60).expect("resize renderer");
        let frame = runtime
            .advance(LiveAdvanceMode::FixedFrames(1), start)
            .expect("advance after resize");

        assert_eq!((frame.image.width, frame.image.height), (80, 60));
        assert_eq!(
            runtime.cmos_ram().expect("cmos snapshot"),
            *ArcadeMachine::new().red_label_cmos_ram()
        );
    }

    #[test]
    fn live_core_frame_mailbox_replaces_stale_frames() {
        let mailbox = LiveCoreFrameMailbox::default();

        mailbox
            .post(Ok(test_live_core_frame(1, 32, 24)))
            .expect("post first frame");
        mailbox
            .post(Ok(test_live_core_frame(2, 48, 36)))
            .expect("replace first frame");

        let frame = mailbox
            .take_latest()
            .expect("take latest frame")
            .expect("latest frame");

        assert_eq!(frame.snapshot.frame, 2);
        assert_eq!((frame.image.width, frame.image.height), (48, 36));
        assert!(mailbox.take_latest().expect("take empty mailbox").is_none());
    }

    #[test]
    fn live_core_thread_async_frame_request_respects_resize_order() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime.resize_renderer(80, 60).expect("resize renderer");
        runtime
            .request_frame(LiveAdvanceMode::FixedFrames(1), start)
            .expect("request async frame");

        let frame = wait_for_latest_frame(&runtime);

        assert_eq!(frame.snapshot.frame, 1);
        assert_eq!((frame.image.width, frame.image.height), (80, 60));
    }

    #[test]
    fn live_core_thread_async_fixed_frame_requests_are_deterministic() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime
            .request_frame(LiveAdvanceMode::FixedFrames(1), start)
            .expect("request first async frame");
        let first = wait_for_latest_frame(&runtime);
        runtime
            .request_frame(LiveAdvanceMode::FixedFrames(1), start + FRAME_DURATION)
            .expect("request second async frame");
        let second = wait_for_latest_frame(&runtime);

        assert_eq!(first.snapshot.frame, 1);
        assert_eq!(second.snapshot.frame, 2);
    }

    #[test]
    fn live_core_thread_drop_joins_in_flight_async_frame_request() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));
        runtime
            .request_frame(LiveAdvanceMode::FixedFrames(1), start)
            .expect("request async frame before drop");
        let (dropped, receiver) = mpsc::channel();

        thread::spawn(move || {
            drop(runtime);
            dropped.send(()).expect("report runtime drop");
        });

        receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("runtime drop should join worker thread");
    }

    #[test]
    fn live_core_thread_shutdown_cmos_ram_returns_final_mutated_cmos() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));
        let mut expected = ArcadeMachine::new();
        expected.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });

        runtime
            .handle_input_event(InputEvent::new(InputKey::Char('5'), InputEventKind::Press))
            .expect("coin input");
        runtime
            .advance(LiveAdvanceMode::FixedFrames(1), start)
            .expect("advance coin frame");

        let cmos = runtime
            .shutdown_cmos_ram()
            .expect("shutdown with final CMOS");

        assert_eq!(cmos, *expected.red_label_cmos_ram());
    }

    #[test]
    fn live_core_thread_failed_command_reports_command_context() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));
        runtime
            .shutdown_cmos_ram()
            .expect("shutdown should return CMOS");

        let error = runtime
            .cmos_ram()
            .expect_err("CMOS command after shutdown should fail");
        let runtime_error = live_core_error(&error);

        assert_eq!(runtime_error.command(), LiveCoreCommandName::CmosRam);
        assert!(matches!(
            runtime_error.kind(),
            LiveCoreErrorKind::CommandSend
        ));
        assert!(error.to_string().contains("cmos_ram"));
    }

    #[test]
    fn live_core_thread_take_latest_frame_reports_stopped_worker_context() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));
        runtime
            .shutdown_cmos_ram()
            .expect("shutdown should return CMOS");

        let error = runtime
            .take_latest_frame()
            .expect_err("taking a frame after shutdown should fail");
        let runtime_error = live_core_error(&error);

        assert_eq!(runtime_error.command(), LiveCoreCommandName::RequestFrame);
        assert!(matches!(
            runtime_error.kind(),
            LiveCoreErrorKind::WorkerTerminated
        ));
        assert!(error.to_string().contains("request_frame"));
    }

    #[test]
    fn live_core_thread_reports_worker_panic_with_command_context() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime.panic_worker_for_test().expect("send panic command");

        let error = runtime
            .cmos_ram()
            .expect_err("command after worker panic should fail");
        let runtime_error = live_core_error(&error);

        assert_eq!(runtime_error.command(), LiveCoreCommandName::CmosRam);
        match runtime_error.kind() {
            LiveCoreErrorKind::WorkerPanic(message) => {
                assert!(message.contains("live core worker panic requested by test"));
            }
            kind => panic!("expected worker panic, got {kind:?}"),
        }
    }

    #[test]
    fn live_core_thread_sync_render_error_reports_advance_context() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime
            .fail_next_render_for_test("forced sync render failure")
            .expect("arm render failure");

        let error = runtime
            .advance(LiveAdvanceMode::FixedFrames(1), start)
            .expect_err("forced render failure should propagate");
        let runtime_error = live_core_error(&error);

        assert_eq!(runtime_error.command(), LiveCoreCommandName::Advance);
        assert_render_error_contains(runtime_error, "forced sync render failure");
    }

    #[test]
    fn live_core_thread_async_render_error_reports_request_frame_context() {
        let start = Instant::now();
        let runtime =
            LiveCoreThread::spawn(InputProfile::Test, ArcadeMachine::new(), start, (64, 48));

        runtime
            .fail_next_render_for_test("forced async render failure")
            .expect("arm render failure");
        runtime
            .request_frame(LiveAdvanceMode::FixedFrames(1), start)
            .expect("request async frame");

        let error = wait_for_latest_frame_error(&runtime);
        let runtime_error = live_core_error(&error);

        assert_eq!(runtime_error.command(), LiveCoreCommandName::RequestFrame);
        assert_render_error_contains(runtime_error, "forced async render failure");
    }

    #[test]
    fn live_core_error_display_and_source_cover_error_kinds() {
        use std::error::Error as _;

        let render_error = LiveCoreWorkerError::render("frame upload failed");
        let failed =
            LiveCoreError::worker_failed(LiveCoreCommandName::Advance, render_error.clone());
        assert_eq!(
            render_error.to_string(),
            "render failed: frame upload failed"
        );
        assert_eq!(
            failed.to_string(),
            "live core command advance failed: render failed: frame upload failed"
        );
        assert!(failed.source().is_some());

        let terminated = LiveCoreError::worker_terminated(LiveCoreCommandName::RequestFrame);
        assert!(terminated.to_string().contains("worker terminated"));
        assert!(terminated.source().is_none());

        let panicked =
            LiveCoreError::worker_panic(LiveCoreCommandName::CmosRam, String::from("boom"));
        assert!(panicked.to_string().contains("worker panicked: boom"));

        let mailbox = LiveCoreError::mailbox_unavailable(LiveCoreCommandName::RequestFrame);
        assert!(mailbox.to_string().contains("frame mailbox is unavailable"));

        let worker_state = LiveCoreError::worker_state_unavailable(LiveCoreCommandName::Shutdown);
        assert!(
            worker_state
                .to_string()
                .contains("worker state is unavailable")
        );
    }

    #[test]
    fn live_core_panic_message_formats_supported_payloads() {
        assert_eq!(
            panic_message(Box::new(String::from("owned panic"))),
            "owned panic"
        );
        assert_eq!(panic_message(Box::new(())), "unknown panic payload");
    }

    #[test]
    fn live_frame_inputs_buffer_pulse_inputs_until_a_core_frame_is_due() {
        let mut pending = CabinetInput::NONE;
        let polled = CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        };

        assert_eq!(
            live_frame_inputs(&mut pending, polled, CabinetInput::NONE, 0),
            None
        );
        assert!(pending.coin);

        let frame_inputs =
            live_frame_inputs(&mut pending, CabinetInput::NONE, CabinetInput::NONE, 1)
                .expect("buffered coin should be consumed on the next core frame");

        assert!(frame_inputs.first.coin);
        assert!(!frame_inputs.catch_up.coin);
        assert_eq!(pending, CabinetInput::NONE);
    }

    #[test]
    fn live_frame_inputs_do_not_replay_pulses_across_catch_up_frames() {
        let mut pending = CabinetInput::NONE;
        let polled = CabinetInput {
            start_one: true,
            ..CabinetInput::NONE
        };
        let held = CabinetInput {
            thrust: true,
            ..CabinetInput::NONE
        };

        let frame_inputs =
            live_frame_inputs(&mut pending, polled, held, 3).expect("frames are due");

        assert!(frame_inputs.first.start_one);
        assert!(frame_inputs.first.thrust);
        assert!(!frame_inputs.catch_up.start_one);
        assert!(frame_inputs.catch_up.thrust);
        assert_eq!(pending, CabinetInput::NONE);
    }

    #[test]
    fn live_core_sound_state_is_presentation_independent() {
        let mut baseline_core = ArcadeMachine::new_cold_boot_trace();
        let mut comparison_core = ArcadeMachine::new_cold_boot_trace();
        let audio = LiveAudioRuntime::disabled();

        step_live_core_frames(
            &mut baseline_core,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &[],
            731,
            &audio,
        );
        step_live_core_frames(
            &mut comparison_core,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &[],
            731,
            &audio,
        );

        let expected = RedLabelSoundBoardSnapshot {
            last_command_latch: Some(SoundCommandLatch::from_main_board_pia_port_b(0xC0)),
            latched_port_b: Some(0xC0),
            command_cb1_asserted: true,
            latch_write_count: 1,
        };
        assert_eq!(baseline_core.red_label_sound_board_snapshot(), expected);
        assert_eq!(comparison_core.red_label_sound_board_snapshot(), expected);
        assert_eq!(
            comparison_core.red_label_sound_board_snapshot(),
            baseline_core.red_label_sound_board_snapshot()
        );
    }

    #[test]
    fn live_cmos_storage_loads_and_saves_machine_cmos() {
        let storage = MemoryCmosStorage::default();
        let mut cmos = [0xF0; CMOS_RAM_SIZE];
        let high_score_offset = usize::from(RED_LABEL_CRHSTD_CELL_OFFSET);
        cmos_sram_write_byte(&mut cmos, high_score_offset, 0x21).expect("write score high byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 2, 0x27)
            .expect("write score middle byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 4, 0x00).expect("write score low byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 6, b'D').expect("write first initial");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 8, b'R').expect("write second initial");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 10, b'J').expect("write third initial");
        cmos_sram_write_byte(&mut cmos, 0x7D, 0x04).expect("write persisted credits");
        *storage.cmos.borrow_mut() = Some(cmos);

        let machine =
            live_machine_from_cmos_storage(Some(&storage)).expect("load machine from CMOS");
        assert_eq!(
            machine.red_label_cmos_range(0x7D..0x7F),
            Some(&cmos[0x7D..0x7F])
        );

        let mut changed_machine = ArcadeMachine::try_new_with_cmos(cmos).expect("machine");
        changed_machine.step(crate::input::CabinetInput {
            coin: true,
            ..crate::input::CabinetInput::NONE
        });
        save_live_cmos_ram(Some(&storage), changed_machine.red_label_cmos_ram())
            .expect("save machine CMOS");
        assert_eq!(
            storage.cmos.borrow().expect("saved CMOS"),
            *changed_machine.red_label_cmos_ram()
        );
    }

    fn wait_for_latest_frame(runtime: &LiveCoreThread) -> LiveCoreFrame {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(frame) = runtime
                .take_latest_frame()
                .expect("take latest async frame")
            {
                return frame;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for async live core frame"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn wait_for_latest_frame_error(runtime: &LiveCoreThread) -> anyhow::Error {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match runtime.take_latest_frame() {
                Ok(Some(_)) => panic!("expected async frame error, got frame"),
                Ok(None) => {}
                Err(error) => return error,
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for async live core frame error"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn live_core_error(error: &anyhow::Error) -> &LiveCoreError {
        error
            .downcast_ref::<LiveCoreError>()
            .expect("expected typed live core error")
    }

    fn assert_render_error_contains(error: &LiveCoreError, expected: &str) {
        match error.kind() {
            LiveCoreErrorKind::WorkerFailed(LiveCoreWorkerError::Render(message)) => {
                assert!(message.contains(expected));
            }
            kind => panic!("expected render error, got {kind:?}"),
        }
    }

    fn test_live_core_frame(frame: u64, width: u32, height: u32) -> LiveCoreFrame {
        let mut snapshot = ArcadeMachine::new().snapshot();
        snapshot.frame = frame;
        let image = RenderedImage::new_blank(width, height, [frame as u8, 0, 0, 255]);
        let scene = live_render_scene(frame, &image).expect("test live scene");
        LiveCoreFrame {
            image,
            scene,
            snapshot,
            next_step_at: Instant::now() + FRAME_DURATION,
        }
    }
}
