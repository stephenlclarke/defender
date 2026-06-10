#[derive(Debug, Clone, PartialEq)]
pub struct ActorFrame {
    pub state: GameState,
    pub report: StepReport,
    pub events: GameEvents,
    pub scene: RenderScene,
}

impl ActorFrame {
    pub fn new(
        report: StepReport,
        state: GameState,
        events: GameEvents,
        scene: RenderScene,
    ) -> Self {
        Self {
            state,
            report,
            events,
            scene,
        }
    }

    pub fn game_frame(&self) -> GameFrame {
        GameFrame {
            state: self.state.clone(),
            events: self.events.clone(),
            scene: self.scene.clone(),
        }
    }
}

pub struct ActorRuntimeAdapter {
    driver: ActorGameDriver,
    sound_bridge: ActorSoundEventBridge,
    render_bridge: ActorRenderSceneBridge,
}

impl ActorRuntimeAdapter {
    pub fn new() -> Self {
        Self::with_driver(ActorGameDriver::new())
    }

    pub fn new_with_free_play_admission() -> Self {
        Self::with_driver(ActorGameDriver::new().with_free_play_admission(true))
    }

    pub fn with_scripts(scripts: ActorDriverScripts) -> Self {
        Self::with_driver(ActorGameDriver::with_scripts(scripts))
    }

    pub fn with_scripts_and_free_play_admission(scripts: ActorDriverScripts) -> Self {
        Self::with_driver(ActorGameDriver::with_scripts(scripts).with_free_play_admission(true))
    }

    pub fn with_driver(driver: ActorGameDriver) -> Self {
        Self::with_components(
            driver,
            ActorSoundEventBridge::new(),
            ActorRenderSceneBridge::new(),
        )
    }

    pub fn with_components(
        driver: ActorGameDriver,
        sound_bridge: ActorSoundEventBridge,
        render_bridge: ActorRenderSceneBridge,
    ) -> Self {
        Self {
            driver,
            sound_bridge,
            render_bridge,
        }
    }

    pub fn driver(&self) -> &ActorGameDriver {
        &self.driver
    }

    pub fn driver_mut(&mut self) -> &mut ActorGameDriver {
        &mut self.driver
    }

    pub fn step(&mut self, input: GameInput) -> ActorFrame {
        let report = self.driver.step(input);
        self.frame_for_report(report)
    }

    pub fn step_clean_input(&mut self, input: CleanGameInput, xyzzy: XyzzyMode) -> ActorFrame {
        self.step(GameInput::from_clean_input(input, xyzzy))
    }

    fn frame_for_report(&mut self, report: StepReport) -> ActorFrame {
        let state = report.game_state();
        let gameplay_events = actor_gameplay_events_for_report(&report);
        let sound_events = self.sound_bridge.sound_events_for_report(&report);
        let scene = self.render_bridge.render_scene_for_report(&report);
        ActorFrame::new(
            report,
            state,
            GameEvents::new(gameplay_events, sound_events),
            scene,
        )
    }
}

impl Default for ActorRuntimeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct HighScoreEntryStep {
    accepted: bool,
    submitted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AppliedCommands {
    sounds: Vec<SoundCue>,
    draws: Vec<DrawCommand>,
    bonus_awarded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingSurvivorBonus {
    next_wave: u16,
    multiplier: u8,
    total_survivors: u8,
    visible_icons: u8,
    remaining_awards: u8,
    astronaut_sleep_steps_remaining: u8,
    wave_advance_sleep_steps_remaining: Option<u8>,
}

impl PendingSurvivorBonus {
    fn new(current_wave: u16, next_wave: u16, survivors: usize) -> Self {
        let total_survivors = u8::try_from(survivors.min(SURVIVOR_BONUS_HUMAN_LIMIT))
            .unwrap_or(SURVIVOR_BONUS_HUMAN_LIMIT as u8);
        Self {
            next_wave,
            multiplier: clean_wave(current_wave).min(5),
            total_survivors,
            visible_icons: 0,
            remaining_awards: total_survivors,
            astronaut_sleep_steps_remaining: 0,
            wave_advance_sleep_steps_remaining: None,
        }
    }

    fn bonus_points(&self) -> u32 {
        u32::from(self.multiplier) * SURVIVOR_BONUS_POINTS_PER_MULTIPLIER
    }

    fn award_next_survivor(&mut self) -> Option<u32> {
        if self.remaining_awards == 0 {
            return None;
        }

        self.remaining_awards = self.remaining_awards.saturating_sub(1);
        self.visible_icons = self
            .visible_icons
            .saturating_add(1)
            .min(self.total_survivors);
        self.astronaut_sleep_steps_remaining = SURVIVOR_BONUS_ASTRONAUT_SLEEP_STEPS;
        Some(self.bonus_points())
    }

    fn report(self, awarded_points: Option<u32>) -> SurvivorBonusReport {
        SurvivorBonusReport {
            next_wave: self.next_wave,
            multiplier: self.multiplier,
            total_survivors: self.total_survivors,
            visible_icons: self.visible_icons,
            remaining_awards: self.remaining_awards,
            awarded_points,
            astronaut_sleep_steps_remaining: self.astronaut_sleep_steps_remaining,
            wave_advance_sleep_steps_remaining: self.wave_advance_sleep_steps_remaining,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingPlayerSwitch {
    sleep_steps_remaining: u8,
    from_player: u8,
    to_player: u8,
}

impl PendingPlayerSwitch {
    const fn new(from_player: u8, to_player: u8) -> Self {
        Self {
            sleep_steps_remaining: PLAYER_SWITCH_DELAY_STEPS,
            from_player,
            to_player,
        }
    }

    const fn report(self) -> PlayerSwitchReport {
        PlayerSwitchReport {
            sleep_steps_remaining: self.sleep_steps_remaining,
            from_player: self.from_player,
            to_player: self.to_player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingPlayerStart {
    delay_steps_remaining: u8,
    player: u8,
}

impl PendingPlayerStart {
    const fn new(player: u8) -> Self {
        Self {
            delay_steps_remaining: PLAYER_START_PLAYFIELD_DELAY_STEPS,
            player,
        }
    }

    const fn report(self) -> PlayerStartReport {
        PlayerStartReport {
            delay_steps_remaining: self.delay_steps_remaining,
            player: self.player,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingActorSoundCommand {
    steps_remaining: u8,
    command: SoundCommand,
    trigger: PendingActorSoundTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingActorSoundTrigger {
    SmartBomb,
    TerrainBlow,
    AstronautRescue,
    FirstWaveLanderRefill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SurvivorBonusStep {
    Waiting,
    Award(u32),
    StartNextWave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerStartStep {
    Waiting,
    StartPlayfield,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScripts {
    pub attract_script: AttractScript,
    pub behavior_script: ActorBehaviorScript,
    pub wave_script: ActorWaveScript,
}

impl ActorDriverScripts {
    pub fn new(
        attract_script: AttractScript,
        behavior_script: ActorBehaviorScript,
        wave_script: ActorWaveScript,
    ) -> Self {
        Self {
            attract_script,
            behavior_script,
            wave_script,
        }
    }

    pub fn parse_text(script_text: &str) -> Result<Self, ActorDriverScriptsParseError> {
        let sections = ParsedActorDriverScriptSections::parse(script_text)?;
        Self::parse_texts(
            sections.attract.as_str(),
            sections.behavior.as_str(),
            sections.wave.as_str(),
        )
    }

    pub fn parse_texts(
        attract_script_text: &str,
        behavior_script_text: &str,
        wave_script_text: &str,
    ) -> Result<Self, ActorDriverScriptsParseError> {
        let attract_script = AttractScript::parse_text(attract_script_text)
            .map_err(ActorDriverScriptsParseError::from_attract)?;
        let behavior_script = ActorBehaviorScript::parse_text(behavior_script_text)
            .map_err(ActorDriverScriptsParseError::from_behavior)?;
        let wave_script =
            ActorWaveScript::parse_text_with_base_behavior(wave_script_text, &behavior_script)
                .map_err(ActorDriverScriptsParseError::from_wave)?;
        Ok(Self::new(attract_script, behavior_script, wave_script))
    }

    pub fn manifest(&self) -> ActorDriverScriptsManifest {
        ActorDriverScriptsManifest {
            attract_script: self.attract_script.manifest(),
            behavior_script: self.behavior_script.manifest(),
            wave_script: self.wave_script.manifest(),
        }
    }
}

impl FromStr for ActorDriverScripts {
    type Err = ActorDriverScriptsParseError;

    fn from_str(script_text: &str) -> Result<Self, Self::Err> {
        Self::parse_text(script_text)
    }
}

impl Default for ActorDriverScripts {
    fn default() -> Self {
        let behavior_script = ActorBehaviorScript::default_script();
        let wave_script =
            ActorWaveScript::parse_text_with_base_behavior(ACTOR_WAVE_SCRIPT, &behavior_script)
                .unwrap_or_else(|error| panic!("embedded actor wave script is invalid: {error}"));
        Self::new(AttractScript::default_title(), behavior_script, wave_script)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptsManifest {
    pub attract_script: AttractScriptManifest,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub wave_script: ActorWaveScriptManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorDriverScriptSection {
    Driver,
    Attract,
    Behavior,
    Wave,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptsParseError {
    pub section: ActorDriverScriptSection,
    pub line: usize,
    pub message: String,
}

impl ActorDriverScriptsParseError {
    fn new(section: ActorDriverScriptSection, line: usize, message: impl Into<String>) -> Self {
        Self {
            section,
            line,
            message: message.into(),
        }
    }

    fn from_attract(error: AttractScriptParseError) -> Self {
        Self::new(ActorDriverScriptSection::Attract, error.line, error.message)
    }

    fn from_behavior(error: ActorBehaviorScriptParseError) -> Self {
        Self::new(
            ActorDriverScriptSection::Behavior,
            error.line,
            error.message,
        )
    }

    fn from_wave(error: ActorWaveScriptParseError) -> Self {
        Self::new(ActorDriverScriptSection::Wave, error.line, error.message)
    }
}

impl fmt::Display for ActorDriverScriptsParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let section = match self.section {
            ActorDriverScriptSection::Driver => "driver",
            ActorDriverScriptSection::Attract => "attract",
            ActorDriverScriptSection::Behavior => "behavior",
            ActorDriverScriptSection::Wave => "wave",
        };
        write!(
            formatter,
            "actor driver {section} script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ActorDriverScriptsParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorDriverScriptBundleSection {
    Attract,
    Behavior,
    Wave,
}

impl ActorDriverScriptBundleSection {
    fn parse(line_number: usize, token: &str) -> Result<Self, ActorDriverScriptsParseError> {
        match normalize_script_token(token).as_str() {
            "attract" | "attract_script" => Ok(Self::Attract),
            "behavior" | "behaviour" | "behavior_script" | "behaviour_script" => Ok(Self::Behavior),
            "wave" | "waves" | "wave_script" => Ok(Self::Wave),
            _ => Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Driver,
                line_number,
                format!("unknown driver script section `{token}`"),
            )),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ParsedActorDriverScriptSections {
    attract: String,
    behavior: String,
    wave: String,
    saw_attract: bool,
    saw_behavior: bool,
    saw_wave: bool,
}

impl ParsedActorDriverScriptSections {
    fn parse(script_text: &str) -> Result<Self, ActorDriverScriptsParseError> {
        let mut sections = Self::default();
        let mut current = None;
        for (line_index, input_line) in script_text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = input_line
                .split_once('#')
                .map_or(input_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            if let Some(section) = parse_actor_driver_script_section_header(line_number, line)? {
                current = Some(section);
                sections.mark_seen(section);
                continue;
            }
            let Some(section) = current else {
                return Err(ActorDriverScriptsParseError::new(
                    ActorDriverScriptSection::Driver,
                    line_number,
                    "driver script line must appear inside [attract], [behavior], or [wave]",
                ));
            };
            sections.push_line(section, line_number, line);
        }
        sections.require_sections()?;
        Ok(sections)
    }

    fn mark_seen(&mut self, section: ActorDriverScriptBundleSection) {
        match section {
            ActorDriverScriptBundleSection::Attract => self.saw_attract = true,
            ActorDriverScriptBundleSection::Behavior => self.saw_behavior = true,
            ActorDriverScriptBundleSection::Wave => self.saw_wave = true,
        }
    }

    fn push_line(
        &mut self,
        section: ActorDriverScriptBundleSection,
        line_number: usize,
        line: &str,
    ) {
        let target = match section {
            ActorDriverScriptBundleSection::Attract => &mut self.attract,
            ActorDriverScriptBundleSection::Behavior => &mut self.behavior,
            ActorDriverScriptBundleSection::Wave => &mut self.wave,
        };
        append_script_line_with_original_number(target, line_number, line);
    }

    fn require_sections(&self) -> Result<(), ActorDriverScriptsParseError> {
        if !self.saw_attract {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Attract,
                0,
                "driver script needs an [attract] section",
            ));
        }
        if !self.saw_behavior {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Behavior,
                0,
                "driver script needs a [behavior] section",
            ));
        }
        if !self.saw_wave {
            return Err(ActorDriverScriptsParseError::new(
                ActorDriverScriptSection::Wave,
                0,
                "driver script needs a [wave] section",
            ));
        }
        Ok(())
    }
}

fn parse_actor_driver_script_section_header(
    line_number: usize,
    line: &str,
) -> Result<Option<ActorDriverScriptBundleSection>, ActorDriverScriptsParseError> {
    let Some(name) = line
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return Ok(None);
    };
    Ok(Some(ActorDriverScriptBundleSection::parse(
        line_number,
        name.trim(),
    )?))
}

fn append_script_line_with_original_number(target: &mut String, line_number: usize, line: &str) {
    let current_line_count = target.lines().count();
    for _ in current_line_count..line_number.saturating_sub(1) {
        target.push('\n');
    }
    target.push_str(line);
    target.push('\n');
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorDriverScriptManifest {
    pub step: u64,
    pub phase: Phase,
    pub wave: u16,
    pub attract_script: AttractScriptManifest,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub wave_script: ActorWaveScriptManifest,
    pub current_wave_profile: ActorWaveProfileManifest,
}

fn actor_gameplay_events_for_report(report: &StepReport) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for command in &report.commands {
        match command {
            GameCommand::Credit => push_unique_game_event(&mut events, GameEvent::CreditAdded),
            GameCommand::StartOnePlayer | GameCommand::StartTwoPlayer => {
                if report.phase == Phase::Playing {
                    push_unique_game_event(&mut events, GameEvent::GameStarted);
                    if report.player_start.is_none() {
                        push_unique_game_event(&mut events, GameEvent::WaveStarted);
                    }
                }
            }
            GameCommand::SmartBomb { .. } => {
                push_unique_game_event(&mut events, GameEvent::SmartBombPressed)
            }
            GameCommand::Hyperspace => {
                push_unique_game_event(&mut events, GameEvent::HyperspacePressed)
            }
            GameCommand::PlayerKilled => {
                push_unique_game_event(&mut events, GameEvent::PlayerDestroyed)
            }
            GameCommand::WaveCleared { .. } => {
                push_unique_game_event(&mut events, GameEvent::WaveCleared);
            }
            GameCommand::AdvanceWave { .. } => {
                push_unique_game_event(&mut events, GameEvent::WaveStarted);
            }
            GameCommand::Spawn(_)
            | GameCommand::Destroy(_)
            | GameCommand::SetWorldScrollLeft(_)
            | GameCommand::AttachHuman { .. }
            | GameCommand::HumanLost(_)
            | GameCommand::AddScore(_)
            | GameCommand::PlaySound(_) => {}
        }
    }
    if report.sounds.contains(&SoundCue::GameOver) {
        push_unique_game_event(&mut events, GameEvent::GameOver);
    }
    if report.phase == Phase::HighScoreEntry {
        push_unique_game_event(&mut events, GameEvent::HighScoreEntryStarted);
    }
    if report.high_score_initial_accepted {
        push_unique_game_event(&mut events, GameEvent::HighScoreInitialAccepted);
    }
    if report.high_score_submitted {
        push_unique_game_event(&mut events, GameEvent::HighScoreSubmitted);
    }
    if report.bonus_awarded {
        push_unique_game_event(&mut events, GameEvent::BonusAwarded);
    }
    events
}

fn push_unique_game_event(events: &mut Vec<GameEvent>, event: GameEvent) {
    if !events.contains(&event) {
        events.push(event);
    }
}
