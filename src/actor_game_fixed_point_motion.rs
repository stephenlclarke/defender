use super::*;

pub(in crate::actor_game) const MOTION_WORD_SIGN_BIT: u16 = 0x8000;

pub(in crate::actor_game) const fn drift_from_motion_word(x_velocity: u16) -> i16 {
    if x_velocity & MOTION_WORD_SIGN_BIT != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

pub(in crate::actor_game) fn absolute_world_x(position: Point, x_fraction: u8) -> u16 {
    u16::from_be_bytes([position.x as u8, x_fraction])
}

pub(in crate::actor_game) fn world_position_words(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x as u8, x_fraction]),
        u16::from_be_bytes([position.y as u8, y_fraction]),
    )
}

pub(in crate::actor_game) fn step_motion_axis(
    position: i16,
    fraction: u8,
    velocity: u16,
) -> (i16, u8) {
    let [position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (i16::from(position), fraction)
}

pub(in crate::actor_game) fn step_wrapping_motion_y(
    position: i16,
    fraction: u8,
    velocity: u16,
) -> (i16, u8) {
    let [mut position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    if position < PLAYFIELD_TOP_EDGE_Y {
        position = PLAYFIELD_BOTTOM_EDGE_Y;
    } else if position > PLAYFIELD_BOTTOM_EDGE_Y {
        position = PLAYFIELD_TOP_EDGE_Y;
    }
    (i16::from(position), fraction)
}

pub(in crate::actor_game) fn screen_velocity_from_motion_words(
    x_velocity: u16,
    y_velocity: u16,
) -> Velocity {
    Velocity::new(
        screen_velocity_component_from_motion_word(x_velocity),
        screen_velocity_component_from_motion_word(y_velocity),
    )
}

pub(in crate::actor_game) fn screen_velocity_component_from_motion_word(velocity: u16) -> i16 {
    let signed = velocity as i16;
    if signed == 0 {
        return 0;
    }

    let pixels = signed / MOTION_WORD_FRACTION_SCALE;
    if pixels == 0 {
        if signed > 0 { 1 } else { -1 }
    } else {
        pixels
    }
}
