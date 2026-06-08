#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;

    use crate::{
        audio::{
            LIVE_AUDIO_TEST_SAMPLE_RATE_HZ, LiveAudioBackend, LiveAudioEventBatch, LiveAudioMode,
            LiveAudioRuntime, LiveAudioStats, LiveAudioWorkerError, LiveAudioWorkerState,
            SampleVoice, SynthMixer, render_sound_event_timeline_to_samples,
            sound_actions_for_event,
        },
        game::{
            Direction, GameEvents, GameFrame, GamePhase, GameState, PlayerSnapshot, ScoreSnapshot,
            SoundEvent, WorldSnapshot, WorldVector,
        },
        renderer::{RenderScene, SurfaceSize},
        sound_board::{GWaveSound, OrganTune, RenderedSound, SoundAction, SpecialSound},
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
                post_game_playfield: None,
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
    fn source_sound_actions_cover_clean_sound_events() {
        assert_eq!(
            sound_actions_for_event(SoundEvent::Startup),
            vec![SoundAction::OrganTune(OrganTune::Phantom)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::CreditAdded),
            vec![SoundAction::Special(SpecialSound::Hyper)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::GameStarted),
            vec![SoundAction::GWave(GWaveSound::Vector(10))]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::ThrustStarted),
            vec![SoundAction::Special(SpecialSound::Thrust)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::ThrustStopped),
            vec![SoundAction::Special(SpecialSound::BackgroundNoise)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xEB }),
            vec![SoundAction::Special(SpecialSound::Turbo)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xEA }),
            vec![SoundAction::Special(SpecialSound::Materialize)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xE6 }),
            vec![SoundAction::Special(SpecialSound::Hyper)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xEE }),
            vec![SoundAction::Special(SpecialSound::Lightning)]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xF7 }),
            vec![SoundAction::GWave(GWaveSound::Vector(8))]
        );
        assert_eq!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xFC }),
            vec![SoundAction::GWave(GWaveSound::Vector(3))]
        );
        assert!(
            sound_actions_for_event(SoundEvent::UnmappedSoundCommand { command: 0xFF }).is_empty()
        );
    }

    #[test]
    fn synth_mixer_renders_source_audio_and_drains() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::UnmappedSoundCommand { command: 0xE8 });
        assert!(mixer.foreground_voice.is_some());

        let mut saw_nonzero_sample = false;
        for _ in 0..LIVE_AUDIO_TEST_SAMPLE_RATE_HZ * 4 {
            saw_nonzero_sample |= mixer.next_sample().abs() > f32::EPSILON;
        }

        assert!(saw_nonzero_sample);
        assert!(mixer.foreground_voice.is_none());
    }

    #[test]
    fn offline_timeline_renderer_places_events_on_frame_boundaries() {
        let samples = render_sound_event_timeline_to_samples(
            &[(1, vec![SoundEvent::UnmappedSoundCommand { command: 0xE8 }])],
            2,
            1_000,
            1_000,
        );

        assert_eq!(samples.len(), 2_000);
        assert!(
            samples[..1_000]
                .iter()
                .all(|sample| sample.abs() <= f32::EPSILON)
        );
        assert!(
            samples[1_000..]
                .iter()
                .any(|sample| sample.abs() > f32::EPSILON)
        );
    }

    #[test]
    fn sample_voice_returns_none_after_duration() {
        let rendered = RenderedSound {
            sample_rate_hz: 1_000,
            samples: vec![0.5],
        };
        let mut voice = SampleVoice::new(1_000, rendered, false).expect("sample voice");

        assert_eq!(voice.next_sample(), Some(0.5));
        assert!(!voice.is_active());
        assert_eq!(voice.next_sample(), None);
    }

    #[test]
    fn synth_mixer_thrust_stop_returns_to_background_noise() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::ThrustStarted);
        assert!(mixer.thrust_voice.is_some());

        let mut saw_nonzero_sample = false;
        for _ in 0..LIVE_AUDIO_TEST_SAMPLE_RATE_HZ {
            saw_nonzero_sample |= mixer.next_sample().abs() > f32::EPSILON;
        }
        assert!(saw_nonzero_sample);
        assert!(mixer.thrust_voice.is_some());

        mixer.queue_event(SoundEvent::ThrustStopped);
        assert!(mixer.thrust_voice.is_some());
        let mut saw_background_sample = false;
        for _ in 0..LIVE_AUDIO_TEST_SAMPLE_RATE_HZ {
            saw_background_sample |= mixer.next_sample().abs() > f32::EPSILON;
        }
        assert!(saw_background_sample);
    }

    #[test]
    fn synth_mixer_background_end_stops_thrust_voice() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::ThrustStarted);
        assert!(mixer.thrust_voice.is_some());

        mixer.queue_event(SoundEvent::UnmappedSoundCommand { command: 0xEC });

        assert!(mixer.thrust_voice.is_none());
    }

    #[test]
    fn synth_mixer_interrupts_foreground_commands_like_the_source_sound_board() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::UnmappedSoundCommand { command: 0xEA });
        assert!(mixer.foreground_voice.is_some());
        mixer.queue_event(SoundEvent::UnmappedSoundCommand { command: 0xEB });

        assert!(mixer.foreground_voice.is_some());

        mixer.clear();
        assert!(mixer.foreground_voice.is_none());
        assert!(mixer.thrust_voice.is_none());
    }

    #[test]
    fn synth_mixer_thrust_start_preempts_foreground_sound() {
        let mut mixer = SynthMixer::new(LIVE_AUDIO_TEST_SAMPLE_RATE_HZ);

        mixer.queue_event(SoundEvent::UnmappedSoundCommand { command: 0xEA });
        assert!(mixer.foreground_voice.is_some());

        mixer.queue_event(SoundEvent::ThrustStarted);

        assert!(mixer.foreground_voice.is_none());
        assert!(mixer.thrust_voice.is_some());
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
