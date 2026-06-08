use std::any::Any;
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
use crate::sound_board::{
    OrganTune, RenderedSound, SoundAction, SoundBoardSynth, SpecialSound, sound_actions_for_command,
};

pub const LIVE_AUDIO_TEST_SAMPLE_RATE_HZ: u32 = 48_000;
pub const LIVE_AUDIO_QUEUE_CAPACITY: usize = 8;
pub const LIVE_AUDIO_EVENT_CAPACITY: usize = 8;

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
