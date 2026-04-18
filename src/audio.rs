//! Generates self-contained synthesized sound cues so the app ships without external audio files.

use std::time::Duration;

use rodio::{OutputStream, OutputStreamBuilder, Sink, Source, source::SineWave};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    LogoFanfare,
    AttractHum,
    EnemySweep,
    PlayerShot,
    HumanSaved,
    Explosion,
    HighScoreChime,
}

impl SoundCue {
    pub const ALL: [Self; 7] = [
        Self::LogoFanfare,
        Self::AttractHum,
        Self::EnemySweep,
        Self::PlayerShot,
        Self::HumanSaved,
        Self::Explosion,
        Self::HighScoreChime,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::LogoFanfare => "logo-fanfare",
            Self::AttractHum => "attract-hum",
            Self::EnemySweep => "enemy-sweep",
            Self::PlayerShot => "player-shot",
            Self::HumanSaved => "human-saved",
            Self::Explosion => "explosion-burst",
            Self::HighScoreChime => "high-score-chime",
        }
    }
}

struct AudioOutput {
    stream: OutputStream,
}

pub struct AudioManager {
    output: Option<AudioOutput>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioOutput {
    fn new() -> Option<Self> {
        let mut stream = OutputStreamBuilder::open_default_stream().ok()?;
        stream.log_on_drop(false);
        Some(Self { stream })
    }

    fn new_sink(&self) -> Sink {
        Sink::connect_new(self.stream.mixer())
    }
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            output: AudioOutput::new(),
        }
    }

    pub fn play_cue_blocking(&self, cue: SoundCue) {
        let Some(output) = self.output.as_ref() else {
            return;
        };

        let sink = output.new_sink();
        append_cue(&sink, cue);
        sink.sleep_until_end();
    }

    pub fn play_demo(&self) {
        for cue in SoundCue::ALL {
            println!("audio cue: {}", cue.label());
            self.play_cue_blocking(cue);
        }
    }
}

fn append_cue(sink: &Sink, cue: SoundCue) {
    match cue {
        SoundCue::LogoFanfare => {
            append_tone(sink, 440.0, 110, 0.10);
            append_tone(sink, 660.0, 110, 0.10);
            append_tone(sink, 880.0, 220, 0.12);
        }
        SoundCue::AttractHum => append_layered_tone(sink, 82.5, 110.0, 650, 0.025, 0.018),
        SoundCue::EnemySweep => {
            append_tone(sink, 720.0, 80, 0.08);
            append_tone(sink, 640.0, 80, 0.08);
            append_tone(sink, 560.0, 110, 0.08);
        }
        SoundCue::PlayerShot => {
            append_tone(sink, 980.0, 60, 0.11);
            append_tone(sink, 760.0, 80, 0.08);
        }
        SoundCue::HumanSaved => {
            append_tone(sink, 520.0, 90, 0.08);
            append_tone(sink, 660.0, 90, 0.08);
            append_tone(sink, 880.0, 130, 0.10);
        }
        SoundCue::Explosion => {
            append_tone(sink, 180.0, 120, 0.10);
            append_tone(sink, 120.0, 140, 0.09);
            append_tone(sink, 90.0, 180, 0.08);
        }
        SoundCue::HighScoreChime => {
            append_tone(sink, 523.25, 120, 0.09);
            append_tone(sink, 659.25, 120, 0.09);
            append_tone(sink, 783.99, 240, 0.11);
        }
    }
}

fn append_tone(sink: &Sink, frequency_hz: f32, duration_ms: u64, amplitude: f32) {
    let source = SineWave::new(frequency_hz)
        .take_duration(Duration::from_millis(duration_ms))
        .amplify(amplitude);
    sink.append(source);
}

fn append_layered_tone(
    sink: &Sink,
    low_hz: f32,
    high_hz: f32,
    duration_ms: u64,
    low_amplitude: f32,
    high_amplitude: f32,
) {
    let source = SineWave::new(low_hz)
        .take_duration(Duration::from_millis(duration_ms))
        .amplify(low_amplitude)
        .mix(
            SineWave::new(high_hz)
                .take_duration(Duration::from_millis(duration_ms))
                .amplify(high_amplitude),
        );
    sink.append(source);
}

#[cfg(test)]
mod tests {
    use super::{AudioManager, SoundCue};

    #[test]
    fn every_sound_cue_has_a_stable_label() {
        for cue in SoundCue::ALL {
            assert!(!cue.label().is_empty());
            assert!(cue.label().contains('-'));
        }
    }

    #[test]
    fn silent_audio_manager_handles_every_cue() {
        let audio = AudioManager { output: None };

        for cue in SoundCue::ALL {
            audio.play_cue_blocking(cue);
        }
    }

    #[test]
    fn silent_audio_manager_can_run_the_demo_path() {
        let audio = AudioManager { output: None };
        audio.play_demo();
    }
}
