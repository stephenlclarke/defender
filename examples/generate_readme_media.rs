use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use defender::{
    Color, GamePhase, RenderLayer, RenderScene, SpriteId, SurfaceSize,
    actor_game::{ActorFrame, ActorRuntimeAdapter, GameInput},
    render_scene_to_rgba,
    renderer::SceneRaster,
};
use gif::{Encoder, Frame, Repeat};

const GAMEPLAY_TARGET: SurfaceSize = SurfaceSize::new(3456, 1864);
const ATTRACT_TARGET: SurfaceSize = SurfaceSize::new(768, 576);
const GAMEPLAY_SHOWCASE_SEARCH_STEPS: u16 = 2_400;
const GAMEPLAY_SHOWCASE_MIN_PLAYING_STEPS: u16 = 180;
const GAMEPLAY_SHOWCASE_TERRAIN_TINT: Color = Color::from_rgba(174, 81, 0, 255);
const ATTRACT_SEQUENCE_STEPS: u16 = 3479;
const ATTRACT_SAMPLE_INTERVAL_STEPS: u16 = 12;
const ATTRACT_FRAME_DELAY_CENTISECONDS: u16 = 20;

#[derive(Debug, Default)]
struct MediaArgs {
    gameplay_path: Option<PathBuf>,
    attract_path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = parse_args()?;
    let generate_both = args.gameplay_path.is_none() && args.attract_path.is_none();

    if generate_both || args.gameplay_path.is_some() {
        let gameplay_path = args
            .gameplay_path
            .as_deref()
            .unwrap_or_else(|| Path::new("docs/defender.png"));
        write_gameplay_png(gameplay_path)?;
    }
    if generate_both || args.attract_path.is_some() {
        let attract_path = args
            .attract_path
            .as_deref()
            .unwrap_or_else(|| Path::new("docs/start-sequence.gif"));
        write_attract_gif(attract_path)?;
    }

    Ok(())
}

fn parse_args() -> anyhow::Result<MediaArgs> {
    let mut parsed = MediaArgs::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gameplay" => {
                parsed.gameplay_path = Some(require_arg_value(&mut args, "--gameplay")?);
            }
            "--attract" => {
                parsed.attract_path = Some(require_arg_value(&mut args, "--attract")?);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }

    Ok(parsed)
}

fn require_arg_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> anyhow::Result<PathBuf> {
    let Some(value) = args.next() else {
        bail!("{flag} requires an output path");
    };
    if value.starts_with('-') {
        bail!("{flag} output path cannot be another flag: {value}");
    }

    Ok(PathBuf::from(value))
}

fn print_help() {
    println!(
        "Generate committed README media from the accepted actor runtime.\n\
\n\
Usage:\n\
  cargo run --example generate_readme_media\n\
  cargo run --example generate_readme_media -- --gameplay docs/defender.png\n\
  cargo run --example generate_readme_media -- --attract docs/start-sequence.gif\n\
\n\
Options:\n\
  --gameplay <path>  Write the gameplay PNG screenshot\n\
  --attract <path>   Write the attract-sequence GIF\n\
"
    );
}

fn write_gameplay_png(path: &Path) -> anyhow::Result<()> {
    let raster = gameplay_raster()?;
    write_png(path, &raster)?;
    println!(
        "wrote gameplay README image to {} ({}x{})",
        path.display(),
        raster.surface.width,
        raster.surface.height
    );
    Ok(())
}

fn gameplay_raster() -> anyhow::Result<SceneRaster> {
    let mut runtime = ActorRuntimeAdapter::new_with_free_play_admission();
    let mut playing_steps = 0_u16;
    let mut best_scene: Option<(u32, RenderScene)> = None;

    for step in 0..GAMEPLAY_SHOWCASE_SEARCH_STEPS {
        let input = gameplay_input(step, playing_steps);
        let frame = runtime.step(input);
        if frame.state.phase != GamePhase::Playing {
            continue;
        }

        playing_steps += 1;
        if playing_steps < GAMEPLAY_SHOWCASE_MIN_PLAYING_STEPS {
            continue;
        }

        let Some(score) = gameplay_showcase_score(&frame) else {
            continue;
        };
        let should_replace = match best_scene.as_ref() {
            Some((best_score, _)) => score > *best_score,
            None => true,
        };
        if should_replace {
            best_scene = Some((score, frame.scene.clone()));
        }
    }

    let Some((_, scene)) = best_scene else {
        bail!("actor runtime did not produce a gameplay showcase frame for README media")
    };
    let scene = gameplay_showcase_scene(scene);

    render_scene_to_rgba(&scene, GAMEPLAY_TARGET).context("rasterizing gameplay scene")
}

fn gameplay_showcase_scene(mut scene: RenderScene) -> RenderScene {
    for sprite in &mut scene.sprites {
        if sprite.layer == RenderLayer::Terrain
            && matches!(
                sprite.sprite,
                SpriteId::TERRAIN_TILE | SpriteId::TERRAIN_TILE_ALT
            )
        {
            sprite.tint = GAMEPLAY_SHOWCASE_TERRAIN_TINT;
        }
    }

    scene
}

fn gameplay_showcase_score(frame: &ActorFrame) -> Option<u32> {
    if frame.state.phase != GamePhase::Playing {
        return None;
    }

    let counts = ShowcaseSpriteCounts::from_scene(&frame.scene);
    counts.score()
}

#[derive(Debug, Default)]
struct ShowcaseSpriteCounts {
    has_player: bool,
    terrain_tiles: u32,
    lower_playfield_terrain_tiles: u32,
    humans: u32,
    hostile_aliens: u32,
    projectiles: u32,
    explosions: u32,
    score_popups: u32,
}

impl ShowcaseSpriteCounts {
    fn from_scene(scene: &RenderScene) -> Self {
        let mut counts = Self::default();
        for sprite in &scene.sprites {
            match sprite.sprite {
                SpriteId::PLAYER_SHIP | SpriteId::PLAYER_SHIP_LEFT => counts.has_player = true,
                SpriteId::TERRAIN_TILE | SpriteId::TERRAIN_TILE_ALT => {
                    counts.terrain_tiles += 1;
                    if sprite.layer == RenderLayer::Terrain && sprite.position[1] >= 170.0 {
                        counts.lower_playfield_terrain_tiles += 1;
                    }
                }
                SpriteId::HUMAN => counts.humans += 1,
                SpriteId::ENEMY_LANDER
                | SpriteId::ENEMY_MUTANT
                | SpriteId::ENEMY_BAITER
                | SpriteId::ENEMY_BOMBER
                | SpriteId::ENEMY_POD
                | SpriteId::ENEMY_SWARMER => counts.hostile_aliens += 1,
                SpriteId::PLAYER_PROJECTILE | SpriteId::ENEMY_BOMB => counts.projectiles += 1,
                SpriteId::BOMB_EXPLOSION
                | SpriteId::SWARMER_EXPLOSION
                | SpriteId::ASTRONAUT_EXPLOSION
                | SpriteId::TERRAIN_EXPLOSION => counts.explosions += 1,
                SpriteId::SCORE_POPUP_250 | SpriteId::SCORE_POPUP_500 => counts.score_popups += 1,
                _ => {}
            }
        }

        counts
    }

    fn score(&self) -> Option<u32> {
        if !self.has_player
            || self.terrain_tiles < 16
            || self.lower_playfield_terrain_tiles < 8
            || self.humans < 2
            || self.hostile_aliens < 2
        {
            return None;
        }

        Some(
            100 + self.terrain_tiles.min(120)
                + self.humans * 40
                + self.hostile_aliens * 90
                + self.projectiles * 35
                + self.explosions * 50
                + self.score_popups * 25,
        )
    }
}

fn gameplay_input(step: u16, playing_steps: u16) -> GameInput {
    let mut input = GameInput::NONE;
    if step == 0 {
        input.start_one = true;
        return input;
    }

    if playing_steps > 20 {
        input.thrust = true;
    }
    if (24..=520).contains(&playing_steps) && playing_steps.is_multiple_of(8) {
        input.fire = true;
    }
    if (36..=88).contains(&playing_steps) {
        input.altitude_up = true;
    }
    if playing_steps == 116 {
        input.reverse = true;
    }
    if (124..=168).contains(&playing_steps) {
        input.altitude_down = true;
        input.thrust = true;
    }
    if playing_steps == 360 {
        input.reverse = true;
    }
    if (372..=430).contains(&playing_steps) {
        input.altitude_up = true;
    }
    if (456..=520).contains(&playing_steps) {
        input.altitude_down = true;
    }

    input
}

fn write_attract_gif(path: &Path) -> anyhow::Result<()> {
    prepare_parent_dir(path)?;
    let mut output =
        File::create(path).with_context(|| format!("creating attract GIF {}", path.display()))?;
    let mut encoder = Encoder::new(
        &mut output,
        ATTRACT_TARGET
            .width
            .try_into()
            .context("attract GIF width fits u16")?,
        ATTRACT_TARGET
            .height
            .try_into()
            .context("attract GIF height fits u16")?,
        &[],
    )
    .context("creating attract GIF encoder")?;
    encoder
        .set_repeat(Repeat::Infinite)
        .context("configuring attract GIF loop")?;

    let mut runtime = ActorRuntimeAdapter::new();
    let mut written = 0_u16;
    for step in 0..ATTRACT_SEQUENCE_STEPS {
        let frame = runtime.step(GameInput::NONE);
        if step % ATTRACT_SAMPLE_INTERVAL_STEPS != 0 {
            continue;
        }

        let raster = render_scene_to_rgba(&frame.scene, ATTRACT_TARGET)
            .context("rasterizing attract scene")?;
        write_gif_frame(&mut encoder, raster)?;
        written += 1;
    }

    println!(
        "wrote attract README GIF to {} ({} frames, {}x{})",
        path.display(),
        written,
        ATTRACT_TARGET.width,
        ATTRACT_TARGET.height
    );
    Ok(())
}

fn write_gif_frame<W: std::io::Write>(
    encoder: &mut Encoder<W>,
    raster: SceneRaster,
) -> anyhow::Result<()> {
    let width = raster
        .surface
        .width
        .try_into()
        .context("GIF frame width fits u16")?;
    let height = raster
        .surface
        .height
        .try_into()
        .context("GIF frame height fits u16")?;
    let mut pixels = raster.into_pixels();
    let mut frame = Frame::from_rgba_speed(width, height, &mut pixels, 10);
    frame.delay = ATTRACT_FRAME_DELAY_CENTISECONDS;
    encoder.write_frame(&frame).context("writing GIF frame")
}

fn write_png(path: &Path, raster: &SceneRaster) -> anyhow::Result<()> {
    prepare_parent_dir(path)?;
    let file = File::create(path).with_context(|| format!("creating PNG {}", path.display()))?;
    let mut encoder = png::Encoder::new(file, raster.surface.width, raster.surface.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().context("writing PNG header")?;
    writer
        .write_image_data(raster.pixels())
        .context("writing PNG pixels")
}

fn prepare_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating parent directory {}", parent.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use defender::SceneSprite;

    #[test]
    fn gameplay_showcase_score_requires_a_rich_gameplay_scene() {
        let sparse = scene_with_sprites([
            SpriteId::PLAYER_SHIP,
            SpriteId::TERRAIN_TILE,
            SpriteId::HUMAN,
            SpriteId::ENEMY_LANDER,
        ]);

        assert_eq!(ShowcaseSpriteCounts::from_scene(&sparse).score(), None);

        let mut rich = RenderScene::empty(0, SurfaceSize::new(292, 240));
        rich.sprites.push(test_sprite(SpriteId::PLAYER_SHIP));
        rich.sprites.push(test_sprite(SpriteId::HUMAN));
        rich.sprites.push(test_sprite(SpriteId::HUMAN));
        rich.sprites.push(test_sprite(SpriteId::ENEMY_LANDER));
        rich.sprites.push(test_sprite(SpriteId::ENEMY_BOMBER));
        rich.sprites.push(test_sprite(SpriteId::PLAYER_PROJECTILE));
        for _ in 0..8 {
            rich.sprites.push(test_terrain_sprite([24.0, 180.0]));
            rich.sprites.push(test_terrain_sprite([32.0, 208.0]));
        }

        assert!(ShowcaseSpriteCounts::from_scene(&rich).score().is_some());
    }

    #[test]
    fn gameplay_showcase_scene_tints_only_main_playfield_terrain() {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));
        scene.sprites.push(test_terrain_sprite([20.0, 210.0]));
        scene
            .sprites
            .push(test_sprite(SpriteId::ATTRACT_SCANNER_TERRAIN_PIXEL));
        scene.sprites.push(test_sprite(SpriteId::PLAYER_SHIP));

        let scene = gameplay_showcase_scene(scene);

        assert_eq!(scene.sprites[0].tint, GAMEPLAY_SHOWCASE_TERRAIN_TINT);
        assert_eq!(scene.sprites[1].tint, Color::WHITE);
        assert_eq!(scene.sprites[2].tint, Color::WHITE);
    }

    fn scene_with_sprites<const N: usize>(sprites: [SpriteId; N]) -> RenderScene {
        let mut scene = RenderScene::empty(0, SurfaceSize::new(292, 240));
        for sprite in sprites {
            scene.sprites.push(test_sprite(sprite));
        }

        scene
    }

    fn test_sprite(sprite: SpriteId) -> SceneSprite {
        SceneSprite {
            sprite,
            layer: RenderLayer::Objects,
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            tint: Color::WHITE,
        }
    }

    fn test_terrain_sprite(position: [f32; 2]) -> SceneSprite {
        SceneSprite {
            sprite: SpriteId::TERRAIN_TILE,
            layer: RenderLayer::Terrain,
            position,
            size: [4.0, 1.0],
            tint: Color::WHITE,
        }
    }
}
