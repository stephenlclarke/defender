#[cfg(test)]
mod tests {
    use super::{
        GWaveSound, LIGHTNING_MIN_SAMPLES, OrganNote, OrganTune, SAMPLE_RATE_HZ, SoundAction,
        SoundBoardSynth, SpecialSound, VariSound, gwave_vector, sound_actions_for_command,
        adjusted_pattern_range, decay_waveform,
    };

    #[test]
    fn special_and_gwave_sounds_generate_samples() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[
            SoundAction::Special(SpecialSound::Radio),
            SoundAction::Special(SpecialSound::Hyper),
            SoundAction::Special(SpecialSound::Cannon),
            SoundAction::Special(SpecialSound::Turbo),
            SoundAction::Special(SpecialSound::Lightning),
            SoundAction::Special(SpecialSound::Materialize),
            SoundAction::GWave(GWaveSound::Vector(10)),
            SoundAction::Vari(VariSound::Falling),
        ]);

        assert_eq!(rendered.sample_rate_hz, SAMPLE_RATE_HZ);
        assert!(!rendered.samples.is_empty());
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn quasar_vari_uses_arcade_restart_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Vari(VariSound::Quasar)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 2);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn waveform_decay_uses_wrapping_sample_subtraction() {
        let base_waveform = [0, 16, 127, 255];
        let mut waveform = base_waveform;

        decay_waveform(&mut waveform, &base_waveform, 17);

        assert_eq!(waveform, [0, 255, 8, 0]);
    }

    #[test]
    fn frequency_delta_keeps_contiguous_valid_period_range() {
        assert_eq!(adjusted_pattern_range(&[1, 2, 3, 4], -1), Some((1, 4)));
        assert_eq!(
            adjusted_pattern_range(&[252, 253, 254, 255], 2),
            Some((0, 2))
        );
        assert_eq!(adjusted_pattern_range(&[1, 2], -2), None);
    }

    #[test]
    fn tonal_hit_gwave_vectors_use_arcade_period_density() {
        let bomber_hit = gwave_vector(GWaveSound::Vector(1)).expect("bomber hit GWAVE vector");
        let pod_hit = gwave_vector(GWaveSound::Vector(5)).expect("pod hit GWAVE vector");

        assert_eq!(bomber_hit.period_divisor, 16);
        assert_eq!(pod_hit.period_divisor, 16);
    }

    #[test]
    fn cannon_generates_player_death_length_noise_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Cannon)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn lightning_generates_human_loss_length_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Lightning)]);

        assert!(rendered.samples.len() >= LIGHTNING_MIN_SAMPLES);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn materialize_uses_arcade_appear_sweep_cadence() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Materialize)]);

        assert!(
            (SAMPLE_RATE_HZ as usize..SAMPLE_RATE_HZ as usize * 6 / 5)
                .contains(&rendered.samples.len())
        );
        let sign_transitions = rendered
            .samples
            .windows(2)
            .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
            .count();
        assert!((600..900).contains(&sign_transitions));
    }

    #[test]
    fn organ_sounds_generate_non_silent_samples() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[
            SoundAction::OrganTune(OrganTune::Phantom),
            SoundAction::OrganNote(OrganNote::Cs2),
        ]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 4);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn scream_generates_a_decaying_tail() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Scream)]);
        let midpoint = rendered.samples.len() / 2;
        let first_half_peak = rendered.samples[..midpoint]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f32::max);
        let second_half_peak = rendered.samples[midpoint..]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0, f32::max);

        assert!(first_half_peak >= second_half_peak);
    }

    #[test]
    fn hyper_generates_a_toggling_square_wave() {
        let mut board = SoundBoardSynth::default();
        let rendered = board.render_actions(&[SoundAction::Special(SpecialSound::Hyper)]);

        assert!(rendered.samples.len() > SAMPLE_RATE_HZ as usize / 2);
        let transitions = rendered
            .samples
            .windows(2)
            .filter(|pair| (pair[0] - pair[1]).abs() > 0.5)
            .count();
        assert!(transitions > 32);
    }

    #[test]
    fn sound_board_commands_decode_to_actions() {
        assert_eq!(
            sound_actions_for_command(crate::SoundCommand::new(0xF5)),
            vec![SoundAction::GWave(GWaveSound::Vector(10))]
        );
        assert_eq!(
            sound_actions_for_command(crate::SoundCommand::new(0xEB)),
            vec![SoundAction::Special(SpecialSound::Turbo)]
        );
        assert_eq!(
            sound_actions_for_command(crate::SoundCommand::new(0xE8)),
            vec![SoundAction::Special(SpecialSound::Cannon)]
        );
        assert_eq!(
            sound_actions_for_command(crate::SoundCommand::new(0xEE)),
            vec![SoundAction::Special(SpecialSound::Lightning)]
        );
        assert_eq!(
            sound_actions_for_command(crate::SoundCommand::new(0xE5)),
            vec![SoundAction::Special(SpecialSound::Scream)]
        );
        assert!(sound_actions_for_command(crate::SoundCommand::new(0xFF)).is_empty());
    }
}
