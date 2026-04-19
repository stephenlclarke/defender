//! Builds runtime gameplay sprites from the red-label `defb6.src` picture tables.
//!
//! The live renderer no longer loads cropped gameplay PNGs for object art.
//! Instead it decodes the cabinet picture descriptors and primary picture bytes
//! extracted from `defb6.src`, then applies the original `CRTAB` / `TCTAB`
//! color families in Rust.

use std::sync::{Arc, OnceLock};

use crate::{
    game::{Entity, EntityKind, HorizontalDirection},
    object_rom::{
        bomb_palette, bomber_palette, cycler_palette, human_palette, lander_palette,
        player_shot_palette, render_picture, score_250_palette, score_500_palette, ship_palette,
        swarmer_palette, tie_palette,
    },
    object_rom_data::{
        ASTP1, ASTP2, ASTP3, ASTP4, BMBP1, BMBP2, BXPIC, C5P1, C25P1, LASP1, LNDP1, LNDP2, LNDP3,
        PLAMIN, PLAPIC, PLBPIC, PRBP1, SCZP1, SWPIC1, SWXP1, TIEP1, TIEP2, TIEP3, TIEP4, UFOP1,
        UFOP2, UFOP3,
    },
    video::RenderedImage,
};

#[derive(Clone, Debug)]
pub struct ArcadeSprites {
    ship_right: Arc<RenderedImage>,
    ship_left: Arc<RenderedImage>,
    little_ship: Arc<RenderedImage>,
    player_shot: Arc<RenderedImage>,
    enemy_shots: [Arc<RenderedImage>; 2],
    humans: [Arc<RenderedImage>; 4],
    landers: [Arc<RenderedImage>; 3],
    mutants: [Arc<RenderedImage>; 4],
    baiters: [Arc<RenderedImage>; 4],
    bombers: [Arc<RenderedImage>; 3],
    pods: [Arc<RenderedImage>; 4],
    swarmer: Arc<RenderedImage>,
    mines: [Arc<RenderedImage>; 2],
    pod_explosions: [Arc<RenderedImage>; 4],
    swarmer_explosion: Arc<RenderedImage>,
    score_250: [Arc<RenderedImage>; 3],
    score_500: [Arc<RenderedImage>; 3],
}

pub fn arcade_sprites() -> &'static ArcadeSprites {
    static SPRITES: OnceLock<ArcadeSprites> = OnceLock::new();
    SPRITES.get_or_init(ArcadeSprites::new)
}

impl ArcadeSprites {
    fn new() -> Self {
        Self {
            ship_right: render(&PLAPIC, ship_palette()),
            ship_left: render(&PLBPIC, ship_palette()),
            little_ship: render(&PLAMIN, ship_palette()),
            player_shot: render(&LASP1, player_shot_palette()),
            enemy_shots: [
                render(&BMBP1, bomb_palette(0)),
                render(&BMBP2, bomb_palette(1)),
            ],
            humans: [
                render(&ASTP1, human_palette()),
                render(&ASTP2, human_palette()),
                render(&ASTP3, human_palette()),
                render(&ASTP4, human_palette()),
            ],
            landers: [
                render(&LNDP1, lander_palette()),
                render(&LNDP2, lander_palette()),
                render(&LNDP3, lander_palette()),
            ],
            mutants: [
                render(&TIEP1, tie_palette(0)),
                render(&TIEP2, tie_palette(1)),
                render(&TIEP3, tie_palette(2)),
                render(&TIEP4, tie_palette(0)),
            ],
            baiters: [
                render(&SCZP1, cycler_palette(0)),
                render(&SCZP1, cycler_palette(1)),
                render(&SCZP1, cycler_palette(2)),
                render(&SCZP1, cycler_palette(3)),
            ],
            bombers: [
                render(&UFOP1, bomber_palette()),
                render(&UFOP2, bomber_palette()),
                render(&UFOP3, bomber_palette()),
            ],
            pods: [
                render(&PRBP1, cycler_palette(0)),
                render(&PRBP1, cycler_palette(1)),
                render(&PRBP1, cycler_palette(2)),
                render(&PRBP1, cycler_palette(3)),
            ],
            swarmer: render(&SWPIC1, swarmer_palette()),
            mines: [
                render(&BMBP1, bomb_palette(2)),
                render(&BMBP2, bomb_palette(3)),
            ],
            pod_explosions: [
                render(&BXPIC, cycler_palette(0)),
                render(&BXPIC, cycler_palette(1)),
                render(&BXPIC, cycler_palette(2)),
                render(&BXPIC, cycler_palette(3)),
            ],
            swarmer_explosion: render(&SWXP1, swarmer_palette()),
            score_250: [
                render(&C25P1, score_250_palette(0)),
                render(&C25P1, score_250_palette(1)),
                render(&C25P1, score_250_palette(2)),
            ],
            score_500: [
                render(&C5P1, score_500_palette(0)),
                render(&C5P1, score_500_palette(1)),
                render(&C5P1, score_500_palette(2)),
            ],
        }
    }

    pub fn sprite_for_entity(
        &self,
        entity: &Entity,
        tick: u32,
        player_facing: HorizontalDirection,
    ) -> Arc<RenderedImage> {
        match entity.kind {
            EntityKind::PlayerShip => match player_facing {
                HorizontalDirection::Right => self.ship_right.clone(),
                HorizontalDirection::Left => self.ship_left.clone(),
            },
            EntityKind::PlayerShot => self.player_shot.clone(),
            EntityKind::EnemyShot => {
                self.enemy_shots[rom_cycle_index(tick, 2, self.enemy_shots.len())].clone()
            }
            EntityKind::Human => self.humans[rom_cycle_index(tick, 8, self.humans.len())].clone(),
            EntityKind::Lander => {
                self.landers[rom_cycle_index(tick, 5, self.landers.len())].clone()
            }
            EntityKind::Mutant => {
                self.mutants[rom_cycle_index(tick, 8, self.mutants.len())].clone()
            }
            EntityKind::Baiter => {
                self.baiters[rom_cycle_index(tick, 6, self.baiters.len())].clone()
            }
            EntityKind::Bomber => {
                self.bombers[rom_cycle_index(tick, 6, self.bombers.len())].clone()
            }
            EntityKind::Pod => self.pods[rom_cycle_index(tick, 6, self.pods.len())].clone(),
            EntityKind::Swarmer => self.swarmer.clone(),
            EntityKind::Mine => self.mines[rom_cycle_index(tick, 3, self.mines.len())].clone(),
        }
    }

    pub fn player_stock_icon(&self) -> Arc<RenderedImage> {
        self.little_ship.clone()
    }

    pub fn player_shot(&self) -> Arc<RenderedImage> {
        self.player_shot.clone()
    }

    pub fn pod_explosion(&self, tick: u32) -> Arc<RenderedImage> {
        self.pod_explosions[rom_cycle_index(tick, 3, self.pod_explosions.len())].clone()
    }

    pub fn swarmer_explosion(&self) -> Arc<RenderedImage> {
        self.swarmer_explosion.clone()
    }

    pub fn score_250(&self, tick: u32) -> Arc<RenderedImage> {
        self.score_250[rom_cycle_index(tick, 4, self.score_250.len())].clone()
    }

    pub fn score_500(&self, tick: u32) -> Arc<RenderedImage> {
        self.score_500[rom_cycle_index(tick, 4, self.score_500.len())].clone()
    }
}

fn render(
    picture: &crate::object_rom_data::RomPictureData,
    palette: crate::object_rom::PaletteOverrides,
) -> Arc<RenderedImage> {
    Arc::new(render_picture(picture, palette))
}

fn rom_cycle_index(tick: u32, speed: u32, len: usize) -> usize {
    ((tick / speed.max(1)) as usize) % len.max(1)
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, HorizontalDirection};

    use super::arcade_sprites;

    #[test]
    fn ship_assets_decode_with_pixels() {
        let sprites = arcade_sprites();
        let ship = sprites.ship_right.as_ref();

        assert_eq!(ship.width, 12);
        assert_eq!(ship.height, 8);
        assert!(ship.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn player_ship_uses_distinct_left_and_right_frames() {
        let sprites = arcade_sprites();
        let player = Entity::new(EntityKind::PlayerShip, 12, 8, 0, 0);

        let right = sprites.sprite_for_entity(&player, 0, HorizontalDirection::Right);
        let left = sprites.sprite_for_entity(&player, 0, HorizontalDirection::Left);

        assert_ne!(right.pixels, left.pixels);
    }

    #[test]
    fn player_ship_uses_rom_left_art_not_a_mirror_of_right() {
        let sprites = arcade_sprites();

        assert_ne!(sprites.ship_right.pixels, sprites.ship_left.pixels);
    }

    #[test]
    fn enemy_shots_cycle_across_frames() {
        let sprites = arcade_sprites();
        let shot = Entity::new(EntityKind::EnemyShot, 9, 7, 0, 0);

        let before = sprites.sprite_for_entity(&shot, 0, HorizontalDirection::Right);
        let after = sprites.sprite_for_entity(&shot, 6, HorizontalDirection::Right);

        assert_ne!(before.pixels, after.pixels);
    }

    #[test]
    fn player_ship_frame_is_stable_across_horizontal_positions() {
        let sprites = arcade_sprites();
        let player_a = Entity::new(EntityKind::PlayerShip, 12, 8, 0, 0);
        let player_b = Entity::new(EntityKind::PlayerShip, 13, 8, 0, 0);

        let frame_a = sprites.sprite_for_entity(&player_a, 0, HorizontalDirection::Right);
        let frame_b = sprites.sprite_for_entity(&player_b, 0, HorizontalDirection::Right);

        assert_eq!(frame_a.pixels, frame_b.pixels);
    }

    #[test]
    fn player_stock_icon_decodes_with_pixels() {
        let icon = arcade_sprites().player_stock_icon();

        assert!(icon.width > 0);
        assert!(icon.height > 0);
        assert!(icon.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }

    #[test]
    fn bonus_score_art_decodes_with_pixels() {
        let sprites = arcade_sprites();

        for image in [
            sprites.score_500(0),
            sprites.score_500(6),
            sprites.score_250(0),
            sprites.score_250(6),
        ] {
            assert!(image.width > 0);
            assert!(image.height > 0);
            assert!(image.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0));
        }
    }

    #[test]
    fn human_animation_uses_distinct_rom_frames() {
        let sprites = arcade_sprites();
        let human = Entity::new(EntityKind::Human, 12, 8, 0, 0);

        let first = sprites.sprite_for_entity(&human, 0, HorizontalDirection::Right);
        let later = sprites.sprite_for_entity(&human, 24, HorizontalDirection::Right);

        assert_ne!(first.pixels, later.pixels);
    }
}
