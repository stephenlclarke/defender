//! Live terminal runner for the new core.

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use crossterm::event::{self, Event};

use crate::{
    cmos_storage::{CmosStorage, FileCmosStorage},
    input::{InputMapper, InputProfile, PolledInput, XyzzyOverlay},
    kitty::KittyGraphics,
    machine::{ArcadeMachine, CompatibilityState},
    terminal::{TerminalSession, geometry},
    video::Renderer,
};

const FRAME_DURATION: Duration = Duration::from_micros(16_639);

pub fn run_live(
    _play_audio: bool,
    input_profile: InputProfile,
    cmos_path: Option<&Path>,
) -> Result<()> {
    ensure_interactive_terminal()?;
    KittyGraphics::ensure_supported()?;

    let mut stdout = io::stdout();
    let _terminal = TerminalSession::enter(&mut stdout)?;
    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);
    let mut input_mapper = InputMapper::new(input_profile);
    let mut xyzzy = XyzzyOverlay::default();
    let cmos_storage = cmos_path.map(FileCmosStorage::new);
    let storage = cmos_storage
        .as_ref()
        .map(|storage| storage as &dyn CmosStorage);
    let mut machine = live_machine_from_cmos_storage(storage)?;

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
        let output = machine.step_with_typed_chars(input.cabinet, &input.typed_chars);
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
    save_live_cmos(
        cmos_storage
            .as_ref()
            .map(|storage| storage as &dyn CmosStorage),
        &machine,
    )?;
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

fn live_machine_from_cmos_storage(storage: Option<&dyn CmosStorage>) -> Result<ArcadeMachine> {
    let Some(storage) = storage else {
        return Ok(ArcadeMachine::new());
    };

    let Some(cmos) = storage.load_cmos().context("loading persisted CMOS RAM")? else {
        return Ok(ArcadeMachine::new());
    };

    ArcadeMachine::try_new_with_cmos(cmos)
        .map_err(|error| anyhow!("loading persisted CMOS RAM into arcade core: {error}"))
}

fn save_live_cmos(storage: Option<&dyn CmosStorage>, machine: &ArcadeMachine) -> Result<()> {
    let Some(storage) = storage else {
        return Ok(());
    };

    storage
        .save_cmos(machine.red_label_cmos_ram())
        .context("saving persisted CMOS RAM")
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
    use std::cell::RefCell;
    use std::io;

    use crate::board::{
        CMOS_RAM_SIZE, CmosRam, RED_LABEL_CRHSTD_CELL_OFFSET, cmos_sram_write_byte,
    };
    use crate::cmos_storage::CmosStorage;
    use crate::machine::ArcadeMachine;

    use super::{
        FRAME_DURATION, live_machine_from_cmos_storage, save_live_cmos,
        validate_interactive_terminal,
    };

    #[derive(Default)]
    struct MemoryCmosStorage {
        cmos: RefCell<Option<CmosRam>>,
    }

    impl CmosStorage for MemoryCmosStorage {
        fn load_cmos(&self) -> io::Result<Option<CmosRam>> {
            Ok(*self.cmos.borrow())
        }

        fn save_cmos(&self, cmos: &CmosRam) -> io::Result<()> {
            *self.cmos.borrow_mut() = Some(*cmos);
            Ok(())
        }
    }

    #[test]
    fn live_mode_rejects_non_interactive_terminal() {
        let error = validate_interactive_terminal(false, true).expect_err("terminal guard");
        assert!(error.to_string().contains("interactive terminal"));
    }

    #[test]
    fn frame_duration_tracks_cabinet_refresh_not_old_ninety_ms_tick() {
        assert!(FRAME_DURATION.as_millis() < 20);
    }

    #[test]
    fn live_cmos_storage_loads_and_saves_machine_cmos() {
        let storage = MemoryCmosStorage::default();
        let mut cmos = [0xF0; CMOS_RAM_SIZE];
        let high_score_offset = usize::from(RED_LABEL_CRHSTD_CELL_OFFSET);
        cmos_sram_write_byte(&mut cmos, high_score_offset, 0x21).expect("write score high byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 2, 0x27)
            .expect("write score middle byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 4, 0x00).expect("write score low byte");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 6, b'D').expect("write first initial");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 8, b'R').expect("write second initial");
        cmos_sram_write_byte(&mut cmos, high_score_offset + 10, b'J').expect("write third initial");
        cmos_sram_write_byte(&mut cmos, 0x7D, 0x04).expect("write persisted credits");
        *storage.cmos.borrow_mut() = Some(cmos);

        let machine =
            live_machine_from_cmos_storage(Some(&storage)).expect("load machine from CMOS");
        assert_eq!(
            machine.red_label_cmos_range(0x7D..0x7F),
            Some(&cmos[0x7D..0x7F])
        );

        let mut changed_machine = ArcadeMachine::try_new_with_cmos(cmos).expect("machine");
        changed_machine.step(crate::input::CabinetInput {
            coin: true,
            ..crate::input::CabinetInput::NONE
        });
        save_live_cmos(Some(&storage), &changed_machine).expect("save machine CMOS");
        assert_eq!(
            storage.cmos.borrow().expect("saved CMOS"),
            *changed_machine.red_label_cmos_ram()
        );
    }
}
