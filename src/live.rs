use std::io::{self, Stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::audio::{AudioManager, SoundCue};
use crate::game::{UpdateInput, World, WorldEvent};
use crate::render;

const FRAME_DURATION: Duration = Duration::from_millis(90);

pub fn run_live(play_audio: bool) -> Result<()> {
    let audio = AudioManager::new();
    let mut stdout = io::stdout();
    let _guard = TerminalGuard::enter(&mut stdout)?;
    let mut input_tracker = InputTracker::default();
    let mut world = World::bootstrap();

    draw_frame(&mut stdout, &world)?;

    loop {
        let frame_started = Instant::now();
        let input = input_tracker.poll().context("polling live input")?;
        if input.quit_requested {
            break;
        }

        let events = world.step_live(input.update);
        draw_frame(&mut stdout, &world)?;

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

fn draw_frame(stdout: &mut Stdout, world: &World) -> Result<()> {
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).context("clearing live frame")?;
    write!(stdout, "{}", render::render(world)).context("writing live frame")?;
    stdout.flush().context("flushing live frame")?;
    Ok(())
}

fn cue_for_events(events: &[WorldEvent]) -> Option<SoundCue> {
    if events.contains(&WorldEvent::GameOver) || events.contains(&WorldEvent::PlayerHit) {
        Some(SoundCue::Explosion)
    } else if events.contains(&WorldEvent::WaveAdvanced) {
        Some(SoundCue::HighScoreChime)
    } else if events.contains(&WorldEvent::EnemyDestroyed) {
        Some(SoundCue::Explosion)
    } else if events.contains(&WorldEvent::HumanLost) {
        Some(SoundCue::AttractHum)
    } else if events.contains(&WorldEvent::ShotFired) {
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
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    fire: bool,
}

#[derive(Debug, Default)]
struct PolledInput {
    update: UpdateInput,
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

        input.update.left |= self.held.left;
        input.update.right |= self.held.right;
        input.update.up |= self.held.up;
        input.update.down |= self.held.down;
        input.update.fire |= self.held.fire;

        Ok(input)
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, input: &mut PolledInput) {
        let pressed = !matches!(key_event.kind, KeyEventKind::Release);

        match key_event.code {
            KeyCode::Esc if pressed => input.quit_requested = true,
            KeyCode::Char('q') | KeyCode::Char('Q') if pressed => input.quit_requested = true,
            KeyCode::Left => {
                set_held_flag(&mut self.held.left, key_event.kind, &mut input.update.left)
            }
            KeyCode::Right => set_held_flag(
                &mut self.held.right,
                key_event.kind,
                &mut input.update.right,
            ),
            KeyCode::Up => set_held_flag(&mut self.held.up, key_event.kind, &mut input.update.up),
            KeyCode::Down => {
                set_held_flag(&mut self.held.down, key_event.kind, &mut input.update.down)
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                set_held_flag(&mut self.held.left, key_event.kind, &mut input.update.left)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => set_held_flag(
                &mut self.held.right,
                key_event.kind,
                &mut input.update.right,
            ),
            KeyCode::Char('w') | KeyCode::Char('W') => {
                set_held_flag(&mut self.held.up, key_event.kind, &mut input.update.up)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                set_held_flag(&mut self.held.down, key_event.kind, &mut input.update.down)
            }
            KeyCode::Char(' ') => {
                set_held_flag(&mut self.held.fire, key_event.kind, &mut input.update.fire)
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

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut Stdout) -> Result<Self> {
        enable_raw_mode().context("enabling raw mode for live session")?;
        execute!(
            stdout,
            EnterAlternateScreen,
            Hide,
            MoveTo(0, 0),
            Clear(ClearType::All)
        )
        .context("switching into the live terminal screen")?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::{InputTracker, PolledInput, cue_for_events};
    use crate::audio::SoundCue;
    use crate::game::WorldEvent;

    #[test]
    fn event_priorities_prefer_game_over_and_hits() {
        assert_eq!(
            cue_for_events(&[WorldEvent::ShotFired, WorldEvent::GameOver]),
            Some(SoundCue::Explosion)
        );
        assert_eq!(
            cue_for_events(&[WorldEvent::WaveAdvanced]),
            Some(SoundCue::HighScoreChime)
        );
    }

    #[test]
    fn input_tracker_maps_letter_keys_and_releases() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
            &mut input,
        );
        assert!(input.update.right);

        let mut released = PolledInput::default();
        tracker.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut released,
        );
        assert!(!tracker.held.right);
    }

    #[test]
    fn input_tracker_maps_quit_keys() {
        let mut tracker = InputTracker::default();
        let mut input = PolledInput::default();

        tracker.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut input);
        assert!(input.quit_requested);
    }
}
