#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveProfile {
    pub wave: u16,
    pub arcade_wave: Option<ArcadeWaveProfile>,
    pub behavior_script: ActorBehaviorScript,
    pub lander_spawns: Vec<ActorLanderSpawn>,
    pub bomber_spawns: Vec<ActorBomberSpawn>,
    pub pod_spawns: Vec<ActorPodSpawn>,
    pub mutant_spawns: Vec<ActorMutantSpawn>,
    pub swarmer_spawns: Vec<ActorSwarmerSpawn>,
    pub baiter_spawns: Vec<ActorBaiterSpawn>,
    pub human_spawns: Vec<ActorHumanSpawn>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

impl ActorWaveProfile {
    pub fn new(wave: u16, behavior_script: ActorBehaviorScript, lander_spawns: Vec<Point>) -> Self {
        Self::with_lander_spawns(
            wave,
            behavior_script,
            lander_spawns
                .into_iter()
                .map(ActorLanderSpawn::new)
                .collect(),
        )
    }

    pub fn with_lander_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
    ) -> Self {
        Self::with_spawns(
            wave,
            behavior_script,
            lander_spawns,
            ACTOR_FIRST_WAVE_HUMAN_SPAWNS.to_vec(),
        )
    }

    pub fn with_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self::with_family_spawns(
            wave,
            behavior_script,
            lander_spawns,
            Vec::new(),
            Vec::new(),
            human_spawns,
        )
    }

    pub fn with_family_spawns(
        wave: u16,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<ActorLanderSpawn>,
        bomber_spawns: Vec<ActorBomberSpawn>,
        pod_spawns: Vec<ActorPodSpawn>,
        human_spawns: Vec<ActorHumanSpawn>,
    ) -> Self {
        Self {
            wave: wave.max(1),
            arcade_wave: None,
            behavior_script,
            lander_spawns,
            bomber_spawns,
            pod_spawns,
            mutant_spawns: Vec::new(),
            swarmer_spawns: Vec::new(),
            baiter_spawns: Vec::new(),
            human_spawns,
            enemy_reserve: EnemyReserveSnapshot::default(),
            spawn_behavior_profiles: Vec::new(),
        }
    }

    pub fn with_mutant_spawns(mut self, mutant_spawns: Vec<ActorMutantSpawn>) -> Self {
        self.mutant_spawns = mutant_spawns;
        self
    }

    pub fn with_arcade_wave(mut self, arcade_wave: ArcadeWaveProfile) -> Self {
        self.arcade_wave = Some(arcade_wave);
        self
    }

    fn with_optional_arcade_wave(mut self, arcade_wave: Option<ArcadeWaveProfile>) -> Self {
        self.arcade_wave = arcade_wave;
        self
    }

    pub fn with_swarmer_spawns(mut self, swarmer_spawns: Vec<ActorSwarmerSpawn>) -> Self {
        self.swarmer_spawns = swarmer_spawns;
        self
    }

    pub fn with_baiter_spawns(mut self, baiter_spawns: Vec<ActorBaiterSpawn>) -> Self {
        self.baiter_spawns = baiter_spawns;
        self
    }

    pub fn with_enemy_reserve(mut self, enemy_reserve: EnemyReserveSnapshot) -> Self {
        self.enemy_reserve = enemy_reserve;
        self
    }

    pub fn with_spawn_behavior_profiles(
        mut self,
        spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
    ) -> Self {
        self.spawn_behavior_profiles = spawn_behavior_profiles;
        self
    }

    pub fn spawn_behavior_profile(
        &self,
        kind: ActorKind,
        spawn_index: usize,
    ) -> Option<ActorBehaviorProfile> {
        self.spawn_behavior_profiles
            .iter()
            .find(|entry| entry.kind == kind && entry.spawn_index == spawn_index)
            .map(|entry| entry.profile)
    }

    pub fn lander_spawn_points(&self) -> Vec<Point> {
        self.lander_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn human_spawn_points(&self) -> Vec<Point> {
        self.human_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn bomber_spawn_points(&self) -> Vec<Point> {
        self.bomber_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn pod_spawn_points(&self) -> Vec<Point> {
        self.pod_spawns.iter().map(|spawn| spawn.position).collect()
    }

    pub fn mutant_spawn_points(&self) -> Vec<Point> {
        self.mutant_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn swarmer_spawn_points(&self) -> Vec<Point> {
        self.swarmer_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn baiter_spawn_points(&self) -> Vec<Point> {
        self.baiter_spawns
            .iter()
            .map(|spawn| spawn.position)
            .collect()
    }

    pub fn manifest(&self) -> ActorWaveProfileManifest {
        ActorWaveProfileManifest {
            wave: self.wave,
            arcade_wave: self.arcade_wave,
            behavior_script: self.behavior_script.manifest(),
            lander_spawns: self.lander_spawns.clone(),
            bomber_spawns: self.bomber_spawns.clone(),
            pod_spawns: self.pod_spawns.clone(),
            mutant_spawns: self.mutant_spawns.clone(),
            swarmer_spawns: self.swarmer_spawns.clone(),
            baiter_spawns: self.baiter_spawns.clone(),
            human_spawns: self.human_spawns.clone(),
            enemy_reserve: self.enemy_reserve,
            spawn_behavior_profiles: self.spawn_behavior_profiles.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorProfile {
    pub kind: ActorKind,
    pub spawn_index: usize,
    pub profile: ActorBehaviorProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveProfileManifest {
    pub wave: u16,
    pub arcade_wave: Option<ArcadeWaveProfile>,
    pub behavior_script: ActorBehaviorScriptManifest,
    pub lander_spawns: Vec<ActorLanderSpawn>,
    pub bomber_spawns: Vec<ActorBomberSpawn>,
    pub pod_spawns: Vec<ActorPodSpawn>,
    pub mutant_spawns: Vec<ActorMutantSpawn>,
    pub swarmer_spawns: Vec<ActorSwarmerSpawn>,
    pub baiter_spawns: Vec<ActorBaiterSpawn>,
    pub human_spawns: Vec<ActorHumanSpawn>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScript {
    name: String,
    waves: Vec<ActorWaveProfile>,
    behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
    spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
}

impl ActorWaveScript {
    pub fn new(name: impl Into<String>, waves: Vec<ActorWaveProfile>) -> Self {
        Self::with_presets(name, waves, Vec::new(), Vec::new())
    }

    fn with_presets(
        name: impl Into<String>,
        mut waves: Vec<ActorWaveProfile>,
        behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
        spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
    ) -> Self {
        if waves.is_empty() {
            waves.push(Self::arcade_backed_profile(1));
        }
        waves.sort_by_key(|profile| profile.wave);
        Self {
            name: name.into(),
            waves,
            behavior_presets,
            spawn_behavior_presets,
        }
    }

    pub fn single_wave(
        name: impl Into<String>,
        behavior_script: ActorBehaviorScript,
        lander_spawns: Vec<Point>,
    ) -> Self {
        Self::new(
            name,
            vec![ActorWaveProfile::new(1, behavior_script, lander_spawns)],
        )
    }

    pub fn default_progression() -> Self {
        Self::parse_text(ACTOR_WAVE_SCRIPT)
            .unwrap_or_else(|error| panic!("embedded actor wave script is invalid: {error}"))
    }

    pub fn arcade_table_progression() -> Self {
        let waves = (1..=ACTOR_DATA_BACKED_WAVES)
            .map(Self::arcade_backed_profile)
            .collect::<Vec<_>>();
        Self::new("actor-arcade-wave-table", waves)
    }

    fn arcade_backed_profile(wave: u16) -> ActorWaveProfile {
        let arcade_profile = ArcadeWaveProfile::for_wave(wave);
        Self::arcade_backed_profile_from_profile(wave, arcade_profile)
    }

    fn arcade_backed_profile_from_profile(
        wave: u16,
        arcade_profile: ArcadeWaveProfile,
    ) -> ActorWaveProfile {
        Self::arcade_backed_profile_from_profile_with_behavior(
            wave,
            arcade_profile,
            &ActorBehaviorScript::from_arcade_profile(),
        )
    }

    fn arcade_backed_profile_from_profile_with_behavior(
        wave: u16,
        arcade_profile: ArcadeWaveProfile,
        base_behavior: &ActorBehaviorScript,
    ) -> ActorWaveProfile {
        let human_spawns = arcade_profile.human_spawns(wave);
        ActorWaveProfile::with_family_spawns(
            wave,
            base_behavior
                .clone()
                .with_kind_behavior(ActorKind::Lander, arcade_profile.lander_behavior())
                .with_kind_behavior(
                    ActorKind::Bomber,
                    ActorBehaviorProfile {
                        bomber_drift_speed: speed_pixels_from_arcade_velocity(
                            arcade_profile.bomber_x_velocity,
                        ),
                        ..ActorBehaviorProfile::default()
                    },
                ),
            arcade_profile.lander_spawns(wave, &human_spawns),
            arcade_profile.bomber_spawns(),
            arcade_profile.pod_spawns(),
            human_spawns,
        )
        .with_arcade_wave(arcade_profile)
        .with_mutant_spawns(arcade_profile.mutant_spawns())
        .with_swarmer_spawns(arcade_profile.swarmer_spawns())
        .with_enemy_reserve(arcade_profile.enemy_reserve_after_active_batch())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn profile_for_wave(&self, wave: u16) -> &ActorWaveProfile {
        self.waves
            .iter()
            .rev()
            .find(|profile| wave >= profile.wave)
            .unwrap_or(&self.waves[0])
    }

    pub fn manifest(&self) -> ActorWaveScriptManifest {
        ActorWaveScriptManifest {
            name: self.name.clone(),
            behavior_presets: self.behavior_presets.clone(),
            spawn_behavior_presets: self.spawn_behavior_presets.clone(),
            waves: self.waves.iter().map(ActorWaveProfile::manifest).collect(),
        }
    }
}

impl FromStr for ActorWaveScript {
    type Err = ActorWaveScriptParseError;

    fn from_str(script_text: &str) -> Result<Self, Self::Err> {
        let base_behavior = ActorBehaviorScript::from_arcade_profile();
        Self::parse_text_with_base_behavior(script_text, &base_behavior)
    }
}

impl ActorWaveScript {
    pub fn parse_text(script_text: &str) -> Result<Self, ActorWaveScriptParseError> {
        script_text.parse()
    }

    pub fn parse_text_with_base_behavior(
        script_text: &str,
        base_behavior: &ActorBehaviorScript,
    ) -> Result<Self, ActorWaveScriptParseError> {
        let mut parser = ParsedActorWaveScript::with_base_behavior(base_behavior.clone());
        for (line_index, input_line) in script_text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = input_line
                .split_once('#')
                .map_or(input_line, |(before_comment, _)| before_comment)
                .trim();
            if line.is_empty() {
                continue;
            }
            parser.parse_line(line_number, line)?;
        }
        parser.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScriptParseError {
    pub line: usize,
    pub message: String,
}

impl ActorWaveScriptParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ActorWaveScriptParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "actor wave script line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ActorWaveScriptParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedActorWaveScript {
    name: String,
    waves: Vec<ParsedActorWaveProfile>,
    behavior_presets: BTreeMap<String, Vec<ParsedBehaviorPresetUpdate>>,
    spawn_behavior_presets: BTreeMap<String, Vec<ParsedSpawnBehaviorPresetUpdate>>,
    base_behavior: ActorBehaviorScript,
}

impl ParsedActorWaveScript {
    fn with_base_behavior(base_behavior: ActorBehaviorScript) -> Self {
        Self {
            name: "parsed-wave-script".to_string(),
            waves: Vec::new(),
            behavior_presets: BTreeMap::new(),
            spawn_behavior_presets: BTreeMap::new(),
            base_behavior,
        }
    }

    fn parse_line(
        &mut self,
        line_number: usize,
        line: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut parts = line.split_whitespace();
        let action = parts
            .next()
            .ok_or_else(|| ActorWaveScriptParseError::new(line_number, "missing action"))?;
        match normalize_script_token(action).as_str() {
            "name" => {
                let name = parts.collect::<Vec<_>>().join(" ");
                if name.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "name action needs a value",
                    ));
                }
                self.name = name;
                Ok(())
            }
            "behavior_preset" | "behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        format!("behavior preset `{name}` needs a profile update"),
                    ));
                }
                self.define_behavior_preset(line_number, name, behavior_line)
            }
            "spawn_behavior_preset" | "spawn_behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                self.define_spawn_behavior_preset(line_number, name, field, &values)
            }
            "wave" => {
                let wave = parse_wave_u16(line_number, parts.next(), "wave")?;
                reject_extra_wave_fields(line_number, parts)?;
                self.push_profile(
                    line_number,
                    ParsedActorWaveProfile::new_with_behavior(wave, self.base_behavior.clone()),
                )
            }
            "arcade_wave" => {
                let wave = parse_wave_u16(line_number, parts.next(), "wave")?;
                let mut arcade_profile = ArcadeWaveProfile::for_wave(wave);
                parse_arcade_wave_profile_updates(line_number, &mut arcade_profile, parts)?;
                self.push_profile(
                    line_number,
                    ParsedActorWaveProfile::arcade_backed_from_profile_with_behavior(
                        wave,
                        arcade_profile,
                        &self.base_behavior,
                    ),
                )
            }
            "arcade_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let arcade_profile_update_tokens = parts.collect::<Vec<_>>();
                if last < first {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        format!("arcade wave range `{first}..{last}` is invalid"),
                    ));
                }
                for wave in first..=last {
                    let mut arcade_profile = ArcadeWaveProfile::for_wave(wave);
                    parse_arcade_wave_profile_updates(
                        line_number,
                        &mut arcade_profile,
                        arcade_profile_update_tokens.iter().copied(),
                    )?;
                    self.push_profile(
                        line_number,
                        ParsedActorWaveProfile::arcade_backed_from_profile_with_behavior(
                            wave,
                            arcade_profile,
                            &self.base_behavior,
                        ),
                    )?;
                }
                Ok(())
            }
            "behavior" => {
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "behavior action needs a profile update",
                    ));
                }
                let profile = self.current_profile_mut(line_number)?;
                parse_behavior_script_line(
                    line_number,
                    &behavior_line,
                    &mut profile.behavior_script,
                )
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
            }
            "use_behavior"
            | "use_behaviour"
            | "apply_behavior_preset"
            | "apply_behaviour_preset" => {
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_behavior_preset_to_current_wave(line_number, &name)
            }
            "behavior_waves" | "behaviour_waves" | "behavior_range" | "behaviour_range" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let behavior_line = parts.collect::<Vec<_>>().join(" ");
                if behavior_line.is_empty() {
                    return Err(ActorWaveScriptParseError::new(
                        line_number,
                        "behavior range action needs a profile update",
                    ));
                }
                self.apply_behavior_to_wave_range(line_number, first, last, &behavior_line)
            }
            "use_behavior_waves"
            | "use_behaviour_waves"
            | "behavior_preset_waves"
            | "behaviour_preset_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_behavior_preset_to_wave_range(line_number, first, last, &name)
            }
            "spawn_behavior" | "spawn_behaviour" => {
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                let profile = self
                    .current_profile_mut(line_number)?
                    .spawn_behavior_profile_mut(kind, spawn_index);
                apply_behavior_profile_field(line_number, profile, field, &values)
                    .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
            }
            "use_spawn_behavior"
            | "use_spawn_behaviour"
            | "apply_spawn_behavior_preset"
            | "apply_spawn_behaviour_preset" => {
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_spawn_behavior_preset_to_current_wave(
                    line_number,
                    kind,
                    spawn_index,
                    &name,
                )
            }
            "spawn_behavior_waves"
            | "spawn_behaviour_waves"
            | "spawn_behavior_range"
            | "spawn_behaviour_range" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let field = parts.next().ok_or_else(|| {
                    ActorWaveScriptParseError::new(line_number, "missing behavior field")
                })?;
                let values = parts.collect::<Vec<_>>();
                self.apply_spawn_behavior_to_wave_range(
                    line_number,
                    first,
                    last,
                    ParsedWaveSpawnBehaviorUpdate {
                        kind,
                        spawn_index,
                        field,
                        values: &values,
                    },
                )
            }
            "use_spawn_behavior_waves"
            | "use_spawn_behaviour_waves"
            | "spawn_behavior_preset_waves"
            | "spawn_behaviour_preset_waves" => {
                let first = parse_wave_u16(line_number, parts.next(), "first wave")?.max(1);
                let last = parse_wave_u16(line_number, parts.next(), "last wave")?.max(1);
                let kind = parse_wave_actor_kind(line_number, parts.next())?;
                let spawn_index = parse_wave_usize(line_number, parts.next(), "spawn index")?;
                let name = parse_wave_behavior_preset_name(line_number, parts.next())?;
                reject_extra_wave_fields(line_number, parts)?;
                self.apply_spawn_behavior_preset_to_wave_range(
                    line_number,
                    first,
                    last,
                    kind,
                    spawn_index,
                    &name,
                )
            }
            "lander" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .lander_spawns
                    .push(ActorLanderSpawn::new(position));
                Ok(())
            }
            "bomber" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .bomber_spawns
                    .push(ActorBomberSpawn::new(position));
                Ok(())
            }
            "pod" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .pod_spawns
                    .push(ActorPodSpawn::new(position));
                Ok(())
            }
            "mutant" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .mutant_spawns
                    .push(ActorMutantSpawn::new(position));
                Ok(())
            }
            "swarmer" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .swarmer_spawns
                    .push(ActorSwarmerSpawn::new(position));
                Ok(())
            }
            "baiter" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .baiter_spawns
                    .push(ActorBaiterSpawn::new(position));
                Ok(())
            }
            "human" => {
                let position = parse_wave_point(line_number, &mut parts)?;
                let mode = parse_wave_human_mode(line_number, parts)?;
                self.current_profile_mut(line_number)?
                    .human_spawns
                    .push(ActorHumanSpawn::new(position, mode));
                Ok(())
            }
            "enemy_reserve" | "reserve" => {
                let landers = parse_wave_u8(line_number, parts.next(), "reserve landers")?;
                let bombers = parse_wave_u8(line_number, parts.next(), "reserve bombers")?;
                let pods = parse_wave_u8(line_number, parts.next(), "reserve pods")?;
                let swarmers = parts
                    .next()
                    .map(|field| parse_wave_u8(line_number, Some(field), "reserve swarmers"))
                    .transpose()?
                    .unwrap_or(0);
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?.enemy_reserve = EnemyReserveSnapshot {
                    landers,
                    bombers,
                    pods,
                    swarmers,
                    ..EnemyReserveSnapshot::default()
                };
                Ok(())
            }
            "enemy_reserve_full" | "reserve_full" => {
                let landers = parse_wave_u8(line_number, parts.next(), "reserve landers")?;
                let bombers = parse_wave_u8(line_number, parts.next(), "reserve bombers")?;
                let pods = parse_wave_u8(line_number, parts.next(), "reserve pods")?;
                let mutants = parse_wave_u8(line_number, parts.next(), "reserve mutants")?;
                let swarmers = parse_wave_u8(line_number, parts.next(), "reserve swarmers")?;
                reject_extra_wave_fields(line_number, parts)?;
                self.current_profile_mut(line_number)?.enemy_reserve = EnemyReserveSnapshot {
                    landers,
                    bombers,
                    pods,
                    mutants,
                    swarmers,
                };
                Ok(())
            }
            _ => Err(ActorWaveScriptParseError::new(
                line_number,
                format!("unknown wave action `{action}`"),
            )),
        }
    }

    fn define_spawn_behavior_preset(
        &mut self,
        line_number: usize,
        name: String,
        field: &str,
        values: &[&str],
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut validation = ActorBehaviorProfile::default();
        apply_behavior_profile_field(line_number, &mut validation, field, values)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        self.spawn_behavior_presets.entry(name).or_default().push(
            ParsedSpawnBehaviorPresetUpdate {
                line_number,
                field: field.to_string(),
                values: values.iter().map(|value| (*value).to_string()).collect(),
            },
        );
        Ok(())
    }

    fn define_behavior_preset(
        &mut self,
        line_number: usize,
        name: String,
        behavior_line: String,
    ) -> Result<(), ActorWaveScriptParseError> {
        let mut validation = ActorBehaviorScript::from_arcade_profile();
        parse_behavior_script_line(line_number, &behavior_line, &mut validation)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        self.behavior_presets
            .entry(name)
            .or_default()
            .push(ParsedBehaviorPresetUpdate {
                line_number,
                line: behavior_line,
            });
        Ok(())
    }

    fn behavior_preset_updates(
        &self,
        line_number: usize,
        name: &str,
    ) -> Result<Vec<ParsedBehaviorPresetUpdate>, ActorWaveScriptParseError> {
        self.behavior_presets.get(name).cloned().ok_or_else(|| {
            ActorWaveScriptParseError::new(line_number, format!("unknown behavior preset `{name}`"))
        })
    }

    fn spawn_behavior_preset_updates(
        &self,
        line_number: usize,
        name: &str,
    ) -> Result<Vec<ParsedSpawnBehaviorPresetUpdate>, ActorWaveScriptParseError> {
        self.spawn_behavior_presets
            .get(name)
            .cloned()
            .ok_or_else(|| {
                ActorWaveScriptParseError::new(
                    line_number,
                    format!("unknown spawn behavior preset `{name}`"),
                )
            })
    }

    fn apply_behavior_preset_to_current_wave(
        &mut self,
        line_number: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let updates = self.behavior_preset_updates(line_number, name)?;
        let profile = self.current_profile_mut(line_number)?;
        apply_behavior_preset_updates(&updates, &mut profile.behavior_script)
    }

    fn apply_spawn_behavior_preset_to_current_wave(
        &mut self,
        line_number: usize,
        kind: ActorKind,
        spawn_index: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        let updates = self.spawn_behavior_preset_updates(line_number, name)?;
        let profile = self
            .current_profile_mut(line_number)?
            .spawn_behavior_profile_mut(kind, spawn_index);
        apply_spawn_behavior_preset_updates(&updates, profile)
    }

    fn current_profile_mut(
        &mut self,
        line_number: usize,
    ) -> Result<&mut ParsedActorWaveProfile, ActorWaveScriptParseError> {
        self.waves.last_mut().ok_or_else(|| {
            ActorWaveScriptParseError::new(line_number, "wave action must appear before this line")
        })
    }

    fn push_profile(
        &mut self,
        line_number: usize,
        profile: ParsedActorWaveProfile,
    ) -> Result<(), ActorWaveScriptParseError> {
        if self
            .waves
            .iter()
            .any(|candidate| candidate.wave == profile.wave)
        {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("duplicate wave `{}`", profile.wave),
            ));
        }
        self.waves.push(profile);
        Ok(())
    }

    fn profile_for_wave_mut(
        &mut self,
        line_number: usize,
        wave: u16,
    ) -> Result<&mut ParsedActorWaveProfile, ActorWaveScriptParseError> {
        self.waves
            .iter_mut()
            .find(|profile| profile.wave == wave)
            .ok_or_else(|| {
                ActorWaveScriptParseError::new(
                    line_number,
                    format!("wave range references undefined wave `{wave}`"),
                )
            })
    }

    fn validate_wave_range(
        line_number: usize,
        first: u16,
        last: u16,
        label: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        if last < first {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("{label} range `{first}..{last}` is invalid"),
            ));
        }
        Ok(())
    }

    fn apply_behavior_preset_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "behavior preset wave")?;
        let updates = self.behavior_preset_updates(line_number, name)?;
        for wave in first..=last {
            let profile = self.profile_for_wave_mut(line_number, wave)?;
            apply_behavior_preset_updates(&updates, &mut profile.behavior_script)?;
        }
        Ok(())
    }

    fn apply_behavior_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        behavior_line: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "behavior wave")?;
        for wave in first..=last {
            let profile = self.profile_for_wave_mut(line_number, wave)?;
            parse_behavior_script_line(line_number, behavior_line, &mut profile.behavior_script)
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        }
        Ok(())
    }

    fn apply_spawn_behavior_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        update: ParsedWaveSpawnBehaviorUpdate<'_>,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "spawn behavior wave")?;
        for wave in first..=last {
            let profile = self
                .profile_for_wave_mut(line_number, wave)?
                .spawn_behavior_profile_mut(update.kind, update.spawn_index);
            apply_behavior_profile_field(line_number, profile, update.field, update.values)
                .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
        }
        Ok(())
    }

    fn apply_spawn_behavior_preset_to_wave_range(
        &mut self,
        line_number: usize,
        first: u16,
        last: u16,
        kind: ActorKind,
        spawn_index: usize,
        name: &str,
    ) -> Result<(), ActorWaveScriptParseError> {
        Self::validate_wave_range(line_number, first, last, "spawn behavior preset wave")?;
        let updates = self.spawn_behavior_preset_updates(line_number, name)?;
        for wave in first..=last {
            let profile = self
                .profile_for_wave_mut(line_number, wave)?
                .spawn_behavior_profile_mut(kind, spawn_index);
            apply_spawn_behavior_preset_updates(&updates, profile)?;
        }
        Ok(())
    }

    fn finish(self) -> Result<ActorWaveScript, ActorWaveScriptParseError> {
        let Self {
            name,
            waves,
            behavior_presets,
            spawn_behavior_presets,
            base_behavior: _,
        } = self;
        if waves.is_empty() {
            return Err(ActorWaveScriptParseError::new(
                0,
                "wave script needs at least one wave",
            ));
        }
        Ok(ActorWaveScript::with_presets(
            name,
            waves
                .into_iter()
                .map(ParsedActorWaveProfile::finish)
                .collect(),
            behavior_presets
                .into_iter()
                .map(|(name, updates)| ActorWaveBehaviorPresetManifest {
                    name,
                    updates: updates.into_iter().map(|update| update.line).collect(),
                })
                .collect(),
            spawn_behavior_presets
                .into_iter()
                .map(|(name, updates)| ActorWaveSpawnBehaviorPresetManifest {
                    name,
                    updates: updates
                        .into_iter()
                        .map(|update| ActorWaveSpawnBehaviorPresetUpdateManifest {
                            field: update.field,
                            values: update.values,
                        })
                        .collect(),
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedBehaviorPresetUpdate {
    line_number: usize,
    line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSpawnBehaviorPresetUpdate {
    line_number: usize,
    field: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ParsedWaveSpawnBehaviorUpdate<'a> {
    kind: ActorKind,
    spawn_index: usize,
    field: &'a str,
    values: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedActorWaveProfile {
    wave: u16,
    arcade_wave: Option<ArcadeWaveProfile>,
    behavior_script: ActorBehaviorScript,
    lander_spawns: Vec<ActorLanderSpawn>,
    bomber_spawns: Vec<ActorBomberSpawn>,
    pod_spawns: Vec<ActorPodSpawn>,
    mutant_spawns: Vec<ActorMutantSpawn>,
    swarmer_spawns: Vec<ActorSwarmerSpawn>,
    baiter_spawns: Vec<ActorBaiterSpawn>,
    human_spawns: Vec<ActorHumanSpawn>,
    enemy_reserve: EnemyReserveSnapshot,
    spawn_behavior_profiles: Vec<ActorWaveSpawnBehaviorProfile>,
}

impl ParsedActorWaveProfile {
    fn new_with_behavior(wave: u16, behavior_script: ActorBehaviorScript) -> Self {
        Self {
            wave: wave.max(1),
            arcade_wave: None,
            behavior_script,
            lander_spawns: Vec::new(),
            bomber_spawns: Vec::new(),
            pod_spawns: Vec::new(),
            mutant_spawns: Vec::new(),
            swarmer_spawns: Vec::new(),
            baiter_spawns: Vec::new(),
            human_spawns: Vec::new(),
            enemy_reserve: EnemyReserveSnapshot::default(),
            spawn_behavior_profiles: Vec::new(),
        }
    }

    fn arcade_backed_from_profile_with_behavior(
        wave: u16,
        arcade_profile: ArcadeWaveProfile,
        base_behavior: &ActorBehaviorScript,
    ) -> Self {
        let profile = ActorWaveScript::arcade_backed_profile_from_profile_with_behavior(
            wave,
            arcade_profile,
            base_behavior,
        );
        Self {
            wave: profile.wave,
            arcade_wave: profile.arcade_wave,
            behavior_script: profile.behavior_script,
            lander_spawns: profile.lander_spawns,
            bomber_spawns: profile.bomber_spawns,
            pod_spawns: profile.pod_spawns,
            mutant_spawns: profile.mutant_spawns,
            swarmer_spawns: profile.swarmer_spawns,
            baiter_spawns: profile.baiter_spawns,
            human_spawns: profile.human_spawns,
            enemy_reserve: profile.enemy_reserve,
            spawn_behavior_profiles: profile.spawn_behavior_profiles,
        }
    }

    fn spawn_behavior_profile_mut(
        &mut self,
        kind: ActorKind,
        spawn_index: usize,
    ) -> &mut ActorBehaviorProfile {
        let entry_index = self
            .spawn_behavior_profiles
            .iter()
            .position(|entry| entry.kind == kind && entry.spawn_index == spawn_index);
        let entry_index = match entry_index {
            Some(entry_index) => entry_index,
            None => {
                self.spawn_behavior_profiles
                    .push(ActorWaveSpawnBehaviorProfile {
                        kind,
                        spawn_index,
                        profile: self.behavior_script.behavior_for(ActorId::new(0), kind),
                    });
                self.spawn_behavior_profiles.len() - 1
            }
        };
        &mut self.spawn_behavior_profiles[entry_index].profile
    }

    fn finish(self) -> ActorWaveProfile {
        ActorWaveProfile::with_family_spawns(
            self.wave,
            self.behavior_script,
            self.lander_spawns,
            self.bomber_spawns,
            self.pod_spawns,
            self.human_spawns,
        )
        .with_optional_arcade_wave(self.arcade_wave)
        .with_mutant_spawns(self.mutant_spawns)
        .with_swarmer_spawns(self.swarmer_spawns)
        .with_baiter_spawns(self.baiter_spawns)
        .with_enemy_reserve(self.enemy_reserve)
        .with_spawn_behavior_profiles(self.spawn_behavior_profiles)
    }
}

fn apply_behavior_preset_updates(
    updates: &[ParsedBehaviorPresetUpdate],
    behavior_script: &mut ActorBehaviorScript,
) -> Result<(), ActorWaveScriptParseError> {
    for update in updates {
        parse_behavior_script_line(update.line_number, &update.line, behavior_script)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
    }
    Ok(())
}

fn apply_spawn_behavior_preset_updates(
    updates: &[ParsedSpawnBehaviorPresetUpdate],
    behavior_profile: &mut ActorBehaviorProfile,
) -> Result<(), ActorWaveScriptParseError> {
    for update in updates {
        let values = update.values.iter().map(String::as_str).collect::<Vec<_>>();
        apply_behavior_profile_field(update.line_number, behavior_profile, &update.field, &values)
            .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))?;
    }
    Ok(())
}

fn parse_wave_point<'a>(
    line_number: usize,
    parts: &mut impl Iterator<Item = &'a str>,
) -> Result<Point, ActorWaveScriptParseError> {
    let x = parse_wave_i16(line_number, parts.next(), "x")?;
    let y = parse_wave_i16(line_number, parts.next(), "y")?;
    Ok(Point::new(x, y))
}

fn parse_wave_human_mode<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<HumanMode, ActorWaveScriptParseError> {
    let Some(mode) = parts.next() else {
        return Ok(HumanMode::Grounded);
    };
    match normalize_script_token(mode).as_str() {
        "grounded" => {
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::Grounded)
        }
        "falling" => {
            let velocity = parse_wave_i16(line_number, parts.next(), "fall velocity")?;
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::Falling { velocity })
        }
        "carried" | "carried_by" => {
            let actor = ActorId::new(parse_wave_u64(line_number, parts.next(), "carrier actor")?);
            reject_extra_wave_fields(line_number, parts)?;
            Ok(HumanMode::CarriedBy(actor))
        }
        _ => Err(ActorWaveScriptParseError::new(
            line_number,
            format!("unknown human mode `{mode}`"),
        )),
    }
}

fn parse_wave_behavior_preset_name(
    line_number: usize,
    token: Option<&str>,
) -> Result<String, ActorWaveScriptParseError> {
    let token = token.ok_or_else(|| {
        ActorWaveScriptParseError::new(line_number, "missing behavior preset name")
    })?;
    let name = normalize_script_token(token);
    if name.is_empty() {
        return Err(ActorWaveScriptParseError::new(
            line_number,
            "missing behavior preset name",
        ));
    }
    Ok(name)
}

fn parse_arcade_wave_profile_updates<'a>(
    line_number: usize,
    arcade_profile: &mut ArcadeWaveProfile,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), ActorWaveScriptParseError> {
    while let Some(field) = parts.next() {
        let value = parts.next().ok_or_else(|| {
            ActorWaveScriptParseError::new(
                line_number,
                format!("arcade wave field `{field}` needs a value"),
            )
        })?;
        apply_arcade_wave_profile_field(line_number, arcade_profile, field, value)?;
    }
    Ok(())
}

fn apply_arcade_wave_profile_field(
    line_number: usize,
    arcade_profile: &mut ArcadeWaveProfile,
    field: &str,
    value: &str,
) -> Result<(), ActorWaveScriptParseError> {
    match normalize_script_token(field).as_str() {
        "landers" => arcade_profile.landers = parse_wave_u8(line_number, Some(value), field)?,
        "bombers" => arcade_profile.bombers = parse_wave_u8(line_number, Some(value), field)?,
        "pods" => arcade_profile.pods = parse_wave_u8(line_number, Some(value), field)?,
        "mutants" => arcade_profile.mutants = parse_wave_u8(line_number, Some(value), field)?,
        "swarmers" => arcade_profile.swarmers = parse_wave_u8(line_number, Some(value), field)?,
        "wave_size" => arcade_profile.wave_size = parse_wave_u8(line_number, Some(value), field)?,
        "lander_x_velocity" => {
            arcade_profile.lander_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_y_velocity_msb" => {
            arcade_profile.lander_y_velocity_msb = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_y_velocity_lsb" => {
            arcade_profile.lander_y_velocity_lsb = parse_wave_u8(line_number, Some(value), field)?
        }
        "bomber_x_velocity" => {
            arcade_profile.bomber_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "swarmer_x_velocity" => {
            arcade_profile.swarmer_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "swarmer_shot_time" => {
            arcade_profile.swarmer_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "swarmer_acceleration_mask" => {
            arcade_profile.swarmer_acceleration_mask =
                parse_wave_u8(line_number, Some(value), field)?
        }
        "baiter_time" | "baiter_delay" => {
            arcade_profile.baiter_delay = parse_wave_u32(line_number, Some(value), field)?
        }
        "baiter_shot_time" => {
            arcade_profile.baiter_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "baiter_seek_probability" => {
            arcade_profile.baiter_seek_probability = parse_wave_u8(line_number, Some(value), field)?
        }
        "lander_shot_time" => {
            arcade_profile.lander_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        "mutant_random_y" => {
            arcade_profile.mutant_random_y = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_y_velocity_msb" => {
            arcade_profile.mutant_y_velocity_msb = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_y_velocity_lsb" => {
            arcade_profile.mutant_y_velocity_lsb = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_x_velocity" => {
            arcade_profile.mutant_x_velocity = parse_wave_u8(line_number, Some(value), field)?
        }
        "mutant_shot_time" => {
            arcade_profile.mutant_shot_time = parse_wave_u32(line_number, Some(value), field)?
        }
        _ => {
            return Err(ActorWaveScriptParseError::new(
                line_number,
                format!("unknown arcade wave field `{field}`"),
            ));
        }
    }
    Ok(())
}

fn parse_wave_actor_kind(
    line_number: usize,
    token: Option<&str>,
) -> Result<ActorKind, ActorWaveScriptParseError> {
    let token =
        token.ok_or_else(|| ActorWaveScriptParseError::new(line_number, "missing actor kind"))?;
    parse_behavior_actor_kind(line_number, token)
        .map_err(|error| ActorWaveScriptParseError::new(error.line, error.message))
}

fn parse_wave_usize(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<usize, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    usize::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u8(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u8, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u8::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u16, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u16::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u32(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u32, ActorWaveScriptParseError> {
    let value = parse_wave_u64(line_number, token, field)?;
    u32::try_from(value).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_i16(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<i16, ActorWaveScriptParseError> {
    let token = token
        .ok_or_else(|| ActorWaveScriptParseError::new(line_number, format!("missing {field}")))?;
    i16::try_from(parse_wave_i64_value(line_number, token, field)?).map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{token}` is invalid: {error}"),
        )
    })
}

fn parse_wave_u64(
    line_number: usize,
    token: Option<&str>,
    field: &str,
) -> Result<u64, ActorWaveScriptParseError> {
    let token = token
        .ok_or_else(|| ActorWaveScriptParseError::new(line_number, format!("missing {field}")))?;
    parse_wave_u64_value(line_number, token, field)
}

fn parse_wave_u64_value(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<u64, ActorWaveScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(|error| {
            ActorWaveScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<u64>().map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn parse_wave_i64_value(
    line_number: usize,
    value: &str,
    field: &str,
) -> Result<i64, ActorWaveScriptParseError> {
    if let Some(hex) = value
        .strip_prefix("-0x")
        .or_else(|| value.strip_prefix("-0X"))
    {
        return i64::from_str_radix(hex, 16)
            .map(|value| -value)
            .map_err(|error| {
                ActorWaveScriptParseError::new(
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
            ActorWaveScriptParseError::new(
                line_number,
                format!("{field} `{value}` is invalid: {error}"),
            )
        });
    }
    value.parse::<i64>().map_err(|error| {
        ActorWaveScriptParseError::new(
            line_number,
            format!("{field} `{value}` is invalid: {error}"),
        )
    })
}

fn reject_extra_wave_fields<'a>(
    line_number: usize,
    mut parts: impl Iterator<Item = &'a str>,
) -> Result<(), ActorWaveScriptParseError> {
    if let Some(extra) = parts.next() {
        Err(ActorWaveScriptParseError::new(
            line_number,
            format!("unexpected extra field `{extra}`"),
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveScriptManifest {
    pub name: String,
    pub behavior_presets: Vec<ActorWaveBehaviorPresetManifest>,
    pub spawn_behavior_presets: Vec<ActorWaveSpawnBehaviorPresetManifest>,
    pub waves: Vec<ActorWaveProfileManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveBehaviorPresetManifest {
    pub name: String,
    pub updates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorPresetManifest {
    pub name: String,
    pub updates: Vec<ActorWaveSpawnBehaviorPresetUpdateManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWaveSpawnBehaviorPresetUpdateManifest {
    pub field: String,
    pub values: Vec<String>,
}

impl Default for ActorWaveScript {
    fn default() -> Self {
        Self::default_progression()
    }
}
