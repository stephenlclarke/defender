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
    (arcade_rng.seed & 0x1F).wrapping_add(1)
}

fn arcade_tie_selected_slot(seed: u8) -> u8 {
    (seed & 0x06) >> 1
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
    let x_delta = (shot_rng.seed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.x as u8)
        .wrapping_sub(position.x as u8);
    let mut x_velocity = actor_sign_extend_u8_to_u16(x_delta).wrapping_shl(2);
    if shot_rng.seed > 120 {
        x_velocity =
            x_velocity.wrapping_add(arcade_velocity_word(player_velocity.dx).wrapping_shl(2));
    }
    let y_delta = (shot_rng.lseed & 0x1F)
        .wrapping_sub(0x10)
        .wrapping_add(player_position.y as u8)
        .wrapping_sub(position.y as u8);
    let y_velocity = actor_sign_extend_u8_to_u16(y_delta).wrapping_shl(2);
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
