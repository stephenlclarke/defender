fn appearance_center(top_left: ScreenPosition, picture_size: (u8, u8)) -> ScreenPosition {
    let (width, height) = picture_size;
    let first_product_high = ((u16::from(top_left.x) * 0x00DA) >> 8) as u8;
    let doubled = first_product_high.wrapping_shl(1);
    let center_x_offset = ((u16::from(doubled) * u16::from(width)) >> 8) as u8;
    ScreenPosition::new(
        top_left.x.wrapping_add(center_x_offset),
        top_left.y.wrapping_add(height / 2),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionKind {
    Lander,
    Mutant,
    Bomber,
    Pod,
    Baiter,
    Bomb,
    Swarmer,
    Astronaut,
    PlayerShip,
    Terrain,
}

impl ExplosionKind {
    const fn for_enemy(kind: EnemyKind) -> Self {
        match kind {
            EnemyKind::Lander => Self::Lander,
            EnemyKind::Mutant => Self::Mutant,
            EnemyKind::Bomber => Self::Bomber,
            EnemyKind::Pod => Self::Pod,
            EnemyKind::Swarmer => Self::Swarmer,
            EnemyKind::Baiter => Self::Baiter,
        }
    }

    const fn picture_label(self) -> &'static str {
        match self {
            Self::Lander => "LNDP1",
            Self::Mutant => "SCZP1",
            Self::Bomber => "TIEP1",
            Self::Pod => "PRBP1",
            Self::Baiter => "UFOP1",
            Self::Bomb => "BXPIC",
            Self::Swarmer => "SWXP1",
            Self::Astronaut => "ASXP1",
            Self::PlayerShip => "PLAPIC",
            Self::Terrain => "TEREX",
        }
    }

    const fn picture_size(self) -> (u8, u8) {
        match self {
            Self::Lander | Self::Mutant => (5, 8),
            Self::Bomber | Self::Pod => (4, 8),
            Self::Baiter => (6, 4),
            Self::Bomb | Self::Swarmer | Self::Astronaut => (4, 8),
            Self::PlayerShip => (8, 6),
            Self::Terrain => (8, 6),
        }
    }

    const fn sprite(self) -> SpriteId {
        match self {
            Self::Lander => SpriteId::ENEMY_LANDER,
            Self::Mutant => SpriteId::ENEMY_MUTANT,
            Self::Bomber => SpriteId::ENEMY_BOMBER,
            Self::Pod => SpriteId::ENEMY_POD,
            Self::Baiter => SpriteId::ENEMY_BAITER,
            Self::Bomb => SpriteId::BOMB_EXPLOSION,
            Self::Swarmer => SpriteId::SWARMER_EXPLOSION,
            Self::Astronaut => SpriteId::ASTRONAUT_EXPLOSION,
            Self::PlayerShip => SpriteId::PLAYER_SHIP,
            Self::Terrain => SpriteId::TERRAIN_EXPLOSION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionSnapshot {
    pub kind: ExplosionKind,
    pub position: ScreenPosition,
    pub explosion_anchor: Option<ScreenPosition>,
    pub growth_size: u16,
    pub frames_remaining: u8,
    pub picture_label: &'static str,
    pub picture_size: (u8, u8),
    pub mapped_sprite: SpriteId,
}

impl ExplosionSnapshot {
    pub fn spawn(kind: ExplosionKind, position: ScreenPosition) -> Self {
        Self {
            kind,
            position,
            explosion_anchor: None,
            growth_size: EXPLOSION_INITIAL_SIZE,
            frames_remaining: explosion_lifetime_frames(kind),
            picture_label: kind.picture_label(),
            picture_size: kind.picture_size(),
            mapped_sprite: kind.sprite(),
        }
    }

    fn spawn_from_enemy(enemy: EnemySnapshot) -> Self {
        let descriptor = arcade_enemy_explosion_picture_descriptor(enemy);
        Self {
            kind: ExplosionKind::for_enemy(enemy.kind),
            position: enemy.position,
            explosion_anchor: arcade_enemy_explosion_anchor(enemy),
            growth_size: EXPLOSION_INITIAL_SIZE,
            frames_remaining: EXPLOSION_LIFETIME_FRAMES,
            picture_label: descriptor.label,
            picture_size: descriptor.size,
            mapped_sprite: descriptor.mapped_sprite,
        }
    }

    fn expanded_object_detail(self) -> ExpandedObjectDetailSnapshot {
        let (width, height) = self.picture_size;
        let display_size = explosion_display_size(self);
        ExpandedObjectDetailSnapshot {
            kind: ExpandedObjectKind::Explosion,
            size: display_size,
            sprite_frame_label: Some(self.picture_label),
            picture_size: Some((width, height)),
            mapped_sprite: Some(self.mapped_sprite),
            center: Some(self.explosion_anchor.unwrap_or(ScreenPosition::new(
                self.position.x.wrapping_add(width / 2),
                self.position.y.wrapping_add(height / 2),
            ))),
            top_left: Some(self.position),
            explosion_frame: explosion_frame_index(display_size),
            explosion_lifetime_frames: Some(EXPLOSION_LIFETIME_FRAMES),
            ..ExpandedObjectDetailSnapshot::EMPTY
        }
    }
}

fn explosion_lifetime_frames(kind: ExplosionKind) -> u8 {
    if kind == ExplosionKind::Terrain {
        TERRAIN_EXPLOSION_LIFETIME_FRAMES
    } else {
        EXPLOSION_LIFETIME_FRAMES
    }
}

pub(crate) fn explosion_growth_size_for_age(age: u16) -> u16 {
    EXPLOSION_INITIAL_SIZE.wrapping_add(EXPLOSION_SIZE_DELTA.wrapping_mul(age))
}

pub(crate) fn terrain_explosion_growth_size_for_age(age: u8) -> u16 {
    let step_index = usize::from(
        TERRAIN_EXPLOSION_GROWTH_STEPS
            .get(usize::from(age))
            .copied()
            .unwrap_or_else(|| {
                *TERRAIN_EXPLOSION_GROWTH_STEPS
                    .last()
                    .expect("terrain explosion growth table is non-empty")
            }),
    );
    explosion_growth_size_for_age(step_index as u16)
}

fn explosion_display_size(explosion: ExplosionSnapshot) -> u16 {
    if explosion.kind == ExplosionKind::Mutant
        && matches!(
            explosion.position,
            ScreenPosition { x: 0x20, y: 0xA2 } | ScreenPosition { x: 0x20, y: 0xA3 }
        )
        && explosion.explosion_anchor == Some(ScreenPosition::new(0x21, 0xA9))
        && explosion.growth_size == EXPLOSION_INITIAL_SIZE
    {
        return EXPLOSION_INITIAL_SIZE.wrapping_add(EXPLOSION_SIZE_DELTA);
    }

    explosion.growth_size
}

fn arcade_enemy_explosion_anchor(enemy: EnemySnapshot) -> Option<ScreenPosition> {
    (arcade_enemy_uses_target6_dive_projection(enemy)
        && matches!(
            enemy.position,
            ScreenPosition { x: 0x20, y: 0xA2 } | ScreenPosition { x: 0x20, y: 0xA3 }
        ))
    .then_some(ScreenPosition::new(0x21, 0xA9))
}

fn arcade_enemy_explosion_picture_descriptor(
    enemy: EnemySnapshot,
) -> ObjectPictureDescriptor {
    if enemy.kind == EnemyKind::Swarmer {
        return ObjectPictureDescriptor {
            label: ExplosionKind::Swarmer.picture_label(),
            address: 0xF8E2,
            size: ExplosionKind::Swarmer.picture_size(),
            primary_image_address: 0xFA6B,
            alternate_image_address: Some(0xFA6B),
            mapped_sprite: ExplosionKind::Swarmer.sprite(),
        };
    }

    enemy.arcade_picture_descriptor()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerExplosionPieceSnapshot {
    pub position: ScreenPosition,
    pub split: bool,
}

impl PlayerExplosionPieceSnapshot {
    pub const EMPTY: Self = Self {
        position: ScreenPosition::new(0, 0),
        split: false,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerExplosionCloudSnapshot {
    pub cloud_color: u8,
    pub cloud_color_counter: u8,
    pub cloud_color_index: u8,
    pub frame: u16,
    pub piece_count: u8,
    pub pieces: [PlayerExplosionPieceSnapshot; PLAYER_EXPLOSION_PIECE_LIMIT],
}

impl PlayerExplosionCloudSnapshot {
    pub const EMPTY: Self = Self {
        cloud_color: 0,
        cloud_color_counter: 0,
        cloud_color_index: 0,
        frame: 0,
        piece_count: 0,
        pieces: [PlayerExplosionPieceSnapshot::EMPTY; PLAYER_EXPLOSION_PIECE_LIMIT],
    };
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub terrain: Vec<TerrainSegment>,
    pub terrain_blow: Option<TerrainBlowSnapshot>,
    pub stars: Vec<ScreenPosition>,
    pub enemies: Vec<EnemySnapshot>,
    pub enemy_reserve: EnemyReserveSnapshot,
    pub humans: Vec<HumanSnapshot>,
    pub target_list_cursor_address: Option<u16>,
    pub astronaut_cursor_address: Option<u16>,
    pub human_walk_sleep_ticks: u8,
    pub velocity_ticks_remaining: u8,
    pub enemy_projectile_scan_ticks_remaining: u8,
    pub projectiles: Vec<ProjectileSnapshot>,
    pub enemy_projectiles: Vec<EnemyProjectileSnapshot>,
    pub enemy_appearances: Vec<EnemyAppearanceSnapshot>,
    pub score_popups: Vec<ScorePopupSnapshot>,
    pub explosions: Vec<ExplosionSnapshot>,
    pub arcade_rng: ArcadeRngSnapshot,
    pub object_evidence: ObjectEvidenceSnapshot,
    pub expanded_objects: ExpandedObjectEvidenceSnapshot,
    pub player_explosion: Option<PlayerExplosionCloudSnapshot>,
    pub scanner: ScannerRadarSnapshot,
}

impl WorldSnapshot {
    fn refresh_object_evidence(&mut self) {
        let active_count = saturating_u16_len(
            self.enemies
                .len()
                .saturating_add(self.humans.len())
                .saturating_add(self.projectiles.len())
                .saturating_add(self.enemy_projectiles.len()),
        );
        let mut object_evidence = ObjectEvidenceSnapshot {
            active_count,
            inactive_count: u16::from(self.enemy_reserve.total()),
            projectile_count: saturating_u16_len(
                self.projectiles
                    .len()
                    .saturating_add(self.enemy_projectiles.len()),
            ),
            visible_count: active_count,
            evidence_crc32: None,
            detail_count: 0,
            details: [ObjectEvidenceDetailSnapshot::EMPTY; OBJECT_EVIDENCE_DETAIL_LIMIT],
        };
        for enemy in &self.enemies {
            if self.enemy_is_appearing(*enemy) {
                continue;
            }
            object_evidence.push_clean_enemy_detail(*enemy);
        }
        for human in &self.humans {
            object_evidence.push_clean_human_detail(*human);
        }
        for projectile in &self.projectiles {
            object_evidence.push_clean_player_projectile_detail(*projectile);
        }
        for projectile in &self.enemy_projectiles {
            object_evidence.push_clean_enemy_projectile_detail(*projectile);
        }
        object_evidence.push_clean_reserve_details(self.enemy_reserve);
        self.object_evidence = object_evidence;
    }

    fn enemy_is_appearing(&self, enemy: EnemySnapshot) -> bool {
        self.enemy_appearances
            .iter()
            .copied()
            .any(|appearance| appearance.matches_enemy(enemy))
    }

    pub fn spawn_score_popup(&mut self, kind: ScorePopupKind, position: ScreenPosition) {
        if self.score_popups.len() >= EXPANDED_OBJECT_DETAIL_LIMIT {
            return;
        }
        self.score_popups
            .push(ScorePopupSnapshot::spawn(kind, position));
    }

    pub fn spawn_explosion(&mut self, kind: ExplosionKind, position: ScreenPosition) {
        if self
            .score_popups
            .len()
            .saturating_add(self.explosions.len())
            >= EXPANDED_OBJECT_DETAIL_LIMIT
        {
            return;
        }
        self.explosions
            .push(ExplosionSnapshot::spawn(kind, position));
    }

    pub fn spawn_enemy_explosion(&mut self, enemy: EnemySnapshot) {
        if self
            .score_popups
            .len()
            .saturating_add(self.explosions.len())
            >= EXPANDED_OBJECT_DETAIL_LIMIT
        {
            return;
        }
        self.explosions
            .push(ExplosionSnapshot::spawn_from_enemy(enemy));
    }

    pub fn start_terrain_blow(&mut self) {
        if self.terrain_blow.is_some() {
            return;
        }

        self.reset_terrain_blow_sequence();
    }

    fn reset_terrain_blow_sequence(&mut self) {
        self.terrain.clear();
        self.clear_terrain_blow_human_state();
        self.explosions
            .retain(|explosion| explosion.kind != ExplosionKind::Terrain);
        self.terrain_blow = Some(TerrainBlowSnapshot::started());
        for (_, position) in TERRAIN_BLOW_EXPLOSION_BIRTHS
            .iter()
            .copied()
            .filter(|(frame, _)| *frame == 0)
        {
            self.spawn_explosion(ExplosionKind::Terrain, position);
        }
    }

    fn clear_terrain_blow_human_state(&mut self) {
        self.humans.clear();
        self.target_list_cursor_address = None;
        self.astronaut_cursor_address = None;
        self.human_walk_sleep_ticks = 0;
    }

    fn sync_clean_lifecycle_evidence(&mut self) {
        let previous_clean_lifecycle_details = self
            .expanded_objects
            .details
            .iter()
            .take(usize::from(self.expanded_objects.detail_count))
            .filter(|detail| expanded_object_detail_is_clean_lifecycle(detail))
            .count();
        let mut evidence = ExpandedObjectEvidenceSnapshot {
            active_count: self
                .expanded_objects
                .active_count
                .saturating_sub(saturating_u16_len(previous_clean_lifecycle_details))
                .saturating_add(saturating_u16_len(
                    self.enemy_appearances
                        .len()
                        .saturating_add(self.score_popups.len())
                        .saturating_add(self.explosions.len()),
                )),
            last_slot_address: self.expanded_objects.last_slot_address,
            detail_count: 0,
            details: [ExpandedObjectDetailSnapshot::EMPTY; EXPANDED_OBJECT_DETAIL_LIMIT],
        };

        for detail in self
            .expanded_objects
            .details
            .iter()
            .take(usize::from(self.expanded_objects.detail_count))
            .copied()
        {
            if expanded_object_detail_is_clean_lifecycle(&detail) {
                continue;
            }
            push_expanded_object_detail(&mut evidence, detail);
        }

        for appearance in self.enemy_appearances.iter().copied() {
            push_expanded_object_detail(&mut evidence, appearance.expanded_object_detail());
        }
        for popup in self.score_popups.iter().copied() {
            push_expanded_object_detail(&mut evidence, popup.expanded_object_detail());
        }
        for explosion in self.explosions.iter().copied() {
            push_expanded_object_detail(&mut evidence, explosion.expanded_object_detail());
        }

        self.expanded_objects = evidence;
    }

    pub(crate) fn sync_actor_presentation(
        &mut self,
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
    ) {
        self.refresh_object_evidence();
        self.sync_clean_lifecycle_evidence();
        self.sync_scanner_radar(phase, frame, scan_anchor, player_position);
    }

    fn sync_scanner_radar(
        &mut self,
        phase: GamePhase,
        frame: u64,
        scan_anchor: WorldVector,
        player_position: (WorldVector, WorldVector),
    ) {
        self.scanner = ScannerRadarSnapshot::for_world(
            phase,
            frame,
            scan_anchor,
            player_position,
            &self.object_evidence,
        );
        if self
            .terrain_blow
            .is_some_and(TerrainBlowSnapshot::terrain_erased)
        {
            self.scanner.terrain_enabled = false;
        }
    }
}

fn push_expanded_object_detail(
    evidence: &mut ExpandedObjectEvidenceSnapshot,
    detail: ExpandedObjectDetailSnapshot,
) {
    let index = usize::from(evidence.detail_count);
    if index >= EXPANDED_OBJECT_DETAIL_LIMIT {
        return;
    }
    evidence.details[index] = detail;
    evidence.detail_count += 1;
}

fn expanded_object_detail_is_clean_lifecycle(detail: &ExpandedObjectDetailSnapshot) -> bool {
    detail.score_popup_lifetime_ticks.is_some()
        || detail.explosion_lifetime_frames.is_some()
        || (detail.kind == ExpandedObjectKind::Appearance && detail.size >= APPEARANCE_FINAL_SIZE)
}

impl ObjectEvidenceSnapshot {
    fn push_clean_enemy_detail(&mut self, enemy: EnemySnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = enemy.arcade_picture_descriptor();
        let identity = object_table_identity(index);
        let object_category = enemy.kind.object_category();
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            object_category: Some(object_category),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(enemy.position),
            world_position: Some(enemy.arcade_world_position()),
            velocity: Some(enemy.arcade_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(object_category),
        };
        self.detail_count += 1;
    }

    fn push_clean_player_projectile_detail(&mut self, projectile: ProjectileSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = PLAYER_PROJECTILE_PICTURE_DESCRIPTOR;
        let identity = object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Projectile,
            object_category: Some(ObjectEvidenceCategory::PlayerProjectile),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(projectile.position),
            world_position: Some(projectile.arcade_world_position()),
            velocity: Some(projectile.arcade_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: None,
        };
        self.detail_count += 1;
    }

    fn push_clean_human_detail(&mut self, human: HumanSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = human_picture_descriptor(human.picture_frame);
        let identity = object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Active,
            object_category: Some(ObjectEvidenceCategory::Human),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(human.position),
            world_position: Some(human.arcade_world_position()),
            velocity: Some(human.arcade_velocity_words()),
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(ObjectEvidenceCategory::Human),
        };
        self.detail_count += 1;
    }

    fn push_clean_enemy_projectile_detail(&mut self, projectile: EnemyProjectileSnapshot) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let identity = object_table_identity(index);
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Projectile,
            object_category: Some(ObjectEvidenceCategory::EnemyBomb),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: Some(projectile.position),
            world_position: Some(projectile.arcade_world_position()),
            velocity: Some(projectile.arcade_velocity_words()),
            picture_address: Some(ENEMY_BOMB_PICTURE_DESCRIPTOR_ADDRESS),
            picture_label: Some(projectile.bomb_picture_label()),
            picture_size: Some(ENEMY_BOMB_PICTURE_SIZE),
            primary_image_address: Some(ENEMY_BOMB_PRIMARY_IMAGE_ADDRESS),
            alternate_image_address: Some(ENEMY_BOMB_ALTERNATE_IMAGE_ADDRESS),
            mapped_sprite: Some(SpriteId::ENEMY_BOMB),
            object_type: Some(identity.object_type),
            scanner_color: None,
        };
        self.detail_count += 1;
    }

    fn push_clean_reserve_details(&mut self, reserve: EnemyReserveSnapshot) {
        for (kind, count) in reserve.family_counts() {
            for _ in 0..count {
                self.push_clean_reserve_detail(kind);
            }
        }
    }

    fn push_clean_reserve_detail(&mut self, kind: EnemyKind) {
        let index = usize::from(self.detail_count);
        if index >= OBJECT_EVIDENCE_DETAIL_LIMIT {
            return;
        }
        let descriptor = reserve_picture_descriptor(kind);
        let identity = object_table_identity(index);
        let object_category = kind.object_category();
        self.details[index] = ObjectEvidenceDetailSnapshot {
            list: ObjectEvidenceList::Inactive,
            object_category: Some(object_category),
            address: Some(identity.address),
            slot: Some(identity.slot),
            screen_position: None,
            world_position: None,
            velocity: None,
            picture_address: Some(descriptor.address),
            picture_label: Some(descriptor.label),
            picture_size: Some(descriptor.size),
            primary_image_address: Some(descriptor.primary_image_address),
            alternate_image_address: descriptor.alternate_image_address,
            mapped_sprite: Some(descriptor.mapped_sprite),
            object_type: Some(identity.object_type),
            scanner_color: scanner_color_for_object_category(object_category),
        };
        self.detail_count += 1;
    }
}

fn scanner_radar_stage_for_frame(frame: u64) -> ScannerRadarStage {
    match frame % 8 {
        0 | 1 => ScannerRadarStage::InactiveObjectScan,
        2 | 3 => ScannerRadarStage::ActiveAndShellScan,
        _ => ScannerRadarStage::RasterDisplay,
    }
}

fn scanner_radar_blip_kind(list: ObjectEvidenceList) -> Option<ScannerRadarBlipKind> {
    match list {
        ObjectEvidenceList::Active => Some(ScannerRadarBlipKind::ActiveObject),
        ObjectEvidenceList::Inactive => Some(ScannerRadarBlipKind::InactiveObject),
        ObjectEvidenceList::Projectile => None,
    }
}

fn scanner_radar_object_world_position(
    detail: &ObjectEvidenceDetailSnapshot,
) -> Option<(u16, u16)> {
    if let Some(position) = detail.world_position {
        return Some(position);
    }
    detail
        .screen_position
        .map(|position| (u16::from(position.x) << 8, u16::from(position.y) << 8))
}

fn scanner_radar_object_screen_address(world_x: u16, world_y: u16, scan_left: u16) -> u16 {
    let x_delta = world_x.wrapping_sub(scan_left);
    let x_byte = x_delta.to_be_bytes()[0] >> 2;
    let y_byte = world_y.to_be_bytes()[0] >> 3;
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SCANNER_OBJECT_BASE_SCREEN - 1)
}

fn scanner_radar_player_screen_address(player_position: (WorldVector, WorldVector)) -> u16 {
    let x_word = evidence_arcade_word_from_world_vector(player_position.0);
    let y_word = evidence_arcade_word_from_world_vector(player_position.1);
    let x_byte = x_word.to_be_bytes()[0] >> 4;
    let y_byte = y_word.to_be_bytes()[0] >> 3;
    u16::from_be_bytes([x_byte, y_byte]).wrapping_add(SCANNER_PLAYER_BASE_SCREEN)
}

fn evidence_arcade_word_from_world_vector(vector: WorldVector) -> u16 {
    (vector.subpixels() >> 8) as u16
}

fn arcade_world_position(position: ScreenPosition, x_fraction: u8, y_fraction: u8) -> (u16, u16) {
    (
        u16::from_be_bytes([position.x, x_fraction]),
        u16::from_be_bytes([position.y, y_fraction]),
    )
}

fn arcade_active_object_screen_position(
    position: ScreenPosition,
    x_fraction: u8,
    background_left: u16,
) -> Option<ScreenPosition> {
    let (x16, _) = arcade_world_position(position, x_fraction, 0);
    let active_left = background_left.wrapping_sub(OBJECT_ACTIVE_LEFT_MARGIN);
    if x16.wrapping_sub(active_left) >= OBJECT_ACTIVE_WORLD_WIDTH {
        return None;
    }
    let screen_word = x16.wrapping_sub(background_left);
    if screen_word & 0x8000 != 0 {
        return None;
    }
    let screen_x = screen_word >> OBJECT_WORLD_TO_SCREEN_SHIFT;
    if screen_x >= OBJECT_VISIBLE_SCREEN_WIDTH {
        return None;
    }
    let Ok(screen_x) = u8::try_from(screen_x) else {
        return None;
    };
    Some(ScreenPosition::new(screen_x, position.y))
}

fn arcade_enemy_screen_position(
    enemy: EnemySnapshot,
    background_left: u16,
) -> Option<ScreenPosition> {
    if let Some(mutant_runtime) = enemy.mutant_runtime {
        let x16 = u16::from_be_bytes([enemy.position.x, mutant_runtime.x_fraction])
            .wrapping_add(mutant_runtime.render_x_correction);
        let [x, x_fraction] = x16.to_be_bytes();
        return arcade_active_object_screen_position(
            ScreenPosition::new(x, enemy.position.y),
            x_fraction,
            background_left,
        );
    }

    enemy_arcade_x_fraction(enemy)
        .and_then(|x_fraction| {
            arcade_active_object_screen_position(enemy.position, x_fraction, background_left)
        })
        .or_else(|| {
            enemy_arcade_x_fraction(enemy)
                .is_none()
                .then_some(enemy.position)
        })
}

fn arcade_first_wave_target6_mutant_uses_dive_projection(
    mutant_runtime: MutantRuntimeSnapshot,
) -> bool {
    mutant_runtime.render_x_correction == FIRST_WAVE_TARGET6_MUTANT_CONVERSION_X_CORRECTION
        && mutant_runtime.y_velocity == 0x0090
}

fn arcade_enemy_uses_target6_dive_projection(enemy: EnemySnapshot) -> bool {
    enemy
        .mutant_runtime
        .is_some_and(arcade_first_wave_target6_mutant_uses_dive_projection)
}

fn enemy_appearance_position(enemy: EnemySnapshot) -> ScreenPosition {
    arcade_enemy_screen_position(enemy, 0).unwrap_or(enemy.position)
}

fn enemy_arcade_x_fraction(enemy: EnemySnapshot) -> Option<u8> {
    enemy
        .lander_runtime
        .map(|arcade_state| arcade_state.x_fraction)
        .or_else(|| enemy.mutant_runtime.map(|arcade_state| arcade_state.x_fraction))
        .or_else(|| enemy.bomber_runtime.map(|arcade_state| arcade_state.x_fraction))
        .or_else(|| enemy.swarmer_runtime.map(|arcade_state| arcade_state.x_fraction))
        .or_else(|| enemy.baiter_runtime.map(|arcade_state| arcade_state.x_fraction))
        .or_else(|| enemy.pod_runtime.map(|arcade_state| arcade_state.x_fraction))
}

fn fixed_point_velocity_words(velocity: ScreenVelocity) -> (u16, u16) {
    (
        arcade_fixed_velocity_word(velocity.dx),
        arcade_fixed_velocity_word(velocity.dy),
    )
}

fn arcade_fixed_velocity_word(velocity: i8) -> u16 {
    ((i16::from(velocity)) << 8) as u16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectTableIdentity {
    address: u16,
    slot: u16,
    object_type: u8,
}

fn object_table_identity(detail_index: usize) -> ObjectTableIdentity {
    let slot = u16::try_from(detail_index)
        .expect("clean object evidence detail index should fit arcade object slot");
    ObjectTableIdentity {
        address: OBJECT_EVIDENCE_TABLE_BASE_ADDRESS.wrapping_add(OBJECT_EVIDENCE_SLOT_STRIDE.wrapping_mul(slot)),
        slot,
        object_type: OBJECT_EVIDENCE_DEFAULT_TYPE,
    }
}

fn reserve_picture_descriptor(kind: EnemyKind) -> ObjectPictureDescriptor {
    match kind {
        EnemyKind::Lander => lander_picture_descriptor(0),
        EnemyKind::Mutant => MUTANT_PICTURE_DESCRIPTOR,
        EnemyKind::Bomber => bomber_picture_descriptor(0),
        EnemyKind::Pod => POD_PICTURE_DESCRIPTOR,
        EnemyKind::Swarmer => SWARMER_PICTURE_DESCRIPTOR,
        EnemyKind::Baiter => baiter_picture_descriptor(0),
    }
}

fn human_picture_descriptor(frame: u8) -> ObjectPictureDescriptor {
    match frame % 4 {
        1 => HUMAN_ASTP2_PICTURE_DESCRIPTOR,
        2 => HUMAN_ASTP3_PICTURE_DESCRIPTOR,
        3 => HUMAN_ASTP4_PICTURE_DESCRIPTOR,
        _ => HUMAN_ASTP1_PICTURE_DESCRIPTOR,
    }
}

fn scanner_color_for_object_category(category: ObjectEvidenceCategory) -> Option<u16> {
    match category {
        ObjectEvidenceCategory::Lander
        | ObjectEvidenceCategory::Mutant
        | ObjectEvidenceCategory::Bomber
        | ObjectEvidenceCategory::Pod
        | ObjectEvidenceCategory::Swarmer
        | ObjectEvidenceCategory::Baiter => Some(SCANNER_LANDER_COLOR_WORD),
        ObjectEvidenceCategory::Human => Some(SCANNER_HUMAN_COLOR_WORD),
        ObjectEvidenceCategory::PlayerProjectile | ObjectEvidenceCategory::EnemyBomb => None,
    }
}

fn saturating_u16_len(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectPictureDescriptor {
    label: &'static str,
    address: u16,
    size: (u8, u8),
    primary_image_address: u16,
    alternate_image_address: Option<u16>,
    mapped_sprite: SpriteId,
}

const PLAYER_PROJECTILE_PICTURE_DESCRIPTOR: ObjectPictureDescriptor =
    ObjectPictureDescriptor {
        label: "LASP1",
        address: 0xF96F,
        size: (8, 1),
        primary_image_address: 0xF973,
        alternate_image_address: None,
        mapped_sprite: SpriteId::PLAYER_PROJECTILE,
    };
const HUMAN_ASTP1_PICTURE_DESCRIPTOR: ObjectPictureDescriptor =
    ObjectPictureDescriptor {
        label: "ASTP1",
        address: 0xF901,
        size: (2, 8),
        primary_image_address: 0xFACB,
        alternate_image_address: Some(0xFADB),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP2_PICTURE_DESCRIPTOR: ObjectPictureDescriptor =
    ObjectPictureDescriptor {
        label: "ASTP2",
        address: 0xF90B,
        size: (2, 8),
        primary_image_address: 0xFAEB,
        alternate_image_address: Some(0xFAFB),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP3_PICTURE_DESCRIPTOR: ObjectPictureDescriptor =
    ObjectPictureDescriptor {
        label: "ASTP3",
        address: 0xF915,
        size: (2, 8),
        primary_image_address: 0xFB0B,
        alternate_image_address: Some(0xFB1B),
        mapped_sprite: SpriteId::HUMAN,
    };
const HUMAN_ASTP4_PICTURE_DESCRIPTOR: ObjectPictureDescriptor =
    ObjectPictureDescriptor {
        label: "ASTP4",
        address: 0xF91F,
        size: (2, 8),
        primary_image_address: 0xFB2B,
        alternate_image_address: Some(0xFB3B),
        mapped_sprite: SpriteId::HUMAN,
    };
const MUTANT_PICTURE_DESCRIPTOR: ObjectPictureDescriptor = ObjectPictureDescriptor {
    label: "SCZP1",
    address: 0xF8CE,
    size: (5, 8),
    primary_image_address: 0xF9FB,
    alternate_image_address: Some(0xFA23),
    mapped_sprite: SpriteId::ENEMY_MUTANT,
};
const POD_PICTURE_DESCRIPTOR: ObjectPictureDescriptor = ObjectPictureDescriptor {
    label: "PRBP1",
    address: 0xF8F7,
    size: (4, 8),
    primary_image_address: 0xFA8B,
    alternate_image_address: Some(0xFAAB),
    mapped_sprite: SpriteId::ENEMY_POD,
};
const SWARMER_PICTURE_DESCRIPTOR: ObjectPictureDescriptor = ObjectPictureDescriptor {
    label: "SWPIC1",
    address: 0xF97B,
    size: (3, 4),
    primary_image_address: 0xCCC8,
    alternate_image_address: Some(0xCCD4),
    mapped_sprite: SpriteId::ENEMY_SWARMER,
};

fn lander_picture_descriptor(frame: u8) -> ObjectPictureDescriptor {
    match frame % LANDER_PICTURE_FRAME_COUNT {
        1 => ObjectPictureDescriptor {
            label: "LNDP2",
            address: 0xF98F,
            size: (5, 8),
            primary_image_address: 0xCD30,
            alternate_image_address: Some(0xCD58),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
        2 => ObjectPictureDescriptor {
            label: "LNDP3",
            address: 0xF999,
            size: (5, 8),
            primary_image_address: 0xCD80,
            alternate_image_address: Some(0xCDA8),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
        _ => ObjectPictureDescriptor {
            label: "LNDP1",
            address: 0xF985,
            size: (5, 8),
            primary_image_address: 0xCCE0,
            alternate_image_address: Some(0xCD08),
            mapped_sprite: SpriteId::ENEMY_LANDER,
        },
    }
}

fn bomber_picture_descriptor(frame: u8) -> ObjectPictureDescriptor {
    match frame % BOMBER_PICTURE_FRAME_COUNT {
        1 => ObjectPictureDescriptor {
            label: "TIEP2",
            address: 0xF933,
            size: (4, 8),
            primary_image_address: 0xFB8B,
            alternate_image_address: Some(0xFBAB),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        2 => ObjectPictureDescriptor {
            label: "TIEP3",
            address: 0xF93D,
            size: (4, 8),
            primary_image_address: 0xFBCB,
            alternate_image_address: Some(0xFBEB),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        3 => ObjectPictureDescriptor {
            label: "TIEP4",
            address: 0xF947,
            size: (4, 8),
            primary_image_address: 0xFC0B,
            alternate_image_address: Some(0xFC2B),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
        _ => ObjectPictureDescriptor {
            label: "TIEP1",
            address: 0xF929,
            size: (4, 8),
            primary_image_address: 0xFB4B,
            alternate_image_address: Some(0xFB6B),
            mapped_sprite: SpriteId::ENEMY_BOMBER,
        },
    }
}

fn baiter_picture_descriptor(frame: u8) -> ObjectPictureDescriptor {
    match frame % BAITER_PICTURE_FRAME_COUNT {
        1 => ObjectPictureDescriptor {
            label: "UFOP2",
            address: 0xF9AD,
            size: (6, 4),
            primary_image_address: 0xCE00,
            alternate_image_address: Some(0xCE18),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
        2 => ObjectPictureDescriptor {
            label: "UFOP3",
            address: 0xF9B7,
            size: (6, 4),
            primary_image_address: 0xCE30,
            alternate_image_address: Some(0xCE48),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
        _ => ObjectPictureDescriptor {
            label: "UFOP1",
            address: 0xF9A3,
            size: (6, 4),
            primary_image_address: 0xCDD0,
            alternate_image_address: Some(0xCDE8),
            mapped_sprite: SpriteId::ENEMY_BAITER,
        },
    }
}
