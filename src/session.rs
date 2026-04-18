use crate::game::{UpdateInput, World, WorldEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Title,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SessionInput {
    pub update: UpdateInput,
    pub start_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    GameStarted,
    GameRestarted,
    HighScoreUpdated,
    World(WorldEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    mode: SessionMode,
    world: World,
    high_score: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            mode: SessionMode::Title,
            world: World::bootstrap(),
            high_score: 0,
        }
    }

    pub fn mode(&self) -> SessionMode {
        self.mode
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn high_score(&self) -> u32 {
        self.high_score
    }

    pub fn tick(&mut self, input: SessionInput) -> Vec<SessionEvent> {
        match self.mode {
            SessionMode::Title => self.tick_title(input),
            SessionMode::Playing => self.tick_playing(input),
            SessionMode::GameOver => self.tick_game_over(input),
        }
    }

    fn tick_title(&mut self, input: SessionInput) -> Vec<SessionEvent> {
        if input.start_requested {
            self.world = World::bootstrap();
            self.mode = SessionMode::Playing;
            vec![SessionEvent::GameStarted]
        } else {
            Vec::new()
        }
    }

    fn tick_playing(&mut self, input: SessionInput) -> Vec<SessionEvent> {
        let mut events: Vec<SessionEvent> = self
            .world
            .step_live(input.update)
            .into_iter()
            .map(SessionEvent::World)
            .collect();

        let score = self.world.status().score;
        if score > self.high_score {
            self.high_score = score;
            events.push(SessionEvent::HighScoreUpdated);
        }

        if self.world.is_game_over() {
            self.mode = SessionMode::GameOver;
        }

        events
    }

    fn tick_game_over(&mut self, input: SessionInput) -> Vec<SessionEvent> {
        if input.start_requested {
            self.world = World::bootstrap();
            self.mode = SessionMode::Playing;
            vec![SessionEvent::GameRestarted]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, Status, UpdateInput, World, WorldEvent};

    use super::{SessionEvent, SessionInput, SessionMode, SessionState};

    #[test]
    fn session_starts_at_the_title_screen() {
        let session = SessionState::new();
        assert_eq!(session.mode(), SessionMode::Title);
        assert_eq!(session.high_score(), 0);
    }

    #[test]
    fn title_screen_requires_start_input() {
        let mut session = SessionState::new();

        let events = session.tick(SessionInput::default());
        assert!(events.is_empty());
        assert_eq!(session.mode(), SessionMode::Title);

        let events = session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        assert_eq!(session.mode(), SessionMode::Playing);
        assert_eq!(events, vec![SessionEvent::GameStarted]);
    }

    #[test]
    fn playing_updates_the_high_score_when_score_increases() {
        let mut session = SessionState::new();
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 3, 0, 0),
                Entity::new(EntityKind::Lander, 5, 3, 0, 0),
                Entity::new(EntityKind::Mutant, 9, 2, 0, 0),
            ],
        );

        let events = session.tick(SessionInput {
            update: UpdateInput {
                fire: true,
                ..UpdateInput::default()
            },
            ..SessionInput::default()
        });

        assert!(events.contains(&SessionEvent::World(WorldEvent::EnemyDestroyed)));
        assert!(events.contains(&SessionEvent::HighScoreUpdated));
        assert_eq!(session.high_score(), 150);
    }

    #[test]
    fn game_over_transitions_to_restartable_state() {
        let mut session = SessionState::new();
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 3, 3, 0, 0),
            ],
        );
        let events = session.tick(SessionInput::default());

        assert_eq!(session.mode(), SessionMode::GameOver);
        assert!(events.contains(&SessionEvent::World(WorldEvent::GameOver)));

        let events = session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        assert_eq!(events, vec![SessionEvent::GameRestarted]);
        assert_eq!(session.mode(), SessionMode::Playing);
        assert_eq!(session.world().status().score, 0);
        assert_eq!(session.world().status().lives, 3);
    }

    #[test]
    fn restart_keeps_the_best_score() {
        let mut session = SessionState::new();
        session.high_score = 900;
        session.mode = SessionMode::GameOver;

        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });

        assert_eq!(session.high_score(), 900);
    }
}
