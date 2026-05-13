//! Temporary gameplay oracle backed by the current implementation.
//!
//! This module is the explicit boundary where the clean rewrite can compare
//! against the existing behavior without letting converted implementation names
//! leak into new production contracts.

use crate::{
    input::CabinetInput,
    machine::ArcadeMachine,
    machine_state::{FrameOutput, MachineEvent, MachineSnapshot},
    red_label::Facing,
};

use super::game::{
    Direction, GameEvent, GameEvents, GameFrame, GameInput, GamePhase, GameState, PlayerSnapshot,
    ScoreSnapshot, SoundEvent, WorldVector,
};
use super::renderer::{Color, RenderLayer, RenderScene, SceneSprite, SpriteId, SurfaceSize};
use super::systems::GameSimulation;

#[derive(Debug)]
pub struct GameplayOracle {
    machine: ArcadeMachine,
}

impl GameplayOracle {
    pub fn new() -> Self {
        Self {
            machine: ArcadeMachine::new(),
        }
    }

    pub fn snapshot(&self) -> GameState {
        adapt_snapshot(self.machine.snapshot())
    }

    pub fn step(&mut self, input: GameInput) -> GameFrame {
        adapt_frame_output(self.machine.step(to_cabinet_input(input)))
    }
}

impl Default for GameplayOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl GameSimulation for GameplayOracle {
    fn state(&self) -> GameState {
        self.snapshot()
    }

    fn step(&mut self, input: GameInput) -> GameFrame {
        GameplayOracle::step(self, input)
    }
}

fn to_cabinet_input(input: GameInput) -> CabinetInput {
    CabinetInput {
        coin: input.coin,
        coin_two: input.coin_two,
        coin_three: input.coin_three,
        start_one: input.start_one,
        start_two: input.start_two,
        altitude_up: input.altitude_up,
        altitude_down: input.altitude_down,
        reverse: input.reverse,
        thrust: input.thrust,
        fire: input.fire,
        smart_bomb: input.smart_bomb,
        hyperspace: input.hyperspace,
        auto_up_manual_down: input.service_auto_up,
        service_advance: input.service_advance,
        high_score_reset: input.high_score_reset,
        tilt: input.tilt,
    }
}

fn adapt_frame_output(output: FrameOutput) -> GameFrame {
    let state = adapt_snapshot(output.snapshot);
    let scene = adapt_scene(&state, output.video_crc32);

    GameFrame {
        state,
        events: GameEvents::new(
            output.events().map(adapt_event).collect(),
            output
                .sound_commands()
                .map(|command| SoundEvent {
                    command: command.raw(),
                })
                .collect(),
        ),
        scene,
    }
}

fn adapt_snapshot(snapshot: MachineSnapshot) -> GameState {
    GameState {
        frame: snapshot.frame,
        phase: adapt_phase(snapshot.phase),
        credits: snapshot.credits,
        current_player: snapshot.current_player,
        wave: snapshot.wave,
        player: PlayerSnapshot {
            position: (
                WorldVector::from_subpixels(snapshot.player.x.0),
                WorldVector::from_subpixels(snapshot.player.y.0),
            ),
            velocity: (
                WorldVector::from_subpixels(snapshot.player.xv.0),
                WorldVector::from_subpixels(snapshot.player.yv.0),
            ),
            direction: adapt_direction(snapshot.player.facing),
            lives: snapshot.player.lives,
            smart_bombs: snapshot.player.smart_bombs,
        },
        scores: ScoreSnapshot {
            player_one: snapshot.scores.player_one,
            player_two: snapshot.scores.player_two,
            high_score: snapshot.scores.high_score,
            next_bonus: snapshot.scores.next_bonus,
        },
    }
}

fn adapt_scene(state: &GameState, visual_hash: Option<u32>) -> RenderScene {
    let (width, height) = crate::video::native_visible_size();
    let mut scene = RenderScene::empty(
        state.frame,
        SurfaceSize::new(u32::from(width), u32::from(height)),
    );
    scene.visual_hash = visual_hash;
    scene.push_sprite(SceneSprite {
        sprite: SpriteId::SCORE_TEXT,
        layer: RenderLayer::Hud,
        position: [0.0, 0.0],
        size: [96.0, 8.0],
        tint: Color::WHITE,
    });

    if state.phase == GamePhase::Playing {
        scene.push_sprite(SceneSprite {
            sprite: SpriteId::PLAYER_SHIP,
            layer: RenderLayer::Objects,
            position: [
                world_vector_pixels(state.player.position.0),
                world_vector_pixels(state.player.position.1),
            ],
            size: [16.0, 8.0],
            tint: Color::WHITE,
        });
    }

    scene
}

fn world_vector_pixels(vector: WorldVector) -> f32 {
    vector.subpixels() as f32 / WorldVector::SUBPIXELS_PER_PIXEL as f32
}

fn adapt_phase(phase: crate::machine_state::GamePhase) -> GamePhase {
    match phase {
        crate::machine_state::GamePhase::Attract => GamePhase::Attract,
        crate::machine_state::GamePhase::Playing => GamePhase::Playing,
        crate::machine_state::GamePhase::GameOver => GamePhase::GameOver,
        crate::machine_state::GamePhase::HighScoreEntry => GamePhase::HighScoreEntry,
    }
}

fn adapt_direction(direction: Facing) -> Direction {
    match direction {
        Facing::Left => Direction::Left,
        Facing::Right => Direction::Right,
    }
}

fn adapt_event(event: MachineEvent) -> GameEvent {
    match event {
        MachineEvent::CreditAdded => GameEvent::CreditAdded,
        MachineEvent::GameStarted => GameEvent::GameStarted,
        MachineEvent::DiagnosticsSelected => GameEvent::DiagnosticsSelected,
        MachineEvent::AuditsSelected => GameEvent::AuditsSelected,
        MachineEvent::HighScoreReset => GameEvent::HighScoreReset,
        MachineEvent::ReversePressed => GameEvent::ReversePressed,
        MachineEvent::FirePressed => GameEvent::FirePressed,
        MachineEvent::SmartBombPressed => GameEvent::SmartBombPressed,
        MachineEvent::HyperspacePressed => GameEvent::HyperspacePressed,
        MachineEvent::BonusAwarded => GameEvent::BonusAwarded,
        MachineEvent::HighScoreEntryStarted => GameEvent::HighScoreEntryStarted,
        MachineEvent::HighScoreInitialAccepted => GameEvent::HighScoreInitialAccepted,
        MachineEvent::HighScoreSubmitted => GameEvent::HighScoreSubmitted,
    }
}

#[cfg(test)]
mod tests {
    use crate::{input::CabinetInput, machine::ArcadeMachine};

    use super::{
        GameEvent, GameInput, GamePhase, GameplayOracle, adapt_event, adapt_scene, adapt_snapshot,
        to_cabinet_input,
    };
    use crate::systems::{GameSimulation, advance_one_frame};

    #[test]
    fn oracle_starts_from_clean_attract_snapshot() {
        let oracle = GameplayOracle::new();
        let snapshot = oracle.snapshot();

        assert_eq!(snapshot.frame, 0);
        assert_eq!(snapshot.phase, GamePhase::Attract);
        assert_eq!(snapshot.current_player, 1);
    }

    #[test]
    fn oracle_steps_through_clean_frame_contract() {
        let mut oracle = GameplayOracle::new();
        let frame = oracle.step(GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(frame.state.phase, GamePhase::Attract);
        assert_eq!(frame.scene.summary().frame, 1);
    }

    #[test]
    fn oracle_maps_clean_input_to_cabinet_input() {
        let input = GameInput {
            coin: true,
            start_one: true,
            fire: true,
            service_auto_up: true,
            ..GameInput::NONE
        };

        let cabinet = to_cabinet_input(input);

        assert_eq!(
            cabinet.bits(),
            CabinetInput {
                coin: true,
                start_one: true,
                fire: true,
                auto_up_manual_down: true,
                ..CabinetInput::NONE
            }
            .bits()
        );
    }

    #[test]
    fn oracle_implements_clean_simulation_trait() {
        let mut oracle = GameplayOracle::new();

        let frame = advance_one_frame(&mut oracle, GameInput::NONE);

        assert_eq!(frame.state.frame, 1);
        assert_eq!(GameSimulation::state(&oracle).frame, 1);
    }

    #[test]
    fn clean_fixture_matches_accepted_oracle_events_and_scene_summaries() {
        let mut clean = GameplayOracle::new();
        let mut legacy = ArcadeMachine::new();
        let mut observed_events = Vec::new();
        let mut saw_playing_scene = false;

        for input in credited_start_and_controls_inputs() {
            let clean_frame = clean.step(input);
            let legacy_output = legacy.step(to_cabinet_input(input));
            let expected_state = adapt_snapshot(legacy_output.snapshot);
            let expected_gameplay_events =
                legacy_output.events().map(adapt_event).collect::<Vec<_>>();
            let expected_sounds = legacy_output
                .sound_commands()
                .map(|command| super::SoundEvent {
                    command: command.raw(),
                })
                .collect::<Vec<_>>();
            let expected_scene_summary =
                adapt_scene(&expected_state, legacy_output.video_crc32).summary();

            assert_eq!(clean_frame.state, expected_state);
            assert_eq!(
                clean_frame.events.gameplay(),
                expected_gameplay_events.as_slice()
            );
            assert_eq!(clean_frame.events.sounds(), expected_sounds.as_slice());
            assert_eq!(clean_frame.scene.summary(), expected_scene_summary);

            observed_events.extend_from_slice(clean_frame.events.gameplay());
            let summary = clean_frame.scene.summary();
            saw_playing_scene |= clean_frame.state.phase == GamePhase::Playing
                && summary.visual_hash.is_some()
                && summary.layers.objects == 1;
        }

        assert!(observed_events.contains(&GameEvent::CreditAdded));
        assert!(observed_events.contains(&GameEvent::GameStarted));
        assert!(saw_playing_scene);
    }

    fn credited_start_and_controls_inputs() -> Vec<GameInput> {
        let mut inputs = vec![GameInput {
            coin: true,
            ..GameInput::NONE
        }];
        for _ in 0..16 {
            inputs.push(GameInput::NONE);
        }
        inputs.push(GameInput {
            start_one: true,
            ..GameInput::NONE
        });
        for _ in 0..16 {
            inputs.push(GameInput {
                altitude_up: true,
                reverse: true,
                thrust: true,
                fire: true,
                hyperspace: true,
                ..GameInput::NONE
            });
        }
        inputs
    }
}
