#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenPosition {
    pub x: u8,
    pub y: u8,
}

impl ScreenPosition {
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub const fn from_packed(value: u16) -> Self {
        let [x, y] = value.to_be_bytes();
        Self { x, y }
    }

    pub const fn packed(self) -> u16 {
        u16::from_be_bytes([self.x, self.y])
    }

    pub const fn wrapping_offset(self, x: u8, y: u8) -> Self {
        Self {
            x: self.x.wrapping_add(x),
            y: self.y.wrapping_add(y),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScreenVelocity {
    pub dx: i8,
    pub dy: i8,
}

impl ScreenVelocity {
    pub const fn new(dx: i8, dy: i8) -> Self {
        Self { dx, dy }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyMotionStep {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnemyMotionSystem;

impl EnemyMotionSystem {
    pub fn step(position: ScreenPosition, velocity: ScreenVelocity) -> EnemyMotionStep {
        EnemyMotionStep {
            position: ScreenPosition::new(
                position.x.wrapping_add_signed(velocity.dx),
                position.y.wrapping_add_signed(velocity.dy),
            ),
            velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMotionState {
    pub position: (WorldVector, WorldVector),
    pub velocity: (WorldVector, WorldVector),
    pub direction: Direction,
    pub camera_left: WorldVector,
}

impl PlayerMotionState {
    pub const fn new(
        position: (WorldVector, WorldVector),
        velocity: (WorldVector, WorldVector),
        direction: Direction,
        camera_left: WorldVector,
    ) -> Self {
        Self {
            position,
            velocity,
            direction,
            camera_left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerMotionStep {
    pub state: PlayerMotionState,
    pub camera_delta: WorldVector,
    pub world_x: WorldVector,
    pub screen_position: ScreenPosition,
    pub blocked_by_vertical_limit: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerMotionSystem;

impl PlayerMotionSystem {
    pub fn step(state: PlayerMotionState, intent: PlayerControlIntent) -> PlayerMotionStep {
        let mut x_velocity = Fixed24::from_world_vector(state.velocity.0).damped();
        if intent.thrust {
            x_velocity = x_velocity.add_signed_word(thrust_acceleration(state.direction));
        }

        let calculated_x = x_velocity.calculated_screen_x(state.direction);
        let previous_x = unsigned_vector_word(state.position.0);
        let (screen_x, camera_delta) = scroll_adjusted_x(previous_x, calculated_x);

        x_velocity = x_velocity.with_high_word(clamp_camera_velocity_word(x_velocity.high_word()));
        let camera_left = unsigned_vector_word(state.camera_left)
            .wrapping_add(x_velocity.high_word())
            .wrapping_sub(camera_delta);
        let world_x = player_world_x(screen_x, camera_left);

        let previous_y = unsigned_vector_word(state.position.1);
        let previous_y_velocity = signed_vector_word(state.velocity.1);
        let vertical = next_vertical_velocity(
            previous_y.to_be_bytes()[0],
            previous_y_velocity,
            intent.vertical,
        );
        let (screen_y, y_velocity, blocked_by_vertical_limit) = match vertical {
            Some(y_velocity) => (previous_y.wrapping_add(y_velocity), y_velocity, false),
            None => (previous_y, previous_y_velocity, true),
        };

        let next_state = PlayerMotionState {
            position: (
                unsigned_word_vector(screen_x),
                unsigned_word_vector(screen_y),
            ),
            velocity: (x_velocity.to_world_vector(), signed_word_vector(y_velocity)),
            direction: state.direction,
            camera_left: unsigned_word_vector(camera_left),
        };

        PlayerMotionStep {
            state: next_state,
            camera_delta: signed_word_vector(camera_delta),
            world_x: unsigned_word_vector(world_x),
            screen_position: ScreenPosition::from_packed(u16::from_be_bytes([
                screen_x.to_be_bytes()[0],
                screen_y.to_be_bytes()[0],
            ])),
            blocked_by_vertical_limit,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileState {
    pub active_projectiles: u8,
}

impl ProjectileState {
    pub const fn new(active_projectiles: u8) -> Self {
        Self { active_projectiles }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileLaunchOutcome {
    Started {
        state: ProjectileState,
        direction: Direction,
        spawn: ScreenPosition,
    },
    CapacityReached {
        state: ProjectileState,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileSystem;

impl ProjectileSystem {
    pub const MAX_ACTIVE_PROJECTILES: u8 = 4;

    pub fn try_launch(
        state: ProjectileState,
        player_position: ScreenPosition,
        direction: Direction,
    ) -> ProjectileLaunchOutcome {
        if state.active_projectiles >= Self::MAX_ACTIVE_PROJECTILES {
            return ProjectileLaunchOutcome::CapacityReached { state };
        }

        let spawn = match direction {
            Direction::Left => player_position.wrapping_offset(0, 4),
            Direction::Right => player_position.wrapping_offset(7, 4),
        };
        ProjectileLaunchOutcome::Started {
            state: ProjectileState::new(state.active_projectiles.wrapping_add(1)),
            direction,
            spawn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileMotionStep {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectileMotionSystem;

const PLAYER_LASER_COLUMNS_PER_STEP: i8 = 4;
const SCREEN_RIGHT_EDGE_X: u8 = u8::MAX;
const SCREEN_LEFT_EDGE_X: u8 = 0x00;

impl ProjectileMotionSystem {
    pub const fn velocity_for_direction(direction: Direction) -> ScreenVelocity {
        match direction {
            Direction::Left => ScreenVelocity::new(-PLAYER_LASER_COLUMNS_PER_STEP, 0),
            Direction::Right => ScreenVelocity::new(PLAYER_LASER_COLUMNS_PER_STEP, 0),
        }
    }

    pub fn step(position: ScreenPosition, velocity: ScreenVelocity) -> ProjectileMotionStep {
        if player_laser_reached_screen_edge(position, velocity) {
            return ProjectileMotionStep {
                position,
                velocity,
                active: false,
            };
        }

        let next_x = i16::from(position.x) + i16::from(velocity.dx);
        let next_y = i16::from(position.y) + i16::from(velocity.dy);
        let active = (0..=i16::from(u8::MAX)).contains(&next_x)
            && (0..=i16::from(u8::MAX)).contains(&next_y);

        ProjectileMotionStep {
            position: if active {
                ScreenPosition::new(next_x as u8, next_y as u8)
            } else {
                position
            },
            velocity,
            active,
        }
    }
}

const fn player_laser_reached_screen_edge(
    position: ScreenPosition,
    velocity: ScreenVelocity,
) -> bool {
    if velocity.dx > 0 {
        position.x == SCREEN_RIGHT_EDGE_X
    } else if velocity.dx < 0 {
        position.x == SCREEN_LEFT_EDGE_X
    } else {
        false
    }
}
