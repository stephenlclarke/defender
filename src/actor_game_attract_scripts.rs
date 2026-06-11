use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteKey {
    WilliamsLogo,
    DefenderCoalescence,
    DefenderWordmark,
    DefenderLogo,
    HighScoreText,
    PlayerRight,
    PlayerLeft,
    Lander,
    Mutant,
    Bomber,
    Bomb,
    Pod,
    Swarmer,
    Baiter,
    Human,
    HumanFalling,
    HumanCarried,
    Laser,
    EnemyLaser,
    Explosion,
    Score250,
    Score500,
    Text,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VisualEffect {
    #[default]
    Static,
    MessageText {
        screen_cell: ScreenAddress,
        visual_offset: Point,
    },
    AttractScoringSurface {
        scoring_tick: TimelineStep,
    },
    WilliamsReveal {
        stroke_step: u16,
        color_step: u16,
    },
    DefenderCoalescence {
        slot: u8,
        row_pair: u8,
    },
    LanderSpriteFrame {
        animation_frame: SpriteFrameIndex,
    },
    BomberSpriteFrame {
        animation_frame: SpriteFrameIndex,
    },
    PodSprite,
    BaiterSpriteFrame {
        animation_frame: SpriteFrameIndex,
    },
    HumanSpriteFrame {
        animation_frame: SpriteFrameIndex,
    },
    ExplosionCloud {
        kind: ExplosionKind,
        age: u16,
        explosion_anchor: Option<Point>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
    Bomb,
    Player,
    Human,
    Terrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    Credit,
    Start,
    Thrust,
    Laser,
    SmartBomb,
    Hyperspace,
    PlayerAppear,
    HyperspaceMaterialize,
    Explosion,
    LanderHit,
    LanderPickup,
    HumanPulled,
    HumanReleased,
    HumanRescued,
    HumanSafeLanding,
    HumanLost,
    MutantSpawn,
    MutantHit,
    BomberHit,
    BombHit,
    PodHit,
    SwarmerHit,
    LanderShot,
    MutantShot,
    SwarmerShot,
    BaiterHit,
    BaiterShot,
    AttractPulse,
    GameOver,
    SoundBoardCommand(SoundCommand),
}

impl SoundCue {
    pub const fn sound_board_command(self) -> Option<SoundCommand> {
        match self {
            Self::Credit => Some(CREDIT_SOUND_COMMAND),
            Self::Start => Some(START_SOUND_COMMAND),
            Self::Thrust => Some(THRUST_SOUND_COMMAND),
            Self::Laser => Some(LASER_SOUND_COMMAND),
            Self::SmartBomb => Some(SMART_BOMB_SOUND_COMMAND),
            Self::Hyperspace => None,
            Self::PlayerAppear => Some(APPEARANCE_SOUND_COMMAND),
            Self::HyperspaceMaterialize => Some(APPEARANCE_SOUND_COMMAND),
            Self::Explosion => Some(EXPLOSION_SOUND_COMMAND),
            Self::LanderHit => Some(LANDER_HIT_SOUND_COMMAND),
            Self::LanderPickup => Some(LANDER_PICKUP_SOUND_COMMAND),
            Self::HumanPulled => Some(HUMAN_PULLED_SOUND_COMMAND),
            Self::HumanReleased => Some(ASTRONAUT_SHORT_CATCH_SOUND_COMMAND),
            Self::HumanRescued => Some(ASTRONAUT_CATCH_SOUND_COMMAND),
            Self::HumanSafeLanding => Some(HUMAN_SAFE_LANDING_SOUND_COMMAND),
            Self::HumanLost => Some(EXPLOSION_SOUND_COMMAND),
            Self::MutantSpawn => Some(EXPLOSION_SOUND_COMMAND),
            Self::MutantHit => Some(MUTANT_HIT_SOUND_COMMAND),
            Self::BomberHit => Some(BOMBER_HIT_SOUND_COMMAND),
            Self::BombHit => Some(EXPLOSION_SOUND_COMMAND),
            Self::PodHit => Some(POD_HIT_SOUND_COMMAND),
            Self::SwarmerHit => Some(SWARMER_OR_BAITER_HIT_SOUND_COMMAND),
            Self::LanderShot => Some(LANDER_OR_BAITER_SHOT_SOUND_COMMAND),
            Self::MutantShot => Some(MUTANT_SHOT_SOUND_COMMAND),
            Self::SwarmerShot => Some(SWARMER_SHOT_SOUND_COMMAND),
            Self::BaiterHit => Some(SWARMER_OR_BAITER_HIT_SOUND_COMMAND),
            Self::BaiterShot => Some(LANDER_OR_BAITER_SHOT_SOUND_COMMAND),
            Self::AttractPulse => None,
            Self::GameOver => Some(GAME_OVER_SOUND_COMMAND),
            Self::SoundBoardCommand(command) => Some(command),
        }
    }

    pub const fn sound_event(self) -> Option<SoundEvent> {
        match self {
            Self::Credit => Some(SoundEvent::CreditAdded),
            Self::Start => Some(SoundEvent::GameStarted),
            Self::Thrust => Some(SoundEvent::ThrustStarted),
            _ => match self.sound_board_command() {
                Some(command) => Some(SoundEvent::UnmappedSoundCommand { command }),
                None => None,
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActorSoundEventBridge {
    pub(in crate::actor_game) thrust_active: bool,
}

impl ActorSoundEventBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sound_events_for_report(&mut self, report: &StepReport) -> Vec<SoundEvent> {
        self.sound_events_for_cues(&report.sounds)
    }

    pub fn sound_events_for_cues(&mut self, cues: &[SoundCue]) -> Vec<SoundEvent> {
        let mut events = Vec::new();
        let mut thrust_requested = false;
        for cue in cues.iter().copied() {
            if cue == SoundCue::Thrust {
                if !thrust_requested && !self.thrust_active {
                    events.push(SoundEvent::ThrustStarted);
                }
                thrust_requested = true;
                continue;
            }

            if let Some(event) = cue.sound_event() {
                events.push(event);
            }
        }

        if !thrust_requested && self.thrust_active {
            events.push(SoundEvent::ThrustStopped);
        }
        self.thrust_active = thrust_requested;
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScript {
    pub(in crate::actor_game) events: Vec<AttractScriptEvent>,
    pub(in crate::actor_game) cycle_steps: Option<u64>,
}

impl AttractScript {
    pub fn new(events: Vec<AttractScriptEvent>) -> Self {
        Self::with_cycle_steps(events, None)
    }

    pub fn with_cycle_steps(mut events: Vec<AttractScriptEvent>, cycle_steps: Option<u64>) -> Self {
        events.sort_by_key(|event| event.start_after_steps);
        Self {
            events,
            cycle_steps: cycle_steps.filter(|steps| *steps > 0),
        }
    }

    pub fn parse_text(script_text: &str) -> Result<Self, AttractScriptParseError> {
        script_text.parse()
    }

    pub fn default_title() -> Self {
        Self::parse_text(ACTOR_ATTRACT_SCRIPT)
            .unwrap_or_else(|error| panic!("embedded actor attract script is invalid: {error}"))
    }

    pub fn default_title_from_events() -> Self {
        let mut events = vec![
            AttractScriptEvent::williams_logo(
                1,
                Some(ATTRACT_WILLIAMS_LOGO_DURATION_STEPS),
                ATTRACT_WILLIAMS_LOGO_POSITION,
            ),
            AttractScriptEvent::message_text(
                ATTRACT_PRESENTS_START_STEP,
                Some(ATTRACT_PRESENTS_DURATION_STEPS),
                PRESENTS_MESSAGE,
                ATTRACT_PRESENTS_ELECTRONICS_CELL,
            ),
            AttractScriptEvent::defender_wordmark(
                DEFENDER_WORDMARK_START_STEP,
                Some(ATTRACT_DEFENDER_WORDMARK_DURATION_STEPS),
                ATTRACT_DEFENDER_WORDMARK_POSITION,
            ),
            AttractScriptEvent::credits_when_nonzero(
                1,
                Some(ATTRACT_HALL_OF_FAME_START_STEP.saturating_sub(1)),
                ATTRACT_CREDIT_LABEL_POSITION,
                ATTRACT_CREDIT_COUNT_POSITION,
            ),
            AttractScriptEvent::credits(
                ATTRACT_HALL_OF_FAME_START_STEP,
                None,
                ATTRACT_CREDIT_LABEL_POSITION,
                ATTRACT_CREDIT_COUNT_POSITION,
            ),
            AttractScriptEvent::message_text_with_offset(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_TITLE_MESSAGE,
                ATTRACT_HALL_TITLE_MESSAGE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::message_text_with_offset(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_TODAYS_MESSAGE,
                ATTRACT_HALL_TODAYS_MESSAGE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::message_text_with_offset(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_ALL_TIME_MESSAGE,
                ATTRACT_HALL_ALL_TIME_MESSAGE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::message_text_with_offset(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_GREATEST_MESSAGE,
                ATTRACT_HALL_GREATEST_LEFT_MESSAGE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::message_text_with_offset(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_GREATEST_MESSAGE,
                ATTRACT_HALL_GREATEST_RIGHT_MESSAGE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
            AttractScriptEvent::sprite(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                SpriteKey::DefenderLogo,
                ATTRACT_HALL_DEFENDER_LOGO_POSITION,
            ),
            AttractScriptEvent::hall_scores(
                ATTRACT_HALL_OF_FAME_START_STEP,
                Some(ATTRACT_HALL_OF_FAME_DURATION_STEPS),
                ATTRACT_HALL_TODAYS_TABLE_CELL,
                ATTRACT_HALL_ALL_TIME_TABLE_CELL,
                ATTRACT_HALL_TABLE_VISUAL_OFFSET,
            ),
        ];
        events.push(AttractScriptEvent::scoring_surface(
            ATTRACT_SCORING_SEQUENCE_START_STEP,
            None,
        ));
        for (line_index, (message, screen_cell)) in
            ATTRACT_INSTRUCTION_TEXT_LINES.iter().copied().enumerate()
        {
            events.push(AttractScriptEvent::message_text_with_offset(
                actor_attract_scoring_instruction_text_start_step(line_index),
                None,
                message,
                screen_cell,
                ATTRACT_SCORING_VISUAL_OFFSET,
            ));
        }
        Self::with_cycle_steps(events, Some(ATTRACT_CYCLE_STEPS))
    }

    pub(in crate::actor_game) fn draws_for(
        &self,
        actor: ActorId,
        step: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        let step = self.cycled_step(step);
        self.events
            .iter()
            .filter(|event| event.active_at(step))
            .flat_map(|event| event.draws(actor, step, high_scores, credits))
            .collect()
    }

    pub fn manifest(&self) -> AttractScriptManifest {
        AttractScriptManifest {
            cycle_steps: self.cycle_steps,
            events: self
                .events
                .iter()
                .map(AttractScriptEvent::manifest)
                .collect(),
        }
    }

    fn cycled_step(&self, step: u64) -> u64 {
        let Some(cycle_steps) = self.cycle_steps else {
            return step;
        };
        let wrapped = step % cycle_steps;
        if wrapped == 0 { 1 } else { wrapped }
    }
}

impl FromStr for AttractScript {
    type Err = AttractScriptParseError;

    fn from_str(script_text: &str) -> Result<Self, Self::Err> {
        let mut events = Vec::new();
        let mut cycle_steps = None;
        for (line_index, input_line) in script_text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = input_line
                .split_once('#')
                .map_or(input_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            if let Some(cycle) = parse_attract_script_cycle_directive(line_number, line)? {
                if cycle_steps.replace(cycle).is_some() {
                    return Err(AttractScriptParseError::new(
                        line_number,
                        "duplicate cycle directive",
                    ));
                }
                continue;
            }
            events.push(parse_attract_script_event(line_number, line)?);
        }
        Ok(Self::with_cycle_steps(events, cycle_steps))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptParseError {
    pub line: usize,
    pub message: String,
}

impl AttractScriptParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for AttractScriptParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "attract script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for AttractScriptParseError {}

pub(in crate::actor_game) fn parse_attract_script_cycle_directive(
    line_number: usize,
    line: &str,
) -> Result<Option<u64>, AttractScriptParseError> {
    let mut parts = line.split_whitespace();
    let action = parts
        .next()
        .ok_or_else(|| AttractScriptParseError::new(line_number, "missing action"))?;
    if !matches!(
        normalize_script_token(action).as_str(),
        "cycle" | "loop" | "repeat"
    ) {
        return Ok(None);
    }

    let cycle_steps = parse_attract_u64(line_number, parts.next(), "cycle steps")?;
    if cycle_steps == 0 {
        return Err(AttractScriptParseError::new(
            line_number,
            "cycle steps must be greater than zero",
        ));
    }
    reject_extra_attract_fields(line_number, parts)?;
    Ok(Some(cycle_steps))
}

pub(in crate::actor_game) fn parse_attract_script_event(
    line_number: usize,
    line: &str,
) -> Result<AttractScriptEvent, AttractScriptParseError> {
    let mut parts = line.split_whitespace();
    let action = parts
        .next()
        .ok_or_else(|| AttractScriptParseError::new(line_number, "missing action"))?;
    let start_after_steps = parse_attract_u64(line_number, parts.next(), "start step")?;
    let duration_steps = parse_attract_duration(line_number, parts.next())?;

    match normalize_script_token(action).as_str() {
        "text" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            let value = parts.collect::<Vec<_>>().join(" ");
            if value.is_empty() {
                return Err(AttractScriptParseError::new(
                    line_number,
                    "text action needs a value",
                ));
            }
            Ok(AttractScriptEvent::text(
                start_after_steps,
                duration_steps,
                position,
                value,
            ))
        }
        "message" => {
            let message = parse_attract_message_id(line_number, parts.next())?;
            let screen_cell =
                parse_attract_screen_anchor(line_number, parts.next(), "message placement anchor")?;
            let visual_offset = parse_optional_attract_point(line_number, &mut parts)?;
            Ok(AttractScriptEvent::message_text_with_offset(
                start_after_steps,
                duration_steps,
                message,
                screen_cell,
                visual_offset,
            ))
        }
        "scoring_surface" | "scoring" | "attract_scoring" => {
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::scoring_surface(
                start_after_steps,
                duration_steps,
            ))
        }
        "sprite" => {
            let sprite_token = parts.next().ok_or_else(|| {
                AttractScriptParseError::new(line_number, "sprite action needs a sprite key")
            })?;
            let sprite = parse_attract_sprite_key(line_number, sprite_token)?;
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::sprite(
                start_after_steps,
                duration_steps,
                sprite,
                position,
            ))
        }
        "williams" | "williams_logo" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::williams_logo(
                start_after_steps,
                duration_steps,
                position,
            ))
        }
        "defender" | "defender_wordmark" | "wordmark" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::defender_wordmark(
                start_after_steps,
                duration_steps,
                position,
            ))
        }
        "high_scores" | "high_score_table" => {
            let position = parse_attract_point(line_number, &mut parts)?;
            let row_height = parse_attract_i16(line_number, parts.next(), "row height")?;
            let rows = parse_attract_usize(line_number, parts.next(), "rows")?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::high_scores(
                start_after_steps,
                duration_steps,
                position,
                row_height,
                rows,
            ))
        }
        "hall_scores" | "hall_of_fame_scores" | "hall_of_fame_table" => {
            let todays_table_cell = parse_attract_screen_anchor(
                line_number,
                parts.next(),
                "today's score-table placement anchor",
            )?;
            let all_time_table_cell = parse_attract_screen_anchor(
                line_number,
                parts.next(),
                "all-time score-table placement anchor",
            )?;
            let visual_offset = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::hall_scores(
                start_after_steps,
                duration_steps,
                todays_table_cell,
                all_time_table_cell,
                visual_offset,
            ))
        }
        "credits" | "credit_count" => {
            let label_position = parse_attract_point(line_number, &mut parts)?;
            let count_position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::credits(
                start_after_steps,
                duration_steps,
                label_position,
                count_position,
            ))
        }
        "credits_nonzero" | "credit_count_nonzero" | "credits_when_nonzero" => {
            let label_position = parse_attract_point(line_number, &mut parts)?;
            let count_position = parse_attract_point(line_number, &mut parts)?;
            reject_extra_attract_fields(line_number, parts)?;
            Ok(AttractScriptEvent::credits_when_nonzero(
                start_after_steps,
                duration_steps,
                label_position,
                count_position,
            ))
        }
        _ => Err(AttractScriptParseError::new(
            line_number,
            format!("unknown action `{action}`"),
        )),
    }
}

pub(in crate::actor_game) fn parse_attract_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, AttractScriptParseError> {
    let x = parse_attract_i16(line_number, parts.next(), "x")?;
    let y = parse_attract_i16(line_number, parts.next(), "y")?;
    Ok(Point::new(x, y))
}

pub(in crate::actor_game) fn parse_optional_attract_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, AttractScriptParseError> {
    let Some(x_token) = parts.next() else {
        return Ok(Point::new(0, 0));
    };
    let x = parse_attract_i16(line_number, Some(x_token), "offset x")?;
    let y = parse_attract_i16(line_number, parts.next(), "offset y")?;
    reject_extra_attract_fields(line_number, parts.by_ref())?;
    Ok(Point::new(x, y))
}

pub(in crate::actor_game) fn parse_attract_duration(
    line_number: usize,
    token: Option<&str>,
) -> Result<Option<u64>, AttractScriptParseError> {
    let token =
        token.ok_or_else(|| AttractScriptParseError::new(line_number, "missing duration"))?;
    if token == "-" {
        return Ok(None);
    }
    match normalize_script_token(token).as_str() {
        "none" | "forever" | "infinite" => Ok(None),
        _ => Ok(Some(parse_attract_u64(
            line_number,
            Some(token),
            "duration",
        )?)),
    }
}

pub(in crate::actor_game) fn parse_attract_u64(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u64, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<u64>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

pub(in crate::actor_game) fn parse_attract_i16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<i16, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<i16>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

pub(in crate::actor_game) fn parse_attract_usize(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<usize, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    token.parse::<usize>().map_err(|error| {
        AttractScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

pub(in crate::actor_game) fn parse_attract_screen_anchor(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<ScreenAddress, AttractScriptParseError> {
    let token = token
        .ok_or_else(|| AttractScriptParseError::new(line_number, format!("missing {field}")))?;
    match normalize_script_token(token).as_str() {
        "williams_electronics" | "presents_text" => Ok(ATTRACT_PRESENTS_ELECTRONICS_CELL),
        "hall_title" => Ok(ATTRACT_HALL_TITLE_MESSAGE_CELL),
        "hall_todays" => Ok(ATTRACT_HALL_TODAYS_MESSAGE_CELL),
        "hall_all_time" => Ok(ATTRACT_HALL_ALL_TIME_MESSAGE_CELL),
        "hall_greatest_left" => Ok(ATTRACT_HALL_GREATEST_LEFT_MESSAGE_CELL),
        "hall_greatest_right" => Ok(ATTRACT_HALL_GREATEST_RIGHT_MESSAGE_CELL),
        "hall_todays_table" | "todays_score_table" => Ok(ATTRACT_HALL_TODAYS_TABLE_CELL),
        "hall_all_time_table" | "all_time_score_table" => Ok(ATTRACT_HALL_ALL_TIME_TABLE_CELL),
        "scanner_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[0].1),
        "lander_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[1].1),
        "mutant_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[2].1),
        "baiter_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[3].1),
        "bomber_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[4].1),
        "swarmer_pod_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[5].1),
        "swarmer_instruction" => Ok(ATTRACT_INSTRUCTION_TEXT_LINES[6].1),
        "player_start_prompt" => Ok(PLAYER_START_PROMPT_SCREEN_CELL),
        _ => Err(AttractScriptParseError::new(
            line_number,
            format!("unknown {field} `{token}`"),
        )),
    }
}

pub(in crate::actor_game) fn parse_attract_message_id(
    line_number: usize,
    token: Option<&str>,
) -> Result<MessageId, AttractScriptParseError> {
    let token = token.ok_or_else(|| {
        AttractScriptParseError::new(line_number, "message action needs a message key")
    })?;
    crate::arcade_assets::message_id_from_script_key(token).ok_or_else(|| {
        AttractScriptParseError::new(line_number, format!("unknown message key `{token}`"))
    })
}

pub(in crate::actor_game) fn parse_attract_sprite_key(
    line_number: usize,
    token: &str,
) -> Result<SpriteKey, AttractScriptParseError> {
    match normalize_script_token(token).as_str() {
        "williams_logo" | "williams" => Ok(SpriteKey::WilliamsLogo),
        "defender_wordmark" | "wordmark" => Ok(SpriteKey::DefenderWordmark),
        "defender_logo" | "defender" => Ok(SpriteKey::DefenderLogo),
        "high_score_text" | "high_scores" => Ok(SpriteKey::HighScoreText),
        "player_right" | "player" => Ok(SpriteKey::PlayerRight),
        "player_left" => Ok(SpriteKey::PlayerLeft),
        "lander" => Ok(SpriteKey::Lander),
        "mutant" => Ok(SpriteKey::Mutant),
        "bomber" => Ok(SpriteKey::Bomber),
        "bomb" => Ok(SpriteKey::Bomb),
        "pod" => Ok(SpriteKey::Pod),
        "swarmer" => Ok(SpriteKey::Swarmer),
        "baiter" => Ok(SpriteKey::Baiter),
        "human" => Ok(SpriteKey::Human),
        "human_falling" => Ok(SpriteKey::HumanFalling),
        "human_carried" => Ok(SpriteKey::HumanCarried),
        "laser" => Ok(SpriteKey::Laser),
        "enemy_laser" => Ok(SpriteKey::EnemyLaser),
        "explosion" => Ok(SpriteKey::Explosion),
        "score250" | "score_250" => Ok(SpriteKey::Score250),
        "score500" | "score_500" => Ok(SpriteKey::Score500),
        "text" => Ok(SpriteKey::Text),
        _ => Err(AttractScriptParseError::new(
            line_number,
            format!("unknown sprite key `{token}`"),
        )),
    }
}

pub(in crate::actor_game) fn reject_extra_attract_fields<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), AttractScriptParseError> {
    if let Some(extra) = parts.next() {
        Err(AttractScriptParseError::new(
            line_number,
            format!("unexpected extra field `{extra}`"),
        ))
    } else {
        Ok(())
    }
}

pub(in crate::actor_game) fn normalize_script_token(token: &str) -> String {
    token.trim().replace('-', "_").to_ascii_lowercase()
}

impl Default for AttractScript {
    fn default() -> Self {
        Self::default_title()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptEvent {
    pub start_after_steps: u64,
    pub duration_steps: Option<u64>,
    pub action: AttractScriptAction,
}

impl AttractScriptEvent {
    pub fn text(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
        value: impl Into<String>,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Text {
                position,
                value: value.into(),
            },
        }
    }

    pub fn message_text(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        message: MessageId,
        screen_cell: ScreenAddress,
    ) -> Self {
        Self::message_text_with_offset(
            start_after_steps,
            duration_steps,
            message,
            screen_cell,
            Point::new(0, 0),
        )
    }

    pub fn message_text_with_offset(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        message: MessageId,
        screen_cell: ScreenAddress,
        visual_offset: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::MessageText {
                message,
                screen_cell,
                visual_offset,
            },
        }
    }

    pub fn sprite(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        sprite: SpriteKey,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Sprite { sprite, position },
        }
    }

    pub fn williams_logo(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::WilliamsLogo {
                position,
                reveal_steps: WILLIAMS_REVEAL_STEPS,
                color_period: WILLIAMS_COLOR_PERIOD,
            },
        }
    }

    pub fn defender_wordmark(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::DefenderWordmark {
                position,
                slots: DEFENDER_WORDMARK_SLOTS,
                row_pairs: DEFENDER_WORDMARK_ROW_PAIRS,
            },
        }
    }

    pub fn high_scores(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        position: Point,
        row_height: i16,
        rows: usize,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::HighScores {
                position,
                row_height,
                rows,
            },
        }
    }

    pub fn hall_scores(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        todays_table_cell: ScreenAddress,
        all_time_table_cell: ScreenAddress,
        visual_offset: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::HallScores {
                todays_table_cell,
                all_time_table_cell,
                visual_offset,
            },
        }
    }

    pub fn scoring_surface(start_after_steps: u64, duration_steps: Option<u64>) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::ScoringSurface,
        }
    }

    pub fn credits(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label_position: Point,
        count_position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Credits {
                label_position,
                count_position,
                minimum_credits: 0,
            },
        }
    }

    pub fn credits_when_nonzero(
        start_after_steps: u64,
        duration_steps: Option<u64>,
        label_position: Point,
        count_position: Point,
    ) -> Self {
        Self {
            start_after_steps,
            duration_steps,
            action: AttractScriptAction::Credits {
                label_position,
                count_position,
                minimum_credits: 1,
            },
        }
    }

    fn active_at(&self, step: u64) -> bool {
        if step < self.start_after_steps {
            return false;
        }
        match self.duration_steps {
            Some(duration_steps) => step < self.start_after_steps.saturating_add(duration_steps),
            None => true,
        }
    }

    fn draws(
        &self,
        actor: ActorId,
        step: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        self.action
            .draws(actor, self.age(step), high_scores, credits)
    }

    fn age(&self, step: u64) -> u64 {
        step.saturating_sub(self.start_after_steps)
            .saturating_add(1)
    }

    fn manifest(&self) -> AttractScriptEventManifest {
        AttractScriptEventManifest {
            start_after_steps: self.start_after_steps,
            duration_steps: self.duration_steps,
            action: self.action.manifest(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractScriptAction {
    Text {
        position: Point,
        value: String,
    },
    MessageText {
        message: MessageId,
        screen_cell: ScreenAddress,
        visual_offset: Point,
    },
    Sprite {
        sprite: SpriteKey,
        position: Point,
    },
    WilliamsLogo {
        position: Point,
        reveal_steps: u16,
        color_period: u16,
    },
    DefenderWordmark {
        position: Point,
        slots: u16,
        row_pairs: u16,
    },
    HighScores {
        position: Point,
        row_height: i16,
        rows: usize,
    },
    HallScores {
        todays_table_cell: ScreenAddress,
        all_time_table_cell: ScreenAddress,
        visual_offset: Point,
    },
    ScoringSurface,
    Credits {
        label_position: Point,
        count_position: Point,
        minimum_credits: u8,
    },
}

impl AttractScriptAction {
    fn draws(
        &self,
        actor: ActorId,
        age: u64,
        high_scores: &[u32; 5],
        credits: u8,
    ) -> Vec<DrawCommand> {
        match self {
            Self::Text { position, value } => {
                vec![DrawCommand::text(actor, *position, value.clone())]
            }
            Self::MessageText {
                message,
                screen_cell,
                visual_offset,
            } => vec![DrawCommand::message_text_with_offset(
                actor,
                crate::arcade_assets::message_text(*message),
                *screen_cell,
                *visual_offset,
            )],
            Self::Sprite { sprite, position } => {
                vec![DrawCommand::sprite(actor, *sprite, *position)]
            }
            Self::WilliamsLogo {
                position,
                reveal_steps,
                color_period: _,
            } => {
                let color_step = u16::try_from(age.saturating_sub(1)).unwrap_or(u16::MAX);
                vec![DrawCommand::sprite_with_effect(
                    actor,
                    SpriteKey::WilliamsLogo,
                    *position,
                    VisualEffect::WilliamsReveal {
                        stroke_step: (age as u16).min(*reveal_steps),
                        color_step,
                    },
                )]
            }
            Self::DefenderWordmark {
                position,
                slots,
                row_pairs,
            } => {
                let row_pairs = (*row_pairs).max(1);
                let progress = age.saturating_sub(1) as u16;
                let total_steps = slots.saturating_mul(row_pairs);
                if progress >= total_steps {
                    vec![DrawCommand::sprite(
                        actor,
                        SpriteKey::DefenderWordmark,
                        *position,
                    )]
                } else {
                    vec![DrawCommand::sprite_with_effect(
                        actor,
                        SpriteKey::DefenderCoalescence,
                        *position,
                        VisualEffect::DefenderCoalescence {
                            slot: (progress / row_pairs) as u8,
                            row_pair: (progress % row_pairs) as u8,
                        },
                    )]
                }
            }
            Self::HighScores {
                position,
                row_height,
                rows,
            } => high_scores
                .iter()
                .copied()
                .take((*rows).min(high_scores.len()))
                .enumerate()
                .map(|(index, score)| {
                    DrawCommand::text(
                        actor,
                        Point::new(
                            position.x,
                            position.y
                                + i16::try_from(index)
                                    .unwrap_or(i16::MAX)
                                    .saturating_mul(*row_height),
                        ),
                        format!("{}. {}", index + 1, format_status_score(score)),
                    )
                })
                .collect(),
            Self::HallScores {
                todays_table_cell,
                all_time_table_cell,
                visual_offset,
            } => {
                let entries = hall_score_seed_entries();
                let mut draws =
                    hall_score_table_draws(actor, entries, *todays_table_cell, *visual_offset);
                draws.extend(hall_score_table_draws(
                    actor,
                    entries,
                    *all_time_table_cell,
                    *visual_offset,
                ));
                draws
            }
            Self::ScoringSurface => vec![DrawCommand::sprite_with_effect(
                actor,
                SpriteKey::Text,
                Point::new(0, 0),
                VisualEffect::AttractScoringSurface {
                    scoring_tick: TimelineStep::new(
                        u16::try_from(age.saturating_sub(1)).unwrap_or(u16::MAX),
                    ),
                },
            )],
            Self::Credits {
                label_position,
                count_position,
                minimum_credits,
            } => {
                if credits < *minimum_credits {
                    Vec::new()
                } else {
                    vec![
                        DrawCommand::text(actor, *label_position, credits_prompt_text()),
                        DrawCommand::text(
                            actor,
                            *count_position,
                            format!("{:02}", credits.min(99)),
                        ),
                    ]
                }
            }
        }
    }

    fn manifest(&self) -> AttractScriptActionManifest {
        match self {
            Self::Text { position, value } => AttractScriptActionManifest::Text {
                position: *position,
                value: value.clone(),
            },
            Self::MessageText {
                message,
                screen_cell,
                visual_offset,
            } => AttractScriptActionManifest::MessageText {
                message: format!("{message:?}"),
                screen_cell: *screen_cell,
                visual_offset: *visual_offset,
            },
            Self::Sprite { sprite, position } => AttractScriptActionManifest::Sprite {
                sprite: *sprite,
                position: *position,
            },
            Self::WilliamsLogo {
                position,
                reveal_steps,
                color_period,
            } => AttractScriptActionManifest::WilliamsLogo {
                position: *position,
                reveal_steps: *reveal_steps,
                color_period: *color_period,
            },
            Self::DefenderWordmark {
                position,
                slots,
                row_pairs,
            } => AttractScriptActionManifest::DefenderWordmark {
                position: *position,
                slots: *slots,
                row_pairs: *row_pairs,
            },
            Self::HighScores {
                position,
                row_height,
                rows,
            } => AttractScriptActionManifest::HighScores {
                position: *position,
                row_height: *row_height,
                rows: *rows,
            },
            Self::HallScores {
                todays_table_cell,
                all_time_table_cell,
                visual_offset,
            } => AttractScriptActionManifest::HallScores {
                todays_table_cell: *todays_table_cell,
                all_time_table_cell: *all_time_table_cell,
                visual_offset: *visual_offset,
            },
            Self::ScoringSurface => AttractScriptActionManifest::ScoringSurface,
            Self::Credits {
                label_position,
                count_position,
                minimum_credits,
            } => AttractScriptActionManifest::Credits {
                label_position: *label_position,
                count_position: *count_position,
                minimum_credits: *minimum_credits,
            },
        }
    }
}

pub(in crate::actor_game) fn credits_prompt_text() -> &'static str {
    crate::arcade_assets::message_text(CREDITS_MESSAGE)
}

pub(in crate::actor_game) fn hall_score_seed_entries()
-> [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES] {
    std::array::from_fn(high_score_seed_entry)
}

pub(in crate::actor_game) fn hall_score_entries(
    high_scores: &[u32; 5],
) -> [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES] {
    std::array::from_fn(|index| {
        let seed = high_score_seed_entry(index);
        HighScoreTableEntrySnapshot {
            rank: seed.rank,
            score: high_scores.get(index).copied().unwrap_or(seed.score),
            initials: seed.initials,
        }
    })
}

pub(in crate::actor_game) fn high_score_seed_entry(index: usize) -> HighScoreTableEntrySnapshot {
    let seed = crate::arcade_assets::HIGH_SCORE_SEEDS
        .get(index)
        .unwrap_or_else(|| panic!("missing embedded high-score seed row {index}"));
    HighScoreTableEntrySnapshot {
        rank: u8::try_from(index + 1).expect("embedded high-score rank fits u8"),
        score: seed.score,
        initials: [
            Some(seed.initials[0]),
            Some(seed.initials[1]),
            Some(seed.initials[2]),
        ],
    }
}

pub(in crate::actor_game) fn hall_score_table_draws(
    actor: ActorId,
    entries: [HighScoreTableEntrySnapshot; HIGH_SCORE_TABLE_ENTRIES],
    top_left_screen_cell: ScreenAddress,
    visual_offset: Point,
) -> Vec<DrawCommand> {
    let mut draws = Vec::with_capacity(entries.len() * 3);
    for (index, entry) in entries.iter().copied().enumerate() {
        let vertical_offset = u8::try_from(index).expect("high-score table index fits in u8")
            * ATTRACT_HALL_TABLE_ROW_STEP;
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                screen_point_with_offset(top_left_screen_cell, 0, vertical_offset),
                visual_offset,
            ),
            char::from(b'1' + u8::try_from(index).expect("high-score rank fits u8")).to_string(),
        ));
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                screen_point_with_offset(
                    top_left_screen_cell,
                    ATTRACT_HALL_TABLE_INITIALS_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            hall_score_initials_text(entry.initials),
        ));
        draws.push(DrawCommand::text(
            actor,
            offset_point(
                screen_point_with_offset(
                    top_left_screen_cell,
                    ATTRACT_HALL_TABLE_SCORE_OFFSET,
                    vertical_offset,
                ),
                visual_offset,
            ),
            hall_score_text(entry.score),
        ));
    }
    draws
}

pub(in crate::actor_game) fn screen_point_with_offset(
    top_left_screen_cell: ScreenAddress,
    horizontal: u8,
    vertical: u8,
) -> Point {
    let [column, row] = top_left_screen_cell.word().to_be_bytes();
    Point::new(
        i16::from(column.wrapping_add(horizontal)) * 2,
        i16::from(row.wrapping_add(vertical)),
    )
}

pub(in crate::actor_game) fn offset_point(point: Point, offset: Point) -> Point {
    Point::new(
        point.x.saturating_add(offset.x),
        point.y.saturating_add(offset.y),
    )
}

pub(in crate::actor_game) fn hall_score_initials_text(initials: [Option<char>; 3]) -> String {
    initials
        .into_iter()
        .map(|initial| initial.unwrap_or(' '))
        .collect()
}

pub(in crate::actor_game) const HALL_SCORE_DISPLAY_MAX: u32 = 999_999;
pub(in crate::actor_game) const HALL_SCORE_START_PLACE: u32 = 100_000;
pub(in crate::actor_game) const HALL_SCORE_RADIX: u32 = 10;

pub(in crate::actor_game) fn hall_score_text(score: u32) -> String {
    let mut text = [b' '; ATTRACT_HALL_SCORE_TEXT_LEN];
    let mut place = HALL_SCORE_START_PLACE;
    let mut seen_non_zero = false;
    for byte in &mut text {
        let digit = ((score.min(HALL_SCORE_DISPLAY_MAX) / place) % HALL_SCORE_RADIX) as u8;
        if digit != 0 || seen_non_zero {
            seen_non_zero = true;
            *byte = b'0' + digit;
        }
        place /= HALL_SCORE_RADIX;
    }
    String::from_utf8_lossy(&text).into_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptManifest {
    pub cycle_steps: Option<u64>,
    pub events: Vec<AttractScriptEventManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttractScriptEventManifest {
    pub start_after_steps: u64,
    pub duration_steps: Option<u64>,
    pub action: AttractScriptActionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttractScriptActionManifest {
    Text {
        position: Point,
        value: String,
    },
    MessageText {
        message: String,
        screen_cell: ScreenAddress,
        visual_offset: Point,
    },
    Sprite {
        sprite: SpriteKey,
        position: Point,
    },
    WilliamsLogo {
        position: Point,
        reveal_steps: u16,
        color_period: u16,
    },
    DefenderWordmark {
        position: Point,
        slots: u16,
        row_pairs: u16,
    },
    HighScores {
        position: Point,
        row_height: i16,
        rows: usize,
    },
    HallScores {
        todays_table_cell: ScreenAddress,
        all_time_table_cell: ScreenAddress,
        visual_offset: Point,
    },
    ScoringSurface,
    Credits {
        label_position: Point,
        count_position: Point,
        minimum_credits: u8,
    },
}
