use super::*;

pub(in crate::actor_game) fn center_of(bounds: Rect) -> Point {
    Point::new(
        (bounds.left + bounds.right) / 2,
        (bounds.top + bounds.bottom) / 2,
    )
}

pub(in crate::actor_game) fn translate_rect(bounds: Rect, delta: Velocity) -> Rect {
    Rect::new(
        bounds.left.saturating_add(delta.dx),
        bounds.top.saturating_add(delta.dy),
        bounds.right.saturating_add(delta.dx),
        bounds.bottom.saturating_add(delta.dy),
    )
}

pub(in crate::actor_game) fn manhattan_distance(left: Point, right: Point) -> i16 {
    (left.x - right.x).abs() + (left.y - right.y).abs()
}

pub(in crate::actor_game) fn is_hostile(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Pod
            | ActorKind::Swarmer
    )
}

pub(in crate::actor_game) fn snapshot_blocks_wave_clear(snapshot: &ActorSnapshot) -> bool {
    is_hostile(snapshot.kind) && snapshot.bounds.is_some()
}

pub(in crate::actor_game) fn clears_for_next_wave(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::Human
            | ActorKind::Laser
            | ActorKind::EnemyLaser
            | ActorKind::Explosion
            | ActorKind::ScorePopup
    )
}

pub(in crate::actor_game) fn clears_for_next_turn(kind: ActorKind) -> bool {
    kind == ActorKind::Player || clears_for_next_wave(kind)
}

pub(in crate::actor_game) fn is_player_laser_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

pub(in crate::actor_game) fn is_enemy_projectile_kind(kind: ActorKind) -> bool {
    matches!(kind, ActorKind::EnemyLaser | ActorKind::Bomb)
}

pub(in crate::actor_game) fn enemy_projectile_slot_available(
    active_enemy_projectiles: usize,
) -> bool {
    active_enemy_projectiles < ENEMY_PROJECTILE_SLOT_LIMIT
}

pub(in crate::actor_game) fn bomb_projectile_slot_available(
    active_bomb_projectiles: usize,
) -> bool {
    active_bomb_projectiles < ACTIVE_BOMBER_BOMB_LIMIT
}

pub(in crate::actor_game) fn enemy_projectile_spawn_in_bounds(position: Point) -> bool {
    position.x < ENEMY_PROJECTILE_MAX_SCREEN_X && position.y > i16::from(PLAYFIELD_TOP_EDGE_Y)
}

pub(in crate::actor_game) fn bomb_projectile_spawn_in_world_bounds(
    position: Point,
    actor_state: Option<EnemyProjectileActorState>,
) -> bool {
    actor_state.is_none() || enemy_projectile_spawn_in_bounds(position)
}

pub(in crate::actor_game) fn reserve_enemy_projectile_slot(
    active_enemy_projectiles: &mut usize,
) -> bool {
    if !enemy_projectile_slot_available(*active_enemy_projectiles) {
        return false;
    }
    *active_enemy_projectiles += 1;
    true
}

pub(in crate::actor_game) fn is_player_hazard(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

pub(in crate::actor_game) fn actor_collision_body_for_snapshot(
    snapshot: &ActorSnapshot,
    background_left: u16,
) -> Option<CollisionBody> {
    let body = snapshot.collision_body()?;
    actor_project_actor_state_collision_body(snapshot, body, background_left)
}

pub(in crate::actor_game) fn actor_project_actor_state_collision_body(
    snapshot: &ActorSnapshot,
    body: CollisionBody,
    background_left: u16,
) -> Option<CollisionBody> {
    let Some(x_fraction) = actor_actor_state_x_fraction(snapshot) else {
        return Some(body);
    };
    if center_of(body.bounds) != snapshot.position && snapshot.kind != ActorKind::Human {
        return Some(body);
    }
    let position =
        actor_screen_position_from_world(snapshot.position, x_fraction, background_left)?;
    let delta = Velocity::new(
        position.x - snapshot.position.x,
        position.y - snapshot.position.y,
    );
    Some(CollisionBody {
        position,
        bounds: translate_rect(body.bounds, delta),
        ..body
    })
}

pub(in crate::actor_game) fn is_player_enemy_collision_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
    )
}

pub(in crate::actor_game) fn is_smart_bomb_target(kind: ActorKind) -> bool {
    matches!(
        kind,
        ActorKind::Lander
            | ActorKind::Mutant
            | ActorKind::Bomber
            | ActorKind::Bomb
            | ActorKind::Pod
            | ActorKind::Swarmer
            | ActorKind::Baiter
            | ActorKind::EnemyLaser
    )
}

pub(in crate::actor_game) fn commands_spawn_hostiles(commands: &[GameCommand]) -> bool {
    commands.iter().any(|command| {
        matches!(
            command,
            GameCommand::Spawn(SpawnRequest::Lander { .. })
                | GameCommand::Spawn(SpawnRequest::Mutant { .. })
                | GameCommand::Spawn(SpawnRequest::Bomber { .. })
                | GameCommand::Spawn(SpawnRequest::Pod { .. })
                | GameCommand::Spawn(SpawnRequest::Swarmer { .. })
        )
    })
}

pub(in crate::actor_game) fn score_for_hostile(kind: ActorKind) -> u32 {
    match kind {
        ActorKind::Lander => LANDER_SCORE,
        ActorKind::Mutant => MUTANT_SCORE,
        ActorKind::Bomber => BOMBER_SCORE,
        ActorKind::Bomb => 0,
        ActorKind::Pod => POD_SCORE,
        ActorKind::Swarmer => SWARMER_SCORE,
        ActorKind::Baiter => BAITER_SCORE,
        _ => 0,
    }
}

pub(in crate::actor_game) fn hit_sound_for_hostile(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Lander => SoundCue::LanderHit,
        ActorKind::Mutant => SoundCue::MutantHit,
        ActorKind::Bomber => SoundCue::BomberHit,
        ActorKind::Bomb => SoundCue::BombHit,
        ActorKind::Pod => SoundCue::PodHit,
        ActorKind::Swarmer => SoundCue::SwarmerHit,
        ActorKind::Baiter => SoundCue::BaiterHit,
        _ => SoundCue::Explosion,
    }
}

pub(in crate::actor_game) fn player_hazard_sound(kind: ActorKind) -> SoundCue {
    match kind {
        ActorKind::Bomb => SoundCue::BombHit,
        _ => SoundCue::Explosion,
    }
}

pub(in crate::actor_game) fn explosion_kind_for_target(kind: ActorKind) -> Option<ExplosionKind> {
    let kind = match kind {
        ActorKind::Lander => ExplosionKind::Lander,
        ActorKind::Mutant => ExplosionKind::Mutant,
        ActorKind::Bomber => ExplosionKind::Bomber,
        ActorKind::Pod => ExplosionKind::Pod,
        ActorKind::Swarmer => ExplosionKind::Swarmer,
        ActorKind::Baiter => ExplosionKind::Baiter,
        ActorKind::Bomb | ActorKind::EnemyLaser => ExplosionKind::Bomb,
        _ => return None,
    };
    Some(kind)
}

pub(in crate::actor_game) fn player_hazard_explosion_kind(kind: ActorKind) -> ExplosionKind {
    match kind {
        ActorKind::Bomb => ExplosionKind::Bomb,
        _ => ExplosionKind::Player,
    }
}

pub(in crate::actor_game) fn accelerated_baiter_timer_steps(
    current_steps: u32,
    profile: ActorWaveTuning,
    enemy_total: usize,
) -> u32 {
    if enemy_total > 8 {
        return current_steps;
    }

    let mut target_steps = profile.baiter_delay / 2;
    if enemy_total <= 3 {
        target_steps /= 2;
    }
    target_steps = target_steps.saturating_add(1).max(1);
    current_steps.min(target_steps)
}

pub(in crate::actor_game) fn baiter_timer_reset_steps(
    profile: ActorWaveTuning,
    enemy_total: usize,
) -> u32 {
    if enemy_total < 4 {
        (profile.baiter_delay / 4).max(1)
    } else {
        profile.baiter_delay.max(1)
    }
}

#[derive(Debug, Clone)]
pub(in crate::actor_game) struct HighScoreTable {
    pub(in crate::actor_game) entries: [u32; 5],
}

impl HighScoreTable {
    pub(in crate::actor_game) fn entries(&self) -> [u32; 5] {
        self.entries
    }

    pub(in crate::actor_game) fn qualifies(&self, score: u32) -> bool {
        self.entries.iter().any(|entry| score > *entry)
    }

    pub(in crate::actor_game) fn record(&mut self, score: u32) {
        if !self.qualifies(score) {
            return;
        }
        let mut entries = self.entries.to_vec();
        entries.push(score);
        entries.sort_by(|left, right| right.cmp(left));
        self.entries.copy_from_slice(&entries[..5]);
    }
}

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            entries: [10_000, 7_500, 5_000, 2_500, 1_000],
        }
    }
}
