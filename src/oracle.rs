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
    use crate::{
        input::{
            CabinetInput, DEFENDER_IN0_ALTITUDE_DOWN, DEFENDER_IN0_FIRE, DEFENDER_IN0_HYPERSPACE,
            DEFENDER_IN0_REVERSE, DEFENDER_IN0_SMART_BOMB, DEFENDER_IN0_THRUST,
            DEFENDER_IN1_ALTITUDE_UP,
        },
        machine::{
            ArcadeMachine, RED_LABEL_SYSTEM_PROCESS_TYPE, RedLabelLaserDirection,
            RedLabelLaserFire, RedLabelPlayerMotion,
        },
        red_label::{Facing, Fixed16},
    };

    use super::{
        Direction, GameEvent, GameInput, GamePhase, GameplayOracle, WorldVector, adapt_event,
        adapt_scene, adapt_snapshot, to_cabinet_input,
    };
    use crate::systems::{
        GameSimulation, PlayerActionTriggers, PlayerControlFrame, PlayerControlIntent,
        PlayerControlSystem, PlayerMotionFrame, PlayerMotionState, PlayerMotionSystem,
        ProjectileLaunchOutcome, ProjectileState, ProjectileSystem, ScreenPosition,
        VerticalControl, advance_one_frame,
    };

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
    fn clean_player_control_history_matches_oracle_switch_scan() {
        let mut clean = PlayerControlSystem::new();
        let mut oracle = ArcadeMachine::new();

        for input in player_control_inputs() {
            let frame = clean.step(input);
            let scan = oracle
                .red_label_scan_translated_player_switches(
                    to_cabinet_input(input).defender_input_ports(),
                )
                .expect("oracle player switch scan should stay valid");

            assert_eq!(trigger_bits(frame), scan.triggered_bits);
            assert_eq!(
                frame.intent.vertical,
                vertical_control_from_ports(scan.current_pia21, scan.current_pia31)
            );
        }
    }

    #[test]
    fn clean_player_motion_matches_oracle_motion_update() {
        for (state, input) in [
            (
                player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
                GameInput {
                    thrust: true,
                    ..GameInput::NONE
                },
            ),
            (
                player_motion_state(0x2000, 0x8000, 0, 0, Direction::Right, 0),
                GameInput {
                    altitude_up: true,
                    ..GameInput::NONE
                },
            ),
        ] {
            let mut oracle = ArcadeMachine::new();
            restore_oracle_motion(&mut oracle, state);
            let ports = to_cabinet_input(input).defender_input_ports();
            oracle.red_label_write_ram_byte_for_test(ORACLE_PLAYER_INPUT_IN0, ports.in0);
            oracle.red_label_write_ram_byte_for_test(ORACLE_PLAYER_INPUT_IN1, ports.in1);

            let clean = PlayerMotionSystem::step(state, PlayerControlIntent::from_input(input));
            let accepted = oracle
                .red_label_update_player_motion_from_pia()
                .expect("oracle player motion update should stay valid");

            assert_motion_update_matches(clean, accepted);
        }
    }

    #[test]
    fn clean_projectile_launch_matches_oracle_fire_entry() {
        for (direction, direction_word, expected_direction) in [
            (Direction::Right, 0x0000, RedLabelLaserDirection::Right),
            (Direction::Left, 0x8000, RedLabelLaserDirection::Left),
        ] {
            let mut oracle = ArcadeMachine::new();
            schedule_oracle_fire_process(&mut oracle);
            write_oracle_word(
                &mut oracle,
                ORACLE_PLAYER_SCREEN_POSITION,
                ScreenPosition::new(0x40, 0x50).packed(),
            );
            write_oracle_word(&mut oracle, ORACLE_NEXT_PLAYER_DIRECTION, direction_word);

            let clean = ProjectileSystem::try_launch(
                ProjectileState::new(0),
                ScreenPosition::new(0x40, 0x50),
                direction,
            );
            let accepted = oracle
                .red_label_start_laser_fire_current_process()
                .expect("oracle fire entry should stay valid");

            assert_projectile_launch_matches(clean, accepted, expected_direction);
        }

        let mut capped = ArcadeMachine::new();
        schedule_oracle_fire_process(&mut capped);
        capped.red_label_write_ram_byte_for_test(
            ORACLE_ACTIVE_PROJECTILES,
            ProjectileSystem::MAX_ACTIVE_PROJECTILES,
        );
        let clean = ProjectileSystem::try_launch(
            ProjectileState::new(ProjectileSystem::MAX_ACTIVE_PROJECTILES),
            ScreenPosition::new(0x40, 0x50),
            Direction::Right,
        );
        let accepted = capped
            .red_label_start_laser_fire_current_process()
            .expect("oracle capped fire entry should stay valid");

        assert!(matches!(
            clean,
            ProjectileLaunchOutcome::CapacityReached {
                state: ProjectileState {
                    active_projectiles: 4,
                },
            }
        ));
        assert!(matches!(accepted, RedLabelLaserFire::Capped(_)));
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

    fn player_control_inputs() -> [GameInput; 8] {
        [
            GameInput {
                thrust: true,
                altitude_up: true,
                ..GameInput::NONE
            },
            GameInput {
                fire: true,
                thrust: true,
                ..GameInput::NONE
            },
            GameInput::NONE,
            GameInput {
                fire: true,
                ..GameInput::NONE
            },
            GameInput::NONE,
            GameInput::NONE,
            GameInput {
                fire: true,
                smart_bomb: true,
                hyperspace: true,
                reverse: true,
                altitude_down: true,
                ..GameInput::NONE
            },
            GameInput {
                altitude_up: true,
                altitude_down: true,
                ..GameInput::NONE
            },
        ]
    }

    fn trigger_bits(frame: PlayerControlFrame) -> u8 {
        let triggers = frame.triggers;
        let mut bits = 0;
        if triggers.fire {
            bits |= DEFENDER_IN0_FIRE;
        }
        if triggers.thrust {
            bits |= DEFENDER_IN0_THRUST;
        }
        if triggers.smart_bomb {
            bits |= DEFENDER_IN0_SMART_BOMB;
        }
        if triggers.hyperspace {
            bits |= DEFENDER_IN0_HYPERSPACE;
        }
        if triggers.reverse {
            bits |= DEFENDER_IN0_REVERSE;
        }
        if triggers.altitude_down {
            bits |= DEFENDER_IN0_ALTITUDE_DOWN;
        }
        bits
    }

    fn vertical_control_from_ports(in0: u8, in1: u8) -> VerticalControl {
        if in1 & DEFENDER_IN1_ALTITUDE_UP != 0 {
            VerticalControl::Up
        } else if in0 & DEFENDER_IN0_ALTITUDE_DOWN != 0 {
            VerticalControl::Down
        } else {
            VerticalControl::Neutral
        }
    }

    const ORACLE_PLAYER_INPUT_IN0: u16 = 0xA07B;
    const ORACLE_PLAYER_INPUT_IN1: u16 = 0xA07D;
    const ORACLE_STATUS: u16 = 0xA0BA;
    const ORACLE_BACKGROUND_LEFT: u16 = 0xA020;
    const ORACLE_PLAYER_SCREEN_POSITION: u16 = 0xA0C1;
    const ORACLE_NEXT_PLAYER_DIRECTION: u16 = 0xA0BB;
    const ORACLE_ACTIVE_PROJECTILES: u16 = 0xA0B5;
    const ORACLE_FIRE_ROUTINE: u16 = 0xE591;

    fn player_motion_state(
        x: u16,
        y: u16,
        x_velocity: i32,
        y_velocity: i16,
        direction: Direction,
        camera_left: u16,
    ) -> PlayerMotionState {
        PlayerMotionState::new(
            (unsigned_vector(x), unsigned_vector(y)),
            (
                WorldVector::from_subpixels(x_velocity << 8),
                WorldVector::from_subpixels(i32::from(y_velocity) << 8),
            ),
            direction,
            unsigned_vector(camera_left),
        )
    }

    fn restore_oracle_motion(oracle: &mut ArcadeMachine, state: PlayerMotionState) {
        let mut snapshot = oracle.snapshot();
        snapshot.phase = crate::machine_state::GamePhase::Playing;
        snapshot.player.x = Fixed16(state.position.0.subpixels());
        snapshot.player.y = Fixed16(state.position.1.subpixels());
        snapshot.player.xv = Fixed16(state.velocity.0.subpixels());
        snapshot.player.yv = Fixed16(state.velocity.1.subpixels());
        snapshot.player.facing = match state.direction {
            Direction::Left => Facing::Left,
            Direction::Right => Facing::Right,
        };
        oracle.restore(snapshot);
        oracle.red_label_write_ram_byte_for_test(ORACLE_STATUS, 0);
        write_oracle_word(oracle, ORACLE_BACKGROUND_LEFT, word(state.camera_left));
    }

    fn assert_motion_update_matches(clean: PlayerMotionFrame, accepted: RedLabelPlayerMotion) {
        let RedLabelPlayerMotion::Updated {
            player_velocity,
            player_x16,
            background_delta,
            background_left,
            absolute_x,
            player_y_velocity,
            player_y16,
            next_player_screen,
            ..
        } = accepted
        else {
            panic!("expected accepted player motion update, got {accepted:?}");
        };

        assert_eq!(velocity_bytes(clean.state.velocity.0), player_velocity);
        assert_eq!(word(clean.state.position.0), player_x16);
        assert_eq!(word(clean.camera_delta), background_delta);
        assert_eq!(word(clean.state.camera_left), background_left);
        assert_eq!(word(clean.world_x), absolute_x);
        assert_eq!(word(clean.state.velocity.1), player_y_velocity);
        assert_eq!(word(clean.state.position.1), player_y16);
        assert_eq!(clean.screen_position.packed(), next_player_screen);
    }

    fn assert_projectile_launch_matches(
        clean: ProjectileLaunchOutcome,
        accepted: RedLabelLaserFire,
        expected_direction: RedLabelLaserDirection,
    ) {
        let RedLabelLaserFire::Started {
            direction,
            laser_count,
            start_address,
            ..
        } = accepted
        else {
            panic!("expected accepted projectile launch, got {accepted:?}");
        };
        let ProjectileLaunchOutcome::Started {
            state,
            direction: clean_direction,
            spawn,
        } = clean
        else {
            panic!("expected clean projectile launch, got {clean:?}");
        };

        let expected_clean_direction = match expected_direction {
            RedLabelLaserDirection::Left => Direction::Left,
            RedLabelLaserDirection::Right => Direction::Right,
        };
        assert_eq!(direction, expected_direction);
        assert_eq!(clean_direction, expected_clean_direction);
        assert_eq!(state.active_projectiles, laser_count);
        assert_eq!(spawn.packed(), start_address);
    }

    fn schedule_oracle_fire_process(oracle: &mut ArcadeMachine) {
        let process = oracle
            .red_label_make_process(ORACLE_FIRE_ROUTINE, RED_LABEL_SYSTEM_PROCESS_TYPE)
            .expect("make oracle fire process")
            .process_address;
        let scheduled = oracle
            .step_red_label_process_scheduler()
            .expect("schedule oracle fire process")
            .expect("scheduled oracle fire process");
        assert_eq!(scheduled.process_address, process);
    }

    fn write_oracle_word(oracle: &mut ArcadeMachine, address: u16, value: u16) {
        let [high, low] = value.to_be_bytes();
        oracle.red_label_write_ram_byte_for_test(address, high);
        oracle.red_label_write_ram_byte_for_test(address + 1, low);
    }

    fn velocity_bytes(vector: WorldVector) -> [u8; 3] {
        let raw = ((vector.subpixels() >> 8) as u32) & 0x00FF_FFFF;
        [
            ((raw >> 16) & 0xFF) as u8,
            ((raw >> 8) & 0xFF) as u8,
            (raw & 0xFF) as u8,
        ]
    }

    fn unsigned_vector(word: u16) -> WorldVector {
        WorldVector::from_subpixels(i32::from(word) << 8)
    }

    fn word(vector: WorldVector) -> u16 {
        (vector.subpixels() >> 8) as u16
    }

    #[test]
    fn player_action_triggers_none_is_empty() {
        assert!(!PlayerActionTriggers::NONE.any());
    }
}
