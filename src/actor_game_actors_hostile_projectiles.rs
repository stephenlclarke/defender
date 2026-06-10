const BOMBER_BOMB_LIFETIME_RANDOM_MASK: u8 = 0x1F;
const BOMBER_TIE_SLOT_MASK: u8 = 0x06;
const BOMBER_TIE_SLOT_SHIFT: u8 = 1;
const ENEMY_FIREBALL_RANDOM_DELTA_MASK: u8 = 0x1F;
const ENEMY_FIREBALL_RANDOM_DELTA_CENTER: u8 = 0x10;
const ENEMY_FIREBALL_VELOCITY_SHIFT: u32 = 2;
const ENEMY_FIREBALL_PLAYER_VELOCITY_SEED_MIN: u8 = 120;

fn actor_enemy_projectile_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| is_enemy_projectile_kind(snapshot.kind))
        .count()
}

fn actor_bomb_projectile_count(prompt: &StepPrompt) -> usize {
    prompt
        .snapshots
        .iter()
        .filter(|snapshot| snapshot.kind == ActorKind::Bomb)
        .count()
}

fn bomber_bomb_lifetime_ticks(arcade_rng: ActorArcadeRngSnapshot) -> u8 {
    (arcade_rng.seed & BOMBER_BOMB_LIFETIME_RANDOM_MASK).wrapping_add(1)
}

fn arcade_tie_selected_slot(seed: u8) -> u8 {
    (seed & BOMBER_TIE_SLOT_MASK) >> BOMBER_TIE_SLOT_SHIFT
}

fn push_arcade_enemy_projectile_command(
    position: Point,
    velocity: Velocity,
    projectile_arcade_state: EnemyProjectileArcadeState,
    sound: SoundCue,
    commands: &mut Vec<GameCommand>,
) {
    commands.push(GameCommand::Spawn(SpawnRequest::EnemyLaser {
        position,
        velocity,
        arcade_state: Some(projectile_arcade_state),
    }));
    commands.push(GameCommand::PlaySound(sound));
}

fn arcade_enemy_fireball(
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    prompt: &StepPrompt,
    shot_rng: ActorArcadeRngSnapshot,
    lifetime_ticks: u8,
) -> Option<(Velocity, EnemyProjectileArcadeState)> {
    if !enemy_projectile_spawn_in_bounds(position)
        || actor_enemy_projectile_count(prompt) >= ENEMY_PROJECTILE_SLOT_LIMIT
    {
        return None;
    }
    let player_position = prompt.player_position()?;
    let player_velocity = prompt.player_velocity().unwrap_or_default();
    let x_delta = (shot_rng.seed & ENEMY_FIREBALL_RANDOM_DELTA_MASK)
        .wrapping_sub(ENEMY_FIREBALL_RANDOM_DELTA_CENTER)
        .wrapping_add(player_position.x as u8)
        .wrapping_sub(position.x as u8);
    let mut x_velocity =
        actor_sign_extend_u8_to_u16(x_delta).wrapping_shl(ENEMY_FIREBALL_VELOCITY_SHIFT);
    if shot_rng.seed > ENEMY_FIREBALL_PLAYER_VELOCITY_SEED_MIN {
        x_velocity = x_velocity.wrapping_add(
            arcade_velocity_word(player_velocity.dx).wrapping_shl(ENEMY_FIREBALL_VELOCITY_SHIFT),
        );
    }
    let y_delta = (shot_rng.lseed & ENEMY_FIREBALL_RANDOM_DELTA_MASK)
        .wrapping_sub(ENEMY_FIREBALL_RANDOM_DELTA_CENTER)
        .wrapping_add(player_position.y as u8)
        .wrapping_sub(position.y as u8);
    let y_velocity =
        actor_sign_extend_u8_to_u16(y_delta).wrapping_shl(ENEMY_FIREBALL_VELOCITY_SHIFT);
    let velocity = arcade_screen_velocity(x_velocity, y_velocity);
    Some((
        velocity,
        EnemyProjectileArcadeState {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
            lifetime_ticks,
        },
    ))
}
