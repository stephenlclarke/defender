//! Synthesizes Defender audio from routines translated out of `VSNDRM1.SRC`.

use rodio::{OutputStream, OutputStreamBuilder, Sink, buffer::SamplesBuffer};

use crate::audio_rom::{
    GWaveSound, OrganNote, OrganTune, RomAction, RomSoundBoard, SpecialRoutine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    LogoFanfare,
    AttractHum,
    EnemySweep,
    Hyperspace,
    PlayerShot,
    HumanSaved,
    Explosion,
    HighScoreChime,
}

impl SoundCue {
    pub const ALL: [Self; 8] = [
        Self::LogoFanfare,
        Self::AttractHum,
        Self::EnemySweep,
        Self::Hyperspace,
        Self::PlayerShot,
        Self::HumanSaved,
        Self::Explosion,
        Self::HighScoreChime,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::LogoFanfare => "organ-phantom",
            Self::AttractHum => "thrust-noise",
            Self::EnemySweep => "radio-sweep",
            Self::Hyperspace => "hyper-square",
            Self::PlayerShot => "cannon-burst",
            Self::HumanSaved => "bonus-gwave",
            Self::Explosion => "scream-burst",
            Self::HighScoreChime => "organ-note",
        }
    }

    pub const fn duration_ms(self) -> u64 {
        match self {
            Self::LogoFanfare => 1_150,
            Self::AttractHum => 220,
            Self::EnemySweep => 320,
            Self::Hyperspace => 280,
            Self::PlayerShot => 190,
            Self::HumanSaved => 260,
            Self::Explosion => 420,
            Self::HighScoreChime => 180,
        }
    }

    fn program(self) -> &'static [RomAction] {
        match self {
            Self::LogoFanfare => &[RomAction::OrganTune(OrganTune::Phantom)],
            Self::AttractHum => &[RomAction::Special(SpecialRoutine::Thrust)],
            Self::EnemySweep => &[RomAction::Special(SpecialRoutine::Radio)],
            Self::Hyperspace => &[RomAction::Special(SpecialRoutine::Hyper)],
            Self::PlayerShot => &[RomAction::Special(SpecialRoutine::Cannon)],
            Self::HumanSaved => &[RomAction::GWave(GWaveSound::Bonus)],
            Self::Explosion => &[RomAction::Special(SpecialRoutine::Scream)],
            Self::HighScoreChime => &[RomAction::OrganNote(OrganNote::Cs2)],
        }
    }
}

struct AudioOutput {
    stream: OutputStream,
}

pub struct AudioManager {
    output: Option<AudioOutput>,
    board: RomSoundBoard,
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
            board: RomSoundBoard::default(),
        }
    }

    pub fn play_cue(&mut self, cue: SoundCue) {
        let rendered = self.board.render_actions(cue.program());
        if rendered.samples.is_empty() {
            return;
        }

        let Some(output) = self.output.as_ref() else {
            return;
        };

        let sink = output.new_sink();
        sink.append(SamplesBuffer::new(
            1,
            rendered.sample_rate,
            rendered.samples,
        ));
        sink.detach();
    }

    pub fn play_cue_blocking(&mut self, cue: SoundCue) {
        let rendered = self.board.render_actions(cue.program());
        if rendered.samples.is_empty() {
            return;
        }

        let Some(output) = self.output.as_ref() else {
            return;
        };

        let sink = output.new_sink();
        sink.append(SamplesBuffer::new(
            1,
            rendered.sample_rate,
            rendered.samples,
        ));
        sink.sleep_until_end();
    }

    pub fn play_demo(&mut self) {
        for cue in SoundCue::ALL {
            println!("audio cue: {}", cue.label());
            self.play_cue_blocking(cue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioManager, SoundCue};

    #[test]
    fn every_sound_cue_has_a_stable_label() {
        for cue in SoundCue::ALL {
            assert!(!cue.label().is_empty());
            assert!(cue.label().contains('-'));
            assert!(cue.duration_ms() > 0);
            assert!(!cue.program().is_empty());
        }
    }

    #[test]
    fn rom_programs_render_audio_buffers() {
        let mut audio = AudioManager {
            output: None,
            board: Default::default(),
        };

        for cue in SoundCue::ALL {
            let rendered = audio.board.render_actions(cue.program());
            assert!(
                !rendered.samples.is_empty(),
                "{cue:?} should render samples"
            );
            assert!(
                rendered.samples.iter().any(|sample| sample.abs() > 0.01),
                "{cue:?} should render non-silent audio"
            );
        }
    }

    #[test]
    fn silent_audio_manager_handles_every_cue() {
        let mut audio = AudioManager {
            output: None,
            board: Default::default(),
        };

        for cue in SoundCue::ALL {
            audio.play_cue(cue);
            audio.play_cue_blocking(cue);
        }
    }

    #[test]
    fn silent_audio_manager_can_run_the_demo_path() {
        let mut audio = AudioManager {
            output: None,
            board: Default::default(),
        };
        audio.play_demo();
    }
}
