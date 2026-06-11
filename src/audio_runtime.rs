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

    pub fn submit_game_step(&self, snapshot: &GameStepSnapshot) {
        if let Some(batch) = LiveAudioEventBatch::from_game_step(snapshot) {
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
