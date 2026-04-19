//! Loads embedded arcade sprite assets for the Kitty renderer.
//!
//! The object art in `assets/arcade/*.png` is cropped from the red-label
//! Defender sprite rip published by Sean Riddle's Williams graphics ripper
//! work, then bundled into the app with `include_bytes!`.

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
    ships_right: [Arc<RenderedImage>; 4],
    ships_left: [Arc<RenderedImage>; 4],
    player_shot: Arc<RenderedImage>,
    enemy_shots: [Arc<RenderedImage>; 4],
    human: Arc<RenderedImage>,
    landers: [Arc<RenderedImage>; 6],
    mutants: [Arc<RenderedImage>; 2],
    baiters: [Arc<RenderedImage>; 6],
    bombers: [Arc<RenderedImage>; 8],
    pods: [Arc<RenderedImage>; 2],
    swarmers: [Arc<RenderedImage>; 2],
    mine: Arc<RenderedImage>,
    pod_explosion: Arc<RenderedImage>,
    swarmer_explosion: Arc<RenderedImage>,
}

pub fn arcade_sprites() -> &'static ArcadeSprites {
    static SPRITES: OnceLock<ArcadeSprites> = OnceLock::new();
    SPRITES.get_or_init(ArcadeSprites::new)
}

impl ArcadeSprites {
    fn new() -> Self {
        let ships_right = [
            load_embedded_png(include_bytes!("../assets/arcade/ship1.png")),
            load_embedded_png(include_bytes!("../assets/arcade/ship2.png")),
            load_embedded_png(include_bytes!("../assets/arcade/ship3.png")),
            load_embedded_png(include_bytes!("../assets/arcade/ship4.png")),
        ];
        let ships_left = std::array::from_fn(|index| Arc::new(mirror(ships_right[index].as_ref())));

        Self {
            ships_right,
            ships_left,
            player_shot: load_embedded_png(include_bytes!("../assets/arcade/player-shot.png")),
            enemy_shots: [
                load_embedded_png(include_bytes!("../assets/arcade/bomb1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomb2.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomb3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomb4.png")),
            ],
            human: load_embedded_png(include_bytes!("../assets/arcade/humanoid1.png")),
            landers: [
                load_embedded_png(include_bytes!("../assets/arcade/lander1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/lander2.png")),
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
                load_embedded_png(include_bytes!("../assets/arcade/baiter2.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter4.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter5.png")),
                load_embedded_png(include_bytes!("../assets/arcade/baiter6.png")),
            ],
            bombers: [
                load_embedded_png(include_bytes!("../assets/arcade/bomber1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber2.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber3.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber4.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber5.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber6.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber7.png")),
                load_embedded_png(include_bytes!("../assets/arcade/bomber8.png")),
            ],
            pods: [
                load_embedded_png(include_bytes!("../assets/arcade/pod1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/pod2.png")),
            ],
            swarmers: [
                load_embedded_png(include_bytes!("../assets/arcade/swarmer1.png")),
                load_embedded_png(include_bytes!("../assets/arcade/swarmer2.png")),
            ],
            mine: load_embedded_png(include_bytes!("../assets/arcade/mine1.png")),
            pod_explosion: load_embedded_png(include_bytes!("../assets/arcade/podexpl.png")),
            swarmer_explosion: load_embedded_png(include_bytes!("../assets/arcade/swarmexpl.png")),
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
                HorizontalDirection::Right => self.ships_right[player_phase(tick)].clone(),
                HorizontalDirection::Left => self.ships_left[player_phase(tick)].clone(),
            },
            EntityKind::PlayerShot => self.player_shot.clone(),
            EntityKind::EnemyShot => self.enemy_shots[phase_index(entity, tick, 2, 4)].clone(),
            EntityKind::Human => self.human.clone(),
            EntityKind::Lander => self.landers[phase_index(entity, tick, 5, 6)].clone(),
            EntityKind::Mutant => self.mutants[phase_index(entity, tick, 8, 2)].clone(),
            EntityKind::Baiter => self.baiters[phase_index(entity, tick, 4, 6)].clone(),
            EntityKind::Bomber => self.bombers[phase_index(entity, tick, 4, 8)].clone(),
            EntityKind::Pod => self.pods[phase_index(entity, tick, 10, 2)].clone(),
            EntityKind::Swarmer => self.swarmers[phase_index(entity, tick, 6, 2)].clone(),
            EntityKind::Mine => self.mine.clone(),
        }
    }

    pub fn pod_explosion(&self) -> Arc<RenderedImage> {
        self.pod_explosion.clone()
    }

    pub fn swarmer_explosion(&self) -> Arc<RenderedImage> {
        self.swarmer_explosion.clone()
    }
}

fn player_phase(tick: u32) -> usize {
    ((tick / 5) as usize) % 4
}

fn phase_index(entity: &Entity, tick: u32, speed: u32, len: usize) -> usize {
    let phase_seed = i64::from(entity.position.x) * 31 + i64::from(entity.position.y) * 17;
    (((tick / speed.max(1)) as i64) + phase_seed).rem_euclid(len as i64) as usize
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

fn mirror(image: &RenderedImage) -> RenderedImage {
    let mut mirrored = vec![0; image.pixels.len()];
    for y in 0..image.height {
        for x in 0..image.width {
            let src = ((y * image.width + x) * 4) as usize;
            let dst_x = image.width - 1 - x;
            let dst = ((y * image.width + dst_x) * 4) as usize;
            mirrored[dst..dst + 4].copy_from_slice(&image.pixels[src..src + 4]);
        }
    }

    RenderedImage {
        width: image.width,
        height: image.height,
        pixels: mirrored,
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, HorizontalDirection};

    use super::arcade_sprites;

    #[test]
    fn ship_assets_decode_with_pixels() {
        let sprites = arcade_sprites();
        let ship = sprites.ships_right[0].as_ref();

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
    fn enemy_shots_cycle_across_frames() {
        let sprites = arcade_sprites();
        let shot = Entity::new(EntityKind::EnemyShot, 9, 7, 0, 0);

        let before = sprites.sprite_for_entity(&shot, 0, HorizontalDirection::Right);
        let after = sprites.sprite_for_entity(&shot, 6, HorizontalDirection::Right);

        assert_ne!(before.pixels, after.pixels);
    }
}
