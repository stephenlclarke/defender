use crate::arcade::arcade_tables;
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
    Baiter,
    Bomber,
    Pod,
    Swarmer,
    Mine,
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
            Self::Baiter => 'B',
            Self::Bomber => 'V',
            Self::Pod => 'P',
            Self::Swarmer => 'S',
            Self::Mine => 'x',
            Self::Human => 'h',
        }
    }

    pub fn is_enemy(self) -> bool {
        matches!(
            self,
            Self::Lander | Self::Mutant | Self::Baiter | Self::Bomber | Self::Pod | Self::Swarmer
        )
    }

    fn can_fire(self) -> bool {
        matches!(
            self,
            Self::Lander | Self::Mutant | Self::Baiter | Self::Swarmer
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityState {
    #[default]
    Normal,
    CarryingHuman,
    Abducted,
    Falling,
    PlayerCarried,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalDirection {
    Left,
    #[default]
    Right,
}

impl HorizontalDirection {
    fn reversed(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn step(self) -> i32 {
        match self {
            Self::Left => -1,
            Self::Right => 1,
        }
    }

    pub fn glyph(self) -> char {
        match self {
            Self::Left => '<',
            Self::Right => '>',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpdateInput {
    pub up: bool,
    pub down: bool,
    pub thrust: bool,
    pub reverse: bool,
    pub fire: bool,
    pub auto_fire: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub secret_mode: bool,
    pub invincible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEvent {
    ShotFired,
    EnemyFired,
    EnemyDestroyed,
    HumanLost,
    HumanRescued,
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
    pub state: EntityState,
    pub rescue_value: u16,
}

impl Entity {
    pub fn new(kind: EntityKind, x: i32, y: i32, dx: i32, dy: i32) -> Self {
        Self::with_state(kind, x, y, dx, dy, EntityState::Normal)
    }

    pub fn with_state(
        kind: EntityKind,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
        state: EntityState,
    ) -> Self {
        Self {
            kind,
            position: Position { x, y },
            velocity: Velocity { dx, dy },
            state,
            rescue_value: 0,
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
    player_facing: HorizontalDirection,
    tick: u32,
    fire_cooldown: u8,
    next_stock_award: u32,
    smart_bombs: u8,
    wave_started_at: u32,
    last_baiter_tick: u32,
    pending_wave_openers: u8,
    spawned_wave_opener_groups: u8,
    next_wave_reinforcement_tick: u32,
    game_over: bool,
    status: Status,
    terrain: Vec<usize>,
    entities: Vec<Entity>,
}

impl World {
    pub fn bootstrap() -> Self {
        let terrain = build_scrolling_terrain(WORLD_SPAN, WORLD_HEIGHT);
        let mut entities = vec![Entity::new(
            EntityKind::PlayerShip,
            PLAYER_START_X,
            PLAYER_START_Y,
            0,
            0,
        )];
        entities.extend(default_attack_wave_openers(
            WORLD_SPAN as i32,
            DEFAULT_WAVE,
            EntityKind::Lander,
            0,
        ));
        entities.extend(default_humans(&terrain));
        let tables = arcade_tables();
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            world_span: WORLD_SPAN as i32,
            camera_x: PLAYER_START_X,
            player_facing: HorizontalDirection::Right,
            tick: 0,
            fire_cooldown: 0,
            next_stock_award: next_stock_award_score(0),
            smart_bombs: DEFAULT_SMART_BOMBS,
            wave_started_at: 0,
            last_baiter_tick: 0,
            pending_wave_openers: tables.attack_wave_total_openers - tables.attack_wave_group_size,
            spawned_wave_opener_groups: 1,
            next_wave_reinforcement_tick: tables.attack_wave_reinforcement_delay,
            game_over: false,
            status: Status {
                score: 0,
                lives: DEFAULT_LIVES,
                wave: DEFAULT_WAVE,
            },
            terrain,
            entities,
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
            player_facing: HorizontalDirection::Right,
            tick: 0,
            fire_cooldown: 0,
            next_stock_award: next_stock_award_score(status.score),
            smart_bombs: DEFAULT_SMART_BOMBS,
            wave_started_at: 0,
            last_baiter_tick: 0,
            pending_wave_openers: 0,
            spawned_wave_opener_groups: 0,
            next_wave_reinforcement_tick: 0,
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
                EntityKind::Lander
                | EntityKind::Mutant
                | EntityKind::Baiter
                | EntityKind::Bomber
                | EntityKind::Pod
                | EntityKind::Swarmer => {
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
                EntityKind::Mine => {}
            }
        }

        self.retain_projectiles(max_x, min_y, max_y);
        self.spawn_attack_wave_reinforcements_if_due();
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

    pub fn player_facing(&self) -> HorizontalDirection {
        self.player_facing
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

    fn remaining_wave_enemy_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|entity| entity.kind.is_enemy() && entity.kind != EntityKind::Baiter)
            .count()
            + usize::from(self.pending_wave_openers)
    }

    pub fn human_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|entity| {
                entity.kind == EntityKind::Human && entity.state != EntityState::Abducted
            })
            .count()
    }

    pub fn planet_destroyed(&self) -> bool {
        !self.has_humanoids()
    }

    pub fn threat_score(&self) -> usize {
        let humans: Vec<Position> = self
            .entities
            .iter()
            .filter(|entity| {
                entity.kind == EntityKind::Human
                    && entity.state != EntityState::Abducted
                    && entity.state != EntityState::PlayerCarried
            })
            .map(|entity| entity.position)
            .collect();

        self.entities
            .iter()
            .filter(|entity| entity.kind.is_enemy() || entity.kind == EntityKind::Mine)
            .filter(|entity| {
                entity.kind == EntityKind::Mine
                    || entity.position.y >= self.safe_altitude_at_world_x(entity.position.x) - 3
                    || humans.iter().any(|human| {
                        (entity.position.x - human.x).abs() <= 6
                            && (entity.position.y - human.y).abs() <= 4
                    })
            })
            .count()
    }

    pub fn add_score(&mut self, delta: u32) {
        let tables = arcade_tables();
        self.status.score = self.status.score.saturating_add(delta);
        while self.status.score >= self.next_stock_award {
            self.status.lives = self.status.lives.saturating_add(1);
            self.smart_bombs = self.smart_bombs.saturating_add(1);
            self.next_stock_award = self
                .next_stock_award
                .saturating_add(tables.bonus_stock_score);
        }
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

        if input.reverse {
            self.player_facing = self.player_facing.reversed();
        }

        let hyperspace_result = input.hyperspace.then(|| {
            if input.secret_mode {
                HyperspaceResult {
                    position: self.safest_hyperspace_destination(max_x, min_y, max_y),
                    facing: self.player_facing,
                    explodes: false,
                }
            } else {
                hyperspace_result(
                    self.tick,
                    self.status.score,
                    self.player_facing,
                    max_x,
                    min_y,
                    max_y,
                )
            }
        });

        let mut shot_origin = None;
        let mut hyperspaced_this_tick = false;
        let mut hyperspace_failure_check = None;
        {
            let terrain = &self.terrain;
            if let Some(player) = self
                .entities
                .iter_mut()
                .find(|entity| entity.kind == EntityKind::PlayerShip)
            {
                let dy = input.down as i32 - input.up as i32;
                if let Some(result) = hyperspace_result {
                    player.position = result.position;
                    player.velocity = Velocity { dx: 0, dy: 0 };
                    self.player_facing = result.facing;
                    player.position.y = player
                        .position
                        .y
                        .min(terrain_surface_y(terrain, player.position.x));
                    hyperspaced_this_tick = true;
                    events.push(WorldEvent::HyperspaceUsed);
                    shot_origin = Some(player.position);
                    if !input.secret_mode {
                        hyperspace_failure_check = Some((player.position, result.explodes));
                    }
                } else {
                    if input.thrust {
                        let tables = arcade_tables();
                        player.velocity.dx = (player.velocity.dx + self.player_facing.step())
                            .clamp(-tables.player_max_speed, tables.player_max_speed);
                    }
                    player.position.x =
                        wrap_coordinate(player.position.x + player.velocity.dx, max_x);
                    player.position.y = (player.position.y + dy).clamp(min_y, max_y);
                    player.position.y = player
                        .position
                        .y
                        .min(terrain_surface_y(terrain, player.position.x));
                    shot_origin = Some(player.position);
                }
            }
        }

        if let Some((player_position, explodes)) = hyperspace_failure_check
            && (explodes || self.hyperspace_destination_is_unsafe(player_position))
        {
            self.lose_player_life(player_position, &mut events);
        }

        if input.smart_bomb && self.can_use_smart_bomb(input.secret_mode) {
            self.detonate_smart_bomb(input.secret_mode, &mut events);
        }

        self.update_enemy_intents(min_y, max_y);
        self.begin_human_abductions();

        let auto_fire = input.secret_mode
            && input.auto_fire
            && !hyperspaced_this_tick
            && self.fire_cooldown == 0
            && shot_origin.is_some_and(|origin| self.should_auto_fire(origin, max_x, min_y, max_y));
        if (input.fire || auto_fire)
            && !hyperspaced_this_tick
            && self.fire_cooldown == 0
            && let Some(origin) = shot_origin
        {
            let shot_dx = self.player_facing.step() * 2;
            self.entities.push(Entity::new(
                EntityKind::PlayerShot,
                wrap_coordinate(origin.x + self.player_facing.step(), max_x),
                origin.y,
                shot_dx,
                0,
            ));
            self.fire_cooldown = 2;
            events.push(WorldEvent::ShotFired);
        }

        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::Human | EntityKind::PlayerShip => {}
                EntityKind::Lander
                | EntityKind::Mutant
                | EntityKind::Baiter
                | EntityKind::Bomber
                | EntityKind::Pod
                | EntityKind::Swarmer => {
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
                EntityKind::Mine => {}
            }
        }

        self.drop_bomber_mines(min_y, max_y);
        self.spawn_enemy_fire(max_x, min_y, max_y, &mut events);
        self.retain_projectiles(max_x, min_y, max_y);

        self.update_abducted_humans(min_y, &mut events);
        self.update_falling_humans(max_y);
        self.handle_player_human_interactions(&mut events);
        self.resolve_falling_human_impacts(input.secret_mode, &mut events);
        self.handle_human_losses(&mut events);
        self.handle_player_shot_hits(&mut events);
        self.handle_player_collisions(
            input.invincible,
            hyperspaced_this_tick && input.secret_mode,
            &mut events,
        );
        self.mutate_landers_if_humans_extinct();
        self.clear_baiters_if_landers_gone();
        self.spawn_attack_wave_reinforcements_if_due();

        self.spawn_baiter_if_needed(min_y, max_y);

        if !self.game_over
            && self.remaining_wave_enemy_count() == 0
            && !self.has_unresolved_humans()
        {
            self.add_score(self.wave_humanoid_bonus());
            self.status.wave = self.status.wave.saturating_add(1);
            self.clear_wave_carryover_entities();
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
            .filter(|entity| entity.kind.can_fire())
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

    fn update_enemy_intents(&mut self, min_y: i32, max_y: i32) {
        let player_position = self.player_position();
        let player_velocity = self.player_velocity();
        let free_humans = self.free_human_positions();
        let world_span = self.world_span;
        let world_max_x = self.world_max_x();
        let swarmer_follow_distance = (self.width as i32 / 2).max(8);

        for enemy in self
            .entities
            .iter_mut()
            .filter(|entity| entity.kind.is_enemy())
        {
            match enemy.kind {
                EntityKind::Lander if enemy.state == EntityState::CarryingHuman => {
                    enemy.velocity.dx = 0;
                    enemy.velocity.dy = -1;
                }
                EntityKind::Lander => {
                    let target = nearest_wrapped_target(enemy.position, &free_humans, world_span)
                        .or(player_position)
                        .unwrap_or(enemy.position);
                    enemy.velocity.dx =
                        wrapped_horizontal_step(enemy.position.x, target.x, world_max_x);
                    enemy.velocity.dy = (target.y - enemy.position.y).signum();
                    if enemy.velocity.dy == 0 && enemy.position.y <= min_y {
                        enemy.velocity.dy = 1;
                    }
                }
                EntityKind::Mutant => {
                    let target = player_position.unwrap_or(enemy.position);
                    enemy.velocity.dx =
                        wrapped_horizontal_step(enemy.position.x, target.x, world_max_x);
                    enemy.velocity.dy = (target.y - enemy.position.y).signum();
                    if enemy.velocity.dy == 0 {
                        enemy.velocity.dy = if self.tick.is_multiple_of(2) { -1 } else { 1 };
                    }
                }
                EntityKind::Baiter => {
                    let tables = arcade_tables();
                    let target = player_position.unwrap_or(enemy.position);
                    let relative_dx =
                        wrapped_horizontal_step(enemy.position.x, target.x, world_max_x)
                            * tables.baiter_speed;
                    let inherited_dx = player_velocity.map(|velocity| velocity.dx).unwrap_or(0);
                    enemy.velocity.dx = (inherited_dx + relative_dx)
                        .clamp(-(tables.baiter_speed + 1), tables.baiter_speed + 1);
                    if enemy.velocity.dx == 0 {
                        enemy.velocity.dx = if self.tick.is_multiple_of(2) {
                            tables.baiter_speed
                        } else {
                            -tables.baiter_speed
                        };
                    }
                    enemy.velocity.dy = (target.y - enemy.position.y).signum();
                }
                EntityKind::Bomber => {
                    let tables = arcade_tables();
                    let target_y = player_position
                        .map(|target| target.y)
                        .unwrap_or(enemy.position.y);
                    let direction = match enemy.velocity.dx.signum() {
                        -1 | 1 => enemy.velocity.dx.signum(),
                        _ => 1,
                    };
                    enemy.velocity.dx = direction
                        * if target_y == enemy.position.y {
                            tables.bomber_evasive_speed
                        } else {
                            tables.bomber_base_speed
                        };
                    if target_y == enemy.position.y {
                        enemy.velocity.dy = 0;
                    } else if (self.tick + enemy.position.x as u32).is_multiple_of(3) {
                        enemy.velocity.dy = (target_y - enemy.position.y).signum();
                    }
                }
                EntityKind::Pod => {
                    if enemy.velocity.dx == 0 {
                        enemy.velocity.dx = if self.tick.is_multiple_of(2) { 1 } else { -1 };
                    }
                    if enemy.velocity.dy == 0 {
                        enemy.velocity.dy = if self.tick.is_multiple_of(3) { 1 } else { -1 };
                    }
                }
                EntityKind::Swarmer => {
                    let tables = arcade_tables();
                    let target = player_position.unwrap_or(enemy.position);
                    let delta_x = shortest_wrapped_delta(enemy.position.x, target.x, world_span);
                    let desired_dx = if delta_x == 0 {
                        enemy.velocity.dx.signum().max(1) * tables.swarmer_speed
                    } else {
                        delta_x.signum() * tables.swarmer_speed
                    };
                    let current_direction = enemy.velocity.dx.signum();
                    let should_reverse = current_direction == 0
                        || (desired_dx.signum() != current_direction
                            && delta_x.abs() > swarmer_follow_distance);
                    if should_reverse {
                        enemy.velocity.dx = desired_dx;
                    }

                    let vertical_delta = target.y - enemy.position.y;
                    if vertical_delta != 0
                        && (self.tick + enemy.position.x as u32).is_multiple_of(2)
                    {
                        enemy.velocity.dy = vertical_delta.signum();
                    }
                    if enemy.velocity.dy == 0 {
                        enemy.velocity.dy = if self.tick.is_multiple_of(2) { 1 } else { -1 };
                    }
                }
                _ => {}
            }

            enemy.velocity.dy = enemy.velocity.dy.clamp(-1, 1);
            enemy.position.y = enemy.position.y.clamp(min_y, max_y);
        }
    }

    fn drop_bomber_mines(&mut self, min_y: i32, max_y: i32) {
        let tables = arcade_tables();
        if !self.tick.is_multiple_of(tables.bomber_mine_drop_delay)
            || self.entity_count_by_kind(EntityKind::Mine) >= tables.max_mines
        {
            return;
        }

        let max_x = self.world_max_x();
        let mut new_mines = Vec::new();

        for bomber in self
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Bomber)
        {
            let mine_x = wrap_coordinate(bomber.position.x - bomber.velocity.dx.signum(), max_x);
            let mine_position = Position {
                x: mine_x,
                y: bomber
                    .position
                    .y
                    .clamp(min_y, self.safe_altitude_at_world_x(mine_x).min(max_y)),
            };

            let already_present =
                self.entities.iter().any(|entity| {
                    entity.kind == EntityKind::Mine && entity.position == mine_position
                }) || new_mines
                    .iter()
                    .any(|entity: &Entity| entity.position == mine_position);
            if !already_present {
                new_mines.push(Entity::new(
                    EntityKind::Mine,
                    mine_position.x,
                    mine_position.y,
                    0,
                    0,
                ));
            }
        }

        let available = tables
            .max_mines
            .saturating_sub(self.entity_count_by_kind(EntityKind::Mine));
        self.entities.extend(new_mines.into_iter().take(available));
    }

    fn free_human_positions(&self) -> Vec<Position> {
        self.entities
            .iter()
            .filter(|entity| {
                entity.kind == EntityKind::Human && entity.state == EntityState::Normal
            })
            .map(|entity| entity.position)
            .collect()
    }

    fn begin_human_abductions(&mut self) {
        let mut claimed_humans = Vec::new();

        for lander_index in 0..self.entities.len() {
            if self.entities[lander_index].kind != EntityKind::Lander
                || self.entities[lander_index].state != EntityState::Normal
            {
                continue;
            }

            let lander_position = self.entities[lander_index].position;
            let Some((human_index, _)) =
                self.entities
                    .iter()
                    .enumerate()
                    .find(|(human_index, entity)| {
                        entity.kind == EntityKind::Human
                            && entity.state == EntityState::Normal
                            && !claimed_humans.contains(human_index)
                            && positions_overlap(lander_position, entity.position, 1, 1)
                    })
            else {
                continue;
            };

            claimed_humans.push(human_index);
            self.entities[lander_index].state = EntityState::CarryingHuman;
            self.entities[lander_index].velocity.dx = 0;
            self.entities[lander_index].velocity.dy = -1;
            self.entities[human_index].state = EntityState::Abducted;
            self.entities[human_index].position.x = lander_position.x;
            self.entities[human_index].position.y = lander_position.y + 1;
            self.entities[human_index].velocity = Velocity { dx: 0, dy: -1 };
        }
    }

    fn update_abducted_humans(&mut self, min_y: i32, events: &mut Vec<WorldEvent>) {
        let mut mutated_landers = Vec::new();

        for lander_index in 0..self.entities.len() {
            if self.entities[lander_index].kind != EntityKind::Lander
                || self.entities[lander_index].state != EntityState::CarryingHuman
            {
                continue;
            }

            let lander_position = self.entities[lander_index].position;
            let Some(human_index) = self.find_abducted_human_near(lander_position) else {
                self.entities[lander_index].state = EntityState::Normal;
                continue;
            };

            self.entities[human_index].position.x = lander_position.x;
            self.entities[human_index].position.y = (lander_position.y + 1).max(min_y);
            self.entities[human_index].velocity = Velocity { dx: 0, dy: -1 };

            if lander_position.y <= min_y + 1 {
                mutated_landers.push((lander_index, human_index));
            }
        }

        let mut lost_humans = Vec::new();
        for (lander_index, human_index) in mutated_landers {
            if let Some(lander) = self.entities.get_mut(lander_index) {
                lander.kind = EntityKind::Mutant;
                lander.state = EntityState::Normal;
                lander.velocity.dy = 1;
            }
            lost_humans.push(human_index);
            events.push(WorldEvent::HumanLost);
        }

        remove_indices(&mut self.entities, &lost_humans);
    }

    fn handle_human_losses(&mut self, events: &mut Vec<WorldEvent>) {
        let mut human_indices = Vec::new();

        for enemy in self.entities.iter().filter(|entity| entity.kind.is_enemy()) {
            if let Some((index, _)) = self.entities.iter().enumerate().find(|(index, entity)| {
                entity.kind == EntityKind::Human
                    && matches!(entity.state, EntityState::Normal | EntityState::Falling)
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
        let mut released_carriers = Vec::new();
        let mut burst_pods = Vec::new();

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
                    if enemy.kind == EntityKind::Lander && enemy.state == EntityState::CarryingHuman
                    {
                        released_carriers.push(enemy.position);
                    } else if enemy.kind == EntityKind::Pod {
                        burst_pods.push(enemy.position);
                    }
                }
            }
        }

        for carrier_position in released_carriers {
            self.release_abducted_human_at(carrier_position);
        }
        self.add_score(score_delta);
        remove_indices(&mut self.entities, &remove_indices_set);
        for pod_position in burst_pods {
            self.spawn_swarmer_burst_at(pod_position);
        }
    }

    fn handle_player_collisions(
        &mut self,
        invincible: bool,
        collision_immunity: bool,
        events: &mut Vec<WorldEvent>,
    ) {
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
                EntityKind::Lander
                | EntityKind::Mutant
                | EntityKind::Baiter
                | EntityKind::Bomber
                | EntityKind::Pod
                | EntityKind::Swarmer => positions_overlap(player_position, enemy.position, 1, 1),
                EntityKind::Mine => positions_overlap(player_position, enemy.position, 0, 0),
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

        if collision_immunity {
            return;
        }

        if invincible {
            let removable_collisions: Vec<usize> = collided_enemies
                .iter()
                .copied()
                .filter(|index| {
                    self.entities
                        .get(*index)
                        .is_some_and(|entity| entity.kind != EntityKind::Mine)
                })
                .collect();
            let released_carriers: Vec<Position> = removable_collisions
                .iter()
                .filter_map(|index| self.entities.get(*index))
                .filter(|entity| {
                    entity.kind == EntityKind::Lander && entity.state == EntityState::CarryingHuman
                })
                .map(|entity| entity.position)
                .collect();
            let score_delta = removable_collisions
                .iter()
                .filter_map(|index| self.entities.get(*index))
                .map(|entity| score_for_enemy(entity.kind))
                .sum();
            for carrier_position in released_carriers {
                self.release_abducted_human_at(carrier_position);
            }
            remove_indices(&mut self.entities, &removable_collisions);
            self.add_score(score_delta);
            if score_delta > 0 {
                events.push(WorldEvent::EnemyDestroyed);
            }
            return;
        }

        let released_carriers: Vec<Position> = collided_enemies
            .iter()
            .filter_map(|index| self.entities.get(*index))
            .filter(|entity| {
                entity.kind == EntityKind::Lander && entity.state == EntityState::CarryingHuman
            })
            .map(|entity| entity.position)
            .collect();
        let score_delta: u32 = collided_enemies
            .iter()
            .filter_map(|index| self.entities.get(*index))
            .map(|entity| score_for_player_collision(entity.kind))
            .sum();
        remove_indices(&mut self.entities, &collided_enemies);
        for carrier_position in released_carriers {
            self.release_abducted_human_at(carrier_position);
        }
        self.add_score(score_delta);
        self.lose_player_life(player_position, events);
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
            player.velocity.dx = 0;
            player.velocity.dy = 0;
        }
        self.player_facing = HorizontalDirection::Right;
        self.sync_camera_to_player();
    }

    fn player_position(&self) -> Option<Position> {
        self.entities
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .map(|entity| entity.position)
    }

    fn player_velocity(&self) -> Option<Velocity> {
        self.entities
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .map(|entity| entity.velocity)
    }

    fn should_auto_fire(&self, origin: Position, max_x: i32, min_y: i32, max_y: i32) -> bool {
        let shot_position = Position {
            x: wrap_coordinate(origin.x + self.player_facing.step() * 3, max_x),
            y: origin.y,
        };

        self.entities
            .iter()
            .filter(|entity| entity.kind.is_enemy())
            .map(|entity| self.project_enemy_position_after_movement(entity, max_x, min_y, max_y))
            .any(|enemy_position| positions_overlap(shot_position, enemy_position, 1, 0))
    }

    fn project_enemy_position_after_movement(
        &self,
        entity: &Entity,
        max_x: i32,
        min_y: i32,
        max_y: i32,
    ) -> Position {
        let x = wrap_coordinate(entity.position.x + entity.velocity.dx, max_x);
        let mut y = entity.position.y + entity.velocity.dy;
        let surface = terrain_surface_y(&self.terrain, x);
        if y <= min_y || y >= max_y.min(surface) {
            y = y.clamp(min_y, max_y.min(surface));
        }

        Position { x, y }
    }

    fn hyperspace_destination_is_unsafe(&self, player_position: Position) -> bool {
        self.entities.iter().any(|entity| match entity.kind {
            EntityKind::Lander
            | EntityKind::Mutant
            | EntityKind::Baiter
            | EntityKind::Bomber
            | EntityKind::Pod
            | EntityKind::Swarmer => positions_overlap(player_position, entity.position, 1, 1),
            EntityKind::Mine => positions_overlap(player_position, entity.position, 0, 0),
            EntityKind::EnemyShot => positions_overlap(player_position, entity.position, 0, 0),
            _ => false,
        })
    }

    fn lose_player_life(&mut self, player_position: Position, events: &mut Vec<WorldEvent>) {
        self.release_player_carried_human_at(player_position);
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

    fn safest_hyperspace_destination(&self, max_x: i32, min_y: i32, max_y: i32) -> Position {
        let fallback = hyperspace_destination(self.tick, self.status.score, max_x, min_y, max_y);
        let hazards: Vec<Position> = self
            .entities
            .iter()
            .filter(|entity| {
                entity.kind == EntityKind::Lander
                    || entity.kind == EntityKind::Mutant
                    || entity.kind == EntityKind::Bomber
                    || entity.kind == EntityKind::Pod
                    || entity.kind == EntityKind::Swarmer
                    || entity.kind == EntityKind::Mine
                    || entity.kind == EntityKind::EnemyShot
            })
            .map(|entity| entity.position)
            .collect();

        if hazards.is_empty() {
            return fallback;
        }

        let span = max_x + 1;
        let mut best = fallback;
        let mut best_score = i32::MIN;
        let mut best_fallback_delta = i32::MAX;

        for x in 0..=max_x {
            let surface = terrain_surface_y(&self.terrain, x);
            let max_candidate_y = max_y.min(surface);
            for y in min_y..=max_candidate_y {
                let candidate = Position { x, y };
                let hazard_distance = hazards
                    .iter()
                    .map(|hazard| {
                        shortest_wrapped_delta(candidate.x, hazard.x, span).abs() * 2
                            + (candidate.y - hazard.y).abs()
                    })
                    .min()
                    .unwrap_or(i32::MAX);
                let fallback_delta = shortest_wrapped_delta(candidate.x, fallback.x, span).abs()
                    + (candidate.y - fallback.y).abs();

                if hazard_distance > best_score
                    || (hazard_distance == best_score && fallback_delta < best_fallback_delta)
                    || (hazard_distance == best_score
                        && fallback_delta == best_fallback_delta
                        && (candidate.x, candidate.y) < (best.x, best.y))
                {
                    best = candidate;
                    best_score = hazard_distance;
                    best_fallback_delta = fallback_delta;
                }
            }
        }

        best
    }

    fn find_abducted_human_near(&self, carrier_position: Position) -> Option<usize> {
        self.entities
            .iter()
            .enumerate()
            .find(|(_, entity)| {
                entity.kind == EntityKind::Human
                    && entity.state == EntityState::Abducted
                    && positions_overlap(carrier_position, entity.position, 1, 2)
            })
            .map(|(index, _)| index)
    }

    fn release_abducted_human_at(&mut self, carrier_position: Position) {
        let Some(human_index) = self.find_abducted_human_near(carrier_position) else {
            return;
        };

        let safe_y = self.safe_altitude_at_world_x(self.entities[human_index].position.x);
        let tables = arcade_tables();
        if let Some(human) = self.entities.get_mut(human_index) {
            human.state = EntityState::Falling;
            human.velocity = Velocity { dx: 0, dy: 1 };
            human.rescue_value = if safe_y - human.position.y <= tables.safe_fall_height {
                tables.safe_fall_score
            } else {
                0
            };
        }
    }

    fn spawn_swarmer_burst_at(&mut self, pod_position: Position) {
        let tables = arcade_tables();
        let available = tables
            .max_swarmers
            .saturating_sub(self.entity_count_by_kind(EntityKind::Swarmer));
        if available == 0 {
            return;
        }

        let count = self.pod_swarmer_burst_count(pod_position).min(available);
        let player_position = self.player_position();
        let direction = player_position
            .map(|player| shortest_wrapped_delta(pod_position.x, player.x, self.world_span))
            .map(|delta| if delta == 0 { 1 } else { delta.signum() })
            .unwrap_or(1);
        let spawn_offsets = [
            (0, -1),
            (direction, 0),
            (0, 1),
            (-direction, 0),
            (direction, -1),
            (direction, 1),
            (-2 * direction, 0),
        ];
        let max_x = self.world_max_x();
        let min_y = 1;

        for &(horizontal_offset, vertical_offset) in spawn_offsets.iter().take(count) {
            let x = wrap_coordinate(pod_position.x + horizontal_offset, max_x);
            let safe_y = self.safe_altitude_at_world_x(x);
            let y = (pod_position.y + vertical_offset).clamp(min_y, safe_y);
            let dy = player_position
                .map(|player| (player.y - y).signum())
                .filter(|delta| *delta != 0)
                .unwrap_or(vertical_offset.signum());
            self.entities.push(Entity::new(
                EntityKind::Swarmer,
                x,
                y,
                direction * tables.swarmer_speed,
                dy,
            ));
        }
    }

    fn pod_swarmer_burst_count(&self, pod_position: Position) -> usize {
        let tables = arcade_tables();
        let variance =
            ((self.tick as usize) + (pod_position.x as usize) + (pod_position.y as usize))
                % tables.pod_swarmer_burst_range;
        tables.pod_swarmer_burst_min + variance
    }

    fn can_use_smart_bomb(&self, secret_mode: bool) -> bool {
        secret_mode || self.smart_bombs > 0
    }

    fn detonate_smart_bomb(&mut self, secret_mode: bool, events: &mut Vec<WorldEvent>) {
        if !self.can_use_smart_bomb(secret_mode) {
            return;
        }

        if !secret_mode {
            self.smart_bombs -= 1;
        }

        let mut remove_indices_set = Vec::new();
        let mut score_delta = 0;
        let mut released_carriers = Vec::new();

        for (index, entity) in self.entities.iter().enumerate() {
            let on_main_screen = self.screen_x_for_world_x(entity.position.x).is_some();
            let destroyed_by_bomb = if secret_mode {
                on_main_screen
                    && (entity.kind.is_enemy()
                        || matches!(entity.kind, EntityKind::EnemyShot | EntityKind::Mine))
            } else {
                on_main_screen && entity.kind.is_enemy()
            };

            if destroyed_by_bomb {
                remove_indices_set.push(index);
                score_delta += score_for_enemy(entity.kind);
                if entity.kind == EntityKind::Lander && entity.state == EntityState::CarryingHuman {
                    released_carriers.push(entity.position);
                }
            }
        }

        for carrier_position in released_carriers {
            self.release_abducted_human_at(carrier_position);
        }
        remove_indices(&mut self.entities, &remove_indices_set);
        self.add_score(score_delta);
        events.push(WorldEvent::SmartBombDetonated);
    }

    fn update_falling_humans(&mut self, max_y: i32) {
        for human in self.entities.iter_mut().filter(|entity| {
            entity.kind == EntityKind::Human && entity.state == EntityState::Falling
        }) {
            human.position.y = (human.position.y + human.velocity.dy.max(1)).min(max_y);
        }
    }

    fn handle_player_human_interactions(&mut self, events: &mut Vec<WorldEvent>) {
        let Some(player_position) = self.player_position() else {
            return;
        };

        if let Some(human_index) = self.find_player_carried_human() {
            let safe_y = self.safe_altitude_at_world_x(player_position.x);
            let should_land = player_position.y >= safe_y - 1;
            let carried_y = (player_position.y + 1).min(safe_y);
            let mut landing_bonus = 0;
            if let Some(human) = self.entities.get_mut(human_index) {
                human.position.x = player_position.x;
                human.position.y = if should_land { safe_y } else { carried_y };
                human.velocity = Velocity { dx: 0, dy: 0 };
                if should_land {
                    human.state = EntityState::Normal;
                    landing_bonus = human.rescue_value;
                    human.rescue_value = 0;
                }
            }
            if should_land {
                self.add_score(u32::from(landing_bonus));
                events.push(WorldEvent::HumanRescued);
            }
            return;
        }

        let Some(human_index) = self.find_collectable_human(player_position) else {
            return;
        };

        let rescue_bonus = if self.entities[human_index].state == EntityState::Falling {
            arcade_tables().human_catch_score
        } else {
            0
        };
        let carried_y =
            (player_position.y + 1).min(self.safe_altitude_at_world_x(player_position.x));
        if let Some(human) = self.entities.get_mut(human_index) {
            human.state = EntityState::PlayerCarried;
            human.position.x = player_position.x;
            human.position.y = carried_y;
            human.velocity = Velocity { dx: 0, dy: 0 };
            human.rescue_value = if rescue_bonus > 0 {
                arcade_tables().human_landing_score
            } else {
                0
            };
        }
        if rescue_bonus > 0 {
            self.add_score(rescue_bonus);
        }
    }

    fn resolve_falling_human_impacts(&mut self, secret_mode: bool, events: &mut Vec<WorldEvent>) {
        let grounded_humans: Vec<usize> = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                entity.kind == EntityKind::Human && entity.state == EntityState::Falling
            })
            .filter(|(_, entity)| {
                entity.position.y >= self.safe_altitude_at_world_x(entity.position.x)
            })
            .map(|(index, _)| index)
            .collect();

        if grounded_humans.is_empty() {
            return;
        }

        let mut lost_humans = Vec::new();
        let mut rescued_count = 0;
        let mut rescued_score = 0;

        for index in grounded_humans {
            let safe_y = self.safe_altitude_at_world_x(self.entities[index].position.x);
            let Some(human) = self.entities.get_mut(index) else {
                continue;
            };
            if secret_mode || human.rescue_value == arcade_tables().safe_fall_score {
                human.state = EntityState::Normal;
                human.position.y = safe_y;
                human.velocity = Velocity { dx: 0, dy: 0 };
                human.rescue_value = 0;
                rescued_count += 1;
                rescued_score += u32::from(arcade_tables().safe_fall_score);
            } else {
                lost_humans.push(index);
            }
        }

        if rescued_count > 0 {
            self.add_score(rescued_score);
            events.extend(std::iter::repeat_n(WorldEvent::HumanRescued, rescued_count));
        }
        if !lost_humans.is_empty() {
            events.extend(std::iter::repeat_n(
                WorldEvent::HumanLost,
                lost_humans.len(),
            ));
            remove_indices(&mut self.entities, &lost_humans);
        }
    }

    fn find_player_carried_human(&self) -> Option<usize> {
        self.entities
            .iter()
            .enumerate()
            .find(|(_, entity)| {
                entity.kind == EntityKind::Human && entity.state == EntityState::PlayerCarried
            })
            .map(|(index, _)| index)
    }

    fn find_collectable_human(&self, player_position: Position) -> Option<usize> {
        self.entities
            .iter()
            .enumerate()
            .find(|(_, entity)| {
                entity.kind == EntityKind::Human
                    && matches!(entity.state, EntityState::Normal | EntityState::Falling)
                    && positions_overlap(player_position, entity.position, 1, 1)
            })
            .map(|(index, _)| index)
    }

    fn release_player_carried_human_at(&mut self, player_position: Position) {
        let Some(human_index) = self.find_player_carried_human() else {
            return;
        };

        if let Some(human) = self.entities.get_mut(human_index) {
            human.state = EntityState::Falling;
            human.position.x = player_position.x;
            human.position.y = player_position.y + 1;
            human.velocity = Velocity { dx: 0, dy: 1 };
            if human.rescue_value > 0 {
                human.rescue_value = arcade_tables().safe_fall_score;
            }
        }
    }

    fn has_unresolved_humans(&self) -> bool {
        self.entities.iter().any(|entity| {
            entity.kind == EntityKind::Human
                && matches!(
                    entity.state,
                    EntityState::Abducted | EntityState::Falling | EntityState::PlayerCarried
                )
        })
    }

    fn wave_humanoid_bonus(&self) -> u32 {
        let per_humanoid =
            (u32::from(self.status.wave) * 100).min(arcade_tables().max_wave_humanoid_bonus);
        let survivors = self
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Human)
            .count() as u32;
        per_humanoid.saturating_mul(survivors)
    }

    fn mutate_landers_if_humans_extinct(&mut self) {
        if self.has_humanoids() {
            return;
        }

        for lander in self
            .entities
            .iter_mut()
            .filter(|entity| entity.kind == EntityKind::Lander)
        {
            lander.kind = EntityKind::Mutant;
            lander.state = EntityState::Normal;
            if lander.velocity.dy == 0 {
                lander.velocity.dy = 1;
            }
        }
    }

    fn clear_baiters_if_landers_gone(&mut self) {
        if self.entity_count_by_kind(EntityKind::Lander) > 0 {
            return;
        }

        self.entities
            .retain(|entity| entity.kind != EntityKind::Baiter);
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

    fn spawn_baiter_if_needed(&mut self, min_y: i32, max_y: i32) {
        let tables = arcade_tables();
        if self.entity_count_by_kind(EntityKind::Baiter) >= tables.max_baiters
            || self.entity_count_by_kind(EntityKind::Lander) == 0
        {
            return;
        }

        let remaining = self.remaining_wave_enemy_count();
        if remaining == 0 {
            return;
        }

        let elapsed = self.tick.saturating_sub(self.wave_started_at);
        let since_last_baiter = self.tick.saturating_sub(self.last_baiter_tick);
        let due_tick = tables.baiter_base_delay + remaining.saturating_sub(1) as u32 * 4;
        if elapsed < due_tick || since_last_baiter < tables.baiter_repeat_delay {
            return;
        }

        let Some(player_position) = self.player_position() else {
            return;
        };

        let phase = (self.tick / tables.baiter_repeat_delay) as i32;
        let horizontal_offset = self.width as i32 / 2 + (phase % 9);
        let x = if phase % 2 == 0 {
            wrap_coordinate(player_position.x + horizontal_offset, self.world_max_x())
        } else {
            wrap_coordinate(player_position.x - horizontal_offset, self.world_max_x())
        };
        let safe_y = self.safe_altitude_at_world_x(x).min(max_y);
        let vertical_offset = if phase % 3 == 0 { -2 } else { 2 };
        let y = (player_position.y + vertical_offset).clamp(min_y, safe_y);

        self.entities
            .push(Entity::new(EntityKind::Baiter, x, y, 0, 0));
        self.last_baiter_tick = self.tick;
    }

    fn clear_wave_carryover_entities(&mut self) {
        self.entities.retain(|entity| {
            !matches!(
                entity.kind,
                EntityKind::PlayerShot | EntityKind::EnemyShot | EntityKind::Baiter
            )
        });
    }

    fn spawn_wave(&mut self) {
        let tables = arcade_tables();
        let width = self.world_span;
        let bomber_origin = self
            .player_position()
            .map(|player| wrap_coordinate(player.x + width / 2, width - 1))
            .unwrap_or(width / 2);
        if self.status.wave.is_multiple_of(5) {
            self.restore_default_humans();
        }

        let opening_enemy = if self.has_humanoids() {
            EntityKind::Lander
        } else {
            EntityKind::Mutant
        };
        let mut enemies = default_attack_wave_openers(width, self.status.wave, opening_enemy, 0);

        if self.status.wave >= 2 {
            enemies.push(Entity::new(
                EntityKind::Bomber,
                bomber_origin,
                self.safe_altitude_at_world_x(bomber_origin)
                    .saturating_sub(1),
                -tables.bomber_base_speed,
                0,
            ));
        }
        if self.status.wave >= 3 {
            let second_bomber_x = wrap_coordinate(bomber_origin + 10, width - 1);
            enemies.push(Entity::new(
                EntityKind::Bomber,
                second_bomber_x,
                self.safe_altitude_at_world_x(second_bomber_x)
                    .saturating_sub(1),
                tables.bomber_base_speed,
                0,
            ));
        }
        if self.status.wave >= 5 {
            let third_bomber_x = wrap_coordinate(bomber_origin - 10, width - 1);
            enemies.push(Entity::new(
                EntityKind::Bomber,
                third_bomber_x,
                self.safe_altitude_at_world_x(third_bomber_x)
                    .saturating_sub(1),
                -tables.bomber_base_speed,
                0,
            ));
        }

        let pod_count = match self.status.wave {
            0 | 1 => 0,
            2 => 1,
            3 => 3,
            _ => 4,
        };
        let pod_slots = [
            (width / 2, 5, -1, 1),
            (wrap_coordinate(width - 20, width - 1), 4, 1, -1),
            (wrap_coordinate(bomber_origin + 18, width - 1), 6, -1, -1),
            (wrap_coordinate(bomber_origin - 18, width - 1), 4, 1, 1),
        ];
        for (x, desired_y, dx, dy) in pod_slots.into_iter().take(pod_count) {
            let y = desired_y.min(self.safe_altitude_at_world_x(x)).max(1);
            enemies.push(Entity::new(EntityKind::Pod, x, y, dx, dy));
        }

        self.entities.extend(enemies);
        self.wave_started_at = self.tick;
        self.last_baiter_tick = self.tick;
        self.pending_wave_openers =
            tables.attack_wave_total_openers - tables.attack_wave_group_size;
        self.spawned_wave_opener_groups = 1;
        self.next_wave_reinforcement_tick = self.tick + tables.attack_wave_reinforcement_delay;
    }

    fn has_humanoids(&self) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.kind == EntityKind::Human)
    }

    fn restore_default_humans(&mut self) {
        self.entities
            .retain(|entity| entity.kind != EntityKind::Human);
        self.entities.extend(default_humans(&self.terrain));
    }

    fn spawn_attack_wave_reinforcements_if_due(&mut self) {
        if self.pending_wave_openers == 0 || self.tick < self.next_wave_reinforcement_tick {
            return;
        }

        let tables = arcade_tables();
        let opening_enemy = if self.has_humanoids() {
            EntityKind::Lander
        } else {
            EntityKind::Mutant
        };
        let group_index = self.spawned_wave_opener_groups;
        let group_size = tables.attack_wave_group_size.min(self.pending_wave_openers);
        let mut reinforcements = default_attack_wave_openers(
            self.world_span,
            self.status.wave,
            opening_enemy,
            group_index,
        );
        reinforcements.truncate(group_size as usize);
        self.entities.extend(reinforcements);

        self.pending_wave_openers = self.pending_wave_openers.saturating_sub(group_size);
        self.spawned_wave_opener_groups = self.spawned_wave_opener_groups.saturating_add(1);
        if self.pending_wave_openers > 0 {
            self.next_wave_reinforcement_tick = self
                .tick
                .saturating_add(tables.attack_wave_reinforcement_delay);
        }
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

fn wrapped_horizontal_step(from: i32, to: i32, max_x: i32) -> i32 {
    shortest_wrapped_delta(from, to, max_x + 1).signum()
}

fn shortest_wrapped_delta(from: i32, to: i32, span: i32) -> i32 {
    let forward = (to - from).rem_euclid(span);
    let backward = forward - span;
    if forward.abs() <= backward.abs() {
        forward
    } else {
        backward
    }
}

fn nearest_wrapped_target(
    origin: Position,
    candidates: &[Position],
    span: i32,
) -> Option<Position> {
    candidates.iter().copied().min_by_key(|candidate| {
        let horizontal = shortest_wrapped_delta(origin.x, candidate.x, span).abs();
        let vertical = (origin.y - candidate.y).abs();
        horizontal * 2 + vertical
    })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HyperspaceResult {
    position: Position,
    facing: HorizontalDirection,
    explodes: bool,
}

fn hyperspace_result(
    tick: u32,
    score: u32,
    current_facing: HorizontalDirection,
    max_x: i32,
    min_y: i32,
    max_y: i32,
) -> HyperspaceResult {
    let position = hyperspace_destination(tick, score, max_x, min_y, max_y);
    let seed = tick
        .wrapping_mul(13)
        .wrapping_add(score.wrapping_mul(7))
        .wrapping_add(5);
    let facing = if seed.is_multiple_of(2) {
        current_facing
    } else {
        current_facing.reversed()
    };

    HyperspaceResult {
        position,
        facing,
        explodes: seed % 4 == 3,
    }
}

fn score_for_enemy(kind: EntityKind) -> u32 {
    match kind {
        EntityKind::Lander => 150,
        EntityKind::Mutant => 250,
        EntityKind::Baiter => 200,
        EntityKind::Bomber => 250,
        EntityKind::Pod => 1_000,
        EntityKind::Swarmer => 150,
        _ => 0,
    }
}

fn score_for_player_collision(kind: EntityKind) -> u32 {
    match kind {
        EntityKind::EnemyShot | EntityKind::Mine => arcade_tables().hazard_collision_score,
        _ if kind.is_enemy() => score_for_enemy(kind),
        _ => 0,
    }
}

fn next_stock_award_score(score: u32) -> u32 {
    let bonus_stock_score = arcade_tables().bonus_stock_score;
    (score / bonus_stock_score)
        .saturating_add(1)
        .saturating_mul(bonus_stock_score)
}

fn default_humans(terrain: &[usize]) -> Vec<Entity> {
    arcade_tables()
        .default_human_world_xs
        .iter()
        .copied()
        .map(|x| Entity::new(EntityKind::Human, x, terrain_surface_y(terrain, x), 0, 0))
        .collect()
}

fn default_attack_wave_openers(
    world_span: i32,
    wave: u8,
    kind: EntityKind,
    group_index: u8,
) -> Vec<Entity> {
    let max_x = world_span - 1;
    let wave_offset = i32::from(wave % 6);
    let group_offset = i32::from(group_index) * 14;
    [
        (world_span - 12 - group_offset, 3 + wave_offset % 3, -1, 1),
        (world_span - 6 - group_offset, 6, -1, 1),
        (18 + wave_offset + group_offset, 8, 1, -1),
        (world_span / 2 + 8 + group_offset, 4, -1, 1),
        (world_span / 2 - 10 - group_offset, 7, 1, -1),
    ]
    .into_iter()
    .map(|(x, y, dx, dy)| Entity::new(kind, wrap_coordinate(x, max_x), y, dx, dy))
    .collect()
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
    use crate::arcade::arcade_tables;
    use crate::constants::{DEFAULT_LIVES, DEFAULT_SMART_BOMBS, PLAYER_START_X};

    use super::{
        Entity, EntityKind, EntityState, HorizontalDirection, Position, Status, UpdateInput, World,
        WorldEvent, hyperspace_result, nearest_wrapped_target, shortest_wrapped_delta,
        wrap_coordinate,
    };

    #[test]
    fn bootstrap_creates_expected_entities() {
        let world = World::bootstrap();

        assert!(world.world_span() > world.width() as i32);
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 5);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 0);
        assert_eq!(world.enemy_count(), 5);
        assert_eq!(world.human_count(), 10);
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
    fn shortest_wrapped_delta_prefers_the_shorter_arc() {
        assert_eq!(shortest_wrapped_delta(1, 14, 16), -3);
        assert_eq!(shortest_wrapped_delta(14, 1, 16), 3);
    }

    #[test]
    fn nearest_wrapped_target_picks_the_closest_candidate() {
        let origin = Position { x: 14, y: 4 };
        let target = nearest_wrapped_target(
            origin,
            &[Position { x: 2, y: 4 }, Position { x: 12, y: 6 }],
            16,
        );

        assert_eq!(target, Some(Position { x: 12, y: 6 }));
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
        assert_eq!(world.enemy_count(), 4);
        assert_eq!(world.human_count(), 11);
    }

    #[test]
    fn score_thresholds_award_extra_lives_and_smart_bombs() {
        let mut world = World::bootstrap();

        world.add_score(20_050);

        assert_eq!(world.status().score, 20_050);
        assert_eq!(world.status().lives, DEFAULT_LIVES + 2);
        assert_eq!(world.smart_bombs(), DEFAULT_SMART_BOMBS + 2);
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
        assert_eq!(world.player_facing(), HorizontalDirection::Right);
        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 1);
        assert!(events.contains(&WorldEvent::ShotFired));
    }

    #[test]
    fn live_step_reverse_flips_player_direction_without_moving() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 14, 1, 0, 0),
            ],
        );

        let start = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position;
        world.step_live(UpdateInput {
            reverse: true,
            ..UpdateInput::default()
        });

        let player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(player.position, start);
        assert_eq!(world.player_facing(), HorizontalDirection::Left);
    }

    #[test]
    fn live_step_thrust_and_fire_follow_reversed_direction() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 14, 1, 0, 0),
            ],
        );

        world.step_live(UpdateInput {
            reverse: true,
            ..UpdateInput::default()
        });
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
        let shot = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShot)
            .expect("shot");

        assert_eq!(world.player_facing(), HorizontalDirection::Left);
        assert_eq!(player.position.x, 5);
        assert_eq!(shot.position.x, 2);
        assert_eq!(shot.velocity.dx, -2);
        assert!(events.contains(&WorldEvent::ShotFired));
    }

    #[test]
    fn reverse_preserves_player_momentum_until_counter_thrust() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 14, 1, 0, 0),
            ],
        );

        world.step_live(UpdateInput {
            thrust: true,
            ..UpdateInput::default()
        });
        world.step_live(UpdateInput {
            reverse: true,
            ..UpdateInput::default()
        });

        let drifting_player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(world.player_facing(), HorizontalDirection::Left);
        assert_eq!(drifting_player.position.x, 8);
        assert_eq!(drifting_player.velocity.dx, 1);

        world.step_live(UpdateInput {
            thrust: true,
            ..UpdateInput::default()
        });

        let braked_player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(braked_player.position.x, 8);
        assert_eq!(braked_player.velocity.dx, 0);
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
    fn live_step_landers_seek_the_nearest_free_human() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 10, 3, 0, 0),
                Entity::new(EntityKind::Human, 14, 5, 0, 0),
                Entity::new(EntityKind::Human, 3, 7, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        assert_eq!(lander.position, Position { x: 11, y: 4 });
    }

    #[test]
    fn live_step_landers_chase_the_player_when_no_free_humans_remain() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 5, 0, 0),
                Entity::new(EntityKind::Lander, 10, 3, 0, 0),
                Entity::with_state(EntityKind::Human, 12, 4, 0, 0, EntityState::Falling),
            ],
        );

        world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        assert_eq!(lander.position, Position { x: 9, y: 4 });
    }

    #[test]
    fn live_step_mutants_home_toward_the_player() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 10, 6, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let mutant = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Mutant)
            .expect("mutant");
        assert_eq!(mutant.position, Position { x: 9, y: 5 });
    }

    #[test]
    fn live_step_baiters_home_toward_the_player() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 1, 0),
                Entity::new(EntityKind::Lander, 18, 1, 0, 0),
                Entity::new(EntityKind::Baiter, 12, 6, 0, 0),
                Entity::new(EntityKind::Pod, 18, 3, 0, 0),
                Entity::new(EntityKind::Human, 1, 8, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let baiter = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Baiter)
            .expect("baiter");
        assert_eq!(baiter.velocity.dx, -1);
        assert_eq!(baiter.position, Position { x: 11, y: 5 });
    }

    #[test]
    fn live_step_bombers_accelerate_when_crossing_the_players_altitude() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 12, 4, -1, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let bomber = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Bomber)
            .expect("bomber");
        assert_eq!(bomber.velocity.dx, -arcade_tables().bomber_evasive_speed);
        assert_eq!(bomber.position, Position { x: 10, y: 4 });
    }

    #[test]
    fn live_step_bombers_leave_mines_that_survive_smart_bombs() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 12, 6, -1, 0),
                Entity::with_state(EntityKind::Human, 2, 2, 0, 0, EntityState::Abducted),
            ],
        );

        for _ in 0..3 {
            world.step_live(UpdateInput::default());
        }
        assert!(world.entity_count_by_kind(EntityKind::Mine) > 0);

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 0);
        assert!(world.entity_count_by_kind(EntityKind::Mine) > 0);
        assert!(events.contains(&WorldEvent::SmartBombDetonated));
    }

    #[test]
    fn smart_bomb_only_destroys_enemies_on_the_main_screen() {
        let mut world = World::bootstrap();
        let player_x = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position
            .x;
        let visible_x = wrap_coordinate(player_x + 5, world.world_max_x());
        let offscreen_x = wrap_coordinate(player_x + world.width() as i32, world.world_max_x());
        let safe_y = world.safe_altitude_at_world_x(player_x).min(4);
        world.camera_x = player_x;
        world.entities = vec![
            Entity::new(EntityKind::PlayerShip, player_x, safe_y, 0, 0),
            Entity::new(EntityKind::Lander, visible_x, safe_y, 0, 0),
            Entity::new(EntityKind::Bomber, offscreen_x, safe_y, 0, 0),
        ];

        assert!(world.screen_x_for_world_x(visible_x).is_some());
        assert!(world.screen_x_for_world_x(offscreen_x).is_none());

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 1);
        assert_eq!(world.status().score, 150);
        assert!(events.contains(&WorldEvent::SmartBombDetonated));
    }

    #[test]
    fn live_step_swarmers_home_toward_the_player() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Swarmer, 12, 6, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let swarmer = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Swarmer)
            .expect("swarmer");
        assert_eq!(swarmer.velocity.dx, -arcade_tables().swarmer_speed);
        assert_eq!(swarmer.position, Position { x: 10, y: 5 });
    }

    #[test]
    fn live_step_carrying_landers_rise_straight_up() {
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
                Entity::with_state(EntityKind::Lander, 8, 5, 1, 1, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 8, 6, 0, -1, EntityState::Abducted),
            ],
        );

        world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        assert_eq!(lander.position, Position { x: 8, y: 4 });
        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("human");
        assert_eq!(human.position, Position { x: 8, y: 5 });
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
    fn live_step_removes_humans_that_are_reached_by_mutants() {
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
                Entity::new(EntityKind::Mutant, 6, 5, 0, 0),
                Entity::new(EntityKind::Human, 6, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.human_count(), 0);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn live_step_starts_abductions_before_counting_humans_as_lost() {
        let mut world = World::with_entities(
            16,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 6, 6, 0, 0),
                Entity::new(EntityKind::Human, 6, 6, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("human");

        assert_eq!(world.entity_count_by_kind(EntityKind::Human), 1);
        assert_eq!(world.human_count(), 0);
        assert_eq!(lander.state, EntityState::CarryingHuman);
        assert_eq!(human.state, EntityState::Abducted);
        assert_eq!(human.position.x, lander.position.x);
        assert_eq!(human.position.y, lander.position.y + 1);
        assert!(!events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn live_step_mutates_landers_that_finish_an_abduction() {
        let mut world = World::with_entities(
            16,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::with_state(EntityKind::Lander, 6, 2, 0, -1, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 6, 3, 0, -1, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Human), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 1);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn losing_the_last_human_mutates_remaining_landers() {
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
                Entity::new(EntityKind::Lander, 16, 5, 0, 0),
                Entity::new(EntityKind::Mutant, 6, 5, 0, 0),
                Entity::new(EntityKind::Human, 6, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Human), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 3);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn last_human_abduction_completion_mutates_all_landers() {
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
                Entity::with_state(EntityKind::Lander, 6, 2, 0, -1, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 6, 3, 0, -1, EntityState::Abducted),
                Entity::new(EntityKind::Lander, 16, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Human), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 2);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn destroying_a_carrying_lander_makes_the_human_fall() {
        let mut world = World::with_entities(
            16,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::with_state(EntityKind::Lander, 5, 4, 0, 0, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 5, 5, 0, 0, EntityState::Abducted),
                Entity::new(EntityKind::Mutant, 11, 3, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("falling human");
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(human.state, EntityState::Falling);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
        assert!(!events.contains(&WorldEvent::HumanRescued));
    }

    #[test]
    fn low_altitude_falling_humans_can_land_safely_on_their_own() {
        let mut world = World::with_entities(
            16,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 4, 0, 0),
                Entity::with_state(EntityKind::Lander, 5, 5, 0, 0, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 5, 6, 0, -1, EntityState::Abducted),
            ],
        );

        world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });
        world.step_live(UpdateInput::default());
        let events = world.step_live(UpdateInput::default());

        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("landed human");
        assert_eq!(human.state, EntityState::Normal);
        assert_eq!(
            human.position.y,
            world.safe_altitude_at_world_x(human.position.x)
        );
        assert_eq!(world.status().score, 500);
        assert_eq!(world.status().wave, 2);
        assert!(events.contains(&WorldEvent::HumanRescued));
    }

    #[test]
    fn player_catches_a_falling_human_and_lands_them_safely() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 5, 0, 0),
                Entity::with_state(EntityKind::Human, 6, 4, 0, 1, EntityState::Falling),
                Entity::new(EntityKind::Mutant, 16, 4, 0, 0),
            ],
        );

        let first_events = world.step_live(UpdateInput::default());
        let carried_human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("carried human");
        assert_eq!(carried_human.state, EntityState::PlayerCarried);
        assert_eq!(world.status().score, 500);
        assert!(!first_events.contains(&WorldEvent::HumanRescued));

        let second_events = world.step_live(UpdateInput {
            down: true,
            ..UpdateInput::default()
        });

        let rescued_human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("rescued human");
        assert_eq!(rescued_human.state, EntityState::Normal);
        assert_eq!(
            rescued_human.position.y,
            world.safe_altitude_at_world_x(rescued_human.position.x)
        );
        assert_eq!(world.status().score, 1000);
        assert!(second_events.contains(&WorldEvent::HumanRescued));
    }

    #[test]
    fn player_picks_up_a_grounded_human_and_redeploys_them() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 6, 0, 0),
                Entity::new(EntityKind::Human, 6, 7, 0, 0),
                Entity::new(EntityKind::Mutant, 16, 4, 0, 0),
            ],
        );

        let pickup_events = world.step_live(UpdateInput::default());
        let carried_human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("carried human");
        assert_eq!(carried_human.state, EntityState::PlayerCarried);
        assert_eq!(world.status().score, 0);
        assert!(!pickup_events.contains(&WorldEvent::HumanRescued));

        let carry_events = world.step_live(UpdateInput {
            up: true,
            thrust: true,
            ..UpdateInput::default()
        });
        let repositioned_human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("repositioned human");
        assert_eq!(repositioned_human.state, EntityState::PlayerCarried);
        assert_eq!(repositioned_human.position.x, 7);
        assert!(!carry_events.contains(&WorldEvent::HumanRescued));

        let landing_events = world.step_live(UpdateInput {
            down: true,
            ..UpdateInput::default()
        });
        let landed_human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("landed human");
        assert_eq!(landed_human.state, EntityState::Normal);
        assert_eq!(landed_human.position.x, 8);
        assert_eq!(
            landed_human.position.y,
            world.safe_altitude_at_world_x(landed_human.position.x)
        );
        assert_eq!(world.status().score, 0);
        assert!(landing_events.contains(&WorldEvent::HumanRescued));
    }

    #[test]
    fn uncaught_falling_humans_are_lost_on_ground_impact() {
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
                Entity::with_state(EntityKind::Human, 14, 6, 0, 1, EntityState::Falling),
                Entity::new(EntityKind::Mutant, 18, 4, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Human), 0);
        assert_eq!(world.status().score, 0);
        assert!(events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn xyzzy_mode_allows_humans_to_survive_full_height_falls() {
        let mut world = World::with_entities(
            20,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 0, 0),
                Entity::with_state(EntityKind::Human, 14, 1, 0, 1, EntityState::Falling),
            ],
        );

        let fall_ticks = world.safe_altitude_at_world_x(14) - 1;

        let mut all_events = Vec::new();
        for _ in 0..fall_ticks {
            all_events.extend(world.step_live(UpdateInput {
                secret_mode: true,
                ..UpdateInput::default()
            }));
        }

        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("surviving human");
        assert_eq!(human.state, EntityState::Normal);
        assert_eq!(
            human.position.y,
            world.safe_altitude_at_world_x(human.position.x)
        );
        assert!(world.status().score >= u32::from(arcade_tables().safe_fall_score));
        assert!(all_events.contains(&WorldEvent::HumanRescued));
        assert!(!all_events.contains(&WorldEvent::HumanLost));
    }

    #[test]
    fn player_hit_drops_a_carried_human_back_into_the_world() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 2,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 6, 4, 0, 0),
                Entity::with_state(EntityKind::Human, 6, 5, 0, 0, EntityState::PlayerCarried),
                Entity::new(EntityKind::Lander, 6, 4, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        let human = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Human)
            .expect("dropped human");
        assert_eq!(human.state, EntityState::Falling);
        assert_eq!(world.status().lives, 1);
        assert!(events.contains(&WorldEvent::PlayerHit));
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
        assert_eq!(world.status().score, arcade_tables().hazard_collision_score);
        assert!(events.contains(&WorldEvent::PlayerHit));
        assert!(events.contains(&WorldEvent::GameOver));
    }

    #[test]
    fn live_step_mines_can_destroy_the_player() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Mine, 3, 3, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().lives, 0);
        assert!(world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::Mine), 0);
        assert_eq!(world.status().score, arcade_tables().hazard_collision_score);
        assert!(events.contains(&WorldEvent::PlayerHit));
        assert!(events.contains(&WorldEvent::GameOver));
    }

    #[test]
    fn live_step_ramming_an_enemy_still_awards_its_score() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Bomber, 1, 3, 0, 0),
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().lives, 0);
        assert!(world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 0);
        assert_eq!(world.status().score, 250);
        assert!(events.contains(&WorldEvent::PlayerHit));
        assert!(events.contains(&WorldEvent::GameOver));
    }

    #[test]
    fn live_step_destroying_a_pod_releases_swarmers_and_scores_bonus() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 4, 0, 0),
                Entity::new(EntityKind::PlayerShot, 8, 4, 1, 0),
                Entity::new(EntityKind::Pod, 9, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());
        let swarmer_count = world.entity_count_by_kind(EntityKind::Swarmer);

        assert_eq!(world.entity_count_by_kind(EntityKind::Pod), 0);
        assert!(swarmer_count >= arcade_tables().pod_swarmer_burst_min);
        assert!(
            swarmer_count
                < arcade_tables().pod_swarmer_burst_min + arcade_tables().pod_swarmer_burst_range
        );
        assert_eq!(world.status().score, 1_000);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
        assert!(world.entities().iter().all(|entity| {
            entity.kind != EntityKind::Swarmer
                || entity.velocity.dx == -arcade_tables().swarmer_speed
        }));
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
    fn wave_two_and_later_add_arcade_like_wave_openers() {
        let mut wave_two = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 9, 0, 0),
            ],
        );
        wave_two.spawn_wave();
        assert_eq!(wave_two.entity_count_by_kind(EntityKind::Lander), 5);
        assert_eq!(wave_two.entity_count_by_kind(EntityKind::Mutant), 0);
        assert_eq!(wave_two.entity_count_by_kind(EntityKind::Bomber), 1);
        assert_eq!(wave_two.entity_count_by_kind(EntityKind::Pod), 1);

        let mut wave_three = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 9, 0, 0),
            ],
        );
        wave_three.spawn_wave();
        assert_eq!(wave_three.entity_count_by_kind(EntityKind::Lander), 5);
        assert_eq!(wave_three.entity_count_by_kind(EntityKind::Mutant), 0);
        assert_eq!(wave_three.entity_count_by_kind(EntityKind::Bomber), 2);
        assert_eq!(wave_three.entity_count_by_kind(EntityKind::Pod), 3);

        let mut wave_four = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 4,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 9, 0, 0),
            ],
        );
        wave_four.spawn_wave();
        assert_eq!(wave_four.entity_count_by_kind(EntityKind::Lander), 5);
        assert_eq!(wave_four.entity_count_by_kind(EntityKind::Bomber), 2);
        assert_eq!(wave_four.entity_count_by_kind(EntityKind::Pod), 4);

        let mut wave_five = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 5,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 9, 0, 0),
            ],
        );
        wave_five.spawn_wave();
        assert_eq!(wave_five.entity_count_by_kind(EntityKind::Lander), 5);
        assert_eq!(wave_five.entity_count_by_kind(EntityKind::Bomber), 3);
        assert_eq!(wave_five.entity_count_by_kind(EntityKind::Pod), 4);
        assert_eq!(wave_five.human_count(), 10);
    }

    #[test]
    fn attack_waves_arrive_in_three_five_ship_groups() {
        let mut world = World::bootstrap();

        for _ in 0..arcade_tables().attack_wave_reinforcement_delay {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 10);

        for _ in 0..arcade_tables().attack_wave_reinforcement_delay {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 15);
    }

    #[test]
    fn mutant_reinforcements_follow_the_same_three_group_schedule() {
        let mut world = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0)],
        );

        world.spawn_wave();
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 5);

        for _ in 0..arcade_tables().attack_wave_reinforcement_delay {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 10);

        for _ in 0..arcade_tables().attack_wave_reinforcement_delay {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 15);
    }

    #[test]
    fn waves_do_not_advance_while_reinforcement_groups_are_still_pending() {
        let mut world = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 9, 0, 0),
            ],
        );

        world.spawn_wave();
        world
            .entities
            .retain(|entity| entity.kind != EntityKind::Lander);

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().wave, 1);
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert!(!events.contains(&WorldEvent::WaveAdvanced));
    }

    #[test]
    fn mutant_stages_replace_landers_until_the_next_fifth_wave() {
        let mut world = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0)],
        );

        world.spawn_wave();

        assert_eq!(world.human_count(), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 5);
        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 1);
        assert_eq!(world.entity_count_by_kind(EntityKind::Pod), 1);
        assert!(world.planet_destroyed());
    }

    #[test]
    fn fifth_waves_restore_the_full_humanoid_population() {
        let mut world = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 5,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Human, 3, 9, 0, 0),
            ],
        );

        world.spawn_wave();

        assert_eq!(world.human_count(), 10);
        assert!(
            !world
                .entities()
                .iter()
                .any(|entity| entity.kind == EntityKind::Human && entity.position.x == 3)
        );
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 5);
        assert!(!world.planet_destroyed());
    }

    #[test]
    fn live_step_spawns_baiters_when_a_wave_drags_on() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 99,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 4, 0, 0),
                Entity::new(EntityKind::Lander, 18, 4, 0, 0),
                Entity::new(EntityKind::Pod, 18, 4, 0, 0),
                Entity::new(EntityKind::Human, 1, 8, 0, 0),
            ],
        );
        world.tick = arcade_tables().baiter_base_delay + 4;

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 1);
    }

    #[test]
    fn live_step_does_not_spawn_baiters_once_landers_are_gone() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 99,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 4, 0, 0),
                Entity::new(EntityKind::Pod, 18, 4, 0, 0),
            ],
        );
        world.tick = arcade_tables().baiter_base_delay;

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 0);
    }

    #[test]
    fn baiters_disappear_when_the_last_lander_is_destroyed() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 4, 0, 0),
                Entity::new(EntityKind::PlayerShot, 8, 4, 1, 0),
                Entity::new(EntityKind::Lander, 9, 4, 0, 0),
                Entity::new(EntityKind::Baiter, 14, 4, 0, 0),
                Entity::new(EntityKind::Pod, 18, 4, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Pod), 1);
        assert_eq!(world.status().score, 150);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
        assert!(!events.contains(&WorldEvent::WaveAdvanced));
    }

    #[test]
    fn baiters_do_not_block_wave_advance() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Baiter, 12, 4, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().wave, 2);
        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 0);
        assert!(world.enemy_count() >= 2);
        assert!(events.contains(&WorldEvent::WaveAdvanced));
    }

    #[test]
    fn wave_advance_awards_humanoid_survivor_bonus() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 5, 3, 0, 0),
                Entity::new(EntityKind::Human, 8, 5, 0, 0),
                Entity::new(EntityKind::Human, 12, 5, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().wave, 4);
        assert_eq!(world.status().score, 750);
        assert!(events.contains(&WorldEvent::WaveAdvanced));
    }

    #[test]
    fn wave_advance_waits_for_falling_humans_to_resolve() {
        let mut world = World::with_entities(
            16,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 4, 0, 0),
                Entity::with_state(EntityKind::Lander, 5, 5, 0, 0, EntityState::CarryingHuman),
                Entity::with_state(EntityKind::Human, 5, 6, 0, -1, EntityState::Abducted),
            ],
        );

        let first_events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().wave, 2);
        assert!(!first_events.contains(&WorldEvent::WaveAdvanced));

        world.step_live(UpdateInput::default());
        let third_events = world.step_live(UpdateInput::default());

        assert_eq!(world.status().wave, 3);
        assert_eq!(world.status().score, 600);
        assert!(third_events.contains(&WorldEvent::WaveAdvanced));
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
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.smart_bombs(), DEFAULT_SMART_BOMBS - 1);
        assert_eq!(world.status().score, 150);
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 1);
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

        let expected = hyperspace_result(1, 0, HorizontalDirection::Right, 19, 1, 7);
        let events = world.step_live(UpdateInput {
            hyperspace: true,
            ..UpdateInput::default()
        });

        let player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(player.position, expected.position);
        assert_eq!(player.velocity.dx, 0);
        assert_eq!(world.player_facing(), expected.facing);
        assert!(events.contains(&WorldEvent::HyperspaceUsed));
        assert!(!events.contains(&WorldEvent::PlayerHit));
    }

    #[test]
    fn live_step_hyperspace_can_destroy_the_player_outside_secret_mode() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 3,
                lives: 2,
                wave: 1,
            },
            vec![Entity::new(EntityKind::PlayerShip, 2, 3, 1, 0)],
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
        assert_eq!(world.status().lives, 1);
        assert_eq!(player.position.x, PLAYER_START_X);
        assert_eq!(player.velocity.dx, 0);
        assert_eq!(world.player_facing(), HorizontalDirection::Right);
        assert!(events.contains(&WorldEvent::HyperspaceUsed));
        assert!(events.contains(&WorldEvent::PlayerHit));
    }

    #[test]
    fn xyzzy_mode_makes_hyperspace_safe_even_when_it_would_fail() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 3,
                lives: 2,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 1, 0),
                Entity::new(EntityKind::Lander, 17, 2, 0, 0),
                Entity::new(EntityKind::Mutant, 8, 4, 0, 0),
                Entity::new(EntityKind::EnemyShot, 12, 5, 0, 0),
            ],
        );
        let destination = world.safest_hyperspace_destination(19, 1, 7);

        let events = world.step_live(UpdateInput {
            hyperspace: true,
            secret_mode: true,
            ..UpdateInput::default()
        });

        let player = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player");
        assert_eq!(world.status().lives, 2);
        assert_eq!(player.position, destination);
        assert_eq!(player.velocity.dx, 0);
        assert!(events.contains(&WorldEvent::HyperspaceUsed));
        assert!(!events.contains(&WorldEvent::PlayerHit));
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
    fn invincible_live_step_ramming_enemy_destroys_it_and_awards_score() {
        let mut world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 2,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 2, 3, 0, 0),
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput {
            invincible: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().lives, 2);
        assert!(!world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 0);
        assert_eq!(world.status().score, 150);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
        assert!(!events.contains(&WorldEvent::PlayerHit));
    }

    #[test]
    fn xyzzy_mode_allows_unlimited_smart_bombs_and_clears_bullets_and_mines() {
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
                Entity::new(EntityKind::Mine, 7, 3, 0, 0),
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );
        world.set_smart_bombs(0);

        let events = world.step_live(UpdateInput {
            smart_bomb: true,
            secret_mode: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.smart_bombs(), 0);
        assert_eq!(world.status().score, 150);
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::Mine), 0);
        assert!(events.contains(&WorldEvent::SmartBombDetonated));
    }

    #[test]
    fn invincible_mode_does_not_grant_unlimited_smart_bombs() {
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
        assert_eq!(world.status().score, 0);
        assert!(!events.contains(&WorldEvent::SmartBombDetonated));
    }

    #[test]
    fn live_step_auto_fire_destroys_enemy_directly_ahead_in_secret_mode() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 9, 4, -1, 0),
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput {
            secret_mode: true,
            auto_fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 0);
        assert_eq!(world.status().score, 250);
        assert!(events.contains(&WorldEvent::ShotFired));
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
    }

    #[test]
    fn live_step_auto_fire_waits_for_a_direct_target() {
        let mut world = World::with_entities(
            16,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 13, 4, -1, 0),
                Entity::with_state(EntityKind::Human, 1, 1, 0, 0, EntityState::Abducted),
            ],
        );

        let events = world.step_live(UpdateInput {
            secret_mode: true,
            auto_fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.entity_count_by_kind(EntityKind::Bomber), 1);
        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 0);
        assert_eq!(world.status().score, 0);
        assert!(!events.contains(&WorldEvent::ShotFired));
    }
}
