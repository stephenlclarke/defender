use crate::arcade::arcade_tables;
use crate::constants::{
    DEFAULT_LIVES, DEFAULT_SMART_BOMBS, DEFAULT_WAVE, GROUND_ROW, PLAYER_START_X, PLAYER_START_Y,
    WORLD_HEIGHT, WORLD_SPAN, WORLD_WIDTH,
};
use crate::red_label_wave::red_label_wave_table;

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
    pub rom_aux: u16,
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
            rom_aux: 0,
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
    next_stock_award: u32,
    smart_bombs: u8,
    baiter_timer: u32,
    pending_wave_openers: u8,
    spawned_wave_opener_groups: u8,
    next_wave_reinforcement_tick: u32,
    game_over: bool,
    status: Status,
    rand_state: RomRandState,
    terrain: Vec<usize>,
    entities: Vec<Entity>,
}

impl World {
    pub fn bootstrap() -> Self {
        let terrain = build_scrolling_terrain(WORLD_SPAN, WORLD_HEIGHT);
        let mut rand_state = RomRandState::default();
        let mut entities = vec![Entity::new(
            EntityKind::PlayerShip,
            PLAYER_START_X,
            PLAYER_START_Y,
            0,
            0,
        )];
        let wave_profile = red_label_wave_table().profile_for_wave(DEFAULT_WAVE);
        entities.extend(default_attack_wave_openers(
            WORLD_SPAN as i32,
            DEFAULT_WAVE,
            EntityKind::Lander,
            0,
            wave_profile.wave_size as usize,
            &mut rand_state,
        ));
        entities.extend(default_humans(&terrain, WORLD_SPAN as i32));
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            world_span: WORLD_SPAN as i32,
            camera_x: PLAYER_START_X,
            player_facing: HorizontalDirection::Right,
            tick: 0,
            next_stock_award: next_stock_award_score(0),
            smart_bombs: DEFAULT_SMART_BOMBS,
            baiter_timer: wave_profile.baiter_delay,
            pending_wave_openers: wave_profile.landers.saturating_sub(wave_profile.wave_size),
            spawned_wave_opener_groups: 1,
            next_wave_reinforcement_tick: wave_profile.wave_time,
            game_over: false,
            status: Status {
                score: 0,
                lives: DEFAULT_LIVES,
                wave: DEFAULT_WAVE,
            },
            rand_state,
            terrain,
            entities,
        }
    }

    pub fn with_entities(
        width: usize,
        height: usize,
        status: Status,
        mut entities: Vec<Entity>,
    ) -> Self {
        let mut init_rand_state = RomRandState::default();
        for entity in &mut entities {
            initialize_rom_enemy_state(entity, status.wave, &mut init_rand_state);
        }
        Self {
            width,
            height,
            world_span: width as i32,
            camera_x: width as i32 / 2,
            player_facing: HorizontalDirection::Right,
            tick: 0,
            next_stock_award: next_stock_award_score(status.score),
            smart_bombs: DEFAULT_SMART_BOMBS,
            baiter_timer: red_label_wave_table()
                .profile_for_wave(status.wave)
                .baiter_delay,
            pending_wave_openers: 0,
            spawned_wave_opener_groups: 0,
            next_wave_reinforcement_tick: 0,
            game_over: false,
            status,
            rand_state: RomRandState::default(),
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

        self.sync_camera_to_player();
        self.retain_projectiles(max_x, min_y, max_y);
        self.spawn_attack_wave_reinforcements_if_due();
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

    pub fn set_player_facing(&mut self, facing: HorizontalDirection) {
        self.player_facing = facing;
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

    fn live_shell_count(&self) -> usize {
        self.entities
            .iter()
            .filter(|entity| matches!(entity.kind, EntityKind::EnemyShot | EntityKind::Mine))
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

    pub fn spawn_entity(&mut self, mut entity: Entity) {
        initialize_rom_enemy_state(&mut entity, self.status.wave, &mut self.rand_state);
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

        if hyperspace_result.is_some() {
            // Red-label `HYPER` walks the shell list through `KILSHL` before the
            // player reappears, so active shell objects do not survive
            // hyperspace.
            self.entities.retain(|entity| {
                !matches!(entity.kind, EntityKind::PlayerShot | EntityKind::EnemyShot)
            });
        }

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
            && self.can_fire_player_shot(input.secret_mode)
            && shot_origin.is_some_and(|origin| self.should_auto_fire(origin, max_x, min_y, max_y));
        if (input.fire || auto_fire)
            && !hyperspaced_this_tick
            && self.can_fire_player_shot(input.secret_mode)
            && let Some(origin) = shot_origin
        {
            let shot_dx = self.player_facing.step() * arcade_tables().player_shot_speed;
            // Red-label `LFIRE`/`LFIREX` is a transient switch process. Hitting
            // the four-shot cap only kills that process; it does not destroy
            // the ship. XYZZY deliberately lifts the cap.
            self.entities.push(Entity::new(
                EntityKind::PlayerShot,
                wrap_coordinate(origin.x + self.player_facing.step(), max_x),
                origin.y,
                shot_dx,
                0,
            ));
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
        self.sync_camera_to_player();
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
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        // `GETSHL` gates the shared hostile shell list through `BMBCNT < 20`.
        // Keep enemy bullets on that cabinet counter instead of the old Rust
        // per-kind cap.
        let available = rom_shell_spawn_limit().saturating_sub(self.live_shell_count());
        let Some((player_position, player_screen_x)) =
            self.player_position().and_then(|position| {
                self.screen_x_for_world_x(position.x)
                    .map(|x| (position, x as i32))
            })
        else {
            return;
        };

        let player_velocity = self.player_velocity().unwrap_or(Velocity { dx: 0, dy: 0 });
        let mut firing_enemies = Vec::new();
        let mut swarmer_resets = Vec::new();

        // StrategyWiki's attack-wave advice calls out that opponents must be on
        // the main screen before they can shoot, so off-screen enemies are
        // filtered out here before any firing solution is attempted.
        for index in 0..self.entities.len() {
            let entity = &self.entities[index];
            if !entity.kind.can_fire() {
                continue;
            }

            let Some(enemy_screen_x) = self.screen_x_for_world_x(entity.position.x) else {
                continue;
            };
            let enemy_screen_x = enemy_screen_x as i32;

            match entity.kind {
                EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter => {
                    if rom_enemy_shot_timer_for_entity(entity, self.status.wave) == 0 {
                        swarmer_resets.push(index);
                    }
                }
                EntityKind::Swarmer
                    if rom_swarmer_shot_timer_for_entity(
                        entity,
                        wave_profile.swarmer_shot_time,
                    ) == 0 =>
                {
                    swarmer_resets.push(index);
                }
                _ => {}
            }

            if firing_enemies.len() >= available {
                continue;
            }

            if let Some(velocity) = self.enemy_shot_velocity(
                entity,
                enemy_screen_x,
                player_position,
                player_screen_x,
                player_velocity,
            ) {
                firing_enemies.push((entity.position, velocity));
            }
        }

        for index in swarmer_resets {
            let Some(entity) = self.entities.get_mut(index) else {
                continue;
            };
            match entity.kind {
                EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter => {
                    entity.rom_aux = rom_pack_enemy_shot_timer(rom_reset_enemy_shot_timer(
                        entity.kind,
                        self.status.wave,
                        &mut self.rand_state,
                    ));
                }
                EntityKind::Swarmer => {
                    let acceleration = rom_swarmer_acceleration_for_entity(
                        entity,
                        wave_profile.swarmer_acceleration_mask,
                    );
                    let shot_timer = rom_seeded_rmax(
                        &mut self.rand_state,
                        wave_profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8,
                    );
                    entity.rom_aux = rom_pack_swarmer_state(acceleration, shot_timer);
                }
                _ => {}
            }
        }

        if firing_enemies.is_empty() {
            return;
        }

        for (enemy_position, velocity) in firing_enemies {
            let shot_x = (enemy_position.x + velocity.dx.signum()).clamp(0, max_x);
            let shot_y = (enemy_position.y + velocity.dy.signum()).clamp(min_y, max_y);
            self.entities.push(Entity::new(
                EntityKind::EnemyShot,
                shot_x,
                shot_y,
                velocity.dx,
                velocity.dy,
            ));
        }
        events.push(WorldEvent::EnemyFired);
    }

    fn enemy_shot_velocity(
        &self,
        enemy: &Entity,
        enemy_screen_x: i32,
        player_position: Position,
        player_screen_x: i32,
        player_velocity: Velocity,
    ) -> Option<Velocity> {
        let tables = arcade_tables();
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        match enemy.kind {
            EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter => {
                if rom_enemy_shot_timer_for_entity(enemy, self.status.wave) != 0 {
                    return None;
                }

                // `defb6.src` routes Landers, Mutants, and Baiters through the
                // shared per-process timer plus `SHOOT` routine, which solves
                // shell velocity from the current screen delta plus `SEED` /
                // `LSEED` jitter rather than alternating between invented
                // chaser and lob shot modes.
                let (seed, lseed) = self.rom_enemy_shot_seed_bytes();
                Some(rom_shoot_velocity(
                    enemy_screen_x,
                    enemy.position.y,
                    player_screen_x,
                    player_position.y,
                    player_velocity,
                    seed,
                    lseed,
                ))
            }
            EntityKind::Swarmer => {
                if rom_swarmer_shot_timer_for_entity(enemy, wave_profile.swarmer_shot_time) != 0 {
                    return None;
                }

                let heading = enemy.velocity.dx.signum();
                let player_delta = player_screen_x - enemy_screen_x;
                if player_delta != 0 && heading != 0 && player_delta.signum() != heading {
                    return None;
                }

                let lead = (self.width as i32 / tables.swarmer_fire_lead_divisor as i32).max(1);
                let vertical_delta = player_position.y - enemy.position.y;
                let mut dy = (vertical_delta / lead.max(1)).clamp(-1, 1);
                if dy == 0 && vertical_delta.abs() >= (lead / 2).max(1) {
                    dy = vertical_delta.signum();
                }
                let dx = if heading == 0 {
                    player_delta.signum().max(1)
                } else {
                    heading
                };
                Some(Velocity { dx, dy })
            }
            _ => None,
        }
    }

    fn update_enemy_intents(&mut self, min_y: i32, max_y: i32) {
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        let player_position = self.player_position();
        let player_velocity = self.player_velocity();
        let free_humans = self.free_human_positions();
        let terrain = &self.terrain;
        let world_span = self.world_span;
        let world_max_x = self.world_max_x();
        let left_edge = self.left_edge();
        let width = self.width;
        let rand_state = &mut self.rand_state;
        let lander_speed_x = rom_lander_horizontal_velocity(wave_profile.lander_x_velocity);
        let lander_speed_y = rom_lander_vertical_velocity(
            wave_profile.lander_y_velocity_msb,
            wave_profile.lander_y_velocity_lsb,
        );
        for enemy in self
            .entities
            .iter_mut()
            .filter(|entity| entity.kind.is_enemy())
        {
            if matches!(
                enemy.kind,
                EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter
            ) {
                let shot_timer = rom_enemy_shot_timer_for_entity(enemy, self.status.wave);
                enemy.rom_aux = rom_pack_enemy_shot_timer(shot_timer.saturating_sub(1));
            }

            match enemy.kind {
                EntityKind::Lander if enemy.state == EntityState::CarryingHuman => {
                    // StrategyWiki gameplay notes and the red-label disassembly
                    // both treat carrying Landers as a straight-up abduction run.
                    enemy.velocity.dx = 0;
                    enemy.velocity.dy = -1;
                }
                EntityKind::Lander => {
                    let cruise_y = rom_lander_cruise_altitude(
                        terrain_surface_y(terrain, enemy.position.x) - 1,
                        min_y,
                        max_y,
                    );
                    let target = nearest_wrapped_target(enemy.position, &free_humans, world_span);
                    // `LANDS0` does not chase the player horizontally. Landers
                    // keep their seeded `LNDXV` drift until they line up with a
                    // target, then switch into the short `LANDG` grab vector.
                    if let Some(target) = target {
                        if rom_lander_target_is_aligned(enemy.position, target, world_span) {
                            enemy.velocity.dx =
                                wrapped_horizontal_step(enemy.position.x, target.x, world_max_x)
                                    * lander_speed_x.max(1);
                            enemy.velocity.dy = rom_lander_grab_velocity(
                                enemy.position.y,
                                target.y,
                                lander_speed_y,
                            );
                        } else {
                            enemy.velocity.dx = normalize_or_seed_horizontal_velocity(
                                enemy.velocity.dx,
                                lander_speed_x,
                                enemy.position.x,
                                self.tick,
                            );
                            enemy.velocity.dy = rom_lander_cruise_velocity(
                                enemy.position.y,
                                cruise_y,
                                lander_speed_y,
                            );
                        }
                    } else {
                        enemy.velocity.dx = normalize_or_seed_horizontal_velocity(
                            enemy.velocity.dx,
                            lander_speed_x,
                            enemy.position.x,
                            self.tick,
                        );
                        enemy.velocity.dy =
                            rom_lander_cruise_velocity(enemy.position.y, cruise_y, lander_speed_y);
                    }
                }
                EntityKind::Mutant => {
                    let mutant_speed_x =
                        rom_mutant_horizontal_velocity(wave_profile.mutant_x_velocity);
                    let mutant_speed_y = rom_mutant_vertical_velocity(
                        wave_profile.mutant_y_velocity_msb,
                        wave_profile.mutant_y_velocity_lsb,
                    );
                    let target = player_position.unwrap_or(enemy.position);
                    let delta_x = shortest_wrapped_delta(enemy.position.x, target.x, world_span);
                    let vertical_delta = target.y - enemy.position.y;
                    let horizontal_close_band = rom_mutant_seek_band();
                    let vertical_close_band = rom_mutant_avoid_band(min_y, max_y);
                    let on_main_screen = screen_x_for_world_x(
                        enemy.position.x,
                        left_edge,
                        width,
                        world_span,
                        world_max_x,
                    )
                    .is_some();

                    // `SCZ0` uses `SZXV` to seek the player horizontally every
                    // cycle, then switches between the `SZYV` seek and avoid
                    // branches depending on the horizontal close band.
                    enemy.velocity.dx = if delta_x == 0 {
                        enemy.velocity.dx.signum().max(1) * mutant_speed_x
                    } else {
                        delta_x.signum() * mutant_speed_x
                    };
                    enemy.velocity.dy = if delta_x.abs() <= horizontal_close_band {
                        vertical_delta.signum() * mutant_speed_y
                    } else if vertical_delta.abs() <= vertical_close_band {
                        -vertical_delta.signum() * mutant_speed_y
                    } else {
                        0
                    };
                    if enemy.velocity.dy == 0 && enemy.position.y <= min_y {
                        enemy.velocity.dy = mutant_speed_y;
                    }
                    if on_main_screen {
                        // `SCZ10` applies a signed `SZRY` hop directly to
                        // `OY16` once the Mutant is on screen, using the live
                        // `SEED` sign bit to pick the direction. Preserve that
                        // visible cabinet jitter here before the motion step.
                        rand_state.advance();
                        enemy.position.y = rom_mutant_random_y_hop(
                            enemy.position,
                            wave_profile.mutant_random_y,
                            rand_state.seed,
                            min_y,
                            max_y,
                        );
                    }
                }
                EntityKind::Baiter => {
                    let target = player_position.unwrap_or(enemy.position);
                    let stationary = enemy.velocity.dx == 0 && enemy.velocity.dy == 0;
                    let (cycle_counter, refresh_due) =
                        rom_baiter_cycle_step(rom_baiter_cycle_counter_for_entity(enemy));
                    enemy.rom_aux = rom_pack_enemy_state(
                        rom_enemy_shot_timer_for_entity(enemy, self.status.wave),
                        cycle_counter,
                    );
                    // `UFOLP` only refreshes Baiter/UFO velocity when the image
                    // cycle wraps, and `UFONV` gates that retarget through the
                    // live `SEED > UFOSK` check before applying the cabinet
                    // close-band exits.
                    if stationary
                        || (refresh_due
                            && rom_baiter_should_seek(
                                rand_state.seed,
                                wave_profile.baiter_seek_probability,
                            ))
                    {
                        enemy.velocity = rom_baiter_seek_velocity(
                            enemy.position,
                            enemy.velocity,
                            RomBaiterSeekContext {
                                target,
                                player_velocity: player_velocity
                                    .unwrap_or(Velocity { dx: 0, dy: 0 }),
                                world_span,
                                screen_width: self.width,
                                min_y,
                                max_y,
                            },
                        );
                    }
                }
                EntityKind::Bomber => {
                    let tie_speed = rom_tie_horizontal_velocity(wave_profile.bomber_x_velocity);
                    let direction = match enemy.velocity.dx.signum() {
                        -1 | 1 => enemy.velocity.dx.signum(),
                        _ => 1,
                    };
                    initialize_tie_cruise_altitude(enemy);
                    let on_main_screen = screen_x_for_world_x(
                        enemy.position.x,
                        left_edge,
                        width,
                        world_span,
                        world_max_x,
                    )
                    .is_some();
                    let (seed, _) =
                        rom_bomber_seed_bytes(self.status.wave, self.tick, enemy.position);
                    enemy.rom_aux = rom_tie_next_cruise_altitude(enemy.rom_aux, seed);
                    let cruise_altitude =
                        scale_rom_display_y_to_world(i32::from(enemy.rom_aux), min_y, max_y);
                    let close_band = rom_tie_close_band(min_y, max_y);
                    let far_band = rom_tie_far_band(min_y, max_y);

                    // `TIEST` seeds a constant horizontal velocity from `TIEXV`;
                    // the later `TIE` process and should not alter that base X
                    // speed. The Y path updates `OYV` through banded accel
                    // solves instead of hard-resetting the velocity every tick.
                    enemy.velocity.dx = direction * tie_speed;

                    let next_vertical_step = if on_main_screen {
                        let delta = enemy.position.y.saturating_sub(
                            player_position
                                .map(|target| target.y)
                                .unwrap_or(enemy.position.y),
                        );
                        if delta >= far_band || (delta < 0 && delta >= -close_band) {
                            -1
                        } else if (delta > 0 && delta <= close_band) || delta <= -far_band {
                            1
                        } else {
                            0
                        }
                    } else {
                        let delta = cruise_altitude - enemy.position.y;
                        if delta.abs() > close_band {
                            delta.signum()
                        } else {
                            0
                        }
                    };
                    enemy.velocity.dy =
                        rom_accumulate_vertical_velocity(enemy.velocity.dy, next_vertical_step);
                }
                EntityKind::Pod => {
                    if enemy.velocity.dx == 0 || enemy.velocity.dy == 0 {
                        // `PRBST` seeds PROBE start velocities from the live
                        // random registers instead of inventing a special AI
                        // roam task. Preserve that shape here by deriving a
                        // deterministic replacement velocity from the current
                        // world position/tick whenever a test or gameplay edge
                        // case creates a zeroed Pod velocity.
                        let fallback = rom_probe_velocity_seed(
                            self.status.wave,
                            ((enemy.position.x as u32) << 8)
                                ^ (enemy.position.y as u32)
                                ^ self.tick,
                        );
                        if enemy.velocity.dx == 0 {
                            enemy.velocity.dx = fallback.dx;
                        }
                        if enemy.velocity.dy == 0 {
                            enemy.velocity.dy = fallback.dy;
                        }
                    }
                }
                EntityKind::Swarmer => {
                    let swarmer_speed =
                        rom_swarmer_horizontal_velocity(wave_profile.swarmer_x_velocity);
                    let swarmer_accel = rom_swarmer_acceleration_for_entity(
                        enemy,
                        wave_profile.swarmer_acceleration_mask,
                    );
                    let swarmer_vertical_limit = rom_swarmer_vertical_speed_limit(swarmer_accel);
                    let target = player_position.unwrap_or(enemy.position);
                    let delta_x = shortest_wrapped_delta(enemy.position.x, target.x, world_span);
                    enemy.velocity.dx = if delta_x == 0 {
                        enemy.velocity.dx.signum().max(1) * swarmer_speed
                    } else {
                        delta_x.signum() * swarmer_speed
                    };
                    let shot_timer =
                        rom_swarmer_shot_timer_for_entity(enemy, wave_profile.swarmer_shot_time)
                            .saturating_sub(1);
                    enemy.rom_aux = rom_pack_swarmer_state(swarmer_accel, shot_timer);

                    let vertical_delta = target.y - enemy.position.y;
                    if vertical_delta != 0
                        && rom_swarmer_vertical_refresh_due(
                            self.tick,
                            enemy.position,
                            swarmer_accel,
                        )
                    {
                        enemy.velocity.dy = (enemy.velocity.dy
                            + vertical_delta.signum() * swarmer_vertical_limit)
                            .clamp(-swarmer_vertical_limit, swarmer_vertical_limit);
                    }
                    if enemy.velocity.dy == 0 {
                        enemy.velocity.dy = if self.tick.is_multiple_of(2) {
                            swarmer_vertical_limit
                        } else {
                            -swarmer_vertical_limit
                        };
                    }
                }
                _ => {}
            }

            let vertical_limit = if enemy.kind == EntityKind::Mutant {
                rom_mutant_vertical_velocity(
                    wave_profile.mutant_y_velocity_msb,
                    wave_profile.mutant_y_velocity_lsb,
                )
            } else {
                1
            };
            enemy.velocity.dy = enemy.velocity.dy.clamp(-vertical_limit, vertical_limit);
            enemy.position.y = enemy.position.y.clamp(min_y, max_y);
        }
    }

    fn drop_bomber_mines(&mut self, min_y: i32, max_y: i32) {
        // `BOMBST` uses the same `BMBCNT` counter as hostile shells but clamps
        // it at ten before allowing a new mine to start.
        if self.live_shell_count() >= rom_bomb_shell_limit() {
            return;
        }

        let max_x = self.world_max_x();
        let mut new_mines = Vec::new();

        for bomber in self
            .entities
            .iter()
            .filter(|entity| entity.kind == EntityKind::Bomber)
        {
            // `TIE31 -> BOMBST` drops bombs on the `LSEED & #$07 == 0` edge
            // rather than on a fixed timer, and `BMBCNT` caps the live bomb
            // list at ten. Keep the port on that cabinet gate.
            if !rom_bomber_should_drop_mine(self.status.wave, self.tick, bomber.position) {
                continue;
            }

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
                if let Some(mine) = new_mines.last_mut() {
                    // `BOMBST` seeds `ODATA` from `SEED & #$1F + 1`, and the
                    // shell scan retires bombs once that countdown reaches
                    // zero. Keep live bombs on that source-backed lifetime.
                    mine.rom_aux =
                        rom_bomber_mine_lifetime(self.status.wave, self.tick, bomber.position);
                }
            }
        }

        let available = rom_bomb_shell_limit().saturating_sub(self.live_shell_count());
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
                lander.rom_aux =
                    rom_pack_enemy_shot_timer(rom_mutant_transformed_shot_timer(self.status.wave));
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

    fn rom_enemy_shot_seed_bytes(&self) -> (u8, u8) {
        // `SHOOT` reads the live shared random-byte state directly.
        (self.rand_state.seed, self.rand_state.lseed)
    }

    fn should_auto_fire(&self, origin: Position, max_x: i32, min_y: i32, max_y: i32) -> bool {
        let shot_position = Position {
            x: wrap_coordinate(
                origin.x + self.player_facing.step() * (arcade_tables().player_shot_speed + 1),
                max_x,
            ),
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
            // Arcade History and StrategyWiki both describe only shallow drops as
            // survivable without an actual catch.
            human.rescue_value = if safe_y - human.position.y <= tables.safe_fall_height {
                tables.safe_fall_score
            } else {
                0
            };
        }
    }

    fn spawn_swarmer_burst_at(&mut self, pod_position: Position) {
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        let available = rom_swarmer_count_limit()
            .saturating_sub(self.entity_count_by_kind(EntityKind::Swarmer));
        if available == 0 {
            return;
        }

        let count = rom_probe_swarmer_burst_count(self.rand_state.seed).min(available);
        let min_y = 1;
        let max_y = self.safe_altitude_at_world_x(pod_position.x);

        for _ in 0..count {
            let velocity = rom_randv_velocity(&mut self.rand_state);
            let acceleration = rom_swarmer_acceleration(
                self.rand_state.lseed,
                wave_profile.swarmer_acceleration_mask,
            );
            let shot_timer = rom_seeded_rmax(
                &mut self.rand_state,
                wave_profile.swarmer_shot_time.min(u32::from(u8::MAX)) as u8,
            );
            let mut swarmer = Entity::new(
                EntityKind::Swarmer,
                pod_position.x,
                pod_position.y.clamp(min_y, max_y),
                velocity.dx,
                velocity.dy,
            );
            swarmer.rom_aux = rom_pack_swarmer_state(acceleration, shot_timer);
            self.entities.push(swarmer);
        }
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
            // Doug Mahugh chapter 02 documents the arcade Smart Bomb rule:
            // enemies on the main screen are destroyed, but bullets and mines
            // survive in normal play. XYZZY intentionally extends that.
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
            // Arcade History and StrategyWiki describe only safe-drop saves in
            // the cabinet game; XYZZY deliberately overrides that rule.
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
            lander.rom_aux =
                rom_pack_enemy_shot_timer(rom_mutant_transformed_shot_timer(self.status.wave));
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
        let width = self.width;
        let world_span = self.world_span;
        let left_edge = self.left_edge();
        for entity in &mut self.entities {
            match entity.kind {
                EntityKind::PlayerShot | EntityKind::EnemyShot => {
                    if entity.rom_aux == 0 {
                        // `GETSHL` seeds the shared shell lifetime in `ODATA`
                        // to 20. Keep raw shell fixtures and freshly spawned
                        // live shells on that cabinet countdown.
                        entity.rom_aux = rom_shell_lifetime();
                    } else {
                        entity.rom_aux -= 1;
                    }
                }
                EntityKind::Mine => {
                    if entity.rom_aux > 0 {
                        entity.rom_aux -= 1;
                    }
                }
                _ => {}
            }
        }
        self.entities.retain(|entity| match entity.kind {
            EntityKind::PlayerShot => {
                screen_x_for_world_x(entity.position.x, left_edge, width, world_span, max_x)
                    .is_some()
                    && entity.rom_aux > 0
                    && (min_y..=max_y).contains(&entity.position.y)
                    && entity.position.y < terrain_surface_y(terrain, entity.position.x) + 1
            }
            EntityKind::EnemyShot => {
                (0..=max_x).contains(&entity.position.x)
                    && entity.rom_aux > 0
                    && (min_y..=max_y).contains(&entity.position.y)
                    && entity.position.y < terrain_surface_y(terrain, entity.position.x) + 1
            }
            EntityKind::Mine => entity.rom_aux > 0,
            _ => true,
        });
    }

    fn can_fire_player_shot(&self, secret_mode: bool) -> bool {
        secret_mode
            || self.entity_count_by_kind(EntityKind::PlayerShot) < arcade_tables().player_shot_limit
    }

    fn spawn_baiter_if_needed(&mut self, min_y: i32, max_y: i32) {
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        if self.entity_count_by_kind(EntityKind::Lander) == 0 {
            return;
        }

        let remaining = self.remaining_wave_enemy_count();
        if remaining == 0 {
            return;
        }

        // `GEXEC` drives Baiter/UFO pressure through the persistent `UFOTMR`
        // countdown instead of a fixed repeat delay. Keep the live port on
        // that timer/cap path.
        self.baiter_timer =
            rom_advance_baiter_timer(self.baiter_timer, wave_profile.baiter_delay, remaining);
        if self.baiter_timer != 0 {
            return;
        }
        self.baiter_timer = rom_reset_baiter_timer(
            self.status.wave,
            self.tick,
            wave_profile.baiter_delay,
            remaining,
        );
        if self.entity_count_by_kind(EntityKind::Baiter) >= rom_baiter_count_limit() {
            return;
        }

        let spawn = rom_baiter_spawn_for_wave(
            self.status.wave,
            self.tick,
            self.left_edge(),
            self.width,
            self.world_max_x(),
            min_y,
            max_y,
        );
        let safe_y = self.safe_altitude_at_world_x(spawn.x).min(max_y);
        let y = spawn.y.clamp(min_y, safe_y);
        let player_position = self.player_position().unwrap_or(spawn);
        let initial_velocity = rom_baiter_seek_velocity(
            Position { x: spawn.x, y },
            Velocity { dx: 0, dy: 0 },
            RomBaiterSeekContext {
                target: player_position,
                player_velocity: self.player_velocity().unwrap_or(Velocity { dx: 0, dy: 0 }),
                world_span: self.world_span,
                screen_width: self.width,
                min_y,
                max_y,
            },
        );

        let mut baiter = Entity::new(
            EntityKind::Baiter,
            spawn.x,
            y,
            initial_velocity.dx,
            initial_velocity.dy,
        );
        initialize_rom_enemy_state(&mut baiter, self.status.wave, &mut self.rand_state);
        self.entities.push(baiter);
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
        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        let width = self.world_span;
        let min_y = 1;
        let max_y = self.height as i32 - 3;
        let tie_speed = rom_tie_horizontal_velocity(wave_profile.bomber_x_velocity);
        let bomber_origin = self
            .player_position()
            .map(|player| wrap_coordinate(player.x + width / 2, width - 1))
            .unwrap_or(width / 2);
        if self.status.wave.is_multiple_of(5) {
            // Doug Mahugh chapter 01 and StrategyWiki both call out the
            // five-wave planet restore with a fresh humanoid population.
            self.restore_default_humans();
        }

        let opening_enemy = if self.has_humanoids() {
            EntityKind::Lander
        } else {
            EntityKind::Mutant
        };
        // `WVTAB` in `blk71.src` drives the baseline red-label wave roster:
        // landers launch in repeated squads, while TIEs/Bombers and
        // PROBEs/Pods are restored immediately for the current wave.
        let group_size = wave_profile.wave_size.min(wave_profile.landers).max(1);
        let mut enemies = default_attack_wave_openers(
            width,
            self.status.wave,
            opening_enemy,
            0,
            group_size as usize,
            &mut self.rand_state,
        );

        let bomber_slots = [
            (bomber_origin, -tie_speed),
            (wrap_coordinate(bomber_origin + 10, width - 1), tie_speed),
            (wrap_coordinate(bomber_origin - 10, width - 1), -tie_speed),
            (wrap_coordinate(bomber_origin + 20, width - 1), tie_speed),
            (wrap_coordinate(bomber_origin - 20, width - 1), -tie_speed),
        ];
        for (x, dx) in bomber_slots.into_iter().take(wave_profile.bombers as usize) {
            enemies.push(Entity::new(
                EntityKind::Bomber,
                x,
                self.safe_altitude_at_world_x(x).saturating_sub(1),
                dx,
                0,
            ));
        }

        for _ in 0..wave_profile.pods {
            let (screen_x, y, velocity) =
                rom_probe_spawn_for_wave(&mut self.rand_state, self.width, min_y, max_y);
            let x = self.world_x_for_screen_x(screen_x);
            let y = y.min(self.safe_altitude_at_world_x(x)).max(min_y);
            enemies.push(Entity::new(EntityKind::Pod, x, y, velocity.dx, velocity.dy));
        }

        self.entities.extend(enemies);
        self.baiter_timer = wave_profile.baiter_delay;
        self.pending_wave_openers = wave_profile.landers.saturating_sub(group_size);
        self.spawned_wave_opener_groups = 1;
        self.next_wave_reinforcement_tick = self.tick + wave_profile.wave_time;
    }

    fn has_humanoids(&self) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.kind == EntityKind::Human)
    }

    fn restore_default_humans(&mut self) {
        self.entities
            .retain(|entity| entity.kind != EntityKind::Human);
        self.entities
            .extend(default_humans(&self.terrain, self.world_span));
    }

    fn spawn_attack_wave_reinforcements_if_due(&mut self) {
        if self.pending_wave_openers == 0 || self.tick < self.next_wave_reinforcement_tick {
            return;
        }

        let wave_profile = red_label_wave_table().profile_for_wave(self.status.wave);
        let opening_enemy = if self.has_humanoids() {
            EntityKind::Lander
        } else {
            EntityKind::Mutant
        };
        let group_index = self.spawned_wave_opener_groups;
        let group_size = wave_profile.wave_size.min(self.pending_wave_openers);
        let reinforcements = default_attack_wave_openers(
            self.world_span,
            self.status.wave,
            opening_enemy,
            group_index,
            group_size as usize,
            &mut self.rand_state,
        );
        self.entities.extend(reinforcements);

        self.pending_wave_openers = self.pending_wave_openers.saturating_sub(group_size);
        self.spawned_wave_opener_groups = self.spawned_wave_opener_groups.saturating_add(1);
        if self.pending_wave_openers > 0 {
            self.next_wave_reinforcement_tick = self.tick.saturating_add(wave_profile.wave_time);
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

fn screen_x_for_world_x(
    world_x: i32,
    left_edge: i32,
    width: usize,
    world_span: i32,
    world_max_x: i32,
) -> Option<usize> {
    if !(0..=world_max_x).contains(&world_x) {
        return None;
    }

    let offset = (world_x - left_edge).rem_euclid(world_span);
    (offset < width as i32).then_some(offset as usize)
}

fn wrapped_horizontal_step(from: i32, to: i32, max_x: i32) -> i32 {
    shortest_wrapped_delta(from, to, max_x + 1).signum()
}

fn rom_shoot_axes_from_seeds(
    enemy_screen_x: i32,
    enemy_y: i32,
    player_screen_x: i32,
    player_y: i32,
    player_velocity: Velocity,
    seed: u8,
    lseed: u8,
) -> (i32, i32) {
    let mut horizontal = i32::from(seed & 0x1F) - 0x10 + player_screen_x - enemy_screen_x;
    horizontal *= 4;
    if seed > 120 {
        horizontal += player_velocity.dx * 4;
    }

    let mut vertical = i32::from(lseed & 0x1F) - 0x10 + player_y - enemy_y;
    vertical *= 4;

    (horizontal, vertical)
}

fn quantize_rom_shot_component(raw: i32) -> i32 {
    match raw {
        i32::MIN..=-48 => -2,
        -47..=-12 => -1,
        -11..=11 => 0,
        12..=47 => 1,
        _ => 2,
    }
}

fn rom_shoot_velocity(
    enemy_screen_x: i32,
    enemy_y: i32,
    player_screen_x: i32,
    player_y: i32,
    player_velocity: Velocity,
    seed: u8,
    lseed: u8,
) -> Velocity {
    let (horizontal, vertical) = rom_shoot_axes_from_seeds(
        enemy_screen_x,
        enemy_y,
        player_screen_x,
        player_y,
        player_velocity,
        seed,
        lseed,
    );
    let mut dx = quantize_rom_shot_component(horizontal);
    let dy = quantize_rom_shot_component(vertical);

    if dx == 0 && dy == 0 {
        dx = match (player_screen_x - enemy_screen_x).signum() {
            -1 | 1 => (player_screen_x - enemy_screen_x).signum(),
            _ => 1,
        };
    }

    Velocity { dx, dy }
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

fn scale_rom_probe_x_fixed_to_screen(rom_x: u16, screen_width: usize) -> usize {
    const ROM_XMIN: u16 = 0x1000;
    const ROM_XMAX: u16 = 0x4FFF;

    if screen_width <= 1 {
        return 0;
    }

    let max_screen_x = screen_width as u32 - 1;
    let span = u32::from(ROM_XMAX - ROM_XMIN);
    let offset = u32::from(rom_x.saturating_sub(ROM_XMIN)).min(span);

    (offset * max_screen_x / span) as usize
}

fn rom_probe_spawn_for_wave(
    rand_state: &mut RomRandState,
    screen_width: usize,
    min_y: i32,
    max_y: i32,
) -> (usize, i32, Velocity) {
    rand_state.advance();
    let rom_x_hi = (rand_state.hseed & 0x3F).wrapping_add(0x10);
    let rom_x = u16::from_be_bytes([rom_x_hi, rand_state.lseed]);
    let rom_y = i32::from(rand_state.lseed >> 1) + 42;
    let screen_x = scale_rom_probe_x_fixed_to_screen(rom_x, screen_width);
    let y = scale_rom_probe_y_to_world(rom_y, min_y, max_y);

    (
        screen_x,
        y,
        Velocity {
            dx: rom_probe_horizontal_velocity(rand_state.seed),
            dy: rom_probe_vertical_velocity(rand_state.lseed),
        },
    )
}

fn rom_probe_velocity_seed(wave: u8, salt: u32) -> Velocity {
    let (_, _, seed, lseed) = rom_probe_seed_bytes(wave, salt, 0);
    Velocity {
        dx: rom_probe_horizontal_velocity(seed),
        dy: rom_probe_vertical_velocity(lseed),
    }
}

fn rom_rmax(max: u8, mut seed: u8) -> usize {
    while seed > max {
        seed >>= 1;
    }
    usize::from(seed) + 1
}

fn rom_probe_swarmer_burst_count(seed: u8) -> usize {
    rom_rmax(6, seed)
}

fn rom_randv_velocity(rand_state: &mut RomRandState) -> Velocity {
    rand_state.advance();
    Velocity {
        dx: rom_probe_horizontal_velocity(rand_state.lseed),
        dy: rom_randv_vertical_velocity(rand_state.seed),
    }
}

fn rom_randv_vertical_velocity(seed: u8) -> i32 {
    let raw = i16::from(seed as i8) * 2;
    match raw {
        ..=-128 => -2,
        -127..=-1 => -1,
        0..=127 => 1,
        _ => 2,
    }
}

fn rom_baiter_count_limit() -> usize {
    12
}

fn rom_baiter_timer_floor(base_delay: u32, remaining: usize) -> u32 {
    if remaining > 8 {
        return base_delay;
    }

    let mut floor = base_delay / 2;
    if remaining <= 3 {
        floor /= 2;
    }
    floor + 1
}

fn rom_advance_baiter_timer(current: u32, base_delay: u32, remaining: usize) -> u32 {
    let mut timer = current.max(1);
    if remaining <= 8 {
        timer = timer.min(rom_baiter_timer_floor(base_delay, remaining));
    }
    timer.saturating_sub(1)
}

fn rom_reset_baiter_timer(wave: u8, tick: u32, base_delay: u32, remaining: usize) -> u32 {
    if remaining >= 4 {
        return base_delay;
    }

    let (_, _, seed, _) = rom_probe_seed_bytes(wave, tick, 0x5546_4f54 ^ remaining as u32);
    rom_rmax((base_delay / 4).min(u32::from(u8::MAX)) as u8, seed) as u32
}

fn rom_probe_seed_bytes(wave: u8, tick: u32, salt: u32) -> (u8, u8, u8, u8) {
    let mut mixed = tick
        .wrapping_mul(0x045d_9f3b)
        .rotate_left(7)
        .wrapping_add(u32::from(wave).wrapping_mul(0x9e37))
        ^ salt.wrapping_mul(0x9e37_79b9);
    mixed ^= mixed >> 16;
    mixed = mixed.wrapping_mul(0x7feb_352d);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x846c_a68b);
    mixed ^= mixed >> 16;

    (
        (mixed >> 24) as u8,
        (mixed >> 16) as u8,
        (mixed >> 8) as u8,
        mixed as u8,
    )
}

fn rom_bomber_seed_bytes(wave: u8, tick: u32, position: Position) -> (u8, u8) {
    let (hseed, lseed, _, _) = rom_probe_seed_bytes(
        wave,
        tick,
        ((position.x as u32) << 8) ^ (position.y as u32) ^ 0x5449_4500,
    );
    (hseed, lseed)
}

fn rom_bomber_should_drop_mine(wave: u8, tick: u32, position: Position) -> bool {
    let (_, lseed) = rom_bomber_seed_bytes(wave, tick, position);
    (lseed & 0x07) == 0
}

fn rom_bomber_mine_lifetime(wave: u8, tick: u32, position: Position) -> u16 {
    let (hseed, _) = rom_bomber_seed_bytes(wave, tick, position);
    u16::from((hseed & 0x1F) + 1)
}

fn rom_shell_lifetime() -> u16 {
    20
}

const ROM_ENEMY_STATE_TAG_MASK: u8 = 0x80;
const ROM_BAITER_CYCLE_LENGTH: u8 = 18;

fn rom_pack_enemy_state(timer: u8, aux: u8) -> u16 {
    u16::from_be_bytes([ROM_ENEMY_STATE_TAG_MASK | aux, timer])
}

fn rom_pack_enemy_shot_timer(timer: u8) -> u16 {
    rom_pack_enemy_state(timer, 0)
}

fn rom_enemy_shot_timer_is_initialized(entity: &Entity) -> bool {
    let [tag, _] = entity.rom_aux.to_be_bytes();
    (tag & ROM_ENEMY_STATE_TAG_MASK) != 0
}

fn rom_enemy_aux_for_entity(entity: &Entity) -> u8 {
    let [tag, _] = entity.rom_aux.to_be_bytes();
    tag & !ROM_ENEMY_STATE_TAG_MASK
}

fn rom_enemy_shot_timer_for_entity(enemy: &Entity, wave: u8) -> u8 {
    let [tag, timer] = enemy.rom_aux.to_be_bytes();
    if (tag & ROM_ENEMY_STATE_TAG_MASK) != 0 {
        timer
    } else {
        rom_initial_enemy_shot_timer(enemy.kind, wave, &mut RomRandState::default())
    }
}

fn rom_enemy_shot_time_limit(kind: EntityKind, wave: u8) -> u8 {
    let wave_profile = red_label_wave_table().profile_for_wave(wave);
    match kind {
        EntityKind::Lander => wave_profile.lander_shot_time,
        EntityKind::Mutant => wave_profile.mutant_shot_time,
        EntityKind::Baiter => wave_profile.baiter_shot_time,
        _ => 0,
    }
    .max(1)
    .min(u32::from(u8::MAX)) as u8
}

fn rom_baiter_initial_shot_timer() -> u8 {
    8
}

fn rom_baiter_cycle_counter_for_entity(enemy: &Entity) -> u8 {
    rom_enemy_aux_for_entity(enemy)
}

fn rom_baiter_cycle_step(counter: u8) -> (u8, bool) {
    let next = (counter + 1) % ROM_BAITER_CYCLE_LENGTH;
    (next, next == 0)
}

fn rom_mutant_transformed_shot_timer(wave: u8) -> u8 {
    rom_enemy_shot_time_limit(EntityKind::Mutant, wave)
}

fn rom_initial_enemy_shot_timer(kind: EntityKind, wave: u8, rand_state: &mut RomRandState) -> u8 {
    match kind {
        EntityKind::Lander | EntityKind::Mutant => {
            rom_seeded_rmax(rand_state, rom_enemy_shot_time_limit(kind, wave))
        }
        EntityKind::Baiter => rom_baiter_initial_shot_timer(),
        _ => 0,
    }
}

fn rom_reset_enemy_shot_timer(kind: EntityKind, wave: u8, rand_state: &mut RomRandState) -> u8 {
    match kind {
        EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter => {
            rom_seeded_rmax(rand_state, rom_enemy_shot_time_limit(kind, wave))
        }
        _ => 0,
    }
}

fn initialize_rom_enemy_state(entity: &mut Entity, wave: u8, rand_state: &mut RomRandState) {
    if rom_enemy_shot_timer_is_initialized(entity) {
        return;
    }

    match entity.kind {
        EntityKind::Lander | EntityKind::Mutant | EntityKind::Baiter => {
            let timer = rom_initial_enemy_shot_timer(entity.kind, wave, rand_state);
            entity.rom_aux = if entity.kind == EntityKind::Baiter {
                rom_pack_enemy_state(timer, 0)
            } else {
                rom_pack_enemy_shot_timer(timer)
            };
        }
        _ => {}
    }
}

fn rom_swarmer_count_limit() -> usize {
    20
}

fn rom_swarmer_acceleration(lseed: u8, mask: u8) -> u8 {
    lseed & mask
}

fn rom_pack_swarmer_state(acceleration: u8, shot_timer: u8) -> u16 {
    u16::from_be_bytes([acceleration, shot_timer])
}

fn rom_swarmer_acceleration_for_entity(enemy: &Entity, wave_mask: u8) -> u8 {
    let [seeded, _] = enemy.rom_aux.to_be_bytes();
    if seeded == 0 { wave_mask } else { seeded }
}

fn rom_swarmer_shot_timer_for_entity(enemy: &Entity, wave_shot_time: u32) -> u8 {
    let [acceleration, shot_timer] = enemy.rom_aux.to_be_bytes();
    if acceleration == 0 && shot_timer == 0 {
        wave_shot_time.min(u32::from(u8::MAX)) as u8
    } else {
        shot_timer
    }
}

fn rom_seeded_rmax(rand_state: &mut RomRandState, max: u8) -> u8 {
    rand_state.advance();
    rom_rmax(max, rand_state.seed) as u8
}

fn rom_shell_spawn_limit() -> usize {
    20
}

fn rom_bomb_shell_limit() -> usize {
    10
}

fn initialize_tie_cruise_altitude(entity: &mut Entity) {
    if entity.rom_aux == 0 {
        entity.rom_aux = 0x50;
    }
}

fn rom_tie_next_cruise_altitude(current: u16, seed: u8) -> u16 {
    let mut next = current.max(0x40);
    if seed <= 0x40 {
        let delta = i32::from(seed & 0x03) - 2;
        next = (i32::from(next) + delta).clamp(0x40, 0x68) as u16;
    }
    next
}

fn rom_accumulate_vertical_velocity(current: i32, step: i32) -> i32 {
    (current + step).clamp(-2, 2)
}

#[cfg(test)]
fn scale_rom_probe_x_to_screen(rom_x: i32, screen_width: usize) -> usize {
    if screen_width <= 1 {
        return 0;
    }

    let max_screen_x = screen_width as i32 - 1;
    ((rom_x - 0x10) * max_screen_x / 0x3F).clamp(0, max_screen_x) as usize
}

fn scale_rom_probe_y_to_world(rom_y: i32, min_y: i32, max_y: i32) -> i32 {
    if max_y <= min_y {
        return min_y;
    }

    let span = max_y - min_y;
    min_y + ((rom_y - 42) * span / 0x7F).clamp(0, span)
}

fn rom_baiter_spawn_for_wave(
    wave: u8,
    tick: u32,
    left_edge: i32,
    screen_width: usize,
    world_max_x: i32,
    min_y: i32,
    max_y: i32,
) -> Position {
    let (hseed_hi, hseed_lo, _, _) = rom_probe_seed_bytes(wave, tick, 0x5546_4f00);
    let max_screen_x = screen_width.saturating_sub(1) as i32;
    let screen_x = (i32::from(hseed_hi & 0x1F) * max_screen_x / 0x1F).clamp(0, max_screen_x);
    let rom_y = i32::from(hseed_lo >> 1) + 42;

    Position {
        x: wrap_coordinate(left_edge + screen_x, world_max_x),
        y: scale_rom_probe_y_to_world(rom_y, min_y, max_y),
    }
}

fn scale_rom_display_y_to_world(rom_y: i32, min_y: i32, max_y: i32) -> i32 {
    const ROM_YMIN: i32 = 42;
    const ROM_YMAX: i32 = 0xA8;

    if max_y <= min_y {
        return min_y;
    }

    let span = max_y - min_y;
    min_y + ((rom_y - ROM_YMIN) * span / (ROM_YMAX - ROM_YMIN)).clamp(0, span)
}

fn rom_tie_horizontal_velocity(raw: u8) -> i32 {
    // `TIEXV` is stored in the cabinet fixed-point scale; map the red-label
    // record back into the coarse live grid without reintroducing the old
    // hand-authored Bomber fast/slow fallback.
    if raw <= 0x28 { 1 } else { 2 }
}

fn rom_lander_horizontal_velocity(raw: u8) -> i32 {
    // `LANDST` seeds `OXV` from `LNDXV`. Waves 1-2 remain in the slower
    // single-cell band, while later waves step into the faster 2-cell band.
    if raw < 0x20 { 1 } else { 2 }
}

fn rom_lander_vertical_velocity(msb: u8, _lsb: u8) -> i32 {
    // `LNDYV` is a 16-bit fixed-point velocity. Later waves step into the
    // faster `+$0100` band, which maps to a 2-cell coarse step.
    if msb == 0 { 1 } else { 2 }
}

fn rom_lander_altitude_offset(min_y: i32, max_y: i32) -> i32 {
    let offset = scale_rom_display_y_to_world(42 + 50, min_y, max_y)
        - scale_rom_display_y_to_world(42, min_y, max_y);
    offset.max(1)
}

fn rom_lander_cruise_altitude(surface_y: i32, min_y: i32, max_y: i32) -> i32 {
    (surface_y - rom_lander_altitude_offset(min_y, max_y)).clamp(min_y, max_y)
}

fn rom_lander_cruise_velocity(current_y: i32, cruise_y: i32, speed: i32) -> i32 {
    let delta = cruise_y - current_y;
    let tolerance = 1;
    if delta > 0 {
        speed
    } else if delta < -tolerance {
        -speed
    } else {
        0
    }
}

fn rom_lander_target_is_aligned(position: Position, target: Position, world_span: i32) -> bool {
    shortest_wrapped_delta(position.x, target.x, world_span).abs() <= 1
}

fn rom_lander_grab_velocity(current_y: i32, target_y: i32, speed: i32) -> i32 {
    let desired_y = target_y.saturating_sub(1);
    (desired_y - current_y).signum() * speed
}

fn normalize_or_seed_horizontal_velocity(current_dx: i32, speed: i32, x: i32, tick: u32) -> i32 {
    if current_dx == 0 {
        if (((tick as i32) + x).rem_euclid(2)) == 0 {
            speed
        } else {
            -speed
        }
    } else {
        current_dx.signum() * speed
    }
}

fn rom_mutant_horizontal_velocity(raw: u8) -> i32 {
    // `SCZ0` reloads `SZXV` every cycle for the Schitzo/Mutant X seek. Map the
    // red-label fixed-point record into the coarse live grid so later waves
    // keep the faster cabinet pursuit speed.
    if raw < 0x20 { 1 } else { 2 }
}

fn rom_mutant_vertical_velocity(msb: u8, _lsb: u8) -> i32 {
    // `SZYV` is stored as a 16-bit fixed-point velocity. Waves 1-2 remain in
    // the sub-`$0100` range, while later waves step into the faster 2-cell
    // band.
    if msb == 0 { 1 } else { 2 }
}

fn rom_mutant_seek_band() -> i32 {
    // `SCZ0` treats the player as X-close once the `PLABX - OX16` delta lands
    // in the `#380 .. #700` range, which maps to about ten coarse cells in the
    // live grid.
    10
}

fn rom_mutant_avoid_band(min_y: i32, max_y: i32) -> i32 {
    let close = scale_rom_display_y_to_world(42 + 8, min_y, max_y)
        - scale_rom_display_y_to_world(42, min_y, max_y);
    close.max(1)
}

fn rom_mutant_random_y_hop(
    position: Position,
    random_y: u8,
    seed: u8,
    min_y: i32,
    max_y: i32,
) -> i32 {
    if random_y == 0 {
        return position.y;
    }

    let delta = if (seed & 0x80) == 0 {
        i32::from(random_y)
    } else {
        -i32::from(random_y)
    };
    let hopped_y = position.y + delta;

    // `SCZ10` applies the signed `SZRY` hop directly to `OY16` and only
    // patches the low underflow case back to `YMAX`. Keep the live world
    // in-bounds without introducing a top-wrap artifact on coarse rows.
    if hopped_y < min_y {
        max_y
    } else {
        hopped_y.clamp(min_y, max_y)
    }
}

fn rom_swarmer_horizontal_velocity(raw: u8) -> i32 {
    // `MSWM` loads `SWXV` directly for Swarm X velocity. Map the red-label
    // fixed-point record into the coarse live grid so later waves keep the
    // faster cabinet Swarmer pace without relying on a global fallback speed.
    if raw < 0x20 { 1 } else { 2 }
}

fn rom_swarmer_vertical_speed_limit(mask: u8) -> i32 {
    if mask > 0x1F { 2 } else { 1 }
}

fn rom_swarmer_vertical_refresh_due(tick: u32, position: Position, mask: u8) -> bool {
    let cadence = (((mask >> 4) & 0x3).max(1) + 1) as u32;
    (tick + position.x as u32 + position.y as u32).is_multiple_of(cadence)
}

fn rom_baiter_should_seek(seed: u8, seek_probability: u8) -> bool {
    seed > seek_probability
}

fn rom_baiter_horizontal_close_band(screen_width: usize) -> i32 {
    ((screen_width as i32 * 20) / 64).max(1)
}

fn rom_baiter_vertical_close_band(min_y: i32, max_y: i32) -> i32 {
    let close = scale_rom_display_y_to_world(42 + 10, min_y, max_y)
        - scale_rom_display_y_to_world(42, min_y, max_y);
    close.max(1)
}

fn rom_baiter_base_speed() -> i32 {
    // `UFONV0` seeds `XTEMP` from `#$4001`; the X seek uses the low byte.
    1
}

#[derive(Clone, Copy)]
struct RomBaiterSeekContext {
    target: Position,
    player_velocity: Velocity,
    world_span: i32,
    screen_width: usize,
    min_y: i32,
    max_y: i32,
}

fn rom_baiter_seek_velocity(
    baiter_position: Position,
    current_velocity: Velocity,
    context: RomBaiterSeekContext,
) -> Velocity {
    let horizontal_delta =
        shortest_wrapped_delta(baiter_position.x, context.target.x, context.world_span);
    let vertical_delta = context.target.y - baiter_position.y;
    let mut next = current_velocity;

    let horizontal_close_band = rom_baiter_horizontal_close_band(context.screen_width);
    if horizontal_delta.abs() > horizontal_close_band {
        let seek_dx = horizontal_delta.signum() * rom_baiter_base_speed();
        next.dx = (context.player_velocity.dx + seek_dx).clamp(-2, 2);
        if next.dx == 0 {
            next.dx = seek_dx;
        }
    }

    let vertical_close_band = rom_baiter_vertical_close_band(context.min_y, context.max_y);
    if vertical_delta.abs() > vertical_close_band {
        let seek_dy = vertical_delta.signum();
        let averaged_dy = seek_dy + context.player_velocity.dy;
        next.dy = if averaged_dy == 0 {
            seek_dy
        } else {
            averaged_dy.signum()
        };
    }

    next
}

fn rom_tie_close_band(min_y: i32, max_y: i32) -> i32 {
    let close = scale_rom_display_y_to_world(42 + 0x10, min_y, max_y)
        - scale_rom_display_y_to_world(42, min_y, max_y);
    close.max(1)
}

fn rom_tie_far_band(min_y: i32, max_y: i32) -> i32 {
    let far = scale_rom_display_y_to_world(42 + 0x20, min_y, max_y)
        - scale_rom_display_y_to_world(42, min_y, max_y);
    far.max(rom_tie_close_band(min_y, max_y) + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RomRandState {
    seed: u8,
    hseed: u8,
    lseed: u8,
}

impl Default for RomRandState {
    fn default() -> Self {
        Self {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        }
    }
}

impl RomRandState {
    fn advance(&mut self) {
        // `RAND` in `defa7.src` updates `SEED`, `HSEED`, and `LSEED` together.
        let mut product_low = self.seed.wrapping_mul(3).wrapping_add(17);
        let mut a = self.lseed >> 3;
        a ^= self.lseed;
        let carry_into_hseed = (a & 0x01) != 0;
        let old_hseed = self.hseed;
        self.hseed = (u8::from(carry_into_hseed) << 7) | (self.hseed >> 1);
        let carry_into_lseed = (old_hseed & 0x01) != 0;
        self.lseed = (u8::from(carry_into_lseed) << 7) | (self.lseed >> 1);
        let (with_lseed, carry) = adc8(product_low, self.lseed, false);
        let (new_seed, _) = adc8(with_lseed, self.hseed, carry);
        product_low = new_seed;
        self.seed = product_low;
    }
}

fn adc8(lhs: u8, rhs: u8, carry: bool) -> (u8, bool) {
    let sum = u16::from(lhs) + u16::from(rhs) + u16::from(u8::from(carry));
    ((sum & 0xFF) as u8, sum > 0xFF)
}

fn rom_probe_horizontal_velocity(seed: u8) -> i32 {
    let raw = i16::from(seed & 0x3F) - 0x20;
    match raw {
        ..=-16 => -2,
        -15..=-1 => -1,
        0..=15 => 1,
        _ => 2,
    }
}

fn rom_probe_vertical_velocity(seed: u8) -> i32 {
    let raw = i16::from(seed & 0x7F) - 0x40;
    if raw < 0 { -1 } else { 1 }
}

fn scale_rom_fixed_x_to_world(rom_x: u16, world_span: i32) -> i32 {
    if world_span <= 1 {
        return 0;
    }

    (((u32::from(rom_x) * world_span as u32) >> 16) as i32).min(world_span - 1)
}

fn rom_astst_spawn_x(addend: u8, world_span: i32, rand_state: &mut RomRandState) -> i32 {
    rand_state.advance();
    let rom_x_hi = (rand_state.hseed & 0x1F).wrapping_add(addend);
    let rom_x = u16::from_be_bytes([rom_x_hi, rand_state.lseed]);
    scale_rom_fixed_x_to_world(rom_x, world_span)
}

fn rom_default_human_world_xs(world_span: i32) -> Vec<i32> {
    const DEFAULT_HUMAN_COUNT: u8 = 10;

    let mut rand_state = RomRandState::default();
    let mut xs = Vec::with_capacity(DEFAULT_HUMAN_COUNT as usize);
    let per_quadrant = DEFAULT_HUMAN_COUNT >> 2;

    if DEFAULT_HUMAN_COUNT > 7 {
        let mut quadrant = 0_u8;
        loop {
            for _ in 0..per_quadrant {
                xs.push(rom_astst_spawn_x(quadrant, world_span, &mut rand_state));
            }
            quadrant = quadrant.wrapping_add(0x40);
            if quadrant == 0 {
                break;
            }
        }
    }

    let remainder = DEFAULT_HUMAN_COUNT - (per_quadrant << 2);
    for _ in 0..remainder {
        let addend = rand_state.hseed;
        xs.push(rom_astst_spawn_x(addend, world_span, &mut rand_state));
    }

    xs
}

fn default_humans(terrain: &[usize], world_span: i32) -> Vec<Entity> {
    rom_default_human_world_xs(world_span)
        .into_iter()
        .map(|x| Entity::new(EntityKind::Human, x, terrain_surface_y(terrain, x), 0, 0))
        .collect()
}

fn default_attack_wave_openers(
    world_span: i32,
    wave: u8,
    kind: EntityKind,
    group_index: u8,
    count: usize,
    rand_state: &mut RomRandState,
) -> Vec<Entity> {
    let max_x = world_span - 1;
    let wave_offset = i32::from(wave % 6);
    let group_offset = i32::from(group_index) * 14;
    let wave_profile = red_label_wave_table().profile_for_wave(wave);
    let lander_speed_x = rom_lander_horizontal_velocity(wave_profile.lander_x_velocity);
    let lander_speed_y = rom_lander_vertical_velocity(
        wave_profile.lander_y_velocity_msb,
        wave_profile.lander_y_velocity_lsb,
    );
    let mutant_speed_x = rom_mutant_horizontal_velocity(wave_profile.mutant_x_velocity);
    let mutant_speed_y = rom_mutant_vertical_velocity(
        wave_profile.mutant_y_velocity_msb,
        wave_profile.mutant_y_velocity_lsb,
    );
    [
        (
            world_span - 12 - group_offset,
            3 + wave_offset % 3,
            -1_i32,
            1_i32,
        ),
        (world_span - 6 - group_offset, 6, -1_i32, 1_i32),
        (18 + wave_offset + group_offset, 8, 1_i32, -1_i32),
        (world_span / 2 + 8 + group_offset, 4, -1_i32, 1_i32),
        (world_span / 2 - 10 - group_offset, 7, 1_i32, -1_i32),
    ]
    .into_iter()
    .take(count)
    .map(|(x, y, dx, dy)| {
        let (dx, dy) = match kind {
            EntityKind::Lander => (dx.signum() * lander_speed_x, lander_speed_y),
            EntityKind::Mutant => (dx.signum() * mutant_speed_x, dy.signum() * mutant_speed_y),
            _ => (dx, dy),
        };
        let mut entity = Entity::new(kind, wrap_coordinate(x, max_x), y, dx, dy);
        initialize_rom_enemy_state(&mut entity, wave, rand_state);
        entity
    })
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
    use crate::red_label_wave::red_label_wave_table;

    use super::{
        Entity, EntityKind, EntityState, HorizontalDirection, Position, RomBaiterSeekContext,
        RomRandState, Status, UpdateInput, Velocity, World, WorldEvent, hyperspace_result,
        initialize_tie_cruise_altitude, nearest_wrapped_target, rom_accumulate_vertical_velocity,
        rom_advance_baiter_timer, rom_baiter_count_limit, rom_baiter_cycle_counter_for_entity,
        rom_baiter_cycle_step, rom_baiter_horizontal_close_band, rom_baiter_initial_shot_timer,
        rom_baiter_seek_velocity, rom_baiter_should_seek, rom_baiter_spawn_for_wave,
        rom_baiter_timer_floor, rom_baiter_vertical_close_band, rom_bomb_shell_limit,
        rom_bomber_mine_lifetime, rom_bomber_should_drop_mine, rom_default_human_world_xs,
        rom_enemy_shot_timer_for_entity, rom_lander_horizontal_velocity,
        rom_lander_vertical_velocity, rom_mutant_horizontal_velocity, rom_mutant_vertical_velocity,
        rom_pack_enemy_shot_timer, rom_pack_swarmer_state, rom_probe_spawn_for_wave,
        rom_probe_swarmer_burst_count, rom_probe_vertical_velocity, rom_randv_velocity,
        rom_reset_baiter_timer, rom_rmax, rom_shell_lifetime, rom_shell_spawn_limit,
        rom_shoot_axes_from_seeds, rom_swarmer_acceleration, rom_swarmer_count_limit,
        rom_swarmer_horizontal_velocity, rom_swarmer_shot_timer_for_entity,
        rom_swarmer_vertical_speed_limit, rom_tie_close_band, rom_tie_far_band,
        rom_tie_horizontal_velocity, rom_tie_next_cruise_altitude, scale_rom_probe_x_to_screen,
        screen_x_for_world_x, shortest_wrapped_delta, wrap_coordinate,
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
        assert_eq!(
            world
                .entities()
                .iter()
                .filter(|entity| entity.kind == EntityKind::Human)
                .map(|entity| entity.position.x)
                .collect::<Vec<_>>(),
            vec![14, 7, 63, 67, 105, 100, 146, 157, 7, 15]
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
    fn rom_rand_matches_the_red_label_seed_walk() {
        let mut rand_state = RomRandState::default();

        rand_state.advance();
        assert_eq!(rand_state.seed, 0x90);
        assert_eq!(rand_state.hseed, 0xD2);
        assert_eq!(rand_state.lseed, 0xAD);

        rand_state.advance();
        assert_eq!(rand_state.seed, 0x81);
        assert_eq!(rand_state.hseed, 0x69);
        assert_eq!(rand_state.lseed, 0x56);

        rand_state.advance();
        assert_eq!(rand_state.seed, 0x74);
        assert_eq!(rand_state.hseed, 0x34);
        assert_eq!(rand_state.lseed, 0xAB);
    }

    #[test]
    fn rom_default_human_world_xs_follow_plres_astst_layout() {
        assert_eq!(
            rom_default_human_world_xs(64),
            vec![4, 2, 21, 22, 35, 33, 48, 52, 2, 5]
        );
        assert_eq!(
            rom_default_human_world_xs(192),
            vec![14, 7, 63, 67, 105, 100, 146, 157, 7, 15]
        );
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
    fn rom_probe_spawn_follows_prbst_seed_walk() {
        let mut rand_state = RomRandState::default();

        assert_eq!(
            rom_probe_spawn_for_wave(&mut rand_state, 64, 1, 9),
            (18, 6, Velocity { dx: -2, dy: -1 })
        );
        assert_eq!(
            rom_probe_spawn_for_wave(&mut rand_state, 64, 1, 9),
            (40, 3, Velocity { dx: -2, dy: 1 })
        );
        assert_eq!(
            rom_probe_spawn_for_wave(&mut rand_state, 64, 1, 9),
            (51, 6, Velocity { dx: 2, dy: -1 })
        );
        assert_eq!(
            rom_probe_spawn_for_wave(&mut rand_state, 64, 1, 9),
            (25, 3, Velocity { dx: -1, dy: 1 })
        );
    }

    #[test]
    fn rom_probe_screen_mapping_preserves_the_rom_seed_range() {
        assert_eq!(scale_rom_probe_x_to_screen(0x10, 64), 0);
        assert_eq!(scale_rom_probe_x_to_screen(0x4f, 64), 63);
    }

    #[test]
    fn rom_probe_vertical_velocity_enforces_the_rom_minimum_magnitude_sign() {
        assert_eq!(rom_probe_vertical_velocity(0x00), -1);
        assert_eq!(rom_probe_vertical_velocity(0x40), 1);
        assert_eq!(rom_probe_vertical_velocity(0x7f), 1);
    }

    #[test]
    fn rom_tie_helpers_follow_the_source_band_and_acceleration() {
        let mut bomber = Entity::new(EntityKind::Bomber, 12, 2, -1, 0);
        initialize_tie_cruise_altitude(&mut bomber);
        assert_eq!(bomber.rom_aux, 0x50);
        assert_eq!(rom_tie_next_cruise_altitude(0x50, 0x41), 0x50);
        assert_eq!(rom_tie_next_cruise_altitude(0x40, 0x00), 0x40);
        assert_eq!(rom_tie_next_cruise_altitude(0x68, 0x03), 0x68);
        assert_eq!(rom_accumulate_vertical_velocity(0, 1), 1);
        assert_eq!(rom_accumulate_vertical_velocity(2, 1), 2);
        assert_eq!(rom_accumulate_vertical_velocity(-2, -1), -2);
        assert!(rom_tie_close_band(1, 9) >= 1);
        assert!(rom_tie_far_band(1, 9) > rom_tie_close_band(1, 9));
    }

    #[test]
    fn rom_baiter_seek_helpers_follow_the_source_thresholds() {
        assert_eq!(rom_baiter_horizontal_close_band(64), 20);
        assert!(rom_baiter_vertical_close_band(1, 9) >= 1);
        assert!(!rom_baiter_should_seek(0x08, 0xF0));
        assert!(rom_baiter_should_seek(0xF1, 0xC8));
        assert_eq!(rom_baiter_cycle_step(16), (17, false));
        assert_eq!(rom_baiter_cycle_step(17), (0, true));
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
    fn live_step_fifth_manual_shot_is_ignored_in_normal_play() {
        let mut world = World::with_entities(
            64,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 10, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 50, 1, 0, 0),
            ],
        );

        for _ in 0..arcade_tables().player_shot_limit {
            world.step_live(UpdateInput {
                fire: true,
                ..UpdateInput::default()
            });
        }
        let events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(
            world.entity_count_by_kind(EntityKind::PlayerShot),
            arcade_tables().player_shot_limit
        );
        assert!(!events.contains(&WorldEvent::ShotFired));
        assert_eq!(world.status().lives, 3);
    }

    #[test]
    fn xyzzy_mode_removes_the_four_shot_cap() {
        let mut world = World::with_entities(
            64,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 10, 4, 0, 0),
                Entity::new(EntityKind::Bomber, 50, 1, 0, 0),
            ],
        );

        for _ in 0..=arcade_tables().player_shot_limit {
            world.step_live(UpdateInput {
                fire: true,
                secret_mode: true,
                ..UpdateInput::default()
            });
        }

        assert!(
            world.entity_count_by_kind(EntityKind::PlayerShot) > arcade_tables().player_shot_limit
        );
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
        let shot_timer = red_label_wave_table().profile_for_wave(1).lander_shot_time as u8;
        let mut lander = Entity::new(EntityKind::Lander, 12, 3, 0, 0);
        lander.rom_aux = rom_pack_enemy_shot_timer(1);
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
                lander,
                Entity::new(EntityKind::Human, 18, 6, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 1);
        assert!(events.contains(&WorldEvent::EnemyFired));
        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        let reset_timer = rom_enemy_shot_timer_for_entity(lander, 1);
        assert!((1..=shot_timer).contains(&reset_timer));
    }

    #[test]
    fn enemy_fire_uses_the_rom_shared_shell_limit() {
        let mut entities = vec![
            Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
            Entity::new(EntityKind::Lander, 20, 3, 0, 0),
        ];
        entities.extend((0..rom_shell_spawn_limit()).map(|index| {
            let mut shot = Entity::new(
                EntityKind::EnemyShot,
                8 + index as i32,
                3 + (index as i32 % 2),
                0,
                0,
            );
            shot.rom_aux = 100;
            shot
        }));
        let mut world = World::with_entities(
            64,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            entities,
        );

        let shot_delay = red_label_wave_table().profile_for_wave(1).lander_shot_time;
        world.tick = shot_delay;
        let mut events = Vec::new();
        world.spawn_enemy_fire(
            world.world_max_x(),
            1,
            world.height() as i32 - 2,
            &mut events,
        );

        assert_eq!(
            world.entity_count_by_kind(EntityKind::EnemyShot),
            rom_shell_spawn_limit()
        );
        assert!(!events.contains(&WorldEvent::EnemyFired));
    }

    #[test]
    fn live_step_does_not_allow_offscreen_enemies_to_fire() {
        let mut world = World::bootstrap();
        let player_x = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position
            .x;
        let offscreen_x = wrap_coordinate(player_x + world.width() as i32, world.world_max_x());
        let safe_y = world.safe_altitude_at_world_x(player_x).min(4);
        world.camera_x = player_x;
        let mut lander = Entity::new(EntityKind::Lander, offscreen_x, safe_y, 0, 0);
        lander.rom_aux = rom_pack_enemy_shot_timer(1);
        world.entities = vec![
            Entity::new(EntityKind::PlayerShip, player_x, safe_y, 0, 0),
            lander,
            Entity::new(
                EntityKind::Human,
                wrap_coordinate(player_x + 8, world.world_max_x()),
                8,
                0,
                0,
            ),
        ];

        world.step_live(UpdateInput::default());

        assert!(world.screen_x_for_world_x(offscreen_x).is_none());
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
    }

    #[test]
    fn swarmer_fire_uses_the_faster_enemy_cycle() {
        let wave_profile = red_label_wave_table().profile_for_wave(3);
        let mut swarmer = Entity::new(
            EntityKind::Swarmer,
            18,
            4,
            -rom_swarmer_horizontal_velocity(wave_profile.swarmer_x_velocity),
            0,
        );
        swarmer.rom_aux = rom_pack_swarmer_state(
            rom_swarmer_acceleration(0xAD, wave_profile.swarmer_acceleration_mask),
            1,
        );
        let mut world = World::with_entities(
            32,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![Entity::new(EntityKind::PlayerShip, 6, 4, 0, 0), swarmer],
        );

        let events = world.step_live(UpdateInput::default());

        let shot = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::EnemyShot)
            .expect("enemy shot");
        assert_eq!(shot.velocity.dx, -1);
        assert_eq!(shot.velocity.dy, 0);
        assert!(events.contains(&WorldEvent::EnemyFired));
    }

    #[test]
    fn swarmer_fire_resets_the_rom_timer_when_the_shell_list_is_full() {
        let wave_profile = red_label_wave_table().profile_for_wave(3);
        let mut swarmer = Entity::new(
            EntityKind::Swarmer,
            18,
            4,
            -rom_swarmer_horizontal_velocity(wave_profile.swarmer_x_velocity),
            0,
        );
        swarmer.rom_aux = rom_pack_swarmer_state(
            rom_swarmer_acceleration(0xAD, wave_profile.swarmer_acceleration_mask),
            1,
        );
        let mut entities = vec![Entity::new(EntityKind::PlayerShip, 6, 4, 0, 0), swarmer];
        entities.extend((0..rom_shell_spawn_limit()).map(|index| {
            let mut shot = Entity::new(
                EntityKind::EnemyShot,
                8 + index as i32,
                3 + (index as i32 % 2),
                0,
                0,
            );
            shot.rom_aux = 100;
            shot
        }));
        let mut world = World::with_entities(
            32,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            entities,
        );

        let events = world.step_live(UpdateInput::default());

        assert_eq!(
            world.entity_count_by_kind(EntityKind::EnemyShot),
            rom_shell_spawn_limit()
        );
        assert!(!events.contains(&WorldEvent::EnemyFired));
        let swarmer = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Swarmer)
            .expect("swarmer");
        assert!(
            rom_swarmer_shot_timer_for_entity(swarmer, wave_profile.swarmer_shot_time) > 0,
            "SWBMB should reseed PD4 after the attempt"
        );
    }

    #[test]
    fn rom_shoot_axes_add_player_velocity_when_seed_exceeds_threshold() {
        let without_lead =
            rom_shoot_axes_from_seeds(10, 5, 8, 5, Velocity { dx: 2, dy: 0 }, 24, 32);
        let with_lead = rom_shoot_axes_from_seeds(10, 5, 8, 5, Velocity { dx: 2, dy: 0 }, 152, 32);

        assert_eq!(without_lead.1, with_lead.1);
        assert_eq!(with_lead.0, without_lead.0 + 8);
    }

    #[test]
    fn rom_shoot_axes_use_lseed_for_vertical_solution() {
        let lower = rom_shoot_axes_from_seeds(8, 6, 10, 5, Velocity { dx: 0, dy: 0 }, 64, 1);
        let higher = rom_shoot_axes_from_seeds(8, 6, 10, 5, Velocity { dx: 0, dy: 0 }, 64, 17);

        assert_eq!(lower.0, higher.0);
        assert_eq!(higher.1, lower.1 + 64);
    }

    #[test]
    fn rom_enemy_shot_seed_bytes_use_the_shared_rand_state() {
        let mut world = World::bootstrap();
        world.rand_state = RomRandState {
            seed: 0x8A,
            hseed: 0x12,
            lseed: 0x34,
        };

        assert_eq!(world.rom_enemy_shot_seed_bytes(), (0x8A, 0x34));
    }

    #[test]
    fn live_step_landers_keep_seeded_horizontal_drift_until_target_alignment() {
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
                Entity::new(EntityKind::Lander, 10, 3, 1, 0),
                Entity::new(EntityKind::Human, 14, 5, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        assert_eq!(lander.position, Position { x: 11, y: 4 });
        assert_eq!(lander.velocity.dx, 1);
    }

    #[test]
    fn live_step_landers_do_not_chase_the_player_when_no_free_humans_remain() {
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
                Entity::new(EntityKind::Lander, 10, 3, 1, 0),
                Entity::with_state(EntityKind::Human, 12, 4, 0, 0, EntityState::Falling),
                Entity::with_state(EntityKind::Human, 2, 2, 0, 0, EntityState::PlayerCarried),
            ],
        );

        world.step_live(UpdateInput::default());

        let lander = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Lander)
            .expect("lander");
        assert_eq!(lander.position, Position { x: 11, y: 4 });
        assert_eq!(lander.velocity.dx, 1);
    }

    #[test]
    fn wave_one_landers_spawn_with_the_rom_velocity_records() {
        let mut world = World::with_entities(
            64,
            12,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0)],
        );

        world.spawn_wave();

        let wave_profile = red_label_wave_table().profile_for_wave(1);
        let lander_speed_x = rom_lander_horizontal_velocity(wave_profile.lander_x_velocity);
        let lander_speed_y = rom_lander_vertical_velocity(
            wave_profile.lander_y_velocity_msb,
            wave_profile.lander_y_velocity_lsb,
        );

        for lander in world
            .entities()
            .iter()
            .filter(|entity| entity.kind == EntityKind::Lander)
        {
            assert_eq!(lander.velocity.dx.abs(), lander_speed_x);
            assert_eq!(lander.velocity.dy, lander_speed_y);
        }
    }

    #[test]
    fn swarmer_acceleration_mask_expands_the_vertical_band_on_later_waves() {
        let wave_one = red_label_wave_table().profile_for_wave(1);
        let wave_four = red_label_wave_table().profile_for_wave(4);

        assert_eq!(
            rom_swarmer_vertical_speed_limit(wave_one.swarmer_acceleration_mask),
            1
        );
        assert_eq!(
            rom_swarmer_vertical_speed_limit(wave_four.swarmer_acceleration_mask),
            2
        );
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
        assert_eq!(mutant.position, Position { x: 9, y: 4 });
    }

    #[test]
    fn live_step_later_wave_mutants_use_the_rom_speed_records() {
        let mut world = World::with_entities(
            32,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 4,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 12, 6, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let mutant = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Mutant)
            .expect("mutant");
        let wave_profile = red_label_wave_table().profile_for_wave(4);
        assert_eq!(
            mutant.velocity.dx,
            -rom_mutant_horizontal_velocity(wave_profile.mutant_x_velocity),
        );
        assert_eq!(
            mutant.velocity.dy,
            -rom_mutant_vertical_velocity(
                wave_profile.mutant_y_velocity_msb,
                wave_profile.mutant_y_velocity_lsb,
            ),
        );
        assert_eq!(mutant.position, Position { x: 10, y: 2 });
    }

    #[test]
    fn live_step_mutants_apply_the_rom_random_y_hop_when_on_screen() {
        let mut world = World::with_entities(
            32,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 4,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 4, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 12, 6, 0, 0),
            ],
        );

        world.step_live(UpdateInput::default());

        let mutant = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Mutant)
            .expect("mutant");
        assert_eq!(mutant.position.y, 2);
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
    fn live_step_baiters_keep_course_inside_the_rom_close_band() {
        let mut world = World::with_entities(
            64,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 30, 5, 0, 0),
                Entity::new(EntityKind::Lander, 50, 1, 0, 0),
                Entity::new(EntityKind::Baiter, 42, 5, 2, 0),
                Entity::new(EntityKind::Human, 8, 8, 0, 0),
            ],
        );
        world.tick = 18;

        world.step_live(UpdateInput::default());

        let baiter = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Baiter)
            .expect("baiter");
        assert_eq!(baiter.velocity.dx, 2);
        assert_eq!(baiter.position, Position { x: 44, y: 5 });
    }

    #[test]
    fn live_step_bombers_keep_their_rom_horizontal_speed_when_crossing_the_players_altitude() {
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
        assert_eq!(
            bomber.velocity.dx,
            -rom_tie_horizontal_velocity(
                red_label_wave_table().profile_for_wave(2).bomber_x_velocity
            )
        );
        assert_eq!(bomber.position, Position { x: 11, y: 4 });
    }

    #[test]
    fn live_step_offscreen_bombers_seek_their_cruise_altitude() {
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
                Entity::new(EntityKind::Bomber, 18, 2, -1, 0),
            ],
        );
        world.camera_x = 4;

        world.step_live(UpdateInput::default());

        let bomber = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Bomber)
            .expect("bomber");
        assert_eq!(bomber.velocity.dy, 1);
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

        for _ in 0..64 {
            world.step_live(UpdateInput::default());
            if world.entity_count_by_kind(EntityKind::Mine) > 0 {
                break;
            }
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
    fn bomber_mines_use_the_rom_shared_shell_limit() {
        let bomber_position = Position { x: 12, y: 6 };
        let mut entities = vec![
            Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0),
            Entity::new(
                EntityKind::Bomber,
                bomber_position.x,
                bomber_position.y,
                -1,
                0,
            ),
        ];
        entities.extend((0..rom_bomb_shell_limit()).map(|index| {
            let mut shot = Entity::new(EntityKind::EnemyShot, 2 + index as i32, 3, 0, 0);
            shot.rom_aux = 100;
            shot
        }));
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            entities,
        );
        world.tick = (0..64)
            .find(|tick| rom_bomber_should_drop_mine(2, *tick, bomber_position))
            .expect("bomb gate");

        world.drop_bomber_mines(1, world.height() as i32 - 2);

        assert_eq!(world.entity_count_by_kind(EntityKind::Mine), 0);
    }

    #[test]
    fn rom_bomber_mine_gate_matches_bombst_seed_edge() {
        let position = Position { x: 12, y: 6 };
        let drops = (0..32)
            .filter(|tick| rom_bomber_should_drop_mine(2, *tick, position))
            .count();

        assert!(drops > 0);
        assert!(drops < 32);
    }

    #[test]
    fn rom_bomber_mine_lifetime_matches_bombst_seed_range() {
        let position = Position { x: 12, y: 6 };
        let lifetimes: Vec<u16> = (0..32)
            .map(|tick| rom_bomber_mine_lifetime(2, tick, position))
            .collect();

        assert!(lifetimes.iter().all(|lifetime| (1..=32).contains(lifetime)));
        assert!(lifetimes.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn rom_shell_lifetime_matches_getshl_default() {
        assert_eq!(rom_shell_lifetime(), 20);
    }

    #[test]
    fn rom_shell_spawn_limit_matches_getshl_bmbcnt_gate() {
        assert_eq!(rom_shell_spawn_limit(), 20);
    }

    #[test]
    fn rom_bomb_shell_limit_matches_bombst_bmbcnt_gate() {
        assert_eq!(rom_bomb_shell_limit(), 10);
    }

    #[test]
    fn live_step_bomber_mines_expire_after_their_rom_lifetime() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 2,
            },
            vec![Entity::new(EntityKind::Mine, 12, 6, 0, 0)],
        );
        if let Some(mine) = world
            .entities
            .iter_mut()
            .find(|entity| entity.kind == EntityKind::Mine)
        {
            mine.rom_aux = 2;
        }

        world.step_live(UpdateInput::default());
        assert_eq!(world.entity_count_by_kind(EntityKind::Mine), 1);

        world.step_live(UpdateInput::default());
        assert_eq!(world.entity_count_by_kind(EntityKind::Mine), 0);
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
        assert_eq!(
            swarmer.velocity.dx,
            -rom_swarmer_horizontal_velocity(
                red_label_wave_table()
                    .profile_for_wave(3)
                    .swarmer_x_velocity
            )
        );
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
                Entity::new(EntityKind::Mutant, 5, 5, 0, 0),
                Entity::new(EntityKind::Mutant, 9, 2, 0, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            fire: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().score, 250);
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
                Entity::new(EntityKind::Human, 5, 3, 0, 0),
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
                Entity::new(EntityKind::Human, 5, 3, 0, 0),
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
        if let Some(mine) = world
            .entities
            .iter_mut()
            .find(|entity| entity.kind == EntityKind::Mine)
        {
            mine.rom_aux = 2;
        }

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
                Entity::new(EntityKind::Pod, 10, 5, -1, -1),
            ],
        );

        let events = world.step_live(UpdateInput::default());
        let swarmer_count = world.entity_count_by_kind(EntityKind::Swarmer);

        assert_eq!(world.entity_count_by_kind(EntityKind::Pod), 0);
        assert_eq!(swarmer_count, rom_probe_swarmer_burst_count(0));
        assert_eq!(world.status().score, 1_000);
        assert!(events.contains(&WorldEvent::EnemyDestroyed));
        assert!(
            world
                .entities()
                .iter()
                .any(|entity| entity.kind == EntityKind::Swarmer),
            "destroyed pod should spawn at least one swarmer"
        );
    }

    #[test]
    fn rom_rmax_matches_the_cabinet_halving_rule() {
        assert_eq!(rom_rmax(6, 0), 1);
        assert_eq!(rom_rmax(6, 6), 7);
        assert_eq!(rom_rmax(6, 7), 4);
        assert_eq!(rom_rmax(6, 255), 4);
    }

    #[test]
    fn rom_probe_swarmer_burst_count_matches_prbkil_range() {
        let counts: Vec<usize> = (0..32).map(rom_probe_swarmer_burst_count).collect();

        assert!(counts.iter().all(|count| (1..=7).contains(count)));
        assert!(counts.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn rom_randv_velocity_follows_the_source_seed_walk() {
        let mut rand_state = RomRandState::default();

        assert_eq!(
            rom_randv_velocity(&mut rand_state),
            Velocity { dx: 1, dy: -2 }
        );
        assert_eq!(
            rom_randv_velocity(&mut rand_state),
            Velocity { dx: -1, dy: -2 }
        );
        assert_eq!(
            rom_randv_velocity(&mut rand_state),
            Velocity { dx: 1, dy: 2 }
        );
    }

    #[test]
    fn rom_swarmer_count_limit_matches_mmsw_swcnt_cap() {
        assert_eq!(rom_swarmer_count_limit(), 20);
    }

    #[test]
    fn rom_swarmer_acceleration_matches_mmsw_pd2_mask() {
        assert_eq!(rom_swarmer_acceleration(0xAD, 0x1F), 0x0D);
        assert_eq!(rom_swarmer_acceleration(0x56, 0x3F), 0x16);
    }

    #[test]
    fn live_step_swarmers_reverse_immediately_toward_the_player() {
        let mut swarmer = Entity::new(EntityKind::Swarmer, 12, 6, 2, 0);
        swarmer.rom_aux = u16::from(rom_swarmer_acceleration(0xAD, 0x1F));
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 3,
            },
            vec![Entity::new(EntityKind::PlayerShip, 4, 4, 0, 0), swarmer],
        );

        world.step_live(UpdateInput::default());

        let swarmer = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Swarmer)
            .expect("swarmer");
        assert_eq!(
            swarmer.velocity.dx,
            -rom_swarmer_horizontal_velocity(
                red_label_wave_table()
                    .profile_for_wave(3)
                    .swarmer_x_velocity
            )
        );
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
    fn live_step_removes_player_shots_that_leave_the_main_screen() {
        let mut world = World::bootstrap();
        let player_x = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::PlayerShip)
            .expect("player")
            .position
            .x;
        let offscreen_x =
            wrap_coordinate(player_x + world.width() as i32 / 2 + 2, world.world_max_x());
        let shot_y = world.safe_altitude_at_world_x(offscreen_x).min(4);
        world.spawn_entity(Entity::new(
            EntityKind::PlayerShot,
            offscreen_x,
            shot_y,
            2,
            0,
        ));

        assert!(world.screen_x_for_world_x(offscreen_x).is_none());

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 0);
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
    fn live_step_enemy_shots_seed_and_expire_on_the_shell_timer() {
        let mut world = World::with_entities(
            24,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 4, 0, 0),
                Entity::new(EntityKind::Lander, 20, 2, 0, 0),
            ],
        );
        let shot_y = world.safe_altitude_at_world_x(12).saturating_sub(3).max(1);
        world.spawn_entity(Entity::new(EntityKind::EnemyShot, 12, shot_y, 0, 0));

        world.step_live(UpdateInput::default());
        let shot = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::EnemyShot)
            .expect("enemy shot");
        assert_eq!(shot.rom_aux, rom_shell_lifetime());

        for _ in 0..rom_shell_lifetime() {
            world.step_live(UpdateInput::default());
        }
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
                Entity::new(EntityKind::Mutant, 5, 5, 0, 0),
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
        let wave_two_profile = red_label_wave_table().profile_for_wave(2);
        let wave_three_profile = red_label_wave_table().profile_for_wave(3);
        let wave_four_profile = red_label_wave_table().profile_for_wave(4);
        let wave_five_profile = red_label_wave_table().profile_for_wave(5);

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
        assert_eq!(
            wave_two.entity_count_by_kind(EntityKind::Bomber),
            wave_two_profile.bombers as usize
        );
        assert_eq!(
            wave_two.entity_count_by_kind(EntityKind::Pod),
            wave_two_profile.pods as usize
        );

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
        assert_eq!(
            wave_three.entity_count_by_kind(EntityKind::Bomber),
            wave_three_profile.bombers as usize
        );
        assert_eq!(
            wave_three.entity_count_by_kind(EntityKind::Pod),
            wave_three_profile.pods as usize
        );

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
        assert_eq!(
            wave_four.entity_count_by_kind(EntityKind::Bomber),
            wave_four_profile.bombers as usize
        );
        assert_eq!(
            wave_four.entity_count_by_kind(EntityKind::Pod),
            wave_four_profile.pods as usize
        );

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
        assert_eq!(
            wave_five.entity_count_by_kind(EntityKind::Bomber),
            wave_five_profile.bombers as usize
        );
        assert_eq!(
            wave_five.entity_count_by_kind(EntityKind::Pod),
            wave_five_profile.pods as usize
        );
        assert_eq!(wave_five.human_count(), 10);
    }

    #[test]
    fn attack_waves_arrive_in_three_five_ship_groups() {
        let mut world = World::bootstrap();
        let wave_profile = red_label_wave_table().profile_for_wave(world.status().wave);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 10);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 15);
    }

    #[test]
    fn wave_two_uses_four_rom_recorded_lander_groups() {
        let mut world = World::with_entities(
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
        let wave_profile = red_label_wave_table().profile_for_wave(2);

        world.spawn_wave();
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 5);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 10);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 15);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Lander), 20);
    }

    #[test]
    fn mutant_reinforcements_follow_the_same_rom_group_schedule() {
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

        let wave_profile = red_label_wave_table().profile_for_wave(world.status().wave);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 10);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 15);

        for _ in 0..wave_profile.wave_time {
            world.step();
        }
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 20);
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
        let wave_profile = red_label_wave_table().profile_for_wave(2);
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
        assert_eq!(
            world.entity_count_by_kind(EntityKind::Bomber),
            wave_profile.bombers as usize
        );
        assert_eq!(
            world.entity_count_by_kind(EntityKind::Pod),
            wave_profile.pods as usize
        );
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
            world
                .entities()
                .iter()
                .filter(|entity| entity.kind == EntityKind::Human)
                .map(|entity| entity.position.x)
                .eq([4, 2, 21, 22, 35, 33, 48, 52, 2, 5].into_iter())
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
        world.tick = red_label_wave_table().profile_for_wave(2).baiter_delay + 4;
        world.baiter_timer = 1;

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 1);
        let baiter = world
            .entities()
            .iter()
            .find(|entity| entity.kind == EntityKind::Baiter)
            .expect("baiter");
        assert!(world.screen_x_for_world_x(baiter.position.x).is_some());
        assert_ne!(baiter.velocity, Velocity { dx: 0, dy: 0 });
        assert_eq!(
            rom_enemy_shot_timer_for_entity(baiter, 2),
            rom_baiter_initial_shot_timer()
        );
        assert_eq!(rom_baiter_cycle_counter_for_entity(baiter), 0);
    }

    #[test]
    fn rom_baiter_spawn_stays_within_the_visible_band() {
        let spawn = rom_baiter_spawn_for_wave(2, 200, -12, 24, 191, 1, 8);

        assert!((0..=191).contains(&spawn.x));
        assert!(screen_x_for_world_x(spawn.x, -12, 24, 192, 191).is_some());
        assert!((1..=8).contains(&spawn.y));
    }

    #[test]
    fn rom_baiter_seek_velocity_matches_the_spawn_side_of_ufonv0() {
        let velocity = rom_baiter_seek_velocity(
            Position { x: 18, y: 3 },
            Velocity { dx: 0, dy: 0 },
            RomBaiterSeekContext {
                target: Position { x: 4, y: 6 },
                player_velocity: Velocity { dx: 1, dy: 0 },
                world_span: 192,
                screen_width: 64,
                min_y: 1,
                max_y: 9,
            },
        );

        assert_eq!(velocity.dx, 0);
        assert!(velocity.dy > 0);
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
        world.tick = red_label_wave_table().profile_for_wave(2).baiter_delay;
        world.baiter_timer = 1;

        world.step_live(UpdateInput::default());

        assert_eq!(world.entity_count_by_kind(EntityKind::Baiter), 0);
    }

    #[test]
    fn rom_baiter_count_limit_matches_gexec_ufo_cap() {
        assert_eq!(rom_baiter_count_limit(), 12);
    }

    #[test]
    fn rom_baiter_timer_floor_matches_gexec_acceleration_bands() {
        assert_eq!(rom_baiter_timer_floor(196, 9), 196);
        assert_eq!(rom_baiter_timer_floor(196, 8), 99);
        assert_eq!(rom_baiter_timer_floor(196, 3), 50);
    }

    #[test]
    fn rom_baiter_timer_reset_randomizes_when_fewer_than_four_enemies_remain() {
        let varied: Vec<u32> = (0..32)
            .map(|tick| rom_reset_baiter_timer(2, tick, 196, 3))
            .collect();

        assert!(varied.iter().all(|value| (1..=50).contains(value)));
        assert!(varied.windows(2).any(|pair| pair[0] != pair[1]));
        assert_eq!(rom_reset_baiter_timer(2, 0, 196, 4), 196);
    }

    #[test]
    fn rom_baiter_timer_advances_on_the_gexec_countdown() {
        assert_eq!(rom_advance_baiter_timer(196, 196, 12), 195);
        assert_eq!(rom_advance_baiter_timer(196, 196, 8), 98);
        assert_eq!(rom_advance_baiter_timer(196, 196, 3), 49);
    }

    #[test]
    fn live_step_baiters_use_the_rom_active_cap() {
        let mut entities = vec![
            Entity::new(EntityKind::PlayerShip, 3, 4, 0, 0),
            Entity::new(EntityKind::Lander, 18, 4, 0, 0),
            Entity::new(EntityKind::Pod, 18, 4, 0, 0),
            Entity::new(EntityKind::Human, 1, 8, 0, 0),
        ];
        entities.extend(
            (0..rom_baiter_count_limit())
                .map(|index| Entity::new(EntityKind::Baiter, 6 + index as i32, 4, 0, 0)),
        );
        let mut world = World::with_entities(
            32,
            10,
            Status {
                score: 0,
                lives: 99,
                wave: 2,
            },
            entities,
        );
        world.tick = 200;
        world.baiter_timer = 1;

        world.step_live(UpdateInput::default());

        assert_eq!(
            world.entity_count_by_kind(EntityKind::Baiter),
            rom_baiter_count_limit()
        );
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
    fn live_step_hyperspace_clears_active_shells() {
        let mut world = World::with_entities(
            20,
            10,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 3, 1, 0),
                Entity::new(EntityKind::PlayerShot, 6, 3, 2, 0),
                Entity::new(EntityKind::PlayerShot, 7, 4, 2, 0),
                Entity::new(EntityKind::EnemyShot, 10, 5, -2, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            hyperspace: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.entity_count_by_kind(EntityKind::PlayerShot), 0);
        assert_eq!(world.entity_count_by_kind(EntityKind::EnemyShot), 0);
        assert!(events.contains(&WorldEvent::HyperspaceUsed));
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
                Entity::new(EntityKind::Mutant, 2, 3, 0, 0),
                Entity::new(EntityKind::Pod, 10, 4, 1, 0),
            ],
        );

        let events = world.step_live(UpdateInput {
            invincible: true,
            ..UpdateInput::default()
        });

        assert_eq!(world.status().lives, 2);
        assert!(!world.is_game_over());
        assert_eq!(world.entity_count_by_kind(EntityKind::Mutant), 0);
        assert_eq!(world.status().score, 250);
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
    fn empty_smart_bomb_press_is_ignored_in_normal_play() {
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
            ..UpdateInput::default()
        });

        assert_eq!(world.smart_bombs(), 0);
        assert_eq!(world.status().score, 0);
        assert!(!events.contains(&WorldEvent::SmartBombDetonated));
        assert_eq!(world.status().lives, 3);
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
