use crate::{ScreenPosition, ScreenVelocity, SpriteId};

use super::*;

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
    pub(super) const fn object_category(self) -> ObjectEvidenceCategory {
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
    pub(crate) lander_actor_state: Option<LanderDebugStateSnapshot>,
    pub(crate) mutant_actor_state: Option<MutantDebugStateSnapshot>,
    pub(crate) bomber_actor_state: Option<BomberDebugStateSnapshot>,
    pub(crate) swarmer_actor_state: Option<SwarmerDebugStateSnapshot>,
    pub(crate) baiter_actor_state: Option<BaiterDebugStateSnapshot>,
    pub(crate) pod_actor_state: Option<PodDebugStateSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActorDebugMotion {
    x_fraction: u8,
    y_fraction: u8,
    x_velocity: u16,
    y_velocity: u16,
}

impl ActorDebugMotion {
    pub(crate) const fn new(
        x_fraction: u8,
        y_fraction: u8,
        x_velocity: u16,
        y_velocity: u16,
    ) -> Self {
        Self {
            x_fraction,
            y_fraction,
            x_velocity,
            y_velocity,
        }
    }

    pub(crate) const fn x_fraction(self) -> u8 {
        self.x_fraction
    }

    pub(crate) const fn y_velocity(self) -> u16 {
        self.y_velocity
    }

    pub(super) fn world_position_words(self, position: ScreenPosition) -> (u16, u16) {
        world_position_words(position, self.x_fraction, self.y_fraction)
    }

    const fn velocity_words(self) -> (u16, u16) {
        (self.x_velocity, self.y_velocity)
    }

    fn baiter_velocity_words(self) -> (u16, u16) {
        (baiter_screen_x_velocity(self.x_velocity), self.y_velocity)
    }
}

impl EnemySnapshot {
    pub const fn new(kind: EnemyKind, position: ScreenPosition, velocity: ScreenVelocity) -> Self {
        Self {
            kind,
            position,
            velocity,
            lander_actor_state: None,
            mutant_actor_state: None,
            bomber_actor_state: None,
            swarmer_actor_state: None,
            baiter_actor_state: None,
            pod_actor_state: None,
        }
    }

    pub(super) fn object_bitmap_descriptor(self) -> ObjectBitmapDescriptor {
        match self.kind {
            EnemyKind::Lander => lander_object_bitmap_descriptor(
                self.lander_actor_state
                    .map(|actor_state| actor_state.animation_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Mutant => MUTANT_OBJECT_BITMAP_DESCRIPTOR,
            EnemyKind::Bomber => bomber_object_bitmap_descriptor(
                self.bomber_actor_state
                    .map(|actor_state| actor_state.animation_frame)
                    .unwrap_or_default(),
            ),
            EnemyKind::Pod => POD_OBJECT_BITMAP_DESCRIPTOR,
            EnemyKind::Swarmer => SWARMER_OBJECT_BITMAP_DESCRIPTOR,
            EnemyKind::Baiter => baiter_object_bitmap_descriptor(
                self.baiter_actor_state
                    .map(|actor_state| actor_state.animation_frame)
                    .unwrap_or_default(),
            ),
        }
    }

    pub(super) fn world_position_words(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .lander_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
            EnemyKind::Mutant => self
                .mutant_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
            EnemyKind::Bomber => self
                .bomber_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
            EnemyKind::Pod => self
                .pod_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
            EnemyKind::Swarmer => self
                .swarmer_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
            EnemyKind::Baiter => self
                .baiter_actor_state
                .map(|actor_state| actor_state.motion.world_position_words(self.position))
                .unwrap_or_else(|| world_position_words(self.position, 0, 0)),
        }
    }

    pub(super) fn motion_velocity_words(self) -> (u16, u16) {
        match self.kind {
            EnemyKind::Lander => self
                .lander_actor_state
                .map(|actor_state| actor_state.motion.velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Mutant => self
                .mutant_actor_state
                .map(|actor_state| actor_state.motion.velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Bomber => self
                .bomber_actor_state
                .map(|actor_state| actor_state.motion.velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Pod => self
                .pod_actor_state
                .map(|actor_state| actor_state.motion.velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Swarmer => self
                .swarmer_actor_state
                .map(|actor_state| actor_state.motion.velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
            EnemyKind::Baiter => self
                .baiter_actor_state
                .map(|actor_state| actor_state.motion.baiter_velocity_words())
                .unwrap_or_else(|| fixed_point_velocity_words(self.velocity)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LanderDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: u8,
    pub(crate) target_human_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MutantDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) hop_rng: GameRngSnapshot,
    pub(crate) render_x_correction: u16,
    pub(crate) dive_entry_shot_deferred: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BomberDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
    pub(crate) animation_frame: u8,
    pub(crate) cruise_altitude: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SwarmerDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
    pub(crate) acceleration: u8,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) horizontal_seek_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BaiterDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
    pub(crate) shot_timer: u8,
    pub(crate) sleep_ticks: u8,
    pub(crate) animation_frame: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PodDebugStateSnapshot {
    pub(crate) motion: ActorDebugMotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRngSnapshot {
    pub seed: u8,
    pub hseed: u8,
    pub lseed: u8,
}

pub(super) const GAME_RNG_DEFAULT_HSEED: u8 = 0xA5;
pub(super) const GAME_RNG_DEFAULT_LSEED: u8 = 0x5A;

impl Default for GameRngSnapshot {
    fn default() -> Self {
        Self {
            seed: 0,
            hseed: GAME_RNG_DEFAULT_HSEED,
            lseed: GAME_RNG_DEFAULT_LSEED,
        }
    }
}

impl GameRngSnapshot {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyProjectileKind {
    Fireball,
    BomberBombShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyProjectileSnapshot {
    pub position: ScreenPosition,
    pub velocity: ScreenVelocity,
    pub kind: EnemyProjectileKind,
    pub(crate) actor_x_fraction: u8,
    pub(crate) actor_y_fraction: u8,
    pub(crate) actor_x_velocity: u16,
    pub(crate) actor_y_velocity: u16,
    pub lifetime_ticks: u8,
}

impl EnemyProjectileSnapshot {
    pub(super) const fn bomb_object_bitmap_name(self) -> &'static str {
        ENEMY_BOMB_OBJECT_BITMAP_NAME
    }

    pub(super) fn world_position_words(self) -> (u16, u16) {
        world_position_words(self.position, self.actor_x_fraction, self.actor_y_fraction)
    }

    pub(super) const fn motion_velocity_words(self) -> (u16, u16) {
        (self.actor_x_velocity, self.actor_y_velocity)
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
    pub(super) fn total(self) -> u8 {
        self.landers
            .saturating_add(self.bombers)
            .saturating_add(self.pods)
            .saturating_add(self.mutants)
            .saturating_add(self.swarmers)
    }

    pub(super) fn family_counts(self) -> [(EnemyKind, u8); 5] {
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
    pub(crate) actor_x_fraction: u8,
    pub animation_frame: u8,
    pub(crate) actor_fall_velocity: u16,
    pub(crate) actor_fall_y_fraction: u8,
    pub(crate) actor_target_slot_address: Option<u16>,
}

impl HumanSnapshot {
    pub const fn new(position: ScreenPosition) -> Self {
        Self {
            position,
            carried: false,
            carried_by_player: false,
            actor_x_fraction: 0,
            animation_frame: 0,
            actor_fall_velocity: 0,
            actor_fall_y_fraction: 0,
            actor_target_slot_address: None,
        }
    }

    pub(super) fn world_position_words(self) -> (u16, u16) {
        world_position_words(
            self.position,
            self.actor_x_fraction,
            self.actor_fall_y_fraction,
        )
    }

    pub(super) fn motion_velocity_words(self) -> (u16, u16) {
        (0, self.actor_fall_velocity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileSnapshot {
    pub position: ScreenPosition,
    pub tail_position: ScreenPosition,
    pub velocity: ScreenVelocity,
}

impl ProjectileSnapshot {
    pub(super) fn world_position_words(self) -> (u16, u16) {
        world_position_words(self.position, 0, 0)
    }

    pub(super) fn motion_velocity_words(self) -> (u16, u16) {
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
    pub(crate) flash_color_byte: u8,
    pub(crate) overload_counter: u8,
    pub(crate) terrain_erase_entries: u16,
    pub(crate) scanner_terrain_erase_entries: u16,
    pub(crate) terrain_words_remaining: u16,
    pub(crate) scanner_terrain_words_remaining: u16,
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

    #[cfg(test)]
    pub(crate) const fn scanner_terrain_erased(self) -> bool {
        self.status_terrain_blown && self.scanner_terrain_words_remaining == 0
    }
}

pub const OBJECT_EVIDENCE_DETAIL_LIMIT: usize = 16;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ObjectEvidenceList {
    #[default]
    Active,
    Inactive,
    Projectile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectEvidenceCategory {
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
pub(crate) struct ObjectEvidenceDetailSnapshot {
    pub(crate) list: ObjectEvidenceList,
    pub(crate) object_category: Option<ObjectEvidenceCategory>,
    pub(crate) address: Option<u16>,
    pub(crate) slot: Option<u16>,
    pub(crate) screen_position: Option<ScreenPosition>,
    pub(crate) world_position: Option<(u16, u16)>,
    pub(crate) velocity: Option<(u16, u16)>,
    pub(crate) object_bitmap_descriptor_address: Option<u16>,
    pub(crate) object_bitmap_name: Option<&'static str>,
    pub(crate) object_bitmap_size: Option<(u8, u8)>,
    pub(crate) primary_image_address: Option<u16>,
    pub(crate) alternate_image_address: Option<u16>,
    pub(crate) mapped_sprite: Option<SpriteId>,
    pub(crate) object_type: Option<u8>,
    pub(crate) scanner_color: Option<u16>,
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
        object_bitmap_descriptor_address: None,
        object_bitmap_name: None,
        object_bitmap_size: None,
        primary_image_address: None,
        alternate_image_address: None,
        mapped_sprite: None,
        object_type: None,
        scanner_color: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ObjectEvidenceSnapshot {
    pub(crate) active_count: u16,
    pub(crate) inactive_count: u16,
    pub(crate) projectile_count: u16,
    pub(crate) visible_count: u16,
    pub(crate) evidence_crc32: Option<u32>,
    pub(crate) detail_count: u8,
    pub(crate) details: [ObjectEvidenceDetailSnapshot; OBJECT_EVIDENCE_DETAIL_LIMIT],
}

pub const SCANNER_RADAR_BLIP_LIMIT: usize = OBJECT_EVIDENCE_DETAIL_LIMIT;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ScannerRadarStage {
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
pub(crate) enum ScannerRadarBlipKind {
    #[default]
    ActiveObject,
    InactiveObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScannerRadarBlipSnapshot {
    pub(crate) kind: ScannerRadarBlipKind,
    pub(crate) object_address: Option<u16>,
    pub(crate) erase_table_address: u16,
    pub(crate) screen_cell: crate::ScreenAddress,
    pub(crate) color_word: u16,
}

impl ScannerRadarBlipSnapshot {
    pub const EMPTY: Self = Self {
        kind: ScannerRadarBlipKind::ActiveObject,
        object_address: None,
        erase_table_address: 0,
        screen_cell: crate::ScreenAddress::new(0),
        color_word: 0,
    };
}

impl Default for ScannerRadarBlipSnapshot {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScannerRadarPlayerBlipSnapshot {
    pub(crate) erase_table_address: u16,
    pub(crate) screen_cell: crate::ScreenAddress,
    pub(crate) body_word: u16,
    pub(crate) tail_byte: u8,
    pub(crate) upper_byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScannerRadarSnapshot {
    pub(crate) enabled: bool,
    pub(crate) stage: ScannerRadarStage,
    pub(crate) stage_sleep_ticks: u8,
    pub(crate) process_sleep_ticks: [u8; 3],
    pub(crate) selected_map: u8,
    pub(crate) scan_left: Option<u16>,
    pub(crate) terrain_enabled: bool,
    pub(crate) object_erase_start: u16,
    pub(crate) setend: u16,
    pub(crate) blip_count: u8,
    pub(crate) blips: [ScannerRadarBlipSnapshot; SCANNER_RADAR_BLIP_LIMIT],
    pub(crate) player_blip: Option<ScannerRadarPlayerBlipSnapshot>,
}

impl ScannerRadarSnapshot {
    pub const DISABLED: Self = Self {
        enabled: false,
        stage: ScannerRadarStage::InactiveObjectScan,
        stage_sleep_ticks: 0,
        process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
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
        step: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
        object_evidence: &ObjectEvidenceSnapshot,
    ) -> Self {
        if phase != GamePhase::Playing
            || player_position == (WorldVector::default(), WorldVector::default())
        {
            return Self::DISABLED;
        }

        let stage = scanner_radar_stage_for_step(step);
        let scan_anchor_word = world_vector_word(scan_anchor);
        let scan_left = scan_anchor_word.wrapping_sub(SCANNER_SCAN_CENTER_OFFSET);
        let mut scanner = Self {
            enabled: true,
            stage,
            stage_sleep_ticks: stage.stage_sleep_ticks(),
            process_sleep_ticks: SCANNER_PROCESS_SLEEP_TICKS,
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
            screen_cell: scanner_radar_player_cell(player_position),
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
                screen_cell: scanner_radar_object_cell(world_x, world_y, scan_left),
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

pub(super) fn world_vector_word(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as u16
}

pub const EXPANDED_OBJECT_DETAIL_LIMIT: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpriteAssetImageSpec {
    pub(crate) bitmap: crate::arcade_assets::ObjectBitmapId,
    pub(crate) rows: u8,
    pub(crate) byte_columns: u8,
}

impl SpriteAssetImageSpec {
    pub(crate) const fn new(
        bitmap: crate::arcade_assets::ObjectBitmapId,
        rows: u8,
        byte_columns: u8,
    ) -> Self {
        Self {
            bitmap,
            rows,
            byte_columns,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ExpandedObjectKind {
    #[default]
    Appearance,
    Explosion,
    ScorePopup,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ExpandedObjectDetailSnapshot {
    pub(crate) kind: ExpandedObjectKind,
    pub(crate) slot_address: Option<u16>,
    pub(crate) size: u16,
    pub(crate) descriptor_address: Option<u16>,
    pub(crate) sprite_asset_image: Option<SpriteAssetImageSpec>,
    pub(crate) object_bitmap_size: Option<(u8, u8)>,
    pub(crate) mapped_sprite: Option<SpriteId>,
    pub(crate) erase_address: Option<u16>,
    pub(crate) center: Option<ScreenPosition>,
    pub(crate) top_left: Option<ScreenPosition>,
    pub(crate) object_address: Option<u16>,
    pub(crate) score_popup_lifetime_ticks: Option<u8>,
    pub(crate) score_popup_value: Option<u16>,
    pub(crate) explosion_step: Option<u8>,
    pub(crate) explosion_lifetime_steps: Option<u8>,
}

impl ExpandedObjectDetailSnapshot {
    pub const EMPTY: Self = Self {
        kind: ExpandedObjectKind::Appearance,
        slot_address: None,
        size: 0,
        descriptor_address: None,
        sprite_asset_image: None,
        object_bitmap_size: None,
        mapped_sprite: None,
        erase_address: None,
        center: None,
        top_left: None,
        object_address: None,
        score_popup_lifetime_ticks: None,
        score_popup_value: None,
        explosion_step: None,
        explosion_lifetime_steps: None,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ExpandedObjectEvidenceSnapshot {
    pub(crate) active_count: u16,
    pub(crate) last_slot_address: Option<u16>,
    pub(crate) detail_count: u8,
    pub(crate) details: [ExpandedObjectDetailSnapshot; EXPANDED_OBJECT_DETAIL_LIMIT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScorePopupKind {
    Points250,
    Points500,
}

pub(super) const SCORE_POPUP_250_POINTS: u16 = 250;
pub(super) const SCORE_POPUP_500_POINTS: u16 = 500;

impl ScorePopupKind {
    const fn value(self) -> u16 {
        match self {
            Self::Points250 => SCORE_POPUP_250_POINTS,
            Self::Points500 => SCORE_POPUP_500_POINTS,
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
    pub ticks_remaining: u8,
    pub lifetime_ticks: u8,
}

impl ScorePopupSnapshot {
    pub fn spawn(kind: ScorePopupKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            ticks_remaining: SCORE_POPUP_LIFETIME_TICKS,
            lifetime_ticks: SCORE_POPUP_LIFETIME_TICKS,
        }
    }

    pub(super) fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::ScorePopup,
            object_bitmap_size: Some((6, 6)),
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
    pub(crate) sprite_asset_image: SpriteAssetImageSpec,
    pub object_bitmap_size: (u8, u8),
    pub mapped_sprite: SpriteId,
}

impl EnemyAppearanceSnapshot {
    pub(super) fn matches_enemy(self, enemy: EnemySnapshot) -> bool {
        self.position == enemy_appearance_position(enemy)
            && self.mapped_sprite == enemy.object_bitmap_descriptor().mapped_sprite
    }

    pub(super) fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.object_bitmap_size;
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Appearance,
            size: self.growth_size,
            sprite_asset_image: Some(self.sprite_asset_image),
            object_bitmap_size: Some((width, height)),
            mapped_sprite: Some(self.mapped_sprite),
            center: Some(appearance_center(self.position, self.object_bitmap_size)),
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
