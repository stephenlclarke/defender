//! Embeds the Defender cue files under `assets/sounds/` and handles playback.

use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};

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

    pub const fn duration_ms(self) -> u64 {
        match self {
            Self::LogoFanfare => 440,
            Self::AttractHum => 650,
            Self::EnemySweep => 270,
            Self::PlayerShot => 140,
            Self::HumanSaved => 310,
            Self::Explosion => 440,
            Self::HighScoreChime => 480,
        }
    }

    const fn bytes(self) -> &'static [u8] {
        match self {
            Self::LogoFanfare => include_bytes!("../assets/sounds/logo-fanfare.wav"),
            Self::AttractHum => include_bytes!("../assets/sounds/attract-hum.wav"),
            Self::EnemySweep => include_bytes!("../assets/sounds/enemy-sweep.wav"),
            Self::PlayerShot => include_bytes!("../assets/sounds/player-shot.wav"),
            Self::HumanSaved => include_bytes!("../assets/sounds/human-saved.wav"),
            Self::Explosion => include_bytes!("../assets/sounds/explosion-burst.wav"),
            Self::HighScoreChime => include_bytes!("../assets/sounds/high-score-chime.wav"),
        }
    }
}

struct AudioOutput {
    stream: OutputStream,
}

pub struct AudioManager {
    output: Option<AudioOutput>,
}

type SoundDecoder = Decoder<Cursor<&'static [u8]>>;

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
        let Some(source) = sound_decoder(cue) else {
            return;
        };

        let sink = output.new_sink();
        sink.append(source);
        sink.sleep_until_end();
    }

    pub fn play_demo(&self) {
        for cue in SoundCue::ALL {
            println!("audio cue: {}", cue.label());
            self.play_cue_blocking(cue);
        }
    }
}

fn sound_decoder(cue: SoundCue) -> Option<SoundDecoder> {
    Decoder::new(Cursor::new(cue.bytes())).ok()
}

#[cfg(test)]
mod tests {
    use super::{AudioManager, SoundCue, sound_decoder};

    #[test]
    fn every_sound_cue_has_a_stable_label() {
        for cue in SoundCue::ALL {
            assert!(!cue.label().is_empty());
            assert!(cue.label().contains('-'));
            assert!(cue.duration_ms() > 0);
            assert!(!cue.bytes().is_empty());
        }
    }

    #[test]
    fn embedded_sound_files_decode() {
        for cue in SoundCue::ALL {
            assert!(sound_decoder(cue).is_some(), "embedded cue should decode");
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
