#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
}

impl EnemyKind {
    const fn object_category(self) -> ObjectEvidenceCategory {
        match self {
            Self::Lander => ObjectEvidenceCategory::Lander,
            Self::Mutant => ObjectEvidenceCategory::Mutant,
            Self::Bomber => ObjectEvidenceCategory::Bomber,
            Self::Pod => ObjectEvidenceCategory::Pod,
            Self::Swarmer => ObjectEvidenceCategory::Swarmer,
            Self::Baiter => ObjectEvidenceCategory::Baiter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemySnapshot {
    pub kind: EnemyKind,
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub lander_runtime: Option<LanderRuntimeSnapshot>,
    pub mutant_runtime: Option<MutantRuntimeSnapshot>,
    pub bomber_runtime: Option<BomberRuntimeSnapshot>,
    pub swarmer_runtime: Option<SwarmerRuntimeSnapshot>,
    pub baiter_runtime: Option<BaiterRuntimeSnapshot>,
    pub pod_runtime: Option<PodRuntimeSnapshot>,
}

impl EnemySnapshot {
    pub const fn new(kind: EnemyKind, position: ScreenPosition, velocity: ScreenVelocity) -> Self {
        Self {
            kind,
            position,
            velocity,
            lander_runtime: None,
            mutant_runtime: None,
            bomber_runtime: None,
            swarmer_runtime: None,
            baiter_runtime: None,
            pod_runtime: None,
        }
    }

    fn arcade_picture_descriptor(self) -> ObjectPictureDescriptor {
        match self.kind {
            EnemyKind::Lander => lander_picture_descriptor(
                self.lander_runtime
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Mutant => MUTANT_PICTURE_DESCRIPTOR,
            EnemyKind::Bomber => bomber_picture_descriptor(
                self.bomber_runtime
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Pod => POD_PICTURE_DESCRIPTOR,
            EnemyKind::Swarmer => SWARMER_PICTURE_DESCRIPTOR,
            EnemyKind::Baiter => baiter_picture_descriptor(
                self.baiter_runtime
                    .map(|source| source.picture_frame)
                    .unwrap_or_default(),
            ),
        }
    }

    fn arcade_world_position(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .lander_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
            EnemyKind::Mutant => self
                .mutant_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
            EnemyKind::Bomber => self
                .bomber_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
            EnemyKind::Pod => self
                .pod_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
            EnemyKind::Swarmer => self
                .swarmer_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
            EnemyKind::Baiter => self
                .baiter_runtime
                .map(|source| {
                    arcade_world_position(self.position, source.x_fraction, source.y_fraction)
                })
                .unwrap_or_else(|| arcade_world_position(self.position, 0, 0)),
        }
    }

    fn arcade_velocity_words(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .lander_runtime
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Mutant => self
                .mutant_runtime
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Bomber => self
                .bomber_runtime
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Pod => self
                .pod_runtime
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Swarmer => self
                .swarmer_runtime
                .map(|source| (source.x_velocity, source.y_velocity))
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Baiter => self
                .baiter_runtime
                .map(|source| {
                    (
                        baiter_screen_x_velocity(source.x_velocity),
                        source.y_velocity,
                    )
                })
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanderRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
    pub target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutantRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub hop_rng: ArcadeRngSnapshot,
    pub render_x_correction: u16,
    pub target6_first_shot_deferred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BomberRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub picture_frame: u8,
    pub cruise_altitude: u8,
    pub sleep_ticks: u8,
    pub slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmerRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub acceleration: u8,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub horizontal_seek_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaiterRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
    pub shot_timer: u8,
    pub sleep_ticks: u8,
    pub picture_frame: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PodRuntimeSnapshot {
    pub x_fraction: u8,
    pub y_fraction: u8,
    pub x_velocity: u16,
    pub y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcadeRngSnapshot {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

impl Default for ArcadeRngSnapshot {
    fn default() -> Self {
        Self {
            seed: 0,
            hseed: 0xA5,
            lseed: 0x5A,
        }
    }
}

impl ArcadeRngSnapshot {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyProjectileKind {
    Fireball,
    BomberBombShell,
}

impl EnemyProjectileKind {
    pub const fn output_routine_address(self) -> u16 {
        match self {
            Self::Fireball => FIREBALL_ARCADE_ROUTINE_ADDRESS,
            Self::BomberBombShell => ENEMY_BOMB_ARCADE_ROUTINE_ADDRESS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyProjectileSnapshot {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub kind: EnemyProjectileKind,
    pub x_subpixel: u8,
    pub y_subpixel: u8,
    pub x_velocity_word: u16,
    pub y_velocity_word: u16,
    pub lifetime_ticks: u8,
}

impl EnemyProjectileSnapshot {
    pub const fn output_routine_address(self) -> u16 {
        self.kind.output_routine_address()
    }

    const fn bomb_picture_label(self) -> &'static str {
        ENEMY_BOMB_PICTURE_LABEL
    }

    fn arcade_world_position(self) -> (u16, u16) {
        arcade_world_position(
            self.position,
            self.x_subpixel,
            self.y_subpixel,
        )
    }

    const fn arcade_velocity_words(self) -> (u16, u16) {
        (self.x_velocity_word, self.y_velocity_word)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnemyReserveSnapshot {
    pub landers: u8,
    pub bombers: u8,
    pub pods: u8,
    pub mutants: u8,
    pub swarmers: u8,
}

impl EnemyReserveSnapshot {
    fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
    }

    fn family_counts(self) -> [(EnemyKind, u8); 5] {
        [
            (EnemyKind::Lander, self.landers),
            (EnemyKind::Bomber, self.bombers),
            (EnemyKind::Pod, self.pods),
            (EnemyKind::Mutant, self.mutants),
            (EnemyKind::Swarmer, self.swarmers),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanSnapshot {
    pub position: ScreenPosition,
    pub carried: bool,
    pub carried_by_player: bool,
    pub x_subpixel: u8,
    pub picture_frame: u8,
    pub fall_velocity: u16,
    pub fall_y_subpixel: u8,
    pub target_slot_address: Option<u16>,
}

impl HumanSnapshot {
    pub const fn new(position: ScreenPosition) -> Self {
        Self {
            position,
            carried: false,
            carried_by_player: false,
            x_subpixel: 0,
            picture_frame: 0,
            fall_velocity: 0,
            fall_y_subpixel: 0,
            target_slot_address: None,
        }
    }

    fn arcade_world_position(self) -> (u16, u16) {
        arcade_world_position(
            self.position,
            self.x_subpixel,
            self.fall_y_subpixel,
        )
    }

    fn arcade_velocity_words(self) -> (u16, u16) {
        (0, self.fall_velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileSnapshot {
    pub position: ScreenPosition,
    pub tail_position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

impl ProjectileSnapshot {
    fn arcade_world_position(self) -> (u16, u16) {
        arcade_world_position(self.position, 0, 0)
    }

    fn arcade_velocity_words(self) -> (u16, u16) {
        fixed_point_velocity_words(self.velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainSegment {
    pub position: ScreenPosition,
    pub size: (u8, u8),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TerrainBlowStage {
    #[default]
    ExplosionPassSleeping,
    FlashClearedSleeping,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainBlowSnapshot {
    pub stage: TerrainBlowStage,
    pub status_terrain_blown: bool,
    pub elapsed_ticks: u16,
    pub explosion_pass: u8,
    pub explosion_pass_limit: u8,
    pub sleep_ticks_remaining: Option<u8>,
    pub flash_color_byte: u8,
    pub overload_counter: u8,
    pub terrain_erase_entries: u16,
    pub scanner_terrain_erase_entries: u16,
    pub terrain_words_remaining: u16,
    pub scanner_terrain_words_remaining: u16,
    pub explosions_per_pass: u8,
}

impl TerrainBlowSnapshot {
    pub const EMPTY: Self = Self {
        stage: TerrainBlowStage::ExplosionPassSleeping,
        status_terrain_blown: false,
        elapsed_ticks: 0,
        explosion_pass: 0,
        explosion_pass_limit: TERRAIN_BLOW_ITERATION_LIMIT,
        sleep_ticks_remaining: None,
        flash_color_byte: 0,
        overload_counter: 0,
        terrain_erase_entries: 0,
        scanner_terrain_erase_entries: 0,
        terrain_words_remaining: 0,
        scanner_terrain_words_remaining: 0,
        explosions_per_pass: 0,
    };

    pub fn started() -> Self {
        Self {
            stage: TerrainBlowStage::ExplosionPassSleeping,
            status_terrain_blown: TERRAIN_BLOW_STATUS_BIT != 0,
            elapsed_ticks: 0,
            explosion_pass: 0,
            explosion_pass_limit: TERRAIN_BLOW_ITERATION_LIMIT,
            sleep_ticks_remaining: Some(1),
            flash_color_byte: 0,
            overload_counter: TERRAIN_BLOW_OVERLOAD_COUNTER,
            terrain_erase_entries: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_erase_entries: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            terrain_words_remaining: 0,
            scanner_terrain_words_remaining: 0,
            explosions_per_pass: TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
        }
    }

    pub fn armed_terrain_visible() -> Self {
        Self {
            stage: TerrainBlowStage::ExplosionPassSleeping,
            status_terrain_blown: false,
            elapsed_ticks: 0,
            explosion_pass: 0,
            explosion_pass_limit: TERRAIN_BLOW_ITERATION_LIMIT,
            sleep_ticks_remaining: Some(1),
            flash_color_byte: 0,
            overload_counter: TERRAIN_BLOW_OVERLOAD_COUNTER,
            terrain_erase_entries: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_erase_entries: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            terrain_words_remaining: TERRAIN_BLOW_TERRAIN_ERASE_ENTRIES,
            scanner_terrain_words_remaining: TERRAIN_BLOW_SCANNER_ERASE_ENTRIES,
            explosions_per_pass: TERRAIN_BLOW_EXPLOSIONS_PER_PASS,
        }
    }

    pub const fn terrain_erased(self) -> bool {
        self.status_terrain_blown && self.terrain_words_remaining == 0
    }

    pub const fn scanner_terrain_erased(self) -> bool {
        self.status_terrain_blown && self.scanner_terrain_words_remaining == 0
    }
}

pub const OBJECT_EVIDENCE_DETAIL_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ObjectEvidenceList {
    #[default]
    Active,
    Inactive,
    Projectile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectEvidenceCategory {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Swarmer,
    Baiter,
    Human,
    PlayerProjectile,
    EnemyBomb,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectEvidenceDetailSnapshot {
    pub list: ObjectEvidenceList,
    pub object_category: Option<ObjectEvidenceCategory>,
    pub address: Option<u16>,
    pub slot: Option<u16>,
    pub screen_position: Option<ScreenPosition>,
    pub world_position: Option<(u16, u16)>,
    pub velocity: Option<(u16, u16)>,
    pub picture_address: Option<u16>,
    pub picture_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub primary_image_address: Option<u16>,
    pub alternate_image_address: Option<u16>,
    pub mapped_sprite: Option<SpriteId>,
    pub object_type: Option<u8>,
    pub scanner_color: Option<u16>,
}

impl ObjectEvidenceDetailSnapshot {
    pub const EMPTY: Self = Self {
        list: ObjectEvidenceList::Active,
        object_category: None,
        address: None,
        slot: None,
        screen_position: None,
        world_position: None,
        velocity: None,
        picture_address: None,
        picture_label: None,
        picture_size: None,
        primary_image_address: None,
        alternate_image_address: None,
        mapped_sprite: None,
        object_type: None,
        scanner_color: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjectEvidenceSnapshot {
    pub active_count: u16,
    pub inactive_count: u16,
    pub projectile_count: u16,
    pub visible_count: u16,
    pub evidence_crc32: Option<u32>,
    pub detail_count: u8,
    pub details: [ObjectEvidenceDetailSnapshot; OBJECT_EVIDENCE_DETAIL_LIMIT],
}

pub const SCANNER_RADAR_BLIP_LIMIT: usize = OBJECT_EVIDENCE_DETAIL_LIMIT;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScannerRadarStage {
    #[default]
    InactiveObjectScan,
    ActiveAndShellScan,
    RasterDisplay,
}

impl ScannerRadarStage {
    const fn stage_sleep_ticks(self) -> u8 {
        match self {
            Self::InactiveObjectScan => SCANNER_PROCESS_SLEEP_TICKS[0],
            Self::ActiveAndShellScan => SCANNER_PROCESS_SLEEP_TICKS[1],
            Self::RasterDisplay => SCANNER_PROCESS_SLEEP_TICKS[2],
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScannerRadarBlipKind {
    #[default]
    ActiveObject,
    InactiveObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarBlipSnapshot {
    pub kind: ScannerRadarBlipKind,
    pub object_address: Option<u16>,
    pub erase_table_address: u16,
    pub screen_address: u16,
    pub color_word: u16,
}

impl ScannerRadarBlipSnapshot {
    pub const EMPTY: Self = Self {
        kind: ScannerRadarBlipKind::ActiveObject,
        object_address: None,
        erase_table_address: 0,
        screen_address: 0,
        color_word: 0,
    };
}

impl Default for ScannerRadarBlipSnapshot {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarPlayerBlipSnapshot {
    pub erase_table_address: u16,
    pub screen_address: u16,
    pub body_word: u16,
    pub tail_byte: u8,
    pub upper_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerRadarSnapshot {
    pub enabled: bool,
    pub stage: ScannerRadarStage,
    pub stage_sleep_ticks: u8,
    pub source_process_sleep_ticks: [u8; 3],
    pub selected_map: u8,
    pub scan_left: Option<u16>,
    pub terrain_enabled: bool,
    pub object_erase_start: u16,
    pub setend: u16,
    pub blip_count: u8,
    pub blips: [ScannerRadarBlipSnapshot; SCANNER_RADAR_BLIP_LIMIT],
    pub player_blip: Option<ScannerRadarPlayerBlipSnapshot>,
}

impl ScannerRadarSnapshot {
    pub const DISABLED: Self = Self {
        enabled: false,
        stage: ScannerRadarStage::InactiveObjectScan,
        stage_sleep_ticks: 0,
        source_process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
        selected_map: 0,
        scan_left: None,
        terrain_enabled: false,
        object_erase_start: SCANNER_OBJECT_ERASE_START,
        setend: SCANNER_OBJECT_ERASE_START,
        blip_count: 0,
        blips: [ScannerRadarBlipSnapshot::EMPTY; SCANNER_RADAR_BLIP_LIMIT],
        player_blip: None,
    };

    pub(crate) fn for_world(
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
        object_evidence: &ObjectEvidenceSnapshot,
    ) -> Self {
        if phase != GamePhase::Playing
            || player_position == (WorldVector::default(), WorldVector::default())
        {
            return Self::DISABLED;
        }

        let stage = scanner_radar_stage_for_frame(frame);
        let scan_anchor_word = source_word_from_world_vector(scan_anchor);
        let scan_left = scan_anchor_word.wrapping_sub(SCANNER_SCAN_CENTER_OFFSET);
        let mut scanner = Self {
            enabled: true,
            stage,
            stage_sleep_ticks: stage.stage_sleep_ticks(),
            source_process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
            selected_map: SCANNER_SELECTED_MAP,
            scan_left: Some(scan_left),
            terrain_enabled: true,
            object_erase_start: SCANNER_OBJECT_ERASE_START,
            setend: SCANNER_OBJECT_ERASE_START,
            blip_count: 0,
            blips: [ScannerRadarBlipSnapshot::EMPTY; SCANNER_RADAR_BLIP_LIMIT],
            player_blip: None,
        };

        scanner.push_object_blips(object_evidence, scan_left);
        scanner.player_blip = Some(ScannerRadarPlayerBlipSnapshot {
            erase_table_address: scanner.setend,
            screen_address: scanner_radar_player_screen_address(player_position),
            body_word: SCANNER_PLAYER_BODY_WORD,
            tail_byte: SCANNER_PLAYER_TAIL_BYTE,
            upper_byte: SCANNER_PLAYER_UPPER_BYTE,
        });
        scanner
    }

    fn push_object_blips(&mut self, object_evidence: &ObjectEvidenceSnapshot, scan_left: u16) {
        let detail_count =
            usize::from(object_evidence.detail_count).min(OBJECT_EVIDENCE_DETAIL_LIMIT);
        for detail in &object_evidence.details[..detail_count] {
            let Some(kind) = scanner_radar_blip_kind(detail.list) else {
                continue;
            };
            let Some(color_word) = detail.scanner_color else {
                continue;
            };
            let Some((world_x, world_y)) = scanner_radar_object_world_position(detail) else {
                continue;
            };
            let index = usize::from(self.blip_count);
            if index >= SCANNER_RADAR_BLIP_LIMIT {
                return;
            }

            self.blips[index] = ScannerRadarBlipSnapshot {
                kind,
                object_address: detail.address,
                erase_table_address: self.setend,
                screen_address: scanner_radar_object_screen_address(world_x, world_y, scan_left),
                color_word,
            };
            self.blip_count += 1;
            self.setend = self.setend.wrapping_add(2);
        }
    }
}

impl Default for ScannerRadarSnapshot {
    fn default() -> Self {
        Self::DISABLED
    }
}

pub const EXPANDED_OBJECT_DETAIL_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExpandedObjectKind {
    #[default]
    Appearance,
    Explosion,
    ScorePopup,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectDetailSnapshot {
    pub kind: ExpandedObjectKind,
    pub slot_address: Option<u16>,
    pub size: u16,
    pub descriptor_address: Option<u16>,
    pub sprite_frame_label: Option<&'static str>,
    pub picture_size: Option<(u8, u8)>,
    pub mapped_sprite: Option<SpriteId>,
    pub erase_address: Option<u16>,
    pub center: Option<ScreenPosition>,
    pub top_left: Option<ScreenPosition>,
    pub object_address: Option<u16>,
    pub score_popup_lifetime_ticks: Option<u8>,
    pub score_popup_value: Option<u16>,
    pub explosion_frame: Option<u8>,
    pub explosion_lifetime_frames: Option<u8>,
}

impl ExpandedObjectDetailSnapshot {
    pub const EMPTY: Self = Self {
        kind: ExpandedObjectKind::Appearance,
        slot_address: None,
        size: 0,
        descriptor_address: None,
        sprite_frame_label: None,
        picture_size: None,
        mapped_sprite: None,
        erase_address: None,
        center: None,
        top_left: None,
        object_address: None,
        score_popup_lifetime_ticks: None,
        score_popup_value: None,
        explosion_frame: None,
        explosion_lifetime_frames: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpandedObjectEvidenceSnapshot {
    pub active_count: u16,
    pub last_slot_address: Option<u16>,
    pub detail_count: u8,
    pub details: [ExpandedObjectDetailSnapshot; EXPANDED_OBJECT_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScorePopupKind {
    Points250,
    Points500,
}

impl ScorePopupKind {
    const fn value(self) -> u16 {
        match self {
            Self::Points250 => 250,
            Self::Points500 => 500,
        }
    }

    const fn picture_label(self) -> &'static str {
        match self {
            Self::Points250 => "C25P1",
            Self::Points500 => "C5P1",
        }
    }

    const fn sprite(self) -> SpriteId {
        match self {
            Self::Points250 => SpriteId::SCORE_POPUP_250,
            Self::Points500 => SpriteId::SCORE_POPUP_500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScorePopupSnapshot {
    pub kind: ScorePopupKind,
    pub position: ScreenPosition,
    pub frames_remaining: u8,
    pub lifetime_ticks: u8,
}

impl ScorePopupSnapshot {
    pub fn spawn(kind: ScorePopupKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            frames_remaining: SCORE_POPUP_LIFETIME_TICKS,
            lifetime_ticks: SCORE_POPUP_LIFETIME_TICKS,
        }
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::ScorePopup,
            sprite_frame_label: Some(self.kind.picture_label()),
            picture_size: Some((6, 6)),
            mapped_sprite: Some(self.kind.sprite()),
            top_left: Some(self.position),
            score_popup_lifetime_ticks: Some(self.lifetime_ticks),
            score_popup_value: Some(self.kind.value()),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyAppearanceSnapshot {
    pub position: ScreenPosition,
    pub growth_size: u16,
    pub sprite_frame_label: &'static str,
    pub picture_size: (u8, u8),
    pub mapped_sprite: SpriteId,
}

impl EnemyAppearanceSnapshot {
    fn matches_enemy(self, enemy: EnemySnapshot) -> bool {
        self.position == source_enemy_appearance_position(enemy)
            && self.mapped_sprite == enemy.arcade_picture_descriptor().mapped_sprite
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.picture_size;
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Appearance,
            size: self.growth_size,
            sprite_frame_label: Some(self.sprite_frame_label),
            picture_size: Some((width, height)),
            mapped_sprite: Some(self.mapped_sprite),
            center: Some(source_appearance_center(self.position, self.picture_size)),
            top_left: Some(self.position),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

pub(crate) fn appearance_growth_size_for_age(age: u16) -> u16 {
    let size_high = APPEARANCE_INITIAL_SIZE.to_be_bytes()[0]
        .saturating_sub(u8::try_from(age).unwrap_or(u8::MAX));
    if size_high <= APPEARANCE_FINAL_SIZE.to_be_bytes()[0] {
        return APPEARANCE_FINAL_SIZE;
    }
    u16::from_be_bytes([size_high, 0])
}
