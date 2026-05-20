//! Runtime boundary for gameplay-facing sound events.

use std::any::Any;
use std::f32::consts::TAU;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    mpsc::{self, SyncSender},
};
use std::thread::{self, JoinHandle};

#[cfg(not(coverage))]
use anyhow::{Context, anyhow};
#[cfg(not(coverage))]
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::game::{GameFrame, SoundEvent};

pub const LIVE_AUDIO_TEST_SAMPLE_RATE_HZ: u32 = 48_000;
pub const LIVE_AUDIO_QUEUE_CAPACITY: usize = 8;
pub const LIVE_AUDIO_EVENT_CAPACITY: usize = 8;
const LIVE_AUDIO_MAX_SYNTH_VOICES: usize = 16;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LiveAudioMode {
    Disabled,
    #[default]
    Device,
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveAudioEventBatch {
    pub frame: u64,
    events: [Option<SoundEvent>; LIVE_AUDIO_EVENT_CAPACITY],
}

impl LiveAudioEventBatch {
    pub fn new(frame: u64, events: &[SoundEvent]) -> Option<Self> {
        if events.is_empty() || events.len() > LIVE_AUDIO_EVENT_CAPACITY {
            return None;
        }

        let mut batch = Self {
            frame,
            events: [None; LIVE_AUDIO_EVENT_CAPACITY],
        };
        for (slot, event) in batch.events.iter_mut().zip(events.iter().copied()) {
            *slot = Some(event);
        }
        Some(batch)
    }

    pub fn from_game_frame(frame: &GameFrame) -> Option<Self> {
        Self::new(frame.state.frame, frame.events.sounds())
    }

    pub fn events(&self) -> impl Iterator<Item = SoundEvent> + '_ {
        self.events.iter().copied().flatten()
    }

    pub fn event_count(&self) -> usize {
        self.events().count()
    }
}

pub trait LiveAudioBackend: Send + 'static {
    fn sample_rate_hz(&self) -> u32 {
        LIVE_AUDIO_TEST_SAMPLE_RATE_HZ
    }

    fn handle_event_batch(&mut self, batch: LiveAudioEventBatch);

    fn flush(&mut self) {}

    fn shutdown(&mut self) {}
}

pub struct NullLiveAudioBackend;

impl LiveAudioBackend for NullLiveAudioBackend {
    fn handle_event_batch(&mut self, _batch: LiveAudioEventBatch) {}
}

#[cfg(not(coverage))]
pub struct DeviceLiveAudioBackend {
    sample_rate_hz: u32,
    mixer: Arc<Mutex<SynthMixer>>,
    _stream: Stream,
}

#[cfg(not(coverage))]
impl DeviceLiveAudioBackend {
    pub fn try_new() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("no default audio output device"))?;
        let supported_config = device
            .default_output_config()
            .context("querying default audio output config")?;
        let sample_rate_hz = supported_config.sample_rate();
        let mixer = Arc::new(Mutex::new(SynthMixer::new(sample_rate_hz)));
        let stream = build_device_stream(&device, &supported_config, Arc::clone(&mixer))?;

        stream.play().context("starting audio output stream")?;
        Ok(Self {
            sample_rate_hz,
            mixer,
            _stream: stream,
        })
    }
}

#[cfg(not(coverage))]
impl LiveAudioBackend for DeviceLiveAudioBackend {
    fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    fn handle_event_batch(&mut self, batch: LiveAudioEventBatch) {
        if let Ok(mut mixer) = self.mixer.lock() {
            for event in batch.events() {
                mixer.queue_event(event);
            }
        }
    }

    fn flush(&mut self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.clear();
        }
    }
}

#[cfg(not(coverage))]
fn build_device_stream(
    device: &cpal::Device,
    supported_config: &SupportedStreamConfig,
    mixer: Arc<Mutex<SynthMixer>>,
) -> anyhow::Result<Stream> {
    let sample_format = supported_config.sample_format();
    let config = supported_config.clone().into();

    match sample_format {
        SampleFormat::F32 => build_typed_device_stream::<f32>(device, &config, mixer),
        SampleFormat::I16 => build_typed_device_stream::<i16>(device, &config, mixer),
        SampleFormat::U16 => build_typed_device_stream::<u16>(device, &config, mixer),
        sample_format => Err(anyhow!(
            "unsupported default audio output sample format {sample_format}"
        )),
    }
}

#[cfg(not(coverage))]
fn build_typed_device_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mixer: Arc<Mutex<SynthMixer>>,
) -> anyhow::Result<Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels).max(1);

    device
        .build_output_stream(
            config,
            move |output: &mut [T], _| write_mixed_samples(output, channels, &mixer),
            move |_error| {},
            None,
        )
        .context("building audio output stream")
}

#[cfg(not(coverage))]
fn write_mixed_samples<T>(output: &mut [T], channels: usize, mixer: &Arc<Mutex<SynthMixer>>)
where
    T: Sample + FromSample<f32>,
{
    let mut mixer = mixer.lock().ok();
    for frame in output.chunks_mut(channels) {
        let sample = mixer
            .as_mut()
            .map(|mixer| mixer.next_sample())
            .unwrap_or(0.0);
        let sample = T::from_sample(sample);
        for output_sample in frame {
            *output_sample = sample;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SynthClipSpec {
    frequency_hz: f32,
    duration_ms: u32,
    amplitude: f32,
}

impl SynthClipSpec {
    const fn new(frequency_hz: f32, duration_ms: u32, amplitude: f32) -> Self {
        Self {
            frequency_hz,
            duration_ms,
            amplitude,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SynthVoice {
    phase: f32,
    phase_step: f32,
    remaining_samples: u32,
    amplitude: f32,
}

impl SynthVoice {
    fn new(sample_rate_hz: u32, spec: SynthClipSpec) -> Self {
        let sample_rate = sample_rate_hz.max(1) as f32;
        let remaining_samples = ((u64::from(sample_rate_hz.max(1)) * u64::from(spec.duration_ms))
            / 1_000)
            .clamp(1, u64::from(u32::MAX)) as u32;
        Self {
            phase: 0.0,
            phase_step: TAU * spec.frequency_hz / sample_rate,
            remaining_samples,
            amplitude: spec.amplitude,
        }
    }

    fn next_sample(&mut self) -> f32 {
        if self.remaining_samples == 0 {
            return 0.0;
        }

        let sample = self.phase.sin() * self.amplitude;
        self.phase = (self.phase + self.phase_step) % TAU;
        self.remaining_samples = self.remaining_samples.saturating_sub(1);
        sample
    }

    const fn is_active(&self) -> bool {
        self.remaining_samples > 0
    }
}

#[derive(Debug)]
struct SynthMixer {
    sample_rate_hz: u32,
    voices: Vec<SynthVoice>,
}

impl SynthMixer {
    fn new(sample_rate_hz: u32) -> Self {
        Self {
            sample_rate_hz: sample_rate_hz.max(1),
            voices: Vec::new(),
        }
    }

    fn queue_event(&mut self, event: SoundEvent) {
        self.queue_clip(synth_clip_for_event(event));
    }

    fn queue_clip(&mut self, spec: SynthClipSpec) {
        if self.voices.len() >= LIVE_AUDIO_MAX_SYNTH_VOICES {
            self.voices.remove(0);
        }
        self.voices.push(SynthVoice::new(self.sample_rate_hz, spec));
    }

    fn clear(&mut self) {
        self.voices.clear();
    }

    fn next_sample(&mut self) -> f32 {
        let mut mixed = 0.0;
        for voice in &mut self.voices {
            mixed += voice.next_sample();
        }
        self.voices.retain(SynthVoice::is_active);
        mixed.clamp(-0.85, 0.85)
    }
}

fn synth_clip_for_event(event: SoundEvent) -> SynthClipSpec {
    match event {
        SoundEvent::Startup => SynthClipSpec::new(220.0, 120, 0.18),
        SoundEvent::CreditAdded => SynthClipSpec::new(880.0, 90, 0.20),
        SoundEvent::GameStarted => SynthClipSpec::new(660.0, 160, 0.22),
        SoundEvent::ThrustStarted => SynthClipSpec::new(140.0, 110, 0.16),
        SoundEvent::ThrustStopped => SynthClipSpec::new(95.0, 70, 0.12),
        SoundEvent::UnmappedSoundCommand { command } => {
            SynthClipSpec::new(300.0 + f32::from(command), 80, 0.12)
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LiveAudioStats {
    pub queued_batches: u64,
    pub queued_events: u64,
    pub dropped_batches: u64,
    pub dropped_events: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveAudioWorkerState {
    Disabled,
    Starting,
    Running,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveAudioDiagnostics {
    pub enabled: bool,
    pub worker_state: LiveAudioWorkerState,
    pub backend_sample_rate_hz: Option<u32>,
    pub stats: LiveAudioStats,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveAudioWorkerError {
    Panicked(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveAudioShutdownReport {
    pub diagnostics: LiveAudioDiagnostics,
    pub worker_error: Option<LiveAudioWorkerError>,
}

#[derive(Default)]
struct SharedLiveAudioDiagnostics {
    queued_batches: AtomicU64,
    queued_events: AtomicU64,
    dropped_batches: AtomicU64,
    dropped_events: AtomicU64,
    backend_sample_rate_hz: AtomicU32,
    worker_started: AtomicBool,
    worker_stopped: AtomicBool,
}

impl SharedLiveAudioDiagnostics {
    fn record_queued(&self, event_count: usize) {
        self.queued_batches.fetch_add(1, Ordering::Relaxed);
        self.queued_events
            .fetch_add(event_count as u64, Ordering::Relaxed);
    }

    fn record_dropped(&self, event_count: usize) {
        self.dropped_batches.fetch_add(1, Ordering::Relaxed);
        self.dropped_events
            .fetch_add(event_count as u64, Ordering::Relaxed);
    }

    fn stats(&self) -> LiveAudioStats {
        LiveAudioStats {
            queued_batches: self.queued_batches.load(Ordering::Relaxed),
            queued_events: self.queued_events.load(Ordering::Relaxed),
            dropped_batches: self.dropped_batches.load(Ordering::Relaxed),
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
        }
    }

    fn diagnostics(&self, enabled: bool) -> LiveAudioDiagnostics {
        let worker_started = self.worker_started.load(Ordering::Relaxed);
        let worker_stopped = self.worker_stopped.load(Ordering::Relaxed);
        let backend_sample_rate_hz = match self.backend_sample_rate_hz.load(Ordering::Relaxed) {
            0 => None,
            sample_rate_hz => Some(sample_rate_hz),
        };
        let worker_state = match (worker_started, worker_stopped, enabled) {
            (true, true, _) => LiveAudioWorkerState::Stopped,
            (true, false, _) => LiveAudioWorkerState::Running,
            (false, false, true) => LiveAudioWorkerState::Starting,
            (false, false, false) | (false, true, _) => LiveAudioWorkerState::Disabled,
        };

        LiveAudioDiagnostics {
            enabled,
            worker_state,
            backend_sample_rate_hz,
            stats: self.stats(),
        }
    }
}

enum LiveAudioMessage {
    EventBatch(LiveAudioEventBatch),
    Flush,
    Shutdown,
}

pub struct LiveAudioRuntime {
    sender: Option<SyncSender<LiveAudioMessage>>,
    diagnostics: Arc<SharedLiveAudioDiagnostics>,
    handle: Option<JoinHandle<()>>,
}

impl LiveAudioRuntime {
    pub fn disabled() -> Self {
        Self {
            sender: None,
            diagnostics: Arc::new(SharedLiveAudioDiagnostics::default()),
            handle: None,
        }
    }

    pub fn for_mode(mode: LiveAudioMode) -> Self {
        match mode {
            LiveAudioMode::Disabled => Self::disabled(),
            LiveAudioMode::Device => Self::device_or_null(),
            LiveAudioMode::Null => Self::spawn(NullLiveAudioBackend),
        }
    }

    fn device_or_null() -> Self {
        #[cfg(not(coverage))]
        {
            match DeviceLiveAudioBackend::try_new() {
                Ok(backend) => Self::spawn(backend),
                Err(_) => Self::spawn(NullLiveAudioBackend),
            }
        }
        #[cfg(coverage)]
        {
            Self::spawn(NullLiveAudioBackend)
        }
    }

    pub fn spawn<B>(backend: B) -> Self
    where
        B: LiveAudioBackend,
    {
        Self::spawn_with_capacity(backend, LIVE_AUDIO_QUEUE_CAPACITY)
    }

    pub fn spawn_with_capacity<B>(backend: B, queue_capacity: usize) -> Self
    where
        B: LiveAudioBackend,
    {
        let diagnostics = Arc::new(SharedLiveAudioDiagnostics::default());
        diagnostics
            .backend_sample_rate_hz
            .store(backend.sample_rate_hz(), Ordering::Relaxed);
        let worker_diagnostics = Arc::clone(&diagnostics);
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let handle = thread::spawn(move || {
            run_live_audio_thread(backend, receiver, worker_diagnostics);
        });

        Self {
            sender: Some(sender),
            diagnostics,
            handle: Some(handle),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.sender.is_some()
    }

    pub fn submit_game_frame(&self, frame: &GameFrame) {
        if let Some(batch) = LiveAudioEventBatch::from_game_frame(frame) {
            self.submit_event_batch(batch);
        }
    }

    pub fn submit_event_batch(&self, batch: LiveAudioEventBatch) {
        let Some(sender) = &self.sender else {
            return;
        };
        let event_count = batch.event_count();
        match sender.try_send(LiveAudioMessage::EventBatch(batch)) {
            Ok(()) => self.diagnostics.record_queued(event_count),
            Err(_) => self.diagnostics.record_dropped(event_count),
        }
    }

    pub fn flush(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.try_send(LiveAudioMessage::Flush);
        }
    }

    pub fn stats(&self) -> LiveAudioStats {
        self.diagnostics.stats()
    }

    pub fn diagnostics(&self) -> LiveAudioDiagnostics {
        self.diagnostics.diagnostics(self.is_enabled())
    }

    pub fn shutdown(&mut self) -> LiveAudioShutdownReport {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(LiveAudioMessage::Shutdown);
        }

        let worker_error = self.handle.take().and_then(|handle| match handle.join() {
            Ok(()) => None,
            Err(payload) => Some(LiveAudioWorkerError::Panicked(panic_message(&*payload))),
        });

        LiveAudioShutdownReport {
            diagnostics: self.diagnostics(),
            worker_error,
        }
    }
}

impl Drop for LiveAudioRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn run_live_audio_thread<B>(
    mut backend: B,
    receiver: mpsc::Receiver<LiveAudioMessage>,
    diagnostics: Arc<SharedLiveAudioDiagnostics>,
) where
    B: LiveAudioBackend,
{
    diagnostics.worker_started.store(true, Ordering::Relaxed);
    for message in receiver {
        match message {
            LiveAudioMessage::EventBatch(batch) => backend.handle_event_batch(batch),
            LiveAudioMessage::Flush => backend.flush(),
            LiveAudioMessage::Shutdown => break,
        }
    }
    backend.shutdown();
    diagnostics.worker_stopped.store(true, Ordering::Relaxed);
}

fn panic_message(payload: &(dyn Any + Send + 'static)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;

    use crate::{
        audio::{
            LIVE_AUDIO_TEST_SAMPLE_RATE_HZ, LiveAudioBackend, LiveAudioEventBatch, LiveAudioMode,
            LiveAudioRuntime, LiveAudioStats, LiveAudioWorkerError, LiveAudioWorkerState,
            SynthClipSpec, SynthMixer, SynthVoice, synth_clip_for_event,
        },
        game::{
            Direction, GameEvents, GameFrame, GamePhase, GameState, PlayerSnapshot, ScoreSnapshot,
            SoundEvent, WorldSnapshot, WorldVector,
        },
        renderer::{RenderScene, SurfaceSize},
    };

    #[derive(Default)]
    struct RecordingAudioBackend {
        batches: Arc<Mutex<Vec<LiveAudioEventBatch>>>,
    }

    impl LiveAudioBackend for RecordingAudioBackend {
        fn handle_event_batch(&mut self, batch: LiveAudioEventBatch) {
            self.batches
                .lock()
                .expect("audio recording lock")
                .push(batch);
        }
    }

    struct BlockingAudioBackend {
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }

    impl LiveAudioBackend for BlockingAudioBackend {
        fn handle_event_batch(&mut self, _batch: LiveAudioEventBatch) {
            self.entered.send(()).expect("signal audio backend entry");
            self.release.recv().expect("release blocked audio backend");
        }
    }

    struct PanickingAudioBackend;

    impl LiveAudioBackend for PanickingAudioBackend {
        fn handle_event_batch(&mut self, _batch: LiveAudioEventBatch) {
            panic!("audio backend failed");
        }
    }

    fn game_frame_with_sounds(frame: u64, sounds: Vec<SoundEvent>) -> GameFrame {
        GameFrame {
            state: GameState {
                frame,
                phase: GamePhase::Attract,
                credits: 0,
                current_player: 1,
                player_count: 1,
                wave: 0,
                wave_profile: crate::WaveProfileSnapshot::for_wave(0),
                player: PlayerSnapshot {
                    position: (WorldVector::default(), WorldVector::default()),
                    velocity: (WorldVector::default(), WorldVector::default()),
                    direction: Direction::Right,
                    lives: 3,
                    smart_bombs: 3,
                },
                player_stocks: [crate::PlayerStockSnapshot::new(3, 3); 2],
                scores: ScoreSnapshot {
                    player_one: 0,
                    player_two: 0,
                    high_score: 0,
                    next_bonus: 10_000,
                },
                attract: crate::AttractPresentationSnapshot::for_page_frame(
                    u16::try_from(frame).unwrap_or(u16::MAX),
                ),
                high_score_initials: crate::systems::HighScoreInitialsState::EMPTY,
                high_score_entry: None,
                high_score_submission: None,
                high_score_tables: crate::HighScoreTablesSnapshot::DEFAULT,
                game_over: crate::GameOverSnapshot::NONE,
                world: WorldSnapshot::default(),
            },
            events: GameEvents::new(Vec::new(), sounds),
            scene: RenderScene::empty(frame, SurfaceSize::new(292, 240)),
        }
    }

    #[test]
    fn live_audio_mode_defaults_to_device_backend() {
        assert_eq!(LiveAudioMode::default(), LiveAudioMode::Device);
    }

    #[test]
    fn null_live_audio_mode_keeps_no_device_test_backend() {
        let mut runtime = LiveAudioRuntime::for_mode(LiveAudioMode::Null);

        assert!(runtime.is_enabled());
        assert_eq!(
            runtime.diagnostics().backend_sample_rate_hz,
            Some(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ)
        );
        assert!(runtime.shutdown().worker_error.is_none());
    }

    #[test]
    fn synth_clip_specs_cover_clean_sound_events() {
        assert_eq!(
            synth_clip_for_event(SoundEvent::Startup).frequency_hz,
            220.0
        );
        assert_eq!(
            synth_clip_for_event(SoundEvent::CreditAdded).frequency_hz,
            880.0
        );
        assert_eq!(
            synth_clip_for_event(SoundEvent::GameStarted).frequency_hz,
            660.0
        );
        assert_eq!(
            synth_clip_for_event(SoundEvent::ThrustStarted).frequency_hz,
            140.0
        );
        assert_eq!(
            synth_clip_for_event(SoundEvent::ThrustStopped).frequency_hz,
            95.0
        );
        assert_eq!(
            synth_clip_for_event(SoundEvent::UnmappedSoundCommand { command: 0xE9 }).frequency_hz,
            533.0
        );
    }

    #[test]
    fn synth_mixer_renders_nonzero_audio_and_drains() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::CreditAdded);
        assert_eq!(mixer.voices.len(), 1);

        let mut saw_nonzero_sample = false;
        for _ in 0..LIVE_AUDIO_TEST_SAMPLE_RATE_HZ {
            saw_nonzero_sample |= mixer.next_sample().abs() > f32::EPSILON;
        }

        assert!(saw_nonzero_sample);
        assert_eq!(mixer.voices.len(), 0);
    }

    #[test]
    fn synth_voice_returns_silence_after_duration() {
        let mut voice = SynthVoice::new(1_000, SynthClipSpec::new(440.0, 1, 0.5));

        let _ = voice.next_sample();
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn synth_mixer_bounds_overlapping_voice_count() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        for _ in 0..(super::LIVE_AUDIO_MAX_SYNTH_VOICES + 4) {
            mixer.queue_event(SoundEvent::ThrustStarted);
        }

        assert_eq!(mixer.voices.len(), super::LIVE_AUDIO_MAX_SYNTH_VOICES);

        mixer.clear();
        assert_eq!(mixer.voices.len(), 0);
    }

    #[cfg(coverage)]
    #[test]
    fn device_live_audio_mode_uses_null_backend_under_coverage() {
        let mut runtime = LiveAudioRuntime::for_mode(LiveAudioMode::Device);

        assert!(runtime.is_enabled());
        assert_eq!(
            runtime.diagnostics().backend_sample_rate_hz,
            Some(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ)
        );
        assert!(runtime.shutdown().worker_error.is_none());
    }

    #[test]
    fn live_audio_event_batch_uses_game_frame_sound_surface() {
        let frame =
            game_frame_with_sounds(42, vec![SoundEvent::CreditAdded, SoundEvent::GameStarted]);
        let batch = LiveAudioEventBatch::from_game_frame(&frame).expect("sound event batch");

        assert_eq!(batch.frame, 42);
        assert_eq!(
            batch.events().collect::<Vec<_>>(),
            vec![SoundEvent::CreditAdded, SoundEvent::GameStarted]
        );
        assert_eq!(batch.event_count(), 2);
    }

    #[test]
    fn live_audio_event_batch_ignores_silent_frames() {
        let frame = game_frame_with_sounds(1, Vec::new());

        assert_eq!(LiveAudioEventBatch::from_game_frame(&frame), None);
    }

    #[test]
    fn live_audio_runtime_queues_event_batches_to_backend_in_order() {
        let backend = RecordingAudioBackend::default();
        let recorded = backend.batches.clone();
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 4);

        runtime.submit_event_batch(
            LiveAudioEventBatch::new(10, &[SoundEvent::CreditAdded]).expect("credit event batch"),
        );
        runtime.submit_event_batch(
            LiveAudioEventBatch::new(11, &[SoundEvent::GameStarted]).expect("start event batch"),
        );
        let report = runtime.shutdown();

        let recorded = recorded.lock().expect("recorded audio batches");
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0].frame, 10);
        assert_eq!(
            recorded[0].events().collect::<Vec<_>>(),
            vec![SoundEvent::CreditAdded]
        );
        assert_eq!(recorded[1].frame, 11);
        assert_eq!(
            recorded[1].events().collect::<Vec<_>>(),
            vec![SoundEvent::GameStarted]
        );
        assert_eq!(
            runtime.stats(),
            LiveAudioStats {
                queued_batches: 2,
                queued_events: 2,
                dropped_batches: 0,
                dropped_events: 0,
            }
        );
        assert_eq!(report.worker_error, None);
        assert_eq!(
            report.diagnostics.worker_state,
            LiveAudioWorkerState::Stopped
        );
    }

    #[test]
    fn live_audio_runtime_reports_backend_lifecycle() {
        let backend = RecordingAudioBackend::default();
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 4);

        assert_eq!(
            runtime.diagnostics().backend_sample_rate_hz,
            Some(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ)
        );
        let report = runtime.shutdown();

        assert_eq!(report.worker_error, None);
        assert_eq!(
            report.diagnostics.worker_state,
            LiveAudioWorkerState::Stopped
        );
        assert_eq!(
            report.diagnostics.backend_sample_rate_hz,
            Some(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ)
        );
    }

    #[test]
    fn live_audio_runtime_reports_worker_panic_on_shutdown() {
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(PanickingAudioBackend, 4);

        runtime.submit_event_batch(
            LiveAudioEventBatch::new(12, &[SoundEvent::CreditAdded]).expect("credit event batch"),
        );
        let report = runtime.shutdown();

        assert_eq!(
            report.worker_error,
            Some(LiveAudioWorkerError::Panicked(
                "audio backend failed".to_owned()
            ))
        );
    }

    #[test]
    fn live_audio_runtime_flushes_backend_best_effort() {
        #[derive(Default)]
        struct FlushCountingBackend {
            flushes: Arc<Mutex<u32>>,
        }

        impl LiveAudioBackend for FlushCountingBackend {
            fn handle_event_batch(&mut self, _batch: LiveAudioEventBatch) {}

            fn flush(&mut self) {
                *self.flushes.lock().expect("flush count lock") += 1;
            }
        }

        let backend = FlushCountingBackend::default();
        let flushes = backend.flushes.clone();
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 4);

        runtime.flush();
        runtime.shutdown();

        assert_eq!(*flushes.lock().expect("flush count lock"), 1);
    }

    #[test]
    fn live_audio_runtime_drops_when_bounded_queue_is_full() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let backend = BlockingAudioBackend {
            entered: entered_tx,
            release: release_rx,
        };
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 1);

        runtime.submit_event_batch(
            LiveAudioEventBatch::new(1, &[SoundEvent::CreditAdded]).expect("first event batch"),
        );
        entered_rx.recv().expect("backend entered");
        runtime.submit_event_batch(
            LiveAudioEventBatch::new(2, &[SoundEvent::GameStarted]).expect("queued event batch"),
        );
        runtime.submit_event_batch(
            LiveAudioEventBatch::new(3, &[SoundEvent::ThrustStarted]).expect("dropped event batch"),
        );

        assert_eq!(runtime.stats().dropped_batches, 1);
        assert_eq!(runtime.stats().dropped_events, 1);

        release_tx.send(()).expect("release first audio batch");
        entered_rx.recv().expect("backend entered queued batch");
        release_tx.send(()).expect("release queued audio batch");
        thread::yield_now();
        runtime.shutdown();
    }

    #[test]
    fn disabled_live_audio_runtime_is_a_noop() {
        let mut runtime = LiveAudioRuntime::disabled();

        assert!(!runtime.is_enabled());
        runtime.submit_event_batch(
            LiveAudioEventBatch::new(1, &[SoundEvent::CreditAdded]).expect("credit event batch"),
        );

        assert_eq!(runtime.stats(), LiveAudioStats::default());
        assert_eq!(
            runtime.diagnostics().worker_state,
            LiveAudioWorkerState::Disabled
        );
        assert_eq!(
            runtime.shutdown().diagnostics.worker_state,
            LiveAudioWorkerState::Disabled
        );
    }
}
