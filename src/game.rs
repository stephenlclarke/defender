use crate::constants::{
    DEFAULT_LIVES, DEFAULT_SMART_BOMBS, DEFAULT_WAVE, GROUND_ROW, PLAYER_START_X, PLAYER_START_Y,
    WORLD_HEIGHT, WORLD_SPAN, WORLD_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    PlayerShip,
    PlayerShot,
    EnemyShot,
    Lander,
    Mutant,
    Human,
}

impl EntityKind {
    pub fn glyph(self) -> char {
        match self {
            Self::PlayerShip => '^',
            Self::PlayerShot => '-',
            Self::EnemyShot => '!',
            Self::Lander => 'L',
            Self::Mutant => 'M',
            Self::Human => 'h',
        }
    }

    pub fn is_enemy(self) -> bool {
        matches!(self, Self::Lander | Self::Mutant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpdateInput {
    pub up: bool,
    pub down: bool,
    pub thrust: bool,
    pub reverse: bool,
    pub fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub invincible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEvent {
    ShotFired,
    EnemyFired,
    EnemyDestroyed,
    HumanLost,
    SmartBombDetonated,
    HyperspaceUsed,
    PlayerHit,
    WaveAdvanced,
    GameOver,
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
    world_span: i32,
    camera_x: i32,
    tick: u32,
    fire_cooldown: u8,
    smart_bombs: u8,
    game_over: bool,
    status: Status,
    terrain: Vec<usize>,
    entities: Vec<Entity>,
}

impl World {
    pub fn bootstrap() -> Self {
        let terrain = build_scrolling_terrain(WORLD_SPAN, WORLD_HEIGHT);
        let human0_y = terrain_surface_y(&terrain, 8);
        let human1_y = terrain_surface_y(&terrain, 30);
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            world_span: WORLD_SPAN as i32,
            camera_x: PLAYER_START_X,
            tick: 0,
            fire_cooldown: 0,
            smart_bombs: DEFAULT_SMART_BOMBS,
            game_over: false,
            status: Status {
                score: 0,
                lives: DEFAULT_LIVES,
                wave: DEFAULT_WAVE,
            },
            terrain,
            entities: vec![
                Entity::new(EntityKind::PlayerShip, PLAYER_START_X, PLAYER_START_Y, 0, 0),
                Entity::new(EntityKind::Lander, 36, 4, -1, 1),
                Entity::new(EntityKind::Mutant, 22, 9, 1, -1),
                Entity::new(EntityKind::Human, 8, human0_y, 0, 0),
                Entity::new(EntityKind::Human, 30, human1_y, 0, 0),
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
            world_span: width as i32,
            camera_x: width as i32 / 2,
            tick: 0,
            fire_cooldown: 0,
            smart_bombs: DEFAULT_SMART_BOMBS,
            game_over: false,
            status,
            terrain: build_flat_terrain(width, height),
            entities,
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;

        let max_x = self.world_max_x();
        let min_y = 1;
        let max_y = self.height as i32 - 3;
        let terrain = &self.terrain;

        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::Human => {}
                EntityKind::PlayerShip => {
                    if self.tick.is_multiple_of(2) {
                        entity.position.y = (entity.position.y + 1).min(max_y);
                    } else {
                        entity.position.y = (entity.position.y - 1).max(min_y);
                    }
                    entity.position.y = entity
                        .position
                        .y
                        .min(terrain_surface_y(terrain, entity.position.x));
                }
                EntityKind::Lander | EntityKind::Mutant => {
                    entity.position.x =
                        wrap_coordinate(entity.position.x + entity.velocity.dx, max_x);
                    entity.position.y += entity.velocity.dy;
                    let surface = terrain_surface_y(terrain, entity.position.x);

                    if entity.position.y <= min_y || entity.position.y >= max_y.min(surface) {
                        entity.velocity.dy *= -1;
                        entity.position.y = entity.position.y.clamp(min_y, max_y.min(surface));
                    }
                }
                EntityKind::PlayerShot | EntityKind::EnemyShot => {
                    entity.position.x += entity.velocity.dx;
                    entity.position.y += entity.velocity.dy;
                }
            }
        }

        self.retain_projectiles(max_x, min_y, max_y);
        self.sync_camera_to_player();
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn world_span(&self) -> i32 {
        self.world_span
    }

    pub fn camera_x(&self) -> i32 {
        self.camera_x
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn ground_row(&self) -> usize {
        self.height.saturating_sub(2)
    }

    pub fn terrain_row_at_world_x(&self, world_x: i32) -> usize {
        let index = wrap_coordinate(world_x, self.world_max_x()) as usize;
        self.terrain[index]
    }

    pub fn terrain_row_at_screen_x(&self, screen_x: usize) -> usize {
        self.terrain_row_at_world_x(self.world_x_for_screen_x(screen_x))
    }

    pub fn safe_altitude_at_world_x(&self, world_x: i32) -> i32 {
        self.terrain_row_at_world_x(world_x) as i32 - 1
    }

    pub fn screen_x_for_world_x(&self, world_x: i32) -> Option<usize> {
        if !(0..=self.world_max_x()).contains(&world_x) {
            return None;
        }

        let offset = (world_x - self.left_edge()).rem_euclid(self.world_span);
        (offset < self.width as i32).then_some(offset as usize)
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn smart_bombs(&self) -> u8 {
        self.smart_bombs
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn entity_count_by_kind(&self, kind: EntityKind) -> usize {
        self.entities
            .iter()
            .filter(|entity| entity.kind == kind)
            .count()
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
                entity.position.y >= self.safe_altitude_at_world_x(entity.position.x) - 3
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

    pub fn set_smart_bombs(&mut self, smart_bombs: u8) {
        self.smart_bombs = smart_bombs;
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

    pub fn step_live(&mut self, input: UpdateInput) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        if self.game_over {
            return events;
        }

        self.tick += 1;
        if self.fire_cooldown > 0 {
            self.fire_cooldown -= 1;
        }

        let max_x = self.world_max_x();
        let min_y = 1;
        let max_y = self.height as i32 - 3;

        let mut shot_origin = None;
        {
            let terrain = &self.terrain;
            if let Some(player) = self
                .entities
                .iter_mut()
                .find(|entity| entity.kind == EntityKind::PlayerShip)
            {
                let dx = input.thrust as i32 - input.reverse as i32;
                let dy = input.down as i32 - input.up as i32;
                player.position.x = wrap_coordinate(player.position.x + dx, max_x);
                player.position.y = (player.position.y + dy).clamp(min_y, max_y);
                player.position.y = player
                    .position
                    .y
                    .min(terrain_surface_y(terrain, player.position.x));

                if input.hyperspace {
                    player.position =
                        hyperspace_destination(self.tick, self.status.score, max_x, min_y, max_y);
                    player.position.y = player
                        .position
                        .y
                        .min(terrain_surface_y(terrain, player.position.x));
                    events.push(WorldEvent::HyperspaceUsed);
                }

                shot_origin = Some(player.position);
            }
        }

        if input.smart_bomb && self.can_use_smart_bomb(input.invincible) {
            self.detonate_smart_bomb(input.invincible, &mut events);
        }

        if input.fire
            && self.fire_cooldown == 0
            && let Some(origin) = shot_origin
        {
            self.entities.push(Entity::new(
                EntityKind::PlayerShot,
                (origin.x + 1).min(max_x),
                origin.y,
                2,
                0,
            ));
            self.fire_cooldown = 2;
            events.push(WorldEvent::ShotFired);
        }

        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::Human | EntityKind::PlayerShip => {}
                EntityKind::Lander | EntityKind::Mutant => {
                    entity.position.x =
                        wrap_coordinate(entity.position.x + entity.velocity.dx, max_x);
                    entity.position.y += entity.velocity.dy;
                    let surface = terrain_surface_y(&self.terrain, entity.position.x);

                    if entity.position.y <= min_y || entity.position.y >= max_y.min(surface) {
                        entity.velocity.dy *= -1;
                        entity.position.y = entity.position.y.clamp(min_y, max_y.min(surface));
                    }
                }
                EntityKind::PlayerShot | EntityKind::EnemyShot => {
                    entity.position.x += entity.velocity.dx;
                    entity.position.y += entity.velocity.dy;
                }
            }
        }

        self.spawn_enemy_fire(max_x, min_y, max_y, &mut events);
        self.retain_projectiles(max_x, min_y, max_y);

        self.handle_human_losses(&mut events);
        self.handle_player_shot_hits(&mut events);
        self.handle_player_collisions(input.invincible, &mut events);

        if !self.game_over && self.enemy_count() == 0 {
            self.status.wave = self.status.wave.saturating_add(1);
            self.spawn_wave();
            events.push(WorldEvent::WaveAdvanced);
        }

        self.sync_camera_to_player();
        events
    }

    fn spawn_enemy_fire(
        &mut self,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        events: &mut Vec<WorldEvent>,
    ) {
        if !self.tick.is_multiple_of(5) {
            return;
        }

        let Some(player_position) = self.player_position() else {
            return;
        };

        let Some(enemy) = self
            .entities
            .iter()
            .filter(|entity| entity.kind.is_enemy())
            .min_by_key(|entity| (entity.position.x - player_position.x).abs())
            .cloned()
        else {
            return;
        };

        let dx = if enemy.position.x >= player_position.x {
            -1
        } else {
            1
        };
        let dy = (player_position.y - enemy.position.y).signum();
        let shot_x = (enemy.position.x + dx).clamp(0, max_x);
        let shot_y = (enemy.position.y + dy).clamp(min_y, max_y);
        self.entities
            .push(Entity::new(EntityKind::EnemyShot, shot_x, shot_y, dx, dy));
        events.push(WorldEvent::EnemyFired);
    }

    fn handle_human_losses(&mut self, events: &mut Vec<WorldEvent>) {
        let mut human_indices = Vec::new();

        for enemy in self.entities.iter().filter(|entity| entity.kind.is_enemy()) {
            if let Some((index, _)) = self.entities.iter().enumerate().find(|(index, entity)| {
                entity.kind == EntityKind::Human
                    && !human_indices.contains(index)
                    && positions_overlap(enemy.position, entity.position, 1, 1)
            }) {
                human_indices.push(index);
                events.push(WorldEvent::HumanLost);
            }
        }

        remove_indices(&mut self.entities, &human_indices);
    }

    fn handle_player_shot_hits(&mut self, events: &mut Vec<WorldEvent>) {
        let mut remove_indices_set = Vec::new();
        let mut score_delta = 0;

        for (shot_index, shot) in self.entities.iter().enumerate() {
            if shot.kind != EntityKind::PlayerShot || remove_indices_set.contains(&shot_index) {
                continue;
            }

            if let Some((enemy_index, enemy)) =
                self.entities
                    .iter()
                    .enumerate()
                    .find(|(enemy_index, entity)| {
                        (entity.kind.is_enemy() || entity.kind == EntityKind::EnemyShot)
                            && !remove_indices_set.contains(enemy_index)
                            && positions_overlap(shot.position, entity.position, 1, 0)
                    })
            {
                remove_indices_set.push(shot_index);
                remove_indices_set.push(enemy_index);
                if enemy.kind.is_enemy() {
                    score_delta += score_for_enemy(enemy.kind);
                    events.push(WorldEvent::EnemyDestroyed);
                }
            }
        }

        self.add_score(score_delta);
        remove_indices(&mut self.entities, &remove_indices_set);
    }

    fn handle_player_collisions(&mut self, invincible: bool, events: &mut Vec<WorldEvent>) {
        let Some(player_position) = self
            .entities
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .map(|entity| entity.position)
        else {
            return;
        };

        let mut collided_enemies = Vec::new();
        for (index, enemy) in self.entities.iter().enumerate() {
            let collision = match enemy.kind {
                EntityKind::Lander | EntityKind::Mutant => {
                    positions_overlap(player_position, enemy.position, 1, 1)
                }
                EntityKind::EnemyShot => positions_overlap(player_position, enemy.position, 0, 0),
                _ => false,
            };

            if collision {
                collided_enemies.push(index);
            }
        }

        if collided_enemies.is_empty() {
            return;
        }

        if invincible {
            let score_delta = collided_enemies
                .iter()
                .filter_map(|index| self.entities.get(*index))
                .map(|entity| score_for_enemy(entity.kind))
                .sum();
            remove_indices(&mut self.entities, &collided_enemies);
            self.add_score(score_delta);
            if score_delta > 0 {
                events.push(WorldEvent::EnemyDestroyed);
            }
            return;
        }

        remove_indices(&mut self.entities, &collided_enemies);
        if self.status.lives > 0 {
            self.status.lives -= 1;
        }
        self.reset_player_position();
        events.push(WorldEvent::PlayerHit);

        if self.status.lives == 0 {
            self.game_over = true;
            events.push(WorldEvent::GameOver);
        }
    }

    fn reset_player_position(&mut self) {
        let terrain = &self.terrain;
        if let Some(player) = self
            .entities
            .iter_mut()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
        {
            player.position.x = PLAYER_START_X;
            player.position.y = PLAYER_START_Y.min(terrain_surface_y(terrain, PLAYER_START_X));
        }
        self.sync_camera_to_player();
    }

    fn player_position(&self) -> Option<Position> {
        self.entities
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .map(|entity| entity.position)
    }

    fn can_use_smart_bomb(&self, invincible: bool) -> bool {
        invincible || self.smart_bombs > 0
    }

    fn detonate_smart_bomb(&mut self, invincible: bool, events: &mut Vec<WorldEvent>) {
        if !self.can_use_smart_bomb(invincible) {
            return;
        }

        if !invincible {
            self.smart_bombs -= 1;
        }

        let mut remove_indices_set = Vec::new();
        let mut score_delta = 0;

        for (index, entity) in self.entities.iter().enumerate() {
            if entity.kind.is_enemy() || entity.kind == EntityKind::EnemyShot {
                remove_indices_set.push(index);
                score_delta += score_for_enemy(entity.kind);
            }
        }

        remove_indices(&mut self.entities, &remove_indices_set);
        self.add_score(score_delta);
        events.push(WorldEvent::SmartBombDetonated);
    }

    fn retain_projectiles(&mut self, max_x: i32, min_y: i32, max_y: i32) {
        let terrain = &self.terrain;
        self.entities.retain(|entity| match entity.kind {
            EntityKind::PlayerShot | EntityKind::EnemyShot => {
                (0..=max_x).contains(&entity.position.x)
                    && (min_y..=max_y).contains(&entity.position.y)
                    && entity.position.y < terrain_surface_y(terrain, entity.position.x) + 1
            }
            _ => true,
        });
    }

    fn spawn_wave(&mut self) {
        let width = self.world_span;
        let base_y = 3 + (self.status.wave as i32 % 4);
        let wave = self.status.wave as i32;
        let mut enemies = vec![
            Entity::new(EntityKind::Lander, width - 12, base_y, -1, 1),
            Entity::new(EntityKind::Mutant, 18 + (wave % 8), 8, 1, -1),
        ];

        if self.status.wave >= 2 {
            enemies.push(Entity::new(EntityKind::Lander, width - 6, 6, -1, 1));
        }
        if self.status.wave >= 3 {
            enemies.push(Entity::new(EntityKind::Mutant, 8, 4, 1, 1));
        }

        self.entities.extend(enemies);
    }

    fn sync_camera_to_player(&mut self) {
        if let Some(player_position) = self.player_position() {
            self.camera_x = player_position.x;
        }
    }

    fn world_max_x(&self) -> i32 {
        self.world_span - 1
    }

    fn left_edge(&self) -> i32 {
        self.camera_x - self.width as i32 / 2
    }

    fn world_x_for_screen_x(&self, screen_x: usize) -> i32 {
        wrap_coordinate(self.left_edge() + screen_x as i32, self.world_max_x())
    }
}

fn wrap_coordinate(value: i32, max: i32) -> i32 {
    value.rem_euclid(max + 1)
}

fn positions_overlap(left: Position, right: Position, horizontal: i32, vertical: i32) -> bool {
    (left.x - right.x).abs() <= horizontal && (left.y - right.y).abs() <= vertical
}

fn build_flat_terrain(width: usize, height: usize) -> Vec<usize> {
    vec![height.saturating_sub(2); width]
}

fn build_scrolling_terrain(span: usize, height: usize) -> Vec<usize> {
    const TERRAIN_STEP_PATTERN: [i32; 24] = [
        0, 0, -1, 0, 1, 0, 0, -1, -1, 1, 0, 1, 0, -1, 0, 1, 0, 0, -1, 1, 0, 0, 1, -1,
    ];

    let min_row = height.saturating_sub(7) as i32;
    let max_row = height.saturating_sub(2) as i32;
    let mut row = GROUND_ROW as i32;
    let mut terrain = Vec::with_capacity(span);

    for x in 0..span {
        if x % 4 == 0 {
            let delta = TERRAIN_STEP_PATTERN[(x / 4) % TERRAIN_STEP_PATTERN.len()];
            row = (row + delta).clamp(min_row, max_row);
        }
        terrain.push(row as usize);
    }

    terrain
}

fn terrain_surface_y(terrain: &[usize], world_x: i32) -> i32 {
    let index = wrap_coordinate(world_x, terrain.len() as i32 - 1) as usize;
    terrain[index] as i32 - 1
}

fn hyperspace_destination(tick: u32, score: u32, max_x: i32, min_y: i32, max_y: i32) -> Position {
    let width = (max_x + 1).max(1) as u32;
    let height = (max_y - min_y + 1).max(1) as u32;
    let seed = tick
        .wrapping_mul(17)
        .wrapping_add(score.wrapping_mul(3))
        .wrapping_add(11);
    Position {
        x: (seed % width) as i32,
        y: min_y + ((seed / width) % height) as i32,
    }
}

fn score_for_enemy(kind: EntityKind) -> u32 {
    match kind {
        EntityKind::Lander => 150,
        EntityKind::Mutant => 250,
        _ => 0,
    }
}

fn remove_indices(entities: &mut Vec<Entity>, indices: &[usize]) {
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    for index in sorted.into_iter().rev() {
        entities.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::DEFAULT_SMART_BOMBS;

    use super::{
        Entity, EntityKind, Position, Status, UpdateInput, World, WorldEvent,
        hyperspace_destination, wrap_coordinate,
    };

    #[test]
    fn bootstrap_creates_expected_entities() {
        let world = World::bootstrap();

        assert!(world.world_span() > world.width() as i32);
        assert_eq!(world.enemy_count(), 2);
        assert_eq!(world.human_count(), 2);
        assert_eq!(world.status().lives, 3);
        assert_eq!(world.entities()[0].kind, EntityKind::PlayerShip);
        assert_eq!(
            world.screen_x_for_world_x(world.entities()[0].position.x),
            Some(world.width() / 2)
        );
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
        world.set_smart_bombs(1);
        world.spawn_entity(Entity::new(EntityKind::Human, 50, 10, 0, 0));
        assert!(world.remove_first_by_kind(EntityKind::Lander));

        assert_eq!(world.status().score, 250);
        assert_eq!(world.status().wave, 2);
        assert_eq!(world.status().lives, 2);
        assert_eq!(world.smart_bombs(), 1);
        assert_eq!(world.enemy_count(), 1);
        assert_eq!(world.human_count(), 3);
    }

    #[test]
    fn live_step_moves_player_and_spawns_shot() {
        let mut world = World::bootstrap();
        let start = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position;

        let events = world.step_live(UpdateInput {
            thrust: true,
            fire: true,
            ..UpdateInput::default()
        });

        let player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(player.position.x, start.x + 1);
        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 1);
        assert!(events.contains(&WorldEvent::ShotFired));
    }

    #[test]
    fn live_step_wraps_player_and_recenters_the_camera() {
        let mut world = World::bootstrap();
        let world_max_x = world.world_span() - 1;
        let player = world
            .entities
            .iter_mut()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        player.position.x = world_max_x;
        world.camera_x = player.position.x;

        world.step_live(UpdateInput {
            thrust: true,
            ..UpdateInput::default()
        });

        let wrapped_player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(wrapped_player.position.x, 0);
        assert_eq!(world.camera_x(), 0);
        assert_eq!(
            world.screen_x_for_world_x(wrapped_player.position.x),
            Some(world.width() / 2)
        );
    }

    #[test]
    fn live_step_spawns_enemy_shots_on_the_fire_tick() {
        let mut world = World::with_entities(
            20,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 0, 0),
                Entity::new(EntityKind::Lander, 12, 3, 0, 0),
            ],
        );

        let mut events = Vec::new();
        for _ in 0..5 {
            events = world.step_live(UpdateInput::default());
        }

        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 1);
        assert!(events.contains(&WorldEvent::EnemyFired));
    }

    #[test]
    fn live_step_scores_when_a_shot_hits_an_enemy() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 5, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 9, 2, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().score, 150);
        assert_eq!(world.enemy_count(), 1);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
    }

    #[test]
    fn live_step_removes_humans_that_are_reached_by_enemies() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 6, 5, 0, 0),
                Entity::new(EntityKind::Human, 6, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.human_count(), 0);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn live_step_enemy_shots_can_hit_the_player() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::EnemyShot, 2, 3, 1, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().lives, 0);
        assert!(world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
        assert!(events.contains(&WorldEvent::PlayerHit));
        assert!(events.contains(&WorldEvent::GameOver));
    }

    #[test]
    fn live_step_removes_enemy_shots_when_they_leave_the_world() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::EnemyShot, 0, 3, -1, 0),
            ],
        );

        world.step_live(UpdateInput::default());
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
    }

    #[test]
    fn live_step_removes_projectiles_that_hit_the_terrain() {
        let mut world = World::bootstrap();
        let player_x = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position
            .x;
        let terrain_row = world.terrain_row_at_world_x(player_x) as i32;
        world.spawn_entity(Entity::new(
            EntityKind::EnemyShot,
            player_x,
            terrain_row - 1,
            0,
            1,
        ));

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
    }

    #[test]
    fn live_step_marks_game_over_when_the_player_loses_the_last_life() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 3, 3, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().lives, 0);
        assert!(world.is_game_over());
        assert!(events.contains(&WorldEvent::PlayerHit));
        assert!(events.contains(&WorldEvent::GameOver));
    }

    #[test]
    fn live_step_advances_the_wave_when_the_last_enemy_is_removed() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 5, 3, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().wave, 2);
        assert!(world.enemy_count() >= 2);
        assert!(events.contains(&WorldEvent::WaveAdvanced));
    }

    #[test]
    fn live_step_detonates_smart_bombs_and_consumes_inventory() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 0, 0),
                Entity::new(EntityKind::Lander, 8, 3, 0, 0),
                Entity::new(EntityKind::EnemyShot, 6, 3, -1, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.smart_bombs(), DEFAULT_SMART_BOMBS - 1);
        assert_eq!(world.status().score, 150);
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
        assert!(events.contains(&WorldEvent::SmartBombDetonated));
    }

    #[test]
    fn live_step_hyperspace_moves_the_player_to_a_deterministic_location() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 0, 0),
                Entity::new(EntityKind::Lander, 12, 3, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            hyperspace: true,
            ..UpdateInput::default()
        });

        let player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(player.position, hyperspace_destination(1, 0, 19, 1, 7));
        assert!(events.contains(&WorldEvent::HyperspaceUsed));
    }

    #[test]
    fn invincible_live_step_blocks_enemy_shots_without_spending_lives() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 10, 3, 0, 0),
                Entity::new(EntityKind::EnemyShot, 2, 3, 1, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            invincible: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().lives, 1);
        assert!(!world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
        assert!(!events.contains(&WorldEvent::PlayerHit));
    }

    #[test]
    fn invincible_mode_allows_unlimited_smart_bombs() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 0, 0),
                Entity::new(EntityKind::Lander, 8, 3, 0, 0),
            ],
        );
        world.set_smart_bombs(0);

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            invincible: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.smart_bombs(), 0);
        assert_eq!(world.status().score, 150);
        assert!(events.contains(&WorldEvent::SmartBombDetonated));
    }
}
