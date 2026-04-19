//! Loads embedded arcade sprite assets for the Kitty renderer.
//!
//! The object art in `assets/arcade/*.png` is cropped from the red-label
//! Defender sprite rip published by Sean Riddle's Williams graphics ripper
//! work, then bundled into the app with `include_bytes!`. The runtime maps
//! those assets onto the original `defb6.src` picture families (`PLAPIC`,
//! `PLBPIC`, `TIEP*`, `UFOP*`, `LNDP*`, and so on) rather than inventing extra
//! mirrored frames at draw time.

use std::{
    io::Cursor,
    sync::{Arc, OnceLock},
};

use anyhow::Context;
use png::{ColorType, Decoder, Transformations};

use crate::{
    game::{Entity, EntityKind, HorizontalDirection},
    video::RenderedImage,
};

#[derive(Clone, Debug)]
pub struct ArcadeSprites {
    ship_right: Arc<RenderedImage>,
    ship_left: Arc<RenderedImage>,
    little_ship: Arc<RenderedImage>,
    player_shot: Arc<RenderedImage>,
    enemy_shots: [Arc<RenderedImage>; 2],
    human: Arc<RenderedImage>,
    landers: [Arc<RenderedImage>; 5],
    mutants: [Arc<RenderedImage>; 2],
    baiters: [Arc<RenderedImage>; 3],
    bombers: [Arc<RenderedImage>; 6],
    pod: Arc<RenderedImage>,
    swarmer: Arc<RenderedImage>,
    mine: Arc<RenderedImage>,
    pod_explosion: Arc<RenderedImage>,
    swarmer_explosion: Arc<RenderedImage>,
    score_250: [Arc<RenderedImage>; 2],
    score_500: [Arc<RenderedImage>; 2],
}

pub fn arcade_sprites() -> &'static ArcadeSprites {
    static SPRITES: OnceLock<ArcadeSprites> = OnceLock::new();
    SPRITES.get_or_init(ArcadeSprites::new)
}

impl ArcadeSprites {
    fn new() -> Self {
        Self {
            // `PLAPIC` and `PLBPIC` are distinct right/left cabinet pictures.
            ship_right: load_embedded_png(include_bytes!("../assets/arcade/ship1.png")),
            ship_left: load_embedded_png(include_bytes!("../assets/arcade/ship3.png")),
            little_ship: load_embedded_png(include_bytes!("../assets/arcade/littleship.png")),
            player_shot: load_embedded_png(include_bytes!("../assets/arcade/player-shot.png")),
            enemy_shots: [
                load_embedded_png(include_bytes!("../assets/arcade/bomb1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomb3.png")),
            ],
            human: load_embedded_png(include_bytes!("../assets/arcade/humanoid1.png")),
            landers: [
                load_embedded_png(include_bytes!("../assets/arcade/lander1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/lander3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/lander4.png")),
                load_embedded_png(include_bytes!("../assets/arcade/lander5.png")),
                load_embedded_png(include_bytes!("../assets/arcade/lander6.png")),
            ],
            mutants: [
                load_embedded_png(include_bytes!("../assets/arcade/mutant1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/mutant2.png")),
            ],
            baiters: [
                load_embedded_png(include_bytes!("../assets/arcade/baiter1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter5.png")),
            ],
            bombers: [
                load_embedded_png(include_bytes!("../assets/arcade/bomber1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber5.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber6.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber7.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber8.png")),
            ],
            pod: load_embedded_png(include_bytes!("../assets/arcade/pod1.png")),
            swarmer: load_embedded_png(include_bytes!("../assets/arcade/swarmer1.png")),
            mine: load_embedded_png(include_bytes!("../assets/arcade/mine1.png")),
            pod_explosion: load_embedded_png(include_bytes!("../assets/arcade/podexpl.png")),
            swarmer_explosion: load_embedded_png(include_bytes!("../assets/arcade/swarmexpl.png")),
            score_250: [
                load_embedded_png(include_bytes!("../assets/arcade/score250_1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/score250_2.png")),
            ],
            score_500: [
                load_embedded_png(include_bytes!("../assets/arcade/score500_1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/score500_2.png")),
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
            // `POUT` / `POUT1` only swap the player between the right-facing
            // `PLAPIC` and left-facing `PLBPIC` pictures.
            EntityKind::PlayerShip => match player_facing {
                HorizontalDirection::Right => self.ship_right.clone(),
                HorizontalDirection::Left => self.ship_left.clone(),
            },
            EntityKind::PlayerShot => self.player_shot.clone(),
            EntityKind::EnemyShot => self.enemy_shots[rom_cycle_index(tick, 2, 2)].clone(),
            EntityKind::Human => self.human.clone(),
            // These visible phases now follow the distinct cabinet picture
            // families from `defb6.src` instead of the older duplicated PNG
            // buckets. The gameplay model still does not preserve raw `OPICT`,
            // so the runtime advances through the family on a shared cadence.
            EntityKind::Lander => self.landers[rom_cycle_index(tick, 5, 5)].clone(),
            EntityKind::Mutant => self.mutants[rom_cycle_index(tick, 8, 2)].clone(),
            EntityKind::Baiter => self.baiters[rom_cycle_index(tick, 4, 3)].clone(),
            EntityKind::Bomber => self.bombers[rom_cycle_index(tick, 4, 6)].clone(),
            EntityKind::Pod => self.pod.clone(),
            EntityKind::Swarmer => self.swarmer.clone(),
            EntityKind::Mine => self.mine.clone(),
        }
    }

    pub fn player_stock_icon(&self) -> Arc<RenderedImage> {
        self.little_ship.clone()
    }

    pub fn player_shot(&self) -> Arc<RenderedImage> {
        self.player_shot.clone()
    }

    pub fn pod_explosion(&self) -> Arc<RenderedImage> {
        self.pod_explosion.clone()
    }

    pub fn swarmer_explosion(&self) -> Arc<RenderedImage> {
        self.swarmer_explosion.clone()
    }

    pub fn score_250(&self, tick: u32) -> Arc<RenderedImage> {
        self.score_250[rom_cycle_index(tick, 5, 2)].clone()
    }

    pub fn score_500(&self, tick: u32) -> Arc<RenderedImage> {
        self.score_500[rom_cycle_index(tick, 5, 2)].clone()
    }
}

fn rom_cycle_index(tick: u32, speed: u32, len: usize) -> usize {
    ((tick / speed.max(1)) as usize) % len.max(1)
}

fn load_embedded_png(bytes: &'static [u8]) -> Arc<RenderedImage> {
    Arc::new(decode_png_image(bytes).expect("embedded arcade sprite should decode"))
}

fn decode_png_image(bytes: &[u8]) -> anyhow::Result<RenderedImage> {
    let cursor = Cursor::new(bytes);
    let mut decoder = Decoder::new(cursor);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info().context("reading embedded png header")?;
    let out_size = reader
        .output_buffer_size()
        .expect("expanded PNG should report an output size");
    let mut buffer = vec![0; out_size];
    let info = reader
        .next_frame(&mut buffer)
        .context("decoding embedded png frame")?;
    let pixels = &buffer[..info.buffer_size()];

    let mut rgba = Vec::with_capacity(info.width as usize * info.height as usize * 4);
    match info.color_type {
        ColorType::Rgba => rgba.extend_from_slice(pixels),
        ColorType::Rgb => {
            for chunk in pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        ColorType::GrayscaleAlpha => {
            for chunk in pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        ColorType::Grayscale => {
            for value in pixels {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        ColorType::Indexed => unreachable!("indexed pngs are expanded before decoding"),
    }

    Ok(RenderedImage {
        width: info.width,
        height: info.height,
        pixels: rgba,
    })
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, HorizontalDirection};

    use super::arcade_sprites;

    #[test]
    fn ship_assets_decode_with_pixels() {
        let sprites = arcade_sprites();
        let ship = sprites.ship_right.as_ref();

        assert!(ship.width > 0);
        assert!(ship.height > 0);
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
    fn player_ship_uses_embedded_left_art_not_a_mirror_of_right() {
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
}
