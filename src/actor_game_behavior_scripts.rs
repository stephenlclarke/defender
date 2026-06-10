fn parse_behavior_actor_kind(
    line_number: usize,
    token: &str,
) -> Result<ActorKind, ActorBehaviorScriptParseError> {
    match normalize_script_token(token).as_str() {
        "attract_director" => Ok(ActorKind::AttractDirector),
        "attract_script" => Ok(ActorKind::AttractScript),
        "status_display" => Ok(ActorKind::StatusDisplay),
        "williams_logo" => Ok(ActorKind::WilliamsLogo),
        "defender_wordmark" => Ok(ActorKind::DefenderWordmark),
        "player" => Ok(ActorKind::Player),
        "lander" => Ok(ActorKind::Lander),
        "mutant" => Ok(ActorKind::Mutant),
        "bomber" => Ok(ActorKind::Bomber),
        "bomb" => Ok(ActorKind::Bomb),
        "pod" => Ok(ActorKind::Pod),
        "swarmer" => Ok(ActorKind::Swarmer),
        "baiter" => Ok(ActorKind::Baiter),
        "human" => Ok(ActorKind::Human),
        "laser" => Ok(ActorKind::Laser),
        "enemy_laser" => Ok(ActorKind::EnemyLaser),
        "explosion" => Ok(ActorKind::Explosion),
        "score_popup" => Ok(ActorKind::ScorePopup),
        "text" => Ok(ActorKind::Text),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("unknown actor kind `{token}`"),
        )),
    }
}

fn parse_behavior_i16_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<i16, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    i16::try_from(parse_behavior_i64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u8_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u8, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    u8::try_from(parse_behavior_u64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u16_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u16, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    u16::try_from(parse_behavior_u64(line_number, value, field)?).map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_u64_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<u64, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    parse_behavior_u64(line_number, value, field)
}

fn parse_behavior_bool_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<bool, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a boolean"),
        )),
    }
}

fn parse_behavior_hyperspace_seed_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<Option<ActorHyperspaceSeed>, ActorBehaviorScriptParseError> {
    if values.len() == 1 && normalize_script_token(values[0]) == "none" {
        return Ok(None);
    }
    if values.len() != 3 {
        return Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} needs `none` or three seed bytes"),
        ));
    }
    Ok(Some(ActorHyperspaceSeed {
        seed: parse_behavior_u8_value(line_number, &values[0..1], "seed")?,
        hseed: parse_behavior_u8_value(line_number, &values[1..2], "hseed")?,
        lseed: parse_behavior_u8_value(line_number, &values[2..3], "lseed")?,
    }))
}

fn parse_lander_behavior_mode_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<LanderBehaviorMode, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "seek_nearest_human" | "seek_human" | "human" => Ok(LanderBehaviorMode::SeekNearestHuman),
        "chase_player" | "chase" | "player" => Ok(LanderBehaviorMode::ChasePlayer),
        "drift" => Ok(LanderBehaviorMode::Drift),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a lander behavior mode"),
        )),
    }
}

fn parse_hostile_movement_mode_value(
    line_number: usize,
    values: &[&str],
    field: &str,
) -> Result<HostileMovementMode, ActorBehaviorScriptParseError> {
    let value = parse_behavior_single_value(line_number, values, field)?;
    match normalize_script_token(value).as_str() {
        "drift" => Ok(HostileMovementMode::Drift),
        "chase_player" | "chase" | "player" => Ok(HostileMovementMode::ChasePlayer),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is not a hostile movement mode"),
        )),
    }
}

fn parse_behavior_single_value<'a>(
    line_number: usize,
    values: &'a [&'a str],
    field: &str,
) -> Result<&'a str, ActorBehaviorScriptParseError> {
    match values {
        [value] => Ok(value),
        [] => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} needs a value"),
        )),
        _ => Err(ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} accepts one value"),
        )),
    }
}

fn parse_behavior_u64(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<u64, ActorBehaviorScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(|error| {
            ActorBehaviorScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<u64>().map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_behavior_i64(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<i64, ActorBehaviorScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("-0x")
        .or_else(|| value.strip_prefix("-0X"))
    {
        return i64::from_str_radix(hex, 16)
            .map(|value| -value)
            .map_err(|error| {
                ActorBehaviorScriptParseError::new(
                    line_number,
                    format!("{field} `{value}` is invalid: {error}"),
                )
            });
    }
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return i64::from_str_radix(hex, 16).map_err(|error| {
            ActorBehaviorScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<i64>().map_err(|error| {
        ActorBehaviorScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

impl Default for ActorBehaviorScript {
    fn default() -> Self {
        Self::default_script()
    }
}
