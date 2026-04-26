//! Live terminal runner for the new core.

use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event};

use crate::{
    input::{InputMapper, InputProfile, PolledInput, XyzzyOverlay},
    kitty::KittyGraphics,
    machine::{ArcadeMachine, CompatibilityState},
    terminal::{TerminalSession, geometry},
    video::Renderer,
};

const FRAME_DURATION: Duration = Duration::from_micros(16_639);

pub fn run_live(_play_audio: bool, input_profile: InputProfile) -> Result<()> {
    ensure_interactive_terminal()?;
    KittyGraphics::ensure_supported()?;

    let mut stdout = io::stdout();
    let _terminal = TerminalSession::enter(&mut stdout)?;
    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);
    let mut input_mapper = InputMapper::new(input_profile);
    let mut xyzzy = XyzzyOverlay::default();
    let mut machine = ArcadeMachine::new();

    loop {
        let frame_started = Instant::now();
        sync_terminal_geometry(&mut terminal_geometry, &mut renderer, &mut graphics)?;

        let input = poll_input(&mut input_mapper)?;
        if input.quit_requested {
            break;
        }

        xyzzy.handle_typed_chars(&input.typed_chars);
        machine.set_compatibility(CompatibilityState {
            xyzzy_active: xyzzy.active(),
            xyzzy_invincible: xyzzy.invincible(),
            xyzzy_auto_fire: xyzzy.auto_fire(),
        });
        let output = machine.step(input.cabinet);
        let image = renderer.render_scaffold(output.snapshot);
        graphics
            .draw_frame(&mut stdout, image)
            .context("drawing kitty graphics frame")?;
        stdout.flush().context("flushing kitty graphics frame")?;

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    graphics.clear(&mut stdout)?;
    stdout.flush().context("flushing kitty clear escape")?;
    Ok(())
}

fn ensure_interactive_terminal() -> Result<()> {
    validate_interactive_terminal(io::stdin().is_terminal(), io::stdout().is_terminal())
}

fn validate_interactive_terminal(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Result<()> {
    if stdin_is_terminal && stdout_is_terminal {
        Ok(())
    } else {
        bail!(
            "live mode requires an interactive terminal; run `cargo run` in a real terminal window"
        )
    }
}

fn sync_terminal_geometry(
    terminal_geometry: &mut crate::terminal::TerminalGeometry,
    renderer: &mut Renderer,
    graphics: &mut KittyGraphics,
) -> Result<()> {
    let latest_geometry = geometry()?;
    if latest_geometry != *terminal_geometry {
        *terminal_geometry = latest_geometry;
        renderer.resize(*terminal_geometry);
        graphics.resize(terminal_geometry.cols, terminal_geometry.rows);
    }
    Ok(())
}

fn poll_input(input_mapper: &mut InputMapper) -> Result<PolledInput> {
    let mut input = PolledInput::default();
    while event::poll(Duration::ZERO)? {
        if let Event::Key(key_event) = event::read()? {
            input_mapper.handle_key_event(key_event, &mut input);
        }
    }
    input_mapper.apply_held(&mut input);
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::{FRAME_DURATION, validate_interactive_terminal};

    #[test]
    fn live_mode_rejects_non_interactive_terminal() {
        let error = validate_interactive_terminal(false, true).expect_err("terminal guard");
        assert!(error.to_string().contains("interactive terminal"));
    }

    #[test]
    fn frame_duration_tracks_cabinet_refresh_not_old_ninety_ms_tick() {
        assert!(FRAME_DURATION.as_millis() < 20);
    }
}
