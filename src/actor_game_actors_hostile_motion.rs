const fn arcade_drift_from_velocity(x_velocity: u16) -> i16 {
    if x_velocity & 0x8000 != 0 {
        -1
    } else if x_velocity == 0 {
        0
    } else {
        1
    }
}

fn drift_direction(drift: i16) -> Direction {
    if drift < 0 {
        Direction::Left
    } else {
        Direction::Right
    }
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
fn step_toward(position: Point, target: Point, speed: i16) -> Point {
    Point::new(
        position.x + axis_step(target.x - position.x, speed),
        position.y + axis_step(target.y - position.y, speed),
    )
}

fn axis_step(delta: i16, speed: i16) -> i16 {
    let speed = speed.max(0);
    if delta == 0 {
        0
    } else if delta > 0 {
        delta.min(speed)
    } else {
        delta.max(-speed)
    }
}

fn move_by_hostile_mode(
    position: Point,
    mode: HostileMovementMode,
    prompt: &StepPrompt,
    speed: i16,
    drift: i16,
) -> Option<Point> {
    match mode {
        HostileMovementMode::Drift => Some(position.offset(Velocity::new(drift * speed.max(0), 0))),
        HostileMovementMode::ChasePlayer => prompt
            .player_position()
            .map(|player| step_toward(position, player, speed)),
    }
}

fn observed_velocity(previous: Point, current: Point) -> Velocity {
    Velocity::new(current.x - previous.x, current.y - previous.y)
}

fn direction_for_velocity(velocity: Velocity, fallback: Direction) -> Direction {
    if velocity.dx < 0 {
        Direction::Left
    } else if velocity.dx > 0 {
        Direction::Right
    } else {
        fallback
    }
}
