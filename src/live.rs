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
    let mut pending_cabinet_input = CabinetInput::NONE;
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
        let held_input = input_mapper.held_cabinet_input();
        if let Some(frame_inputs) = live_frame_inputs(
            &mut pending_cabinet_input,
            input.cabinet,
            held_input,
            core_steps,
        ) {
            step_live_core_frames(
                &mut machine,
                frame_inputs.first,
                frame_inputs.catch_up,
                &pending_typed_chars,
                core_steps,
            );
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
    first_input: CabinetInput,
    catch_up_input: CabinetInput,
    typed_chars: &[char],
    frames: u32,
) {
    if frames == 0 {
        return;
    }

    machine.step_with_typed_chars(first_input, typed_chars);
    for _ in 1..frames {
        machine.step(catch_up_input);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LiveFrameInputs {
    first: CabinetInput,
    catch_up: CabinetInput,
}

fn live_frame_inputs(
    pending_pulses: &mut CabinetInput,
    polled_pulses: CabinetInput,
    held_input: CabinetInput,
    frames: u32,
) -> Option<LiveFrameInputs> {
    pending_pulses.merge(polled_pulses);
    if frames == 0 {
        return None;
    }

    let mut first = *pending_pulses;
    first.merge(held_input);
    *pending_pulses = CabinetInput::NONE;

    Some(LiveFrameInputs {
        first,
        catch_up: held_input,
    })
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
    Ok(input)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::io;
    use std::time::Instant;

    use crate::board::{
        CMOS_RAM_SIZE, CmosRam, RED_LABEL_CRHSTD_CELL_OFFSET, cmos_sram_write_byte,
    };
    use crate::cmos_storage::CmosStorage;
    use crate::input::CabinetInput;
    use crate::machine::{
        ArcadeMachine, FRAME_RATE_MILLIHZ, GamePhase, MachineEvent, RedLabelSoundBoardSnapshot,
        VISIBLE_HEIGHT, VISIBLE_WIDTH,
    };
    use crate::rom::crc32;
    use crate::sound::SoundCommandLatch;
    use crate::video::{RenderedImage, Renderer, defender_visible_byte_offset};

    use super::{
        FRAME_DURATION, LiveCoreClock, cabinet_frame_duration_micros, live_frame_inputs,
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
    fn render_live_machine_frame_survives_williams_handoff_and_remains_playable() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut render_crcs = BTreeSet::new();
        let mut color_cycle_render_crcs = BTreeSet::new();
        let mut color_cycle_video_crcs = BTreeSet::new();
        let mut saw_non_blank_frame = false;
        let mut saw_post_williams_non_blank_frame = false;
        let mut post_non_blank_blank_frames = Vec::new();

        for frame in 0..1_220 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(
                output.snapshot.phase,
                GamePhase::Attract,
                "unexpected phase while rendering idle live attract frame {frame}"
            );
            if frame < 700 && frame % 20 != 0 {
                continue;
            }
            let (render_crc, non_blank) = {
                let image =
                    render_live_machine_frame(&mut renderer, &mut machine).expect("render frame");
                (
                    crc32(&image.pixels),
                    image
                        .pixels
                        .chunks_exact(4)
                        .any(|pixel| pixel != [0, 0, 0, 255].as_slice()),
                )
            };
            render_crcs.insert(render_crc);
            if saw_non_blank_frame && !non_blank {
                post_non_blank_blank_frames.push(frame);
            }
            saw_non_blank_frame |= non_blank;
            if frame >= 420 {
                saw_post_williams_non_blank_frame |= non_blank;
            }
            if (900..=1_040).contains(&frame) {
                color_cycle_render_crcs.insert(render_crc);
                color_cycle_video_crcs.insert(
                    machine
                        .red_label_visible_video_crc32()
                        .expect("video CRC remains available during Williams color cycle"),
                );
                assert!(
                    non_blank,
                    "Williams color-cycle frame {frame} should not render blank"
                );
            }
        }

        assert!(saw_non_blank_frame);
        assert!(saw_post_williams_non_blank_frame);
        assert!(
            post_non_blank_blank_frames.is_empty(),
            "render path returned blank frames after startup became visible: {post_non_blank_blank_frames:?}"
        );
        assert!(
            color_cycle_render_crcs.len() >= 2,
            "expected Williams color cycle to change rendered palette CRCs"
        );
        assert_eq!(
            color_cycle_video_crcs.len(),
            1,
            "Williams color cycle should not blank or rewrite visible video RAM"
        );
        assert!(
            render_crcs.len() > 8,
            "expected animated rendered frames through Williams handoff, got {} distinct CRCs",
            render_crcs.len()
        );

        let _ = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });
        let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render coin press");
        let mut credit_added = false;
        for _ in 0..32 {
            let output = machine.step(CabinetInput::NONE);
            let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render credit");
            credit_added |= output
                .events()
                .any(|event| event == MachineEvent::CreditAdded);
            if credit_added {
                break;
            }
        }
        assert!(credit_added);
        assert!(machine.snapshot().credits > 0);

        let mut game_started = false;
        for _ in 0..16 {
            let output = machine.step(CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            });
            let _ = render_live_machine_frame(&mut renderer, &mut machine).expect("render start");
            game_started |= output
                .events()
                .any(|event| event == MachineEvent::GameStarted);
            if game_started {
                break;
            }
        }
        assert!(game_started);
        assert_eq!(machine.snapshot().phase, GamePhase::Playing);
    }

    #[test]
    fn live_credited_start_renders_terrain_and_enemy_objects() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);

        start_live_one_player_game(&mut machine, &mut renderer);

        let mut saw_terrain = false;
        let mut saw_rendered_terrain = false;
        let mut saw_enemy_object = false;
        let mut saw_post_reverse_rendered_terrain = false;
        for frame in 0..1_200 {
            assert_live_target_list_valid(&machine);
            let input = if matches!(frame, 360 | 720) {
                CabinetInput {
                    reverse: true,
                    ..CabinetInput::NONE
                }
            } else {
                CabinetInput::NONE
            };
            machine.step(input);
            assert_live_target_list_valid(&machine);
            let image =
                render_live_machine_frame(&mut renderer, &mut machine).expect("render gameplay");
            saw_terrain |= native_nonzero_pixels_in_band(&machine, 180..240) >= 100;
            let rendered_terrain = rendered_nonblack_pixels_in_band(image, 180..240) >= 100;
            saw_rendered_terrain |= rendered_terrain;
            saw_post_reverse_rendered_terrain |= frame > 720 && rendered_terrain;
            saw_enemy_object |= live_visible_enemy_object_count(&machine) > 0
                || live_visible_enemy_appearance_count(&machine) > 0;

            // Keep running after the first good world frame. The release
            // build was able to show player/HUD/world briefly and then crash
            // from later process/object corruption.
        }

        assert!(
            saw_terrain,
            "credited live start did not render terrain/ground pixels"
        );
        assert!(
            saw_rendered_terrain,
            "credited live start did not present terrain/ground pixels in the rendered cabinet frame"
        );
        assert!(
            saw_enemy_object,
            "credited live start did not render visible active enemy/object pixels"
        );
        assert!(
            saw_post_reverse_rendered_terrain,
            "credited live reverse path did not preserve rendered terrain/ground pixels"
        );
    }

    #[test]
    fn rendered_live_attract_visibly_advances_after_title_page() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut title_render_crc = None;
        let mut saw_post_title_render = false;

        for tick in 1..=6_000 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            if tick == 900 || tick > 900 && tick % 30 == 0 {
                let image =
                    render_live_machine_frame(&mut renderer, &mut machine).expect("render attract");
                let render_crc = crc32(&image.pixels);
                if tick == 900 {
                    title_render_crc = Some(render_crc);
                }
                if let Some(title_render_crc) = title_render_crc {
                    saw_post_title_render |= tick > 900 && render_crc != title_render_crc;
                }
            }
            if saw_post_title_render {
                return;
            }
        }

        assert!(
            saw_post_title_render,
            "rendered live attract did not visibly leave the title page"
        );
    }

    #[test]
    fn rendered_live_attract_action_scene_has_objects_without_vertical_trails() {
        let mut machine = ArcadeMachine::new();
        let mut renderer = Renderer::with_size(292, 240);
        let mut saw_action_scene = false;
        let mut worst_vertical_streak = 0;

        for tick in 1..=4_400 {
            let output = machine.step(CabinetInput::NONE);
            assert_eq!(output.snapshot.phase, GamePhase::Attract);
            if tick < 4_000 || tick % 30 != 0 {
                continue;
            }

            let image =
                render_live_machine_frame(&mut renderer, &mut machine).expect("render attract");
            let has_ship = live_object_screen_from_pointer(&machine, 0xA18B).is_some();
            let has_astronaut = live_object_screen_from_pointer(&machine, 0xA189).is_some();
            let has_terrain =
                rendered_nonblack_pixels_in_band(image, 180..usize::from(VISIBLE_HEIGHT)) >= 100;
            let vertical_streak = rendered_max_nonblack_vertical_streak(image, 40..220);
            worst_vertical_streak = worst_vertical_streak.max(vertical_streak);
            if has_ship && has_astronaut && has_terrain {
                saw_action_scene = true;
                assert!(
                    vertical_streak <= 16,
                    "attract action scene retained a vertical object trail of {vertical_streak} pixels"
                );
                break;
            }
        }

        assert!(
            saw_action_scene,
            "rendered live attract did not show the instruction-scene ship, astronaut, and terrain"
        );
        assert!(
            worst_vertical_streak <= 16,
            "rendered live attract retained vertical object trails; worst streak was {worst_vertical_streak} pixels"
        );
    }

    #[test]
    fn frame_duration_tracks_cabinet_refresh_not_old_ninety_ms_tick() {
        assert_eq!(
            FRAME_DURATION.as_micros(),
            u128::from(cabinet_frame_duration_micros(FRAME_RATE_MILLIHZ))
        );
        assert_eq!(FRAME_DURATION.as_micros(), 16_639);
    }

    fn start_live_one_player_game(machine: &mut ArcadeMachine, renderer: &mut Renderer) {
        let _ = machine.step(CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        });
        let _ = render_live_machine_frame(renderer, machine).expect("render coin press");
        let mut credit_added = false;
        for _ in 0..32 {
            let output = machine.step(CabinetInput::NONE);
            let _ = render_live_machine_frame(renderer, machine).expect("render credit");
            credit_added |= output
                .events()
                .any(|event| event == MachineEvent::CreditAdded);
            if credit_added {
                break;
            }
        }
        assert!(credit_added);

        let mut game_started = false;
        for _ in 0..16 {
            let output = machine.step(CabinetInput {
                start_one: true,
                ..CabinetInput::NONE
            });
            let _ = render_live_machine_frame(renderer, machine).expect("render start");
            game_started |= output
                .events()
                .any(|event| event == MachineEvent::GameStarted);
            if game_started {
                break;
            }
        }
        assert!(game_started);
        assert_eq!(machine.snapshot().phase, GamePhase::Playing);
    }

    fn native_nonzero_pixels_in_band(
        machine: &ArcadeMachine,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let nibbles = machine
            .red_label_visible_pixel_nibbles()
            .expect("native visible pixel nibbles");
        let width = usize::from(VISIBLE_WIDTH);
        y_range
            .flat_map(|y| {
                let row = y * width;
                nibbles[row..row + width].iter()
            })
            .filter(|nibble| **nibble != 0)
            .count()
    }

    fn rendered_nonblack_pixels_in_band(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        y_range
            .flat_map(|y| {
                let row = y * width * 4;
                image.pixels[row..row + width * 4].chunks_exact(4)
            })
            .filter(|pixel| *pixel != [0, 0, 0, 255])
            .count()
    }

    fn rendered_max_nonblack_vertical_streak(
        image: &RenderedImage,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let width = image.width as usize;
        let mut max_streak = 0;
        for x in 0..width {
            let mut streak = 0;
            for y in y_range.clone() {
                let offset = (y * width + x) * 4;
                if &image.pixels[offset..offset + 4] != [0, 0, 0, 255].as_slice() {
                    streak += 1;
                    max_streak = max_streak.max(streak);
                } else {
                    streak = 0;
                }
            }
        }
        max_streak
    }

    fn live_object_screen_from_pointer(
        machine: &ArcadeMachine,
        pointer_address: u16,
    ) -> Option<u16> {
        let object_address = read_word(machine, pointer_address)?;
        if !live_object_address_is_valid(object_address) {
            return None;
        }
        let screen_address = read_word(machine, object_address + 0x04)?;
        (screen_address != 0).then_some(screen_address)
    }

    fn live_visible_enemy_object_count(machine: &ArcadeMachine) -> usize {
        let mut count = 0;
        let mut object_address = read_word(machine, 0xA065).expect("active object head");
        for _ in 0..95 {
            if object_address == 0 {
                break;
            }
            if !live_object_address_is_valid(object_address) {
                break;
            }

            let next_object = read_word(machine, object_address).expect("active object link word");
            let picture = read_word(machine, object_address + 0x02).expect("object OPICT");
            let screen = read_word(machine, object_address + 0x04).expect("object screen");
            let collision_vector =
                read_word(machine, object_address + 0x08).expect("object OCVECT");
            if screen != 0 && picture != 0xF8EC && live_enemy_collision_vector(collision_vector) {
                count += 1;
            }
            object_address = next_object;
        }
        count
    }

    fn assert_live_target_list_valid(machine: &ArcadeMachine) {
        let target_pointer = read_word(machine, 0xA09B).expect("TPTR");
        if target_pointer == 0 {
            return;
        }
        assert!(
            (0xA11A..0xA142).contains(&target_pointer)
                && (target_pointer - 0xA11A).is_multiple_of(2),
            "live target cursor drifted outside TLIST: 0x{target_pointer:04X}"
        );
        for slot_address in (0xA11A..0xA142).step_by(2) {
            let object_address = read_word(machine, slot_address).expect("TLIST slot");
            assert!(
                object_address == 0 || live_object_address_is_valid(object_address),
                "live TLIST slot 0x{slot_address:04X} points outside object table: 0x{object_address:04X}"
            );
        }
    }

    fn live_visible_enemy_appearance_count(machine: &ArcadeMachine) -> usize {
        let mut count = 0;
        for slot in 0..16 {
            let slot_address = 0x9C00 + slot * 0x40;
            let size = read_word(machine, slot_address).expect("appearance RSIZE");
            if size == 0 {
                continue;
            }
            let object_address =
                read_word(machine, slot_address + 0x0A).expect("appearance OBJPTR");
            if !live_object_address_is_valid(object_address) {
                continue;
            }
            let collision_vector =
                read_word(machine, object_address + 0x08).expect("appearance object OCVECT");
            if live_enemy_collision_vector(collision_vector) {
                count += 1;
            }
        }
        count
    }

    fn live_object_address_is_valid(object_address: u16) -> bool {
        (0xA23C..0xA23C + 95 * 0x17).contains(&object_address)
            && (object_address - 0xA23C).is_multiple_of(0x17)
    }

    fn live_enemy_collision_vector(collision_vector: u16) -> bool {
        matches!(collision_vector, 0xEB2B | 0xEBE9 | 0xEF6D | 0xF20B)
    }

    fn read_word(machine: &ArcadeMachine, address: u16) -> Option<u16> {
        let bytes = machine.red_label_ram_range(address..address + 2)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
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

        step_live_core_frames(
            &mut machine,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &['a'],
            3,
        );

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
    fn live_frame_inputs_buffer_pulse_inputs_until_a_core_frame_is_due() {
        let mut pending = CabinetInput::NONE;
        let polled = CabinetInput {
            coin: true,
            ..CabinetInput::NONE
        };

        assert_eq!(
            live_frame_inputs(&mut pending, polled, CabinetInput::NONE, 0),
            None
        );
        assert!(pending.coin);

        let frame_inputs =
            live_frame_inputs(&mut pending, CabinetInput::NONE, CabinetInput::NONE, 1)
                .expect("buffered coin should be consumed on the next core frame");

        assert!(frame_inputs.first.coin);
        assert!(!frame_inputs.catch_up.coin);
        assert_eq!(pending, CabinetInput::NONE);
    }

    #[test]
    fn live_frame_inputs_do_not_replay_pulses_across_catch_up_frames() {
        let mut pending = CabinetInput::NONE;
        let polled = CabinetInput {
            start_one: true,
            ..CabinetInput::NONE
        };
        let held = CabinetInput {
            thrust: true,
            ..CabinetInput::NONE
        };

        let frame_inputs =
            live_frame_inputs(&mut pending, polled, held, 3).expect("frames are due");

        assert!(frame_inputs.first.start_one);
        assert!(frame_inputs.first.thrust);
        assert!(!frame_inputs.catch_up.start_one);
        assert!(frame_inputs.catch_up.thrust);
        assert_eq!(pending, CabinetInput::NONE);
    }

    #[test]
    fn live_core_sound_state_is_independent_of_audio_output_mode() {
        let mut audible_core = ArcadeMachine::new_cold_boot_trace();
        let mut muted_core = ArcadeMachine::new_cold_boot_trace();

        step_live_core_frames(
            &mut audible_core,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &[],
            731,
        );
        step_live_core_frames(
            &mut muted_core,
            CabinetInput::NONE,
            CabinetInput::NONE,
            &[],
            731,
        );

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
