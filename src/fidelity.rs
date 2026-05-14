//! Clean equivalence contracts for retiring legacy oracle checks.

use crate::{
    game::{GameEvent, GameFrame, GameSnapshot, SoundEvent},
    renderer::RenderSceneSummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayEquivalenceSignature {
    pub state: GameSnapshot,
    pub gameplay_events: Vec<GameEvent>,
    pub sound_events: Vec<SoundEvent>,
    pub render: RenderSceneSummary,
}

impl GameplayEquivalenceSignature {
    pub fn from_frame(frame: &GameFrame) -> Self {
        Self {
            state: frame.state.clone(),
            gameplay_events: frame.events.gameplay().to_vec(),
            sound_events: frame.events.sounds().to_vec(),
            render: frame.scene.summary(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        game::{
            Direction, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, GameState,
            PlayerSnapshot, ScoreSnapshot, SoundEvent, WorldVector,
        },
        oracle::{GameplayOracle, test_support::ReferenceFrameProbe},
        renderer::{RenderScene, SurfaceSize},
    };

    use super::GameplayEquivalenceSignature;

    #[test]
    fn frame_signature_captures_state_events_sound_and_render_summary() {
        let mut scene = RenderScene::empty(7, SurfaceSize::new(292, 240));
        scene.visual_hash = Some(0x1234_5678);
        let state = GameState {
            frame: 7,
            phase: GamePhase::Playing,
            credits: 1,
            current_player: 1,
            wave: 2,
            player: PlayerSnapshot {
                position: (
                    WorldVector::from_subpixels(0x2000),
                    WorldVector::from_subpixels(0x6000),
                ),
                velocity: (
                    WorldVector::from_subpixels(0x0100),
                    WorldVector::from_subpixels(-0x0200),
                ),
                direction: Direction::Right,
                lives: 3,
                smart_bombs: 2,
            },
            scores: ScoreSnapshot {
                player_one: 1_000,
                player_two: 0,
                high_score: 10_000,
                next_bonus: 10_000,
            },
        };
        let frame = GameFrame {
            state: state.clone(),
            events: GameEvents::new(vec![GameEvent::CreditAdded], vec![SoundEvent::CreditAdded]),
            scene,
        };

        let signature = GameplayEquivalenceSignature::from_frame(&frame);

        assert_eq!(signature.state, state);
        assert_eq!(signature.gameplay_events, [GameEvent::CreditAdded]);
        assert_eq!(signature.sound_events, [SoundEvent::CreditAdded]);
        assert_eq!(signature.render.frame, 7);
        assert_eq!(signature.render.visual_hash, Some(0x1234_5678));
    }

    #[test]
    fn clean_frame_signatures_match_reference_probe_for_start_and_controls() {
        let mut clean = GameplayOracle::new();
        let mut reference = ReferenceFrameProbe::new();
        let mut observed_gameplay = Vec::new();
        let mut observed_sounds = Vec::new();
        let mut saw_playing_render = false;

        for input in credited_start_and_controls_inputs() {
            let clean_frame = clean.step(input);
            let expected_frame = reference.step(input);
            let clean_signature = GameplayEquivalenceSignature::from_frame(&clean_frame);
            let expected_signature = GameplayEquivalenceSignature::from_frame(&expected_frame);

            assert_eq!(clean_signature, expected_signature);
            observed_gameplay.extend_from_slice(&clean_signature.gameplay_events);
            observed_sounds.extend_from_slice(&clean_signature.sound_events);
            saw_playing_render |= clean_signature.state.phase == GamePhase::Playing
                && clean_signature.render.visual_hash.is_some()
                && clean_signature.render.layers.objects == 1;
        }

        assert!(observed_gameplay.contains(&GameEvent::CreditAdded));
        assert!(observed_gameplay.contains(&GameEvent::GameStarted));
        assert!(observed_sounds.contains(&SoundEvent::CreditAdded));
        assert!(observed_sounds.contains(&SoundEvent::GameStarted));
        assert!(saw_playing_render);
    }

    fn credited_start_and_controls_inputs() -> Vec<GameInput> {
        let mut inputs = vec![GameInput {
            coin: true,
            ..GameInput::NONE
        }];
        inputs.extend(std::iter::repeat_n(GameInput::NONE, 96));
        inputs.push(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        inputs.push(GameInput::NONE);
        inputs.extend(std::iter::repeat_n(
            GameInput {
                altitude_up: true,
                reverse: true,
                thrust: true,
                fire: true,
                hyperspace: true,
                ..GameInput::NONE
            },
            16,
        ));
        inputs
    }
}
