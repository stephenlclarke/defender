use std::io::{self, IsTerminal, Stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyboardEnhancementFlags, ModifierKeyCode,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, supports_keyboard_enhancement,
    },
};

use crate::audio::{AudioManager, SoundCue};
use crate::render;
use crate::session::{SessionEvent, SessionInput, SessionMode, SessionState};

const FRAME_DURATION: Duration = Duration::from_millis(90);

pub fn run_live(play_audio: bool) -> Result<()> {
    ensure_interactive_terminal()?;
    let audio = AudioManager::new();
    let mut stdout = io::stdout();
    let _guard = TerminalGuard::enter(&mut stdout)?;
    let mut input_tracker = InputTracker::default();
    let mut session = SessionState::load();

    draw_frame(&mut stdout, &session)?;

    loop {
        let frame_started = Instant::now();
        let input = input_tracker.poll().context("polling live input")?;
        if input.quit_requested {
            break;
        }

        let events = session.tick(input.session);
        draw_frame(&mut stdout, &session)?;

        if play_audio && let Some(cue) = cue_for_events(&events) {
            audio.play_cue_blocking(cue);
        }

        let elapsed = frame_started.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }

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

fn draw_frame(stdout: &mut Stdout, session: &SessionState) -> Result<()> {
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).context("clearing live frame")?;
    let text = match session.mode() {
        SessionMode::Title => render::render_title_screen_with_flags(
            session.high_score(),
            session.xyzzy_active(),
            session.invincible(),
        ),
        SessionMode::EnteringInitials => {
            let pending = session
                .pending_initials()
                .expect("initials mode should have pending entry");
            let display_letters = pending.display_letters();
            render::render_initials_entry_screen(
                session.world(),
                &render::InitialsEntryView {
                    high_score: session.high_score(),
                    high_scores: session.high_scores(),
                    entry_score: pending.score(),
                    entry_rank: pending.rank(),
                    initials: &display_letters,
                    xyzzy_active: session.xyzzy_active(),
                    invincible: session.invincible(),
                },
            )
        }
        SessionMode::Playing => render::render_with_flags(
            session.world(),
            session.xyzzy_active(),
            session.invincible(),
        ),
        SessionMode::GameOver => render::render_game_over_screen_with_flags(
            session.world(),
            session.high_score(),
            session.xyzzy_active(),
            session.invincible(),
        ),
    };
    write!(stdout, "{text}").context("writing live frame")?;
    stdout.flush().context("flushing live frame")?;
    Ok(())
}

fn cue_for_events(events: &[SessionEvent]) -> Option<SoundCue> {
    if events.contains(&SessionEvent::GameStarted) || events.contains(&SessionEvent::GameRestarted)
    {
        Some(SoundCue::LogoFanfare)
    } else if events.contains(&SessionEvent::HighScoreUpdated)
        || events.contains(&SessionEvent::HighScoreSaved)
    {
        Some(SoundCue::HighScoreChime)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::EnemyFired))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::HyperspaceUsed,
        ))
    {
        Some(SoundCue::EnemySweep)
    } else if events.contains(&SessionEvent::World(crate::game::WorldEvent::GameOver))
        || events.contains(&SessionEvent::World(crate::game::WorldEvent::PlayerHit))
        || events.contains(&SessionEvent::World(
            crate::game::WorldEvent::SmartBombDetonated,
        ))
    {
        Some(SoundCue::Explosion)
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
    reverse: bool,
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
        input.session.update.reverse |= self.held.reverse;

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
            KeyCode::Char(' ') => set_held_flag(
                &mut self.held.reverse,
                key_event.kind,
                &mut input.session.update.reverse,
            ),
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

struct TerminalGuard {
    keyboard_enhancement_supported: bool,
}

impl TerminalGuard {
    fn enter(stdout: &mut Stdout) -> Result<Self> {
        enable_raw_mode().context("enabling raw mode for live session")?;
        let mut keyboard_enhancement_supported = false;
        let result = (|| {
            keyboard_enhancement_supported = supports_keyboard_enhancement().unwrap_or(false);
            if keyboard_enhancement_supported {
                execute!(
                    stdout,
                    EnterAlternateScreen,
                    Hide,
                    MoveTo(0, 0),
                    Clear(ClearType::All),
                    PushKeyboardEnhancementFlags(
                        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                            | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    )
                )
                .context("switching into the live terminal screen with modifier reporting")?;
            } else {
                execute!(
                    stdout,
                    EnterAlternateScreen,
                    Hide,
                    MoveTo(0, 0),
                    Clear(ClearType::All)
                )
                .context("switching into the live terminal screen")?;
            }
            Ok(Self {
                keyboard_enhancement_supported,
            })
        })();

        if result.is_err() {
            if keyboard_enhancement_supported {
                let _ = execute!(
                    stdout,
                    PopKeyboardEnhancementFlags,
                    Show,
                    LeaveAlternateScreen
                );
            } else {
                let _ = execute!(stdout, Show, LeaveAlternateScreen);
            }
            let _ = disable_raw_mode();
        }

        result
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        if self.keyboard_enhancement_supported {
            let _ = execute!(
                stdout,
                PopKeyboardEnhancementFlags,
                Show,
                LeaveAlternateScreen
            );
        } else {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
        }
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode};

    use super::{InputTracker, PolledInput, cue_for_events, validate_interactive_terminal};
    use crate::audio::SoundCue;
    use crate::game::WorldEvent;
    use crate::session::SessionEvent;

    #[test]
    fn event_priorities_prefer_game_over_and_hits() {
        assert_eq!(
            cue_for_events(&[
                SessionEvent::World(WorldEvent::ShotFired),
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
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
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
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
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
    fn input_tracker_collects_typed_chars_for_xyzzy_and_god_mode() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        for key in ['x', 'y', 'z', 'z', 'y', 'g'] {
            tracker.handle_key_event(
                KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE),
                &mut input,
            );
        }

        assert_eq!(
            input.session.typed_chars,
            vec!['x', 'y', 'z', 'z', 'y', 'g']
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

        tracker.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut input);
        assert!(input.quit_requested);
    }

    #[test]
    fn live_mode_rejects_non_interactive_terminals() {
        let error = validate_interactive_terminal(false, true).expect_err("terminal guard");
        assert!(error.to_string().contains("interactive terminal"));
    }
}
