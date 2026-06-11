use super::*;

pub(in crate::actor_game) fn drift_direction(drift: i16) -> Direction {
    if drift < 0 {
        Direction::Left
    } else {
        Direction::Right
    }
}
pub(in crate::actor_game) fn step_toward(position: Point, target: Point, speed: i16) -> Point {
    Point::new(
        position.x + axis_step(target.x - position.x, speed),
        position.y + axis_step(target.y - position.y, speed),
    )
}

pub(in crate::actor_game) fn axis_step(delta: i16, speed: i16) -> i16 {
    let speed = speed.max(0);
    if delta == 0 {
        0
    } else if delta > 0 {
        delta.min(speed)
    } else {
        delta.max(-speed)
    }
}

pub(in crate::actor_game) fn move_by_hostile_mode(
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

pub(in crate::actor_game) fn observed_velocity(previous: Point, current: Point) -> Velocity {
    Velocity::new(current.x - previous.x, current.y - previous.y)
}

pub(in crate::actor_game) fn direction_for_velocity(
    velocity: Velocity,
    fallback: Direction,
) -> Direction {
    if velocity.dx < 0 {
        Direction::Left
    } else if velocity.dx > 0 {
        Direction::Right
    } else {
        fallback
    }
}
