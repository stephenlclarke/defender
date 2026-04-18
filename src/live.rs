use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, ModifierKeyCode};

use crate::attract::{AttractBeat, SceneKind, beat_for_elapsed_ms};
use crate::audio::{AudioManager, SoundCue};
use crate::kitty::KittyGraphics;
use crate::session::{SessionEvent, SessionInput, SessionMode, SessionState};
use crate::terminal::{TerminalSession, geometry};
use crate::video::{RenderedImage, Renderer, Screen};

const FRAME_DURATION: Duration = Duration::from_millis(90);
const TITLE_STATIC_HOLD_TICKS: u64 = 24;

pub fn run_live(play_audio: bool) -> Result<()> {
    ensure_interactive_terminal()?;
    KittyGraphics::ensure_supported()?;

    let audio = AudioManager::new();
    let mut stdout = io::stdout();
    let _terminal = TerminalSession::enter(&mut stdout)?;
    let mut input_tracker = InputTracker::default();
    let mut session = SessionState::load();
    let mut terminal_geometry = geometry()?;
    let mut renderer = Renderer::new(terminal_geometry);
    let mut graphics = KittyGraphics::new(terminal_geometry.cols, terminal_geometry.rows);

    draw_frame(&mut stdout, &session, &mut renderer, &mut graphics)?;

    loop {
        let frame_started = Instant::now();
        sync_terminal_geometry(&mut terminal_geometry, &mut renderer, &mut graphics)?;

        let input = input_tracker.poll().context("polling live input")?;
        if input.quit_requested {
            break;
        }

        let events = session.tick(input.session);
        draw_frame(&mut stdout, &session, &mut renderer, &mut graphics)?;

        if play_audio && let Some(cue) = cue_for_events(&events) {
            audio.play_cue_blocking(cue);
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
) -> Result<()> {
    let image = render_session_frame(renderer, session);
    graphics
        .draw_frame(stdout, image)
        .context("drawing kitty graphics frame")?;
    stdout.flush().context("flushing kitty graphics frame")?;
    Ok(())
}

fn render_session_frame<'a>(
    renderer: &'a mut Renderer,
    session: &SessionState,
) -> &'a RenderedImage {
    match session.mode() {
        SessionMode::Title => render_title_frame(renderer, session),
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

fn render_title_frame<'a>(renderer: &'a mut Renderer, session: &SessionState) -> &'a RenderedImage {
    if let Some(beat) = title_beat_for_session(session) {
        match beat.kind {
            SceneKind::Logo => renderer.render(Screen::Logo),
            SceneKind::Attract => {
                let mut world = crate::game::World::bootstrap();
                for _ in 0..beat.world_steps {
                    world.step();
                }
                renderer.render(Screen::Attract {
                    world: &world,
                    revealed_score_entries: beat.revealed_score_entries,
                })
            }
            SceneKind::HighScore => renderer.render(Screen::HighScores {
                todays: session.todays_high_scores(),
                all_time: session.high_scores(),
            }),
        }
    } else {
        renderer.render(Screen::Title {
            high_score: session.high_score(),
            xyzzy_active: session.xyzzy_active(),
            invincible: session.invincible(),
            auto_fire: session.auto_fire(),
        })
    }
}

fn title_beat_for_session(session: &SessionState) -> Option<AttractBeat> {
    if session.title_ticks() < TITLE_STATIC_HOLD_TICKS {
        return None;
    }

    let attract_elapsed_ms = (session.title_ticks() - TITLE_STATIC_HOLD_TICKS)
        .saturating_mul(FRAME_DURATION.as_millis() as u64);
    Some(beat_for_elapsed_ms(attract_elapsed_ms))
}

fn cue_for_events(events: &[SessionEvent]) -> Option<SoundCue> {
    if events.contains(&SessionEvent::GameStarted) || events.contains(&SessionEvent::GameRestarted)
    {
        Some(SoundCue::LogoFanfare)
    } else if events.contains(&SessionEvent::HighScoreUpdated)
        || events.contains(&SessionEvent::HighScoreSaved)
    {
        Some(SoundCue::HighScoreChime)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::GameOver))
        || events.contains(&SessionEvent::World(crate::game::WorldEvent::PlayerHit))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::SmartBombDetonated,
        ))
    {
        Some(SoundCue::Explosion)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::EnemyFired))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::HyperspaceUsed,
        ))
    {
        Some(SoundCue::EnemySweep)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::WaveAdvanced)) {
        Some(SoundCue::HighScoreChime)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::HumanRescued)) {
        Some(SoundCue::HumanSaved)
    } else if events.contains(&SessionEvent::World(
        crate::game::WorldEvent::EnemyDestroyed,
    )) {
        Some(SoundCue::Explosion)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::HumanLost)) {
        Some(SoundCue::AttractHum)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::ShotFired)) {
        Some(SoundCue::PlayerShot)
    } else {
        None
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
        InputTracker, PolledInput, TITLE_STATIC_HOLD_TICKS, cue_for_events, render_title_frame,
        title_beat_for_session, validate_interactive_terminal,
    };
    use crate::audio::SoundCue;
    use crate::game::WorldEvent;
    use crate::high_scores::HighScoreTable;
    use crate::session::{SessionEvent, SessionInput, SessionState};
    use crate::video::Renderer;

    #[test]
    fn event_priorities_prefer_game_over_and_hits() {
        assert_eq!(
            cue_for_events(&[
                SessionEvent::World(WorldEvent::HyperspaceUsed),
                SessionEvent::World(WorldEvent::GameOver),
            ]),
            Some(SoundCue::Explosion)
        );
        assert_eq!(
            cue_for_events(&[SessionEvent::World(WorldEvent::WaveAdvanced)]),
            Some(SoundCue::HighScoreChime)
        );
        assert_eq!(
            cue_for_events(&[SessionEvent::World(WorldEvent::EnemyFired)]),
            Some(SoundCue::EnemySweep)
        );
        assert_eq!(
            cue_for_events(&[SessionEvent::World(WorldEvent::SmartBombDetonated)]),
            Some(SoundCue::Explosion)
        );
        assert_eq!(
            cue_for_events(&[SessionEvent::World(WorldEvent::HumanRescued)]),
            Some(SoundCue::HumanSaved)
        );
        assert_eq!(
            cue_for_events(&[SessionEvent::HighScoreSaved]),
            Some(SoundCue::HighScoreChime)
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
    fn title_mode_switches_to_attract_pages_after_the_hold_period() {
        let mut session = SessionState::with_high_scores(HighScoreTable::default());
        for _ in 0..=TITLE_STATIC_HOLD_TICKS {
            session.tick(SessionInput::default());
        }

        let beat = title_beat_for_session(&session).expect("title should be in attract mode");
        assert!(matches!(
            beat.kind,
            crate::attract::SceneKind::Logo
                | crate::attract::SceneKind::Attract
                | crate::attract::SceneKind::HighScore
        ));

        let mut renderer = Renderer::with_size(960, 720);
        let image = render_title_frame(&mut renderer, &session);
        assert!(image.pixels.iter().any(|pixel| *pixel != 0));
    }
}
