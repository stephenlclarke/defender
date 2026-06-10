const ARCADE_VELOCITY_SIGN_BIT: u16 = 0x8000;

const fn arcade_drift_from_velocity(x_velocity: u16) -> i16 {
    if x_velocity & ARCADE_VELOCITY_SIGN_BIT != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

fn arcade_absolute_x(position: Point, x_fraction: u8) -> u16 {
    u16::from_be_bytes([position.x as u8, x_fraction])
}

fn arcade_world_position(position: Point, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x as u8, x_fraction]),
        u16::from_be_bytes([position.y as u8, y_fraction]),
    )
}

fn arcade_axis_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
    let [position, fraction] = u16::from_be_bytes([position as u8, fraction])
        .wrapping_add(velocity)
        .to_be_bytes();
    (i16::from(position), fraction)
}

fn arcade_active_object_y_step(position: i16, fraction: u8, velocity: u16) -> (i16, u8) {
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

fn arcade_screen_velocity(x_velocity: u16, y_velocity: u16) -> Velocity {
    Velocity::new(
        arcade_screen_velocity_component(x_velocity),
        arcade_screen_velocity_component(y_velocity),
    )
}

fn arcade_screen_velocity_component(velocity: u16) -> i16 {
    let signed = velocity as i16;
    if signed == 0 {
        return 0;
    }

    let pixels = signed / 256;
    if pixels == 0 {
        if signed > 0 { 1 } else { -1 }
    } else {
        pixels
    }
}
