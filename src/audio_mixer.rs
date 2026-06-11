const CREDIT_ADDED_SOUND_COMMAND: crate::SoundCommand = crate::SoundCommand::new(0xE6);
const GAME_STARTED_SOUND_COMMAND: crate::SoundCommand = crate::SoundCommand::new(0xF5);
const THRUST_STOPPED_SOUND_COMMAND: crate::SoundCommand = crate::SoundCommand::new(0xF0);

#[derive(Debug)]
struct SynthMixer {
    sample_rate_hz: u32,
    board: SoundBoardSynth,
    foreground_voice: Option<SampleVoice>,
    thrust_voice: Option<SampleVoice>,
}

const MIXER_OUTPUT_MIN_SAMPLE: f32 = -0.85;
const MIXER_OUTPUT_MAX_SAMPLE: f32 = 0.85;
const MILLIHZ_PER_HZ: u128 = 1_000;

impl SynthMixer {
    fn new(sample_rate_hz: u32) -> Self {
        Self {
            sample_rate_hz: sample_rate_hz.max(1),
            board: SoundBoardSynth::default(),
            foreground_voice: None,
            thrust_voice: None,
        }
    }

    fn queue_event(&mut self, event: SoundEvent) {
        if event == SoundEvent::ThrustStopped {
            self.thrust_voice = None;
            let actions = sound_actions_for_event(event);
            if actions.is_empty() {
                return;
            }
            let rendered = self.board.render_actions(&actions);
            self.thrust_voice = SampleVoice::new(self.sample_rate_hz, rendered, true);
            return;
        }

        let actions = sound_actions_for_event(event);
        if actions.is_empty() {
            return;
        }
        if actions.contains(&SoundAction::Special(SpecialSound::BackgroundEnd)) {
            self.thrust_voice = None;
        }
        let rendered = self.board.render_actions(&actions);
        if event == SoundEvent::ThrustStarted {
            self.foreground_voice = None;
            self.thrust_voice = SampleVoice::new(self.sample_rate_hz, rendered, true);
        } else {
            self.queue_rendered(rendered);
        }
    }

    fn queue_rendered(&mut self, rendered: RenderedSound) {
        self.foreground_voice = SampleVoice::new(self.sample_rate_hz, rendered, false);
    }

    fn clear(&mut self) {
        self.foreground_voice = None;
        self.thrust_voice = None;
    }

    fn next_sample(&mut self) -> f32 {
        let mut mixed = 0.0;
        if let Some(thrust_voice) = &mut self.thrust_voice
            && let Some(sample) = thrust_voice.next_sample()
        {
            mixed += sample;
        }
        if let Some(foreground_voice) = &mut self.foreground_voice
            && let Some(sample) = foreground_voice.next_sample()
        {
            mixed += sample;
        }
        if self
            .foreground_voice
            .as_ref()
            .is_some_and(|voice| !voice.is_active())
        {
            self.foreground_voice = None;
        }
        mixed.clamp(MIXER_OUTPUT_MIN_SAMPLE, MIXER_OUTPUT_MAX_SAMPLE)
    }
}

pub fn render_sound_event_timeline_to_samples(
    timeline: &[(u64, Vec<SoundEvent>)],
    total_steps: u64,
    step_rate_millihz: u32,
    sample_rate_hz: u32,
) -> Vec<f32> {
    let step_rate_millihz = step_rate_millihz.max(1);
    let sample_rate_hz = sample_rate_hz.max(1);
    let target_samples = sample_count_for_step(total_steps, step_rate_millihz, sample_rate_hz);
    let mut entries = timeline
        .iter()
        .filter(|(step, events)| *step < total_steps && !events.is_empty())
        .map(|(step, events)| (*step, events.as_slice()))
        .collect::<Vec<_>>();
    entries.sort_by_key(|(step, _)| *step);

    let mut entry_index = 0;
    let mut mixer = SynthMixer::new(sample_rate_hz);
    let mut samples = Vec::with_capacity(target_samples);
    for step in 0..total_steps {
        while let Some((event_step, events)) = entries.get(entry_index)
            && *event_step == step
        {
            for event in *events {
                mixer.queue_event(*event);
            }
            entry_index += 1;
        }

        let next_step_samples =
            sample_count_for_step(step + 1, step_rate_millihz, sample_rate_hz);
        while samples.len() < next_step_samples {
            samples.push(mixer.next_sample());
        }
    }

    debug_assert_eq!(samples.len(), target_samples);
    samples
}

fn sample_count_for_step(step: u64, step_rate_millihz: u32, sample_rate_hz: u32) -> usize {
    let numerator = u128::from(step) * u128::from(sample_rate_hz) * MILLIHZ_PER_HZ;
    let denominator = u128::from(step_rate_millihz);
    usize::try_from(numerator.div_ceil(denominator)).unwrap_or(usize::MAX)
}

#[derive(Debug, Clone)]
struct SampleVoice {
    samples: Vec<f32>,
    cursor: f32,
    step: f32,
    looping: bool,
}

impl SampleVoice {
    fn new(output_sample_rate_hz: u32, rendered: RenderedSound, looping: bool) -> Option<Self> {
        if rendered.samples.is_empty() {
            return None;
        }
        let step = rendered.sample_rate_hz.max(1) as f32 / output_sample_rate_hz.max(1) as f32;
        Some(Self {
            samples: rendered.samples,
            cursor: 0.0,
            step,
            looping,
        })
    }

    fn next_sample(&mut self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        if self.cursor >= self.samples.len() as f32 {
            if !self.looping {
                return None;
            }
            self.cursor %= self.samples.len() as f32;
        }
        let sample = self.samples[self.cursor as usize];
        self.cursor += self.step;
        if self.looping && self.cursor >= self.samples.len() as f32 {
            self.cursor %= self.samples.len() as f32;
        }
        Some(sample)
    }

    fn is_active(&self) -> bool {
        self.looping || self.cursor < self.samples.len() as f32
    }
}

fn sound_actions_for_event(event: SoundEvent) -> Vec<SoundAction> {
    match event {
        SoundEvent::Startup => vec![SoundAction::OrganTune(OrganTune::Phantom)],
        SoundEvent::CreditAdded => sound_board_command_actions(CREDIT_ADDED_SOUND_COMMAND),
        SoundEvent::GameStarted => sound_board_command_actions(GAME_STARTED_SOUND_COMMAND),
        SoundEvent::ThrustStarted => vec![SoundAction::Special(SpecialSound::Thrust)],
        SoundEvent::ThrustStopped => sound_board_command_actions(THRUST_STOPPED_SOUND_COMMAND),
        SoundEvent::UnmappedSoundCommand { command } => sound_board_command_actions(command),
    }
}

fn sound_board_command_actions(command: crate::SoundCommand) -> Vec<SoundAction> {
    sound_actions_for_command(command)
}
