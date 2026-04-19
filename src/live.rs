use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, ModifierKeyCode};

use crate::attract::{AttractBeat, SceneKind, attract_frame_for_beat, beat_for_elapsed_ms};
use crate::audio::{AudioManager, SoundCue};
use crate::kitty::KittyGraphics;
use crate::session::{SessionEvent, SessionInput, SessionMode, SessionState};
use crate::terminal::{TerminalSession, geometry};
use crate::video::{RenderedImage, Renderer, Screen};

const FRAME_DURATION: Duration = Duration::from_millis(90);

pub fn run_live(play_audio: bool) -> Result<()> {
    ensure_interactive_terminal()?;
    KittyGraphics::ensure_supported()?;

    let mut audio = AudioManager::new();
    let mut stdout = io::stdout();
    let _terminal = TerminalSession::enter(&mut stdout)?;
    let mut input_tracker = InputTracker::default();
    let mut session = SessionState::load();
    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);
    let mut title_started_at = Instant::now();
    let mut previous_mode = session.mode();

    draw_frame(
        &mut stdout,
        &session,
        &mut renderer,
        &mut graphics,
        title_started_at.elapsed().as_millis() as u64,
    )?;

    loop {
        let frame_started = Instant::now();
        sync_terminal_geometry(&mut terminal_geometry, &mut renderer, &mut graphics)?;

        let input = input_tracker.poll().context("polling live input")?;
        if input.quit_requested {
            break;
        }

        let events = session.tick(input.session);
        if session.mode() != previous_mode {
            if session.mode() == SessionMode::Title {
                title_started_at = Instant::now();
            }
            previous_mode = session.mode();
        }
        draw_frame(
            &mut stdout,
            &session,
            &mut renderer,
            &mut graphics,
            title_started_at.elapsed().as_millis() as u64,
        )?;

        if play_audio {
            for cue in cues_for_events(&events) {
                audio.play_cue(cue);
            }
        }

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

fn draw_frame(
    stdout: &mut io::Stdout,
    session: &SessionState,
    renderer: &mut Renderer,
    graphics: &mut KittyGraphics,
    attract_elapsed_ms: u64,
) -> Result<()> {
    let image = render_session_frame(renderer, session, attract_elapsed_ms);
    graphics
        .draw_frame(stdout, image)
        .context("drawing kitty graphics frame")?;
    stdout.flush().context("flushing kitty graphics frame")?;
    Ok(())
}

fn render_session_frame<'a>(
    renderer: &'a mut Renderer,
    session: &SessionState,
    attract_elapsed_ms: u64,
) -> &'a RenderedImage {
    match session.mode() {
        SessionMode::Title => render_title_frame(renderer, session, attract_elapsed_ms),
        SessionMode::Playing => renderer.render(Screen::Playing {
            world: session.world(),
            xyzzy_active: session.xyzzy_active(),
            invincible: session.invincible(),
            auto_fire: session.auto_fire(),
        }),
        SessionMode::GameOver => renderer.render(Screen::GameOver {
            world: session.world(),
            high_score: session.high_score(),
            xyzzy_active: session.xyzzy_active(),
            invincible: session.invincible(),
            auto_fire: session.auto_fire(),
        }),
        SessionMode::EnteringInitials => {
            let pending = session
                .pending_initials()
                .expect("initials mode should have pending entry");
            let display_letters = pending.display_letters();
            let view = crate::render::InitialsEntryView {
                high_score: session.high_score(),
                todays_high_scores: session.todays_high_scores(),
                all_time_high_scores: session.high_scores(),
                entry_score: pending.score(),
                entry_rank: pending.rank(),
                initials: &display_letters,
                xyzzy_active: session.xyzzy_active(),
                invincible: session.invincible(),
                auto_fire: session.auto_fire(),
            };
            renderer.render(Screen::InitialsEntry {
                world: session.world(),
                view: &view,
            })
        }
    }
}

fn render_title_frame<'a>(
    renderer: &'a mut Renderer,
    session: &SessionState,
    attract_elapsed_ms: u64,
) -> &'a RenderedImage {
    let beat = title_beat_for_elapsed_ms(attract_elapsed_ms);
    match beat.kind {
        SceneKind::Logo => renderer.render(Screen::Logo {
            palette_phase: beat.palette_phase,
            elapsed_ms: attract_elapsed_ms,
            trace_points: beat.logo_trace_points,
            show_title_text: beat.logo_show_title_text,
            show_full_defender: beat.logo_show_full_defender,
            defender_appear_tick: beat.logo_defender_appear_tick,
            show_copyright: beat.logo_show_copyright,
        }),
        SceneKind::Attract => {
            let frame = attract_frame_for_beat(beat);
            renderer.render(Screen::Attract {
                frame: &frame,
                palette_phase: beat.palette_phase,
            })
        }
        SceneKind::HighScore => renderer.render(Screen::HighScores {
            todays: session.todays_high_scores(),
            all_time: session.high_scores(),
            palette_phase: beat.palette_phase,
            elapsed_ms: attract_elapsed_ms,
        }),
    }
}

fn title_beat_for_elapsed_ms(elapsed_ms: u64) -> AttractBeat {
    beat_for_elapsed_ms(elapsed_ms)
}

fn cues_for_events(events: &[SessionEvent]) -> Vec<SoundCue> {
    let mut cues = Vec::new();

    if events.contains(&SessionEvent::GameStarted) || events.contains(&SessionEvent::GameRestarted)
    {
        cues.push(SoundCue::LogoFanfare);
    }
    if events.contains(&SessionEvent::HighScoreUpdated)
        || events.contains(&SessionEvent::HighScoreSaved)
        || events.contains(&SessionEvent::World(crate::game::WorldEvent::WaveAdvanced))
    {
        push_unique_cue(&mut cues, SoundCue::HighScoreChime);
    }
    if events.contains(&SessionEvent::World(crate::game::WorldEvent::GameOver))
        || events.contains(&SessionEvent::World(crate::game::WorldEvent::PlayerHit))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::SmartBombDetonated,
        ))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::EnemyDestroyed,
        ))
    {
        push_unique_cue(&mut cues, SoundCue::Explosion);
    }
    if events.contains(&SessionEvent::World(crate::game::WorldEvent::EnemyFired))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::HyperspaceUsed,
        ))
    {
        push_unique_cue(&mut cues, SoundCue::EnemySweep);
    }
    if events.contains(&SessionEvent::World(crate::game::WorldEvent::HumanRescued)) {
        push_unique_cue(&mut cues, SoundCue::HumanSaved);
    }
    if events.contains(&SessionEvent::World(crate::game::WorldEvent::HumanLost)) {
        push_unique_cue(&mut cues, SoundCue::AttractHum);
    }
    if events.contains(&SessionEvent::World(crate::game::WorldEvent::ShotFired)) {
        push_unique_cue(&mut cues, SoundCue::PlayerShot);
    }

    cues
}

fn push_unique_cue(cues: &mut Vec<SoundCue>, cue: SoundCue) {
    if !cues.contains(&cue) {
        cues.push(cue);
    }
}

#[derive(Debug, Default)]
struct InputTracker {
    held: HeldInput,
}

#[derive(Debug, Default)]
struct HeldInput {
    up: bool,
    down: bool,
    thrust: bool,
}

#[derive(Debug, Default)]
struct PolledInput {
    session: SessionInput,
    quit_requested: bool,
}

impl InputTracker {
    fn poll(&mut self) -> Result<PolledInput> {
        let mut input = PolledInput::default();

        while event::poll(Duration::ZERO)? {
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event, &mut input);
            }
        }

        input.session.update.up |= self.held.up;
        input.session.update.down |= self.held.down;
        input.session.update.thrust |= self.held.thrust;

        Ok(input)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        let pressed = !matches!(key_event.kind, KeyEventKind::Release);
        if matches!(key_event.kind, KeyEventKind::Press)
            && let KeyCode::Char(character) = key_event.code
        {
            input
                .session
                .typed_chars
                .push(character.to_ascii_lowercase());
        }

        match key_event.code {
            KeyCode::Esc if pressed => input.quit_requested = true,
            KeyCode::Char('q') | KeyCode::Char('Q') if pressed => input.quit_requested = true,
            KeyCode::Backspace if pressed => input.session.backspace_requested = true,
            KeyCode::Enter if pressed => {
                input.session.start_requested = true;
                input.session.update.fire = true;
            }
            KeyCode::Char('1') if pressed => input.session.start_requested = true,
            KeyCode::Char('a') | KeyCode::Char('A') => set_held_flag(
                &mut self.held.up,
                key_event.kind,
                &mut input.session.update.up,
            ),
            KeyCode::Char('z') | KeyCode::Char('Z') => set_held_flag(
                &mut self.held.down,
                key_event.kind,
                &mut input.session.update.down,
            ),
            KeyCode::Modifier(ModifierKeyCode::LeftShift)
            | KeyCode::Modifier(ModifierKeyCode::RightShift) => set_held_flag(
                &mut self.held.thrust,
                key_event.kind,
                &mut input.session.update.thrust,
            ),
            KeyCode::Char(' ') if pressed => input.session.update.reverse = true,
            KeyCode::Tab if pressed => input.session.update.smart_bomb = true,
            KeyCode::Char('h') | KeyCode::Char('H') if pressed => {
                input.session.update.hyperspace = true;
            }
            _ => {}
        }
    }
}

fn set_held_flag(held: &mut bool, kind: KeyEventKind, output: &mut bool) {
    match kind {
        KeyEventKind::Press | KeyEventKind::Repeat => {
            *held = true;
            *output = true;
        }
        KeyEventKind::Release => *held = false,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};

    use super::{
        InputTracker, PolledInput, cues_for_events, render_title_frame, title_beat_for_elapsed_ms,
        validate_interactive_terminal,
    };
    use crate::audio::SoundCue;
    use crate::game::WorldEvent;
    use crate::high_scores::HighScoreTable;
    use crate::session::{SessionEvent, SessionInput, SessionState};
    use crate::video::Renderer;

    #[test]
    fn event_audio_groups_allow_overlap_without_duplicate_cues() {
        assert_eq!(
            cues_for_events(&[
                SessionEvent::World(WorldEvent::HyperspaceUsed),
                SessionEvent::World(WorldEvent::GameOver),
            ]),
            vec![SoundCue::Explosion, SoundCue::EnemySweep]
        );
        assert_eq!(
            cues_for_events(&[SessionEvent::World(WorldEvent::WaveAdvanced)]),
            vec![SoundCue::HighScoreChime]
        );
        assert_eq!(
            cues_for_events(&[SessionEvent::World(WorldEvent::EnemyFired)]),
            vec![SoundCue::EnemySweep]
        );
        assert_eq!(
            cues_for_events(&[
                SessionEvent::World(WorldEvent::SmartBombDetonated),
                SessionEvent::World(WorldEvent::EnemyDestroyed),
            ]),
            vec![SoundCue::Explosion]
        );
        assert_eq!(
            cues_for_events(&[SessionEvent::World(WorldEvent::HumanRescued)]),
            vec![SoundCue::HumanSaved]
        );
        assert_eq!(
            cues_for_events(&[
                SessionEvent::HighScoreSaved,
                SessionEvent::World(WorldEvent::ShotFired),
            ]),
            vec![SoundCue::HighScoreChime, SoundCue::PlayerShot]
        );
    }

    #[test]
    fn input_tracker_maps_live_controls_and_releases() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT),
            &mut input,
        );
        assert!(input.session.update.up);

        tracker.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Modifier(ModifierKeyCode::LeftShift),
                KeyModifiers::NONE,
                KeyEventKind::Press,
            ),
            &mut input,
        );
        assert!(input.session.update.thrust);

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
            &mut input,
        );
        assert!(input.session.update.reverse);

        tracker.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), &mut input);
        assert!(input.session.update.smart_bomb);

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT),
            &mut input,
        );
        assert!(input.session.update.hyperspace);

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut input,
        );
        assert!(input.session.update.fire);

        let mut released = PolledInput::default();
        tracker.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Modifier(ModifierKeyCode::LeftShift),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut released,
        );
        assert!(!tracker.held.thrust);

        let mut space_release = PolledInput::default();
        tracker.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char(' '),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut space_release,
        );
        assert!(!space_release.session.update.reverse);
    }

    #[test]
    fn input_tracker_maps_start_keys() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut input,
        );
        assert!(input.session.start_requested);
    }

    #[test]
    fn input_tracker_collects_typed_chars_for_xyzzy_god_mode_and_auto_fire() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        for key in ['X', 'Y', 'Z', 'Z', 'Y', 'G', 'F'] {
            tracker.handle_key_event(
                KeyEvent::new(KeyCode::Char(key), KeyModifiers::SHIFT),
                &mut input,
            );
        }

        assert_eq!(
            input.session.typed_chars,
            vec!['x', 'y', 'z', 'z', 'y', 'g', 'f']
        );
    }

    #[test]
    fn input_tracker_maps_backspace_for_initials_entry() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            &mut input,
        );

        assert!(input.session.backspace_requested);
    }

    #[test]
    fn input_tracker_maps_quit_keys() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            &mut input,
        );
        assert!(input.quit_requested);

        let mut input = PolledInput::default();
        tracker.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut input);
        assert!(input.quit_requested);
    }

    #[test]
    fn live_mode_rejects_non_interactive_terminals() {
        let error = validate_interactive_terminal(false, true).expect_err("terminal guard");
        assert!(error.to_string().contains("interactive terminal"));
    }

    #[test]
    fn title_mode_renders_the_arcade_attract_cycle_immediately() {
        let mut session = SessionState::with_high_scores(HighScoreTable::default());
        let beat = title_beat_for_elapsed_ms(0);
        assert_eq!(beat.kind, crate::attract::SceneKind::Logo);

        for _ in 0..30 {
            session.tick(SessionInput::default());
        }

        let beat = title_beat_for_elapsed_ms(2_000);
        assert!(matches!(
            beat.kind,
            crate::attract::SceneKind::Logo
                | crate::attract::SceneKind::Attract
                | crate::attract::SceneKind::HighScore
        ));

        let mut renderer = Renderer::with_size(960, 720);
        let image = render_title_frame(&mut renderer, &session, 2_000);
        assert!(image.pixels.iter().any(|pixel| *pixel != 0));
    }
}
