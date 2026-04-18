use crate::constants::{
    DEFAULT_LIVES, DEFAULT_WAVE, GROUND_ROW, PLAYER_START_X, PLAYER_START_Y, WORLD_HEIGHT,
    WORLD_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    PlayerShip,
    Lander,
    Mutant,
    Human,
}

impl EntityKind {
    pub fn glyph(self) -> char {
        match self {
            Self::PlayerShip => '^',
            Self::Lander => 'L',
            Self::Mutant => 'M',
            Self::Human => 'h',
        }
    }

    pub fn is_enemy(self) -> bool {
        matches!(self, Self::Lander | Self::Mutant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub kind: EntityKind,
    pub position: Position,
    pub velocity: Velocity,
}

impl Entity {
    pub fn new(kind: EntityKind, x: i32, y: i32, dx: i32, dy: i32) -> Self {
        Self {
            kind,
            position: Position { x, y },
            velocity: Velocity { dx, dy },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    pub score: u32,
    pub lives: u8,
    pub wave: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    width: usize,
    height: usize,
    tick: u32,
    status: Status,
    entities: Vec<Entity>,
}

impl World {
    pub fn bootstrap() -> Self {
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            tick: 0,
            status: Status {
                score: 0,
                lives: DEFAULT_LIVES,
                wave: DEFAULT_WAVE,
            },
            entities: vec![
                Entity::new(EntityKind::PlayerShip, PLAYER_START_X, PLAYER_START_Y, 0, 0),
                Entity::new(EntityKind::Lander, 48, 4, -1, 1),
                Entity::new(EntityKind::Mutant, 28, 9, 1, -1),
                Entity::new(EntityKind::Human, 18, GROUND_ROW as i32 - 1, 0, 0),
                Entity::new(EntityKind::Human, 42, GROUND_ROW as i32 - 1, 0, 0),
            ],
        }
    }

    pub fn with_entities(
        width: usize,
        height: usize,
        status: Status,
        entities: Vec<Entity>,
    ) -> Self {
        Self {
            width,
            height,
            tick: 0,
            status,
            entities,
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;

        let max_x = self.width as i32 - 1;
        let min_y = 1;
        let max_y = self.height as i32 - 3;

        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::Human => {}
                EntityKind::PlayerShip => {
                    if self.tick.is_multiple_of(2) {
                        entity.position.y = (entity.position.y + 1).min(max_y);
                    } else {
                        entity.position.y = (entity.position.y - 1).max(min_y);
                    }
                }
                EntityKind::Lander | EntityKind::Mutant => {
                    entity.position.x =
                        wrap_coordinate(entity.position.x + entity.velocity.dx, max_x);
                    entity.position.y += entity.velocity.dy;

                    if entity.position.y <= min_y || entity.position.y >= max_y {
                        entity.velocity.dy *= -1;
                        entity.position.y = entity.position.y.clamp(min_y, max_y);
                    }
                }
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn ground_row(&self) -> usize {
        self.height.saturating_sub(2)
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn enemy_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|entity| entity.kind.is_enemy())
            .count()
    }

    pub fn human_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Human)
            .count()
    }

    pub fn threat_score(&self) -> usize {
        let humans: Vec<Position> = self
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Human)
            .map(|entity| entity.position)
            .collect();

        self.entities
            .iter()
            .filter(|entity| entity.kind.is_enemy())
            .filter(|entity| {
                entity.position.y >= self.ground_row() as i32 - 4
                    || humans.iter().any(|human| {
                        (entity.position.x - human.x).abs() <= 6
                            && (entity.position.y - human.y).abs() <= 4
                    })
            })
            .count()
    }

    pub fn add_score(&mut self, delta: u32) {
        self.status.score = self.status.score.saturating_add(delta);
    }

    pub fn set_wave(&mut self, wave: u8) {
        self.status.wave = wave;
    }

    pub fn set_lives(&mut self, lives: u8) {
        self.status.lives = lives;
    }

    pub fn spawn_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn remove_first_by_kind(&mut self, kind: EntityKind) -> bool {
        if let Some(index) = self.entities.iter().position(|entity| entity.kind == kind) {
            self.entities.remove(index);
            true
        } else {
            false
        }
    }
}

fn wrap_coordinate(value: i32, max: i32) -> i32 {
    if value < 0 {
        max
    } else if value > max {
        0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, EntityKind, Position, Status, World, wrap_coordinate};

    #[test]
    fn bootstrap_creates_expected_entities() {
        let world = World::bootstrap();

        assert_eq!(world.enemy_count(), 2);
        assert_eq!(world.human_count(), 2);
        assert_eq!(world.status().lives, 3);
        assert_eq!(world.entities()[0].kind, EntityKind::PlayerShip);
    }

    #[test]
    fn wrap_coordinate_wraps_both_edges() {
        assert_eq!(wrap_coordinate(-1, 63), 63);
        assert_eq!(wrap_coordinate(64, 63), 0);
        assert_eq!(wrap_coordinate(12, 63), 12);
    }

    #[test]
    fn step_wraps_horizontal_movement_and_bounces_vertical_movement() {
        let mut world = World::with_entities(
            10,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![Entity::new(EntityKind::Lander, 0, 5, -1, 1)],
        );

        world.step();

        let lander = &world.entities()[0];
        assert_eq!(lander.position, Position { x: 9, y: 5 });
        assert_eq!(lander.velocity.dy, -1);
    }

    #[test]
    fn threat_score_counts_enemies_near_humans_or_ground() {
        let world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::Human, 5, 7, 0, 0),
                Entity::new(EntityKind::Lander, 7, 6, 0, 0),
                Entity::new(EntityKind::Mutant, 18, 1, 0, 0),
            ],
        );

        assert_eq!(world.threat_score(), 1);
    }

    #[test]
    fn player_ship_bobs_between_ticks() {
        let mut world = World::bootstrap();
        let start_y = world.entities()[0].position.y;

        world.step();
        let after_first_tick = world.entities()[0].position.y;
        world.step();
        let after_second_tick = world.entities()[0].position.y;

        assert_eq!(after_first_tick, start_y - 1);
        assert_eq!(after_second_tick, start_y);
    }

    #[test]
    fn world_can_update_status_and_entities_for_scripted_sequences() {
        let mut world = World::bootstrap();

        world.add_score(250);
        world.set_wave(2);
        world.set_lives(2);
        world.spawn_entity(Entity::new(EntityKind::Human, 50, 10, 0, 0));
        assert!(world.remove_first_by_kind(EntityKind::Lander));

        assert_eq!(world.status().score, 250);
        assert_eq!(world.status().wave, 2);
        assert_eq!(world.status().lives, 2);
        assert_eq!(world.enemy_count(), 1);
        assert_eq!(world.human_count(), 3);
    }
}
