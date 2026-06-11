use super::*;

pub(in crate::actor_game) const MUTANT_DIVE_ENTRY_SHOT_SCREEN: Point = Point::new(0x13, 0x46);
pub(in crate::actor_game) const MUTANT_DIVE_FIRST_PATH_SHOT_SCREEN: Point = Point::new(0x1E, 0x70);
pub(in crate::actor_game) const MUTANT_DIVE_SECOND_PATH_SHOT_SCREEN: Point = Point::new(0x21, 0x87);

pub(in crate::actor_game) const MUTANT_DIVE_FORCED_FIRST_PROJECTILE_PATTERN:
    MutantDiveProjectilePattern = MutantDiveProjectilePattern {
    position: Point::new(0x1E, 0x54),
    motion: ActorMotion::new(0x33, 0x56, 0xFFE0, 0x0138),
};
pub(in crate::actor_game) const MUTANT_DIVE_FORCED_SECOND_PROJECTILE_PATTERN:
    MutantDiveProjectilePattern = MutantDiveProjectilePattern {
    position: Point::new(0x21, 0x7F),
    motion: ActorMotion::new(0x6F, 0xE1, 0xFFF0, 0x00C0),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) struct MutantDiveProjectilePattern {
    pub(in crate::actor_game) position: Point,
    pub(in crate::actor_game) motion: ActorMotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) struct MutantDivePathAnchor {
    pub(in crate::actor_game) world: MutantDiveWorldPosition,
    pub(in crate::actor_game) screen: Point,
}

impl MutantDivePathAnchor {
    const fn new(world_x_word: u16, world_y_word: u16, screen_x: i16, screen_y: i16) -> Self {
        Self {
            world: MutantDiveWorldPosition::new(world_x_word, world_y_word),
            screen: Point::new(screen_x, screen_y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) struct MutantDiveWorldPosition {
    pub(in crate::actor_game) x_word: u16,
    pub(in crate::actor_game) y_word: u16,
}

impl MutantDiveWorldPosition {
    pub(in crate::actor_game) const fn new(x_word: u16, y_word: u16) -> Self {
        Self { x_word, y_word }
    }

    pub(in crate::actor_game) const fn y_word(self) -> u16 {
        self.y_word
    }

    pub(in crate::actor_game) const fn matches(self, x_word: u16, y_word: u16) -> bool {
        self.x_word == x_word && self.y_word == y_word
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::actor_game) struct MutantDiveVisualRow {
    pub(in crate::actor_game) world_x_word: u16,
    pub(in crate::actor_game) screen_y: i16,
}

impl MutantDiveVisualRow {
    const fn new(world_x_word: u16, screen_y: i16) -> Self {
        Self {
            world_x_word,
            screen_y,
        }
    }

    pub(in crate::actor_game) const fn screen_y_for_world_x(
        self,
        world_x_word: u16,
    ) -> Option<i16> {
        if self.world_x_word == world_x_word {
            Some(self.screen_y)
        } else {
            None
        }
    }
}

pub(in crate::actor_game) const MUTANT_DIVE_PATH_ANCHORS: &[MutantDivePathAnchor] = &[
    MutantDivePathAnchor::new(0x031C, 0x3360, 0x12, 0x43),
    MutantDivePathAnchor::new(0x037C, 0x3380, 0x13, 0x46),
    MutantDivePathAnchor::new(0x034C, 0x33F0, 0x12, 0x43),
    MutantDivePathAnchor::new(0x03AC, 0x3410, 0x14, 0x46),
    MutantDivePathAnchor::new(0x037C, 0x3480, 0x13, 0x44),
    MutantDivePathAnchor::new(0x085C, 0x47A0, 0x1F, 0x5B),
    MutantDivePathAnchor::new(0x085C, 0x6120, 0x1F, 0x71),
    MutantDivePathAnchor::new(0x088C, 0x61B0, 0x1E, 0x71),
    MutantDivePathAnchor::new(0x085C, 0x6140, 0x1F, 0x71),
    MutantDivePathAnchor::new(0x082C, 0x7770, 0x20, 0x87),
    MutantDivePathAnchor::new(0x07FC, 0x7800, 0x21, 0x88),
    MutantDivePathAnchor::new(0x082C, 0x7990, 0x20, 0x87),
    MutantDivePathAnchor::new(0x082C, 0x81E0, 0x20, 0x90),
    MutantDivePathAnchor::new(0x082C, 0x9730, 0x21, 0x9F),
    MutantDivePathAnchor::new(0x07FC, 0x97A0, 0x20, 0x9E),
    MutantDivePathAnchor::new(0x085C, 0x97C0, 0x20, 0xA0),
    MutantDivePathAnchor::new(0x088C, 0x9850, 0x1F, 0xA0),
    MutantDivePathAnchor::new(0x085C, 0x99E0, 0x1E, 0xA2),
    MutantDivePathAnchor::new(0x082C, 0x9A70, 0x20, 0xA3),
    MutantDivePathAnchor::new(0x088C, 0xA200, 0x20, 0xA2),
    MutantDivePathAnchor::new(0x08EC, 0xA320, 0x20, 0xA2),
];

pub(in crate::actor_game) const MUTANT_DIVE_VISUAL_ROWS: &[MutantDiveVisualRow] = &[
    MutantDiveVisualRow::new(0x0004, 0x36),
    MutantDiveVisualRow::new(0x0034, 0x36),
    MutantDiveVisualRow::new(0x0064, 0x37),
    MutantDiveVisualRow::new(0x0094, 0x37),
    MutantDiveVisualRow::new(0x00C4, 0x37),
    MutantDiveVisualRow::new(0x00F4, 0x37),
    MutantDiveVisualRow::new(0x0124, 0x36),
    MutantDiveVisualRow::new(0x0154, 0x36),
    MutantDiveVisualRow::new(0x0184, 0x37),
    MutantDiveVisualRow::new(0x01B4, 0x37),
    MutantDiveVisualRow::new(0x01E4, 0x37),
    MutantDiveVisualRow::new(0x0214, 0x37),
    MutantDiveVisualRow::new(0x0244, 0x36),
    MutantDiveVisualRow::new(0x0274, 0x36),
    MutantDiveVisualRow::new(0x02A4, 0x36),
    MutantDiveVisualRow::new(0x02D4, 0x35),
    MutantDiveVisualRow::new(0x0304, 0x34),
    MutantDiveVisualRow::new(0x0334, 0x34),
    MutantDiveVisualRow::new(0x0364, 0x32),
    MutantDiveVisualRow::new(0x0394, 0x31),
    MutantDiveVisualRow::new(0x03C4, 0x30),
    MutantDiveVisualRow::new(0x03F4, 0x2F),
    MutantDiveVisualRow::new(0x0424, 0x2F),
    MutantDiveVisualRow::new(0x0454, 0x2E),
    MutantDiveVisualRow::new(0x0484, 0x2D),
    MutantDiveVisualRow::new(0x04B4, 0x2C),
    MutantDiveVisualRow::new(0x04E4, 0x2B),
    MutantDiveVisualRow::new(0x0514, 0x2C),
    MutantDiveVisualRow::new(0x0544, 0x2B),
    MutantDiveVisualRow::new(0x0574, 0x2B),
    MutantDiveVisualRow::new(0x05A4, 0x2B),
    MutantDiveVisualRow::new(0x05D4, 0x2B),
    MutantDiveVisualRow::new(0x0604, 0x2A),
    MutantDiveVisualRow::new(0x0634, 0x2C),
    MutantDiveVisualRow::new(0x0664, 0x2C),
    MutantDiveVisualRow::new(0x0694, 0x2D),
    MutantDiveVisualRow::new(0x06C4, 0x2B),
    MutantDiveVisualRow::new(0x06F4, 0x2B),
    MutantDiveVisualRow::new(0x0724, 0x2A),
    MutantDiveVisualRow::new(0x0754, 0x2C),
];
