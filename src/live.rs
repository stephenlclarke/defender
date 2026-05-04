//! Live terminal runner for the new core.
#![cfg_attr(coverage, allow(dead_code, unused_imports))]

#[cfg(not(test))]
use std::io;
#[cfg(not(test))]
use std::io::IsTerminal;
#[cfg(not(test))]
use std::io::Write;
use std::path::Path;
#[cfg(not(test))]
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
#[cfg(not(test))]
use crossterm::event::{self, Event};

use crate::{
    cmos_storage::CmosStorage,
    input::{CabinetInput, InputProfile},
    machine::{ArcadeMachine, FRAME_RATE_MILLIHZ},
    video::{RenderedImage, Renderer},
};
#[cfg(not(test))]
use crate::{
    cmos_storage::FileCmosStorage,
    input::{InputMapper, PolledInput, XyzzyOverlay},
    kitty::KittyGraphics,
    machine::CompatibilityState,
    terminal::{TerminalSession, geometry},
};

const FRAME_DURATION: Duration =
    Duration::from_micros(cabinet_frame_duration_micros(FRAME_RATE_MILLIHZ));

const fn cabinet_frame_duration_micros(frame_rate_millihz: u32) -> u64 {
    let rate = frame_rate_millihz as u64;
    (1_000_000_000 + (rate / 2)) / rate
}

struct LiveCoreClock {
    next_step: Instant,
}

impl LiveCoreClock {
    fn new(now: Instant) -> Self {
        Self { next_step: now }
    }

    fn steps_due(&mut self, now: Instant) -> u32 {
        let mut steps = 0;
        while now >= self.next_step {
            steps += 1;
            self.next_step += FRAME_DURATION;
        }
        steps
    }

    fn sleep_until_next_step(&self, now: Instant) -> Duration {
        self.next_step.saturating_duration_since(now)
    }
}

#[cfg(all(not(test), not(coverage)))]
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
    let mut core_clock = LiveCoreClock::new(Instant::now());
    let mut pending_typed_chars = Vec::new();

    loop {
        sync_terminal_geometry(&mut terminal_geometry, &mut renderer, &mut graphics)?;

        let input = poll_input(&mut input_mapper)?;
        if input.quit_requested {
            break;
        }

        xyzzy.handle_typed_chars(&input.typed_chars);
        pending_typed_chars.extend(input.typed_chars.iter().copied());
        machine.set_compatibility(CompatibilityState {
            xyzzy_active: xyzzy.active(),
            xyzzy_invincible: xyzzy.invincible(),
            xyzzy_auto_fire: xyzzy.auto_fire(),
        });
        let core_steps = core_clock.steps_due(Instant::now());
        step_live_core_frames(
            &mut machine,
            input.cabinet,
            &pending_typed_chars,
            core_steps,
        );
        if core_steps != 0 {
            pending_typed_chars.clear();
        }

        let image = render_live_machine_frame(&mut renderer, &mut machine)
            .context("rendering live machine frame")?;
        graphics
            .draw_frame(&mut stdout, image)
            .context("drawing kitty graphics frame")?;
        stdout.flush().context("flushing kitty graphics frame")?;

        let sleep_duration = core_clock.sleep_until_next_step(Instant::now());
        if !sleep_duration.is_zero() {
            thread::sleep(sleep_duration);
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

#[cfg(any(test, coverage))]
pub fn run_live(
    _play_audio: bool,
    _input_profile: InputProfile,
    _cmos_path: Option<&Path>,
) -> Result<()> {
    Ok(())
}

fn render_live_machine_frame<'a>(
    renderer: &'a mut Renderer,
    machine: &mut ArcadeMachine,
) -> Result<&'a RenderedImage> {
    machine
        .red_label_copy_color_mapping_to_palette_ram()
        .map_err(|error| anyhow!("copying red-label color mapping to palette RAM: {error}"))?;
    let native_frame = machine
        .red_label_visible_rgba_image()
        .context("red-label visible frame is unavailable")?;
    Ok(render_live_frame(renderer, native_frame))
}

fn render_live_frame(renderer: &mut Renderer, native_frame: RenderedImage) -> &RenderedImage {
    renderer.render_cabinet_frame(&native_frame)
}

fn step_live_core_frames(
    machine: &mut ArcadeMachine,
    input: CabinetInput,
    typed_chars: &[char],
    frames: u32,
) {
    if frames == 0 {
        return;
    }

    machine.step_with_typed_chars(input, typed_chars);
    for _ in 1..frames {
        machine.step(input);
    }
}

#[cfg(not(test))]
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

#[cfg(not(test))]
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

#[cfg(not(test))]
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
    use std::time::Instant;

    use crate::board::{
        CMOS_RAM_SIZE, CmosRam, RED_LABEL_CRHSTD_CELL_OFFSET, cmos_sram_write_byte,
    };
    use crate::cmos_storage::CmosStorage;
    use crate::input::CabinetInput;
    use crate::machine::{
        ArcadeMachine, FRAME_RATE_MILLIHZ, GamePhase, RedLabelSoundBoardSnapshot,
    };
    use crate::sound::SoundCommandLatch;
    use crate::video::{RenderedImage, Renderer, defender_visible_byte_offset};

    use super::{
        FRAME_DURATION, LiveCoreClock, cabinet_frame_duration_micros,
        live_machine_from_cmos_storage, render_live_frame, render_live_machine_frame,
        save_live_cmos, step_live_core_frames, validate_interactive_terminal,
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
    fn render_live_frame_uses_native_cabinet_frame_even_when_blank() {
        let mut renderer = Renderer::with_size(32, 24);
        let blank_native = RenderedImage::new_blank(2, 1, [0, 0, 0, 255]);

        let image = render_live_frame(&mut renderer, blank_native);

        assert_eq!((image.width, image.height), (32, 24));
        assert!(
            image
                .pixels
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 0, 255])
        );
    }

    #[test]
    fn render_live_frame_uses_visible_native_frames_and_machine_wrapper() {
        let mut machine = ArcadeMachine::new();
        let visible = RenderedImage {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 0, 95, 255, 255],
        };
        let mut renderer = Renderer::with_size(32, 16);

        let native = render_live_frame(&mut renderer, visible);
        assert!(
            native
                .pixels
                .chunks_exact(4)
                .any(|pixel| pixel == [0, 95, 255, 255].as_slice())
        );

        let source_frame =
            render_live_machine_frame(&mut renderer, &mut machine).expect("render machine");
        assert_eq!((source_frame.width, source_frame.height), (32, 16));
    }

    #[test]
    fn render_live_machine_frame_applies_source_color_mapping_before_scaling() {
        let mut machine = ArcadeMachine::new();
        let visible_offset =
            defender_visible_byte_offset(0, 0).expect("visible origin maps into video RAM");
        machine.red_label_write_ram_byte_for_test(visible_offset as u16, 0xAB);
        machine.red_label_write_ram_byte_for_test(0xA026 + 0x0A, 0xD6);
        machine.red_label_write_ram_byte_for_test(0xA026 + 0x0B, 0x29);
        let mut renderer = Renderer::with_size(292, 240);

        let image = render_live_machine_frame(&mut renderer, &mut machine).expect("render machine");

        assert_eq!(machine.red_label_palette_ram()[0x0A], 0xD6);
        assert_eq!(machine.red_label_palette_ram()[0x0B], 0x29);
        assert_eq!(&image.pixels[0..4], &[217, 81, 255, 255]);
        assert_eq!(&image.pixels[4..8], &[38, 174, 0, 255]);
    }

    #[test]
    fn frame_duration_tracks_cabinet_refresh_not_old_ninety_ms_tick() {
        assert_eq!(
            FRAME_DURATION.as_micros(),
            u128::from(cabinet_frame_duration_micros(FRAME_RATE_MILLIHZ))
        );
        assert_eq!(FRAME_DURATION.as_micros(), 16_639);
    }

    #[test]
    fn core_clock_reports_due_frames_independent_of_draw_calls() {
        let start = Instant::now();
        let mut clock = LiveCoreClock::new(start);

        assert_eq!(clock.steps_due(start), 1);
        assert_eq!(clock.sleep_until_next_step(start), FRAME_DURATION);
        assert_eq!(clock.steps_due(start + (FRAME_DURATION / 2)), 0);
        assert_eq!(
            clock.sleep_until_next_step(start + (FRAME_DURATION / 2)),
            FRAME_DURATION / 2
        );

        let stalled_until = start + FRAME_DURATION + FRAME_DURATION + FRAME_DURATION;
        assert_eq!(clock.steps_due(stalled_until), 3);
        assert_eq!(clock.sleep_until_next_step(stalled_until), FRAME_DURATION);
    }

    #[test]
    fn live_core_steps_catch_up_without_replaying_typed_chars() {
        let mut machine = ArcadeMachine::new();
        let mut snapshot = machine.snapshot();
        snapshot.phase = GamePhase::HighScoreEntry;
        machine.restore(snapshot);
        machine
            .red_label_begin_live_high_score_entry(50_000)
            .expect("high score table should be valid");

        step_live_core_frames(&mut machine, CabinetInput::NONE, &['a'], 3);

        let snapshot = machine.snapshot();
        assert_eq!(snapshot.frame, 3);
        assert_eq!(
            snapshot
                .high_score_entry
                .expect("entry still active")
                .initials,
            [b'A', b' ', b' ']
        );
    }

    #[test]
    fn live_core_sound_state_is_independent_of_audio_output_mode() {
        let mut audible_core = ArcadeMachine::new_cold_boot_trace();
        let mut muted_core = ArcadeMachine::new_cold_boot_trace();

        step_live_core_frames(&mut audible_core, CabinetInput::NONE, &[], 731);
        step_live_core_frames(&mut muted_core, CabinetInput::NONE, &[], 731);

        let expected = RedLabelSoundBoardSnapshot {
            last_command_latch: Some(SoundCommandLatch::from_main_board_pia_port_b(0xC0)),
            latched_port_b: Some(0xC0),
            command_cb1_asserted: true,
            latch_write_count: 1,
        };
        assert_eq!(audible_core.red_label_sound_board_snapshot(), expected);
        assert_eq!(muted_core.red_label_sound_board_snapshot(), expected);
        assert_eq!(
            muted_core.red_label_sound_board_snapshot(),
            audible_core.red_label_sound_board_snapshot()
        );
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
