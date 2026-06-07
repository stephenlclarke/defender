use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use defender::{
    GamePhase, SurfaceSize,
    actor_game::{ActorRuntimeAdapter, GameInput},
    render_scene_to_rgba,
    renderer::SceneRaster,
};
use gif::{Encoder, Frame, Repeat};

const GAMEPLAY_TARGET: SurfaceSize = SurfaceSize::new(3456, 1864);
const ATTRACT_TARGET: SurfaceSize = SurfaceSize::new(768, 576);
const ATTRACT_SEQUENCE_STEPS: u16 = 3367;
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

    for step in 0..900_u16 {
        let input = gameplay_input(step, playing_steps);
        let frame = runtime.step(input);
        if frame.state.phase == GamePhase::Playing {
            playing_steps += 1;
            if playing_steps >= 180 {
                return render_scene_to_rgba(&frame.scene, GAMEPLAY_TARGET)
                    .context("rasterizing gameplay scene");
            }
        }
    }

    bail!("actor runtime did not reach a stable gameplay frame for README media")
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
    if (24..=220).contains(&playing_steps) && playing_steps.is_multiple_of(8) {
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
