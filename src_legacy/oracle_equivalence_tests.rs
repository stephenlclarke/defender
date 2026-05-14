use crate::{
    accepted::AcceptedFrame,
    accepted_behavior::cabinet_input_for_test,
    game::{Direction, GameEvent, GameInput, GamePhase, WorldVector},
    input::{
        DEFENDER_IN0_ALTITUDE_DOWN, DEFENDER_IN0_FIRE, DEFENDER_IN0_HYPERSPACE,
        DEFENDER_IN0_REVERSE, DEFENDER_IN0_SMART_BOMB, DEFENDER_IN0_THRUST,
        DEFENDER_IN1_ALTITUDE_UP,
    },
    machine::{
        ArcadeMachine, RED_LABEL_SYSTEM_PROCESS_TYPE, RedLabelLaserDirection, RedLabelLaserFire,
        RedLabelPlayerMotion,
    },
    machine_state,
    oracle::{
        GameplayOracle,
        test_support::{
            adapt_accepted_event, adapt_accepted_scene, adapt_accepted_snapshot,
            adapt_accepted_sound_command,
        },
    },
    red_label::{Facing, Fixed16},
    systems::{
        PlayerControlFrame, PlayerControlIntent, PlayerControlSystem, PlayerMotionFrame,
        PlayerMotionState, PlayerMotionSystem, ProjectileLaunchOutcome, ProjectileState,
        ProjectileSystem, ScreenPosition, VerticalControl,
    },
};

#[test]
fn clean_player_control_history_matches_oracle_switch_scan() {
    let mut clean = PlayerControlSystem::new();
    let mut oracle = ArcadeMachine::new();

    for input in player_control_inputs() {
        let frame = clean.step(input);
        let scan = oracle
            .red_label_scan_translated_player_switches(
                cabinet_input_for_test(input).defender_input_ports(),
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
        let ports = cabinet_input_for_test(input).defender_input_ports();
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
        let accepted_frame = AcceptedFrame::from(legacy.step(cabinet_input_for_test(input)));
        let expected_state = adapt_accepted_snapshot(accepted_frame.snapshot);
        let expected_gameplay_events = accepted_frame
            .events
            .into_iter()
            .map(adapt_accepted_event)
            .collect::<Vec<_>>();
        let expected_sounds = accepted_frame
            .sound_commands
            .into_iter()
            .map(adapt_accepted_sound_command)
            .collect::<Vec<_>>();
        let expected_scene_summary =
            adapt_accepted_scene(&expected_state, accepted_frame.visual_signature).summary();

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
            && summary.visual_signature.is_some()
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
    snapshot.phase = machine_state::GamePhase::Playing;
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
