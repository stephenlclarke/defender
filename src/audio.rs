//! Live audio command delivery boundary.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc::{self, SyncSender},
};
use std::thread::{self, JoinHandle};

use crate::{
    machine_state::FrameOutput,
    sound::{FRAME_SOUND_COMMAND_CAPACITY, SoundCommand},
};

pub const LIVE_AUDIO_TEST_SAMPLE_RATE_HZ: u32 = 48_000;
pub const LIVE_AUDIO_QUEUE_CAPACITY: usize = 8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LiveAudioMode {
    Disabled,
    #[default]
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveAudioCommandBatch {
    pub frame: u64,
    commands: [Option<SoundCommand>; FRAME_SOUND_COMMAND_CAPACITY],
}

impl LiveAudioCommandBatch {
    pub fn new(frame: u64, commands: &[SoundCommand]) -> Option<Self> {
        if commands.is_empty() {
            return None;
        }

        let mut output = [None; FRAME_SOUND_COMMAND_CAPACITY];
        for (slot, command) in output.iter_mut().zip(commands.iter().copied()) {
            *slot = Some(command);
        }

        Some(Self {
            frame,
            commands: output,
        })
    }

    pub fn from_frame_output(output: &FrameOutput) -> Option<Self> {
        let mut commands = [None; FRAME_SOUND_COMMAND_CAPACITY];
        for (slot, command) in commands.iter_mut().zip(output.sound_commands()) {
            *slot = Some(command);
        }
        if commands.iter().all(Option::is_none) {
            return None;
        }

        Some(Self {
            frame: output.snapshot.frame,
            commands,
        })
    }

    pub fn commands(&self) -> impl Iterator<Item = SoundCommand> + '_ {
        self.commands.iter().filter_map(|command| *command)
    }

    pub fn command_count(&self) -> usize {
        self.commands().count()
    }
}

pub trait LiveAudioBackend: Send + 'static {
    fn sample_rate_hz(&self) -> u32 {
        LIVE_AUDIO_TEST_SAMPLE_RATE_HZ
    }

    fn handle_command_batch(&mut self, batch: LiveAudioCommandBatch);

    fn flush(&mut self) {}

    fn shutdown(&mut self) {}
}

#[derive(Debug, Default)]
pub struct NullLiveAudioBackend;

impl LiveAudioBackend for NullLiveAudioBackend {
    fn handle_command_batch(&mut self, _batch: LiveAudioCommandBatch) {}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveAudioStats {
    pub queued_batches: u64,
    pub queued_commands: u64,
    pub dropped_batches: u64,
    pub dropped_commands: u64,
}

#[derive(Default)]
struct SharedLiveAudioStats {
    queued_batches: AtomicU64,
    queued_commands: AtomicU64,
    dropped_batches: AtomicU64,
    dropped_commands: AtomicU64,
}

impl SharedLiveAudioStats {
    fn queued(&self, command_count: u64) {
        self.queued_batches.fetch_add(1, Ordering::Relaxed);
        self.queued_commands
            .fetch_add(command_count, Ordering::Relaxed);
    }

    fn dropped(&self, command_count: u64) {
        self.dropped_batches.fetch_add(1, Ordering::Relaxed);
        self.dropped_commands
            .fetch_add(command_count, Ordering::Relaxed);
    }

    fn snapshot(&self) -> LiveAudioStats {
        LiveAudioStats {
            queued_batches: self.queued_batches.load(Ordering::Relaxed),
            queued_commands: self.queued_commands.load(Ordering::Relaxed),
            dropped_batches: self.dropped_batches.load(Ordering::Relaxed),
            dropped_commands: self.dropped_commands.load(Ordering::Relaxed),
        }
    }
}

enum LiveAudioMessage {
    CommandBatch(LiveAudioCommandBatch),
    Flush,
    Shutdown,
}

pub struct LiveAudioRuntime {
    sender: Option<SyncSender<LiveAudioMessage>>,
    stats: Arc<SharedLiveAudioStats>,
    worker: Option<JoinHandle<()>>,
}

impl LiveAudioRuntime {
    pub fn disabled() -> Self {
        Self {
            sender: None,
            stats: Arc::default(),
            worker: None,
        }
    }

    pub fn for_mode(mode: LiveAudioMode) -> Self {
        match mode {
            LiveAudioMode::Disabled => Self::disabled(),
            LiveAudioMode::Null => Self::spawn(NullLiveAudioBackend),
        }
    }

    pub fn spawn<B>(backend: B) -> Self
    where
        B: LiveAudioBackend,
    {
        Self::spawn_with_capacity(backend, LIVE_AUDIO_QUEUE_CAPACITY)
    }

    pub(crate) fn spawn_with_capacity<B>(backend: B, queue_capacity: usize) -> Self
    where
        B: LiveAudioBackend,
    {
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let worker = thread::spawn(move || run_live_audio_thread(backend, receiver));
        Self {
            sender: Some(sender),
            stats: Arc::default(),
            worker: Some(worker),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.sender.is_some()
    }

    pub fn submit_frame_output(&self, output: &FrameOutput) {
        if let Some(batch) = LiveAudioCommandBatch::from_frame_output(output) {
            self.submit_command_batch(batch);
        }
    }

    pub fn submit_command_batch(&self, batch: LiveAudioCommandBatch) {
        let command_count = batch.command_count() as u64;
        let Some(sender) = &self.sender else {
            return;
        };

        match sender.try_send(LiveAudioMessage::CommandBatch(batch)) {
            Ok(()) => self.stats.queued(command_count),
            Err(_) => self.stats.dropped(command_count),
        }
    }

    pub fn flush(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.try_send(LiveAudioMessage::Flush);
        }
    }

    pub fn stats(&self) -> LiveAudioStats {
        self.stats.snapshot()
    }

    pub fn shutdown(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(LiveAudioMessage::Shutdown);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for LiveAudioRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run_live_audio_thread<B>(mut backend: B, receiver: mpsc::Receiver<LiveAudioMessage>)
where
    B: LiveAudioBackend,
{
    let _sample_rate_hz = backend.sample_rate_hz();
    while let Ok(message) = receiver.recv() {
        match message {
            LiveAudioMessage::CommandBatch(batch) => backend.handle_command_batch(batch),
            LiveAudioMessage::Flush => backend.flush(),
            LiveAudioMessage::Shutdown => break,
        }
    }
    backend.shutdown();
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    };
    use std::time::Duration;

    use crate::{input::CabinetInput, machine::ArcadeMachine, sound::SoundCommand};

    use super::{
        LIVE_AUDIO_TEST_SAMPLE_RATE_HZ, LiveAudioBackend, LiveAudioCommandBatch, LiveAudioMode,
        LiveAudioRuntime,
    };

    #[derive(Clone, Default)]
    struct RecordingAudioBackend {
        batches: Arc<Mutex<Vec<LiveAudioCommandBatch>>>,
        flushes: Arc<AtomicU64>,
        shutdowns: Arc<AtomicU64>,
    }

    impl LiveAudioBackend for RecordingAudioBackend {
        fn handle_command_batch(&mut self, batch: LiveAudioCommandBatch) {
            self.batches.lock().expect("recording lock").push(batch);
        }

        fn flush(&mut self) {
            self.flushes.fetch_add(1, Ordering::Relaxed);
        }

        fn shutdown(&mut self) {
            self.shutdowns.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct BlockingAudioBackend {
        batches: Arc<Mutex<Vec<LiveAudioCommandBatch>>>,
        started: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
        first_batch_blocked: AtomicBool,
    }

    impl LiveAudioBackend for BlockingAudioBackend {
        fn handle_command_batch(&mut self, batch: LiveAudioCommandBatch) {
            self.batches.lock().expect("blocking lock").push(batch);
            if !self.first_batch_blocked.swap(true, Ordering::Relaxed) {
                self.started.send(()).expect("signal first batch");
                self.release.recv().expect("release first batch");
            }
        }
    }

    #[test]
    fn live_audio_mode_defaults_to_null_backend() {
        assert_eq!(LiveAudioMode::default(), LiveAudioMode::Null);
        let mut runtime = LiveAudioRuntime::for_mode(LiveAudioMode::Null);
        assert!(runtime.is_enabled());
        runtime.shutdown();
    }

    #[test]
    fn live_audio_command_batch_uses_frame_output_sound_command_surface() {
        let mut machine = ArcadeMachine::new_cold_boot_trace();
        let mut output = None;
        for _ in 0..731 {
            output = Some(machine.step(CabinetInput::NONE));
        }
        let output = output.expect("stepped frame");

        let batch = LiveAudioCommandBatch::from_frame_output(&output).expect("sound command batch");

        assert_eq!(batch.frame, output.snapshot.frame);
        assert_eq!(
            batch.commands().collect::<Vec<_>>(),
            vec![SoundCommand::from_main_board_pia_port_b(0xC0)]
        );
    }

    #[test]
    fn live_audio_command_batch_ignores_silent_frames() {
        let output = ArcadeMachine::new().step(CabinetInput::NONE);

        assert_eq!(LiveAudioCommandBatch::from_frame_output(&output), None);
    }

    #[test]
    fn live_audio_runtime_queues_batches_to_backend_in_order() {
        let backend = RecordingAudioBackend::default();
        let recorded = backend.batches.clone();
        let shutdowns = backend.shutdowns.clone();
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 4);
        let first =
            LiveAudioCommandBatch::new(10, &[SoundCommand::from_main_board_pia_port_b(0xE6)])
                .expect("first batch");
        let second =
            LiveAudioCommandBatch::new(11, &[SoundCommand::from_main_board_pia_port_b(0xF5)])
                .expect("second batch");

        runtime.submit_command_batch(first);
        runtime.submit_command_batch(second);
        runtime.shutdown();

        let recorded = recorded.lock().expect("recorded batches");
        assert_eq!(recorded.as_slice(), &[first, second]);
        assert_eq!(
            runtime.stats(),
            super::LiveAudioStats {
                queued_batches: 2,
                queued_commands: 2,
                dropped_batches: 0,
                dropped_commands: 0,
            }
        );
        assert_eq!(shutdowns.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn live_audio_runtime_flushes_backend_best_effort() {
        let backend = RecordingAudioBackend::default();
        let flushes = backend.flushes.clone();
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 4);

        runtime.flush();
        runtime.shutdown();

        assert_eq!(flushes.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn live_audio_runtime_drops_when_bounded_queue_is_full() {
        let (started, first_started) = mpsc::channel();
        let (release_first, release) = mpsc::channel();
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let backend = BlockingAudioBackend {
            batches: recorded.clone(),
            started,
            release,
            first_batch_blocked: AtomicBool::new(false),
        };
        let mut runtime = LiveAudioRuntime::spawn_with_capacity(backend, 1);
        let first =
            LiveAudioCommandBatch::new(1, &[SoundCommand::from_main_board_pia_port_b(0xE6)])
                .expect("first batch");
        let second =
            LiveAudioCommandBatch::new(2, &[SoundCommand::from_main_board_pia_port_b(0xF5)])
                .expect("second batch");
        let third =
            LiveAudioCommandBatch::new(3, &[SoundCommand::from_main_board_pia_port_b(0xE9)])
                .expect("third batch");

        runtime.submit_command_batch(first);
        first_started
            .recv_timeout(Duration::from_secs(2))
            .expect("worker should receive first batch");
        runtime.submit_command_batch(second);
        runtime.submit_command_batch(third);

        assert_eq!(runtime.stats().dropped_batches, 1);
        assert_eq!(runtime.stats().dropped_commands, 1);

        release_first.send(()).expect("release blocked backend");
        runtime.shutdown();
        assert_eq!(recorded.lock().expect("recorded batches").len(), 2);
    }

    #[test]
    fn disabled_live_audio_runtime_is_a_noop() {
        let runtime = LiveAudioRuntime::disabled();
        let batch =
            LiveAudioCommandBatch::new(1, &[SoundCommand::from_main_board_pia_port_b(0xE6)])
                .expect("batch");

        runtime.submit_command_batch(batch);

        assert!(!runtime.is_enabled());
        assert_eq!(runtime.stats(), super::LiveAudioStats::default());
    }

    #[test]
    fn live_audio_backend_default_sample_rate_is_test_fixture_rate() {
        let backend = RecordingAudioBackend::default();

        assert_eq!(backend.sample_rate_hz(), LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);
    }
}
