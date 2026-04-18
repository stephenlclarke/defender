use crate::game::{UpdateInput, World, WorldEvent};
use crate::high_scores::{HighScoreTable, sanitize_initials};

const EASTER_EGG_CODE: [char; 5] = ['x', 'y', 'z', 'z', 'y'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Title,
    Playing,
    EnteringInitials,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionInput {
    pub update: UpdateInput,
    pub start_requested: bool,
    pub backspace_requested: bool,
    pub typed_chars: Vec<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEvent {
    GameStarted,
    GameRestarted,
    HighScoreUpdated,
    HighScoreSaved,
    World(WorldEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    mode: SessionMode,
    world: World,
    todays_high_scores: HighScoreTable,
    high_scores: HighScoreTable,
    high_score: u32,
    title_ticks: u64,
    pending_initials: Option<PendingInitials>,
    persist_high_scores: bool,
    xyzzy: XyzzyState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingInitials {
    score: u32,
    todays_rank: Option<usize>,
    all_time_rank: Option<usize>,
    letters: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct XyzzyState {
    active: bool,
    sequence_index: usize,
    invincible: bool,
    auto_fire: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self::with_high_scores(HighScoreTable::default())
    }

    pub fn load() -> Self {
        let mut session = Self::with_high_scores(HighScoreTable::load_default());
        session.persist_high_scores = true;
        session
    }

    pub fn with_high_scores(high_scores: HighScoreTable) -> Self {
        Self {
            mode: SessionMode::Title,
            world: World::bootstrap(),
            // Red-label `THSTAB` starts from the ROM defaults each power cycle,
            // while `CRHSTD` persists in CMOS. We mirror that split here by
            // resetting the left-hand today's table on load/new session.
            todays_high_scores: HighScoreTable::default(),
            high_score: high_scores.top_score(),
            high_scores,
            title_ticks: 0,
            pending_initials: None,
            persist_high_scores: false,
            xyzzy: XyzzyState::default(),
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
            .max(self.todays_high_scores.top_score())
            .max(self.high_scores.top_score())
    }

    pub fn high_scores(&self) -> &HighScoreTable {
        &self.high_scores
    }

    pub fn todays_high_scores(&self) -> &HighScoreTable {
        &self.todays_high_scores
    }

    pub fn title_ticks(&self) -> u64 {
        self.title_ticks
    }

    pub fn pending_initials(&self) -> Option<&PendingInitials> {
        self.pending_initials.as_ref()
    }

    pub fn xyzzy_active(&self) -> bool {
        self.xyzzy.active
    }

    pub fn invincible(&self) -> bool {
        self.xyzzy.invincible
    }

    pub fn auto_fire(&self) -> bool {
        self.xyzzy.auto_fire
    }

    pub fn tick(&mut self, input: SessionInput) -> Vec<SessionEvent> {
        if !matches!(self.mode, SessionMode::EnteringInitials) {
            self.handle_easter_egg_input(&input.typed_chars);
        }

        match self.mode {
            SessionMode::Title => self.tick_title(input.start_requested),
            SessionMode::Playing => self.tick_playing(input.update),
            SessionMode::EnteringInitials => self.tick_entering_initials(
                &input.typed_chars,
                input.backspace_requested,
                input.start_requested,
            ),
            SessionMode::GameOver => self.tick_game_over(input.start_requested),
        }
    }

    fn tick_title(&mut self, start_requested: bool) -> Vec<SessionEvent> {
        if start_requested {
            self.world = World::bootstrap();
            self.title_ticks = 0;
            self.pending_initials = None;
            self.mode = SessionMode::Playing;
            vec![SessionEvent::GameStarted]
        } else {
            self.title_ticks = self.title_ticks.saturating_add(1);
            Vec::new()
        }
    }

    fn tick_playing(&mut self, mut input: UpdateInput) -> Vec<SessionEvent> {
        input.secret_mode = self.xyzzy.active;
        input.invincible = self.xyzzy.invincible;
        input.auto_fire = self.xyzzy.auto_fire;
        let mut events: Vec<SessionEvent> = self
            .world
            .step_live(input)
            .into_iter()
            .map(SessionEvent::World)
            .collect();

        let score = self.world.status().score;
        if score > self.high_score {
            self.high_score = score;
            events.push(SessionEvent::HighScoreUpdated);
        }

        if self.world.is_game_over() {
            if self.todays_high_scores.qualifies(score) || self.high_scores.qualifies(score) {
                self.begin_initials_entry(score);
            } else {
                self.mode = SessionMode::GameOver;
            }
        }

        events
    }

    fn tick_entering_initials(
        &mut self,
        typed_chars: &[char],
        backspace_requested: bool,
        confirm_requested: bool,
    ) -> Vec<SessionEvent> {
        let Some(initials) = self.pending_initials.as_mut() else {
            self.mode = SessionMode::GameOver;
            return Vec::new();
        };

        if backspace_requested {
            initials.letters.pop();
        }

        for &character in typed_chars {
            if initials.letters.len() >= 3 {
                break;
            }

            if character.is_ascii_alphabetic() {
                initials.letters.push(character.to_ascii_uppercase());
            }
        }

        if confirm_requested && initials.letters.len() == 3 {
            self.commit_initials();
            return vec![SessionEvent::HighScoreSaved];
        }

        Vec::new()
    }

    fn tick_game_over(&mut self, start_requested: bool) -> Vec<SessionEvent> {
        if start_requested {
            self.world = World::bootstrap();
            self.title_ticks = 0;
            self.pending_initials = None;
            self.mode = SessionMode::Playing;
            vec![SessionEvent::GameRestarted]
        } else {
            Vec::new()
        }
    }

    fn handle_easter_egg_input(&mut self, typed_chars: &[char]) {
        for &character in typed_chars {
            let character = character.to_ascii_lowercase();
            self.update_easter_egg_sequence(character);
            if self.xyzzy.active && character == 'g' {
                self.xyzzy.invincible = !self.xyzzy.invincible;
            } else if self.xyzzy.active && character == 'f' {
                self.xyzzy.auto_fire = !self.xyzzy.auto_fire;
            }
        }
    }

    fn update_easter_egg_sequence(&mut self, character: char) -> bool {
        if character == EASTER_EGG_CODE[self.xyzzy.sequence_index] {
            self.xyzzy.sequence_index += 1;
            if self.xyzzy.sequence_index == EASTER_EGG_CODE.len() {
                self.toggle_easter_egg_mode();
            }
            return true;
        }

        self.xyzzy.sequence_index = usize::from(character == EASTER_EGG_CODE[0]);
        character == EASTER_EGG_CODE[0]
    }

    fn toggle_easter_egg_mode(&mut self) {
        self.xyzzy.active = !self.xyzzy.active;
        self.xyzzy.sequence_index = 0;
        if !self.xyzzy.active {
            self.xyzzy.invincible = false;
            self.xyzzy.auto_fire = false;
        }
    }

    fn begin_initials_entry(&mut self, score: u32) {
        self.mode = SessionMode::EnteringInitials;
        self.pending_initials = Some(PendingInitials {
            score,
            todays_rank: self.todays_high_scores.projected_rank(score),
            all_time_rank: self.high_scores.projected_rank(score),
            letters: String::new(),
        });
    }

    fn commit_initials(&mut self) {
        let Some(initials) = self.pending_initials.take() else {
            return;
        };

        if initials.todays_rank.is_some() {
            self.todays_high_scores
                .insert(&initials.letters, initials.score);
        }
        if initials.all_time_rank.is_some() {
            self.high_scores.insert(&initials.letters, initials.score);
        }
        self.high_score = self.high_score.max(self.high_scores.top_score());
        self.mode = SessionMode::GameOver;

        if self.persist_high_scores {
            let _ = self.high_scores.save_default();
        }
    }
}

impl PendingInitials {
    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn rank(&self) -> usize {
        self.todays_rank.or(self.all_time_rank).unwrap_or(1)
    }

    pub fn letters(&self) -> &str {
        &self.letters
    }

    pub fn display_letters(&self) -> String {
        let mut display = sanitize_initials(&self.letters);
        display.truncate(3);
        display
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, Status, UpdateInput, World, WorldEvent};
    use crate::high_scores::HighScoreTable;

    use super::{SessionEvent, SessionInput, SessionMode, SessionState};

    #[test]
    fn session_starts_at_the_title_screen_with_seeded_high_score() {
        let session = SessionState::new();
        assert_eq!(session.mode(), SessionMode::Title);
        assert_eq!(session.high_score(), 21_270);
        assert_eq!(session.high_scores().entries().len(), 8);
    }

    #[test]
    fn title_screen_requires_start_input() {
        let mut session = SessionState::new();

        let events = session.tick(SessionInput::default());
        assert!(events.is_empty());
        assert_eq!(session.mode(), SessionMode::Title);
        assert_eq!(session.title_ticks(), 1);

        let events = session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        assert_eq!(session.mode(), SessionMode::Playing);
        assert_eq!(session.title_ticks(), 0);
        assert_eq!(events, vec![SessionEvent::GameStarted]);
    }

    #[test]
    fn title_ticks_accumulate_while_waiting_at_the_title_screen() {
        let mut session = SessionState::new();

        session.tick(SessionInput::default());
        session.tick(SessionInput::default());
        session.tick(SessionInput::default());

        assert_eq!(session.mode(), SessionMode::Title);
        assert_eq!(session.title_ticks(), 3);
    }

    #[test]
    fn playing_updates_the_high_score_when_score_increases() {
        let mut session = SessionState::with_high_scores(HighScoreTable::default());
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 249_900,
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
        assert_eq!(session.high_score(), 250_050);
    }

    #[test]
    fn non_qualifying_game_over_transitions_to_restartable_state() {
        let mut session = SessionState::new();
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 100,
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
    fn qualifying_game_over_enters_initials_mode() {
        let mut session = SessionState::new();
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 60_000,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 3, 3, 0, 0),
            ],
        );

        session.tick(SessionInput::default());

        assert_eq!(session.mode(), SessionMode::EnteringInitials);
        assert_eq!(
            session.pending_initials().map(|entry| entry.rank()),
            Some(1)
        );
    }

    #[test]
    fn initials_entry_accepts_letters_backspace_and_confirm() {
        let mut session = SessionState::new();
        session.mode = SessionMode::EnteringInitials;
        session.pending_initials = Some(super::PendingInitials {
            score: 60_000,
            todays_rank: Some(1),
            all_time_rank: Some(1),
            letters: String::new(),
        });

        session.tick(SessionInput {
            typed_chars: vec!['r', 'o'],
            ..SessionInput::default()
        });
        assert_eq!(
            session
                .pending_initials()
                .map(|entry| entry.display_letters()),
            Some(String::from("RO_"))
        );

        session.tick(SessionInput {
            backspace_requested: true,
            typed_chars: vec!['m', 'x'],
            ..SessionInput::default()
        });
        assert_eq!(
            session
                .pending_initials()
                .map(|entry| entry.display_letters()),
            Some(String::from("RMX"))
        );

        let events = session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        assert_eq!(events, vec![SessionEvent::HighScoreSaved]);
        assert_eq!(session.mode(), SessionMode::GameOver);
        assert_eq!(session.todays_high_scores().entries()[0].initials, "RMX");
        assert_eq!(session.todays_high_scores().entries()[0].score, 60_000);
        assert_eq!(session.high_scores().entries()[0].initials, "RMX");
        assert_eq!(session.high_scores().entries()[0].score, 60_000);
    }

    #[test]
    fn todays_greatest_accepts_scores_that_do_not_make_all_time() {
        let mut all_time = HighScoreTable::default();
        all_time.insert("ZZZ", 30_000);
        let mut session = SessionState::with_high_scores(all_time);
        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });
        session.world = World::with_entities(
            12,
            8,
            Status {
                score: 7_500,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 3, 3, 0, 0),
                Entity::new(EntityKind::Lander, 3, 3, 0, 0),
            ],
        );

        session.tick(SessionInput::default());

        assert_eq!(session.mode(), SessionMode::EnteringInitials);
        assert_eq!(session.pending_initials().map(|entry| entry.rank()), Some(8));
    }

    #[test]
    fn restart_keeps_the_best_score() {
        let mut session = SessionState::new();
        session.mode = SessionMode::GameOver;

        session.tick(SessionInput {
            start_requested: true,
            ..SessionInput::default()
        });

        assert_eq!(session.high_score(), 21_270);
    }

    #[test]
    fn xyzzy_toggles_secret_mode_and_resets_hidden_toggles() {
        let mut session = SessionState::new();

        session.tick(SessionInput {
            typed_chars: vec!['X', 'Y', 'Z', 'Z', 'Y'],
            ..SessionInput::default()
        });
        assert!(session.xyzzy_active());
        assert!(!session.invincible());
        assert!(!session.auto_fire());

        session.tick(SessionInput {
            typed_chars: vec!['G', 'F'],
            ..SessionInput::default()
        });
        assert!(session.invincible());
        assert!(session.auto_fire());

        session.tick(SessionInput {
            typed_chars: vec!['X', 'Y', 'Z', 'Z', 'Y'],
            ..SessionInput::default()
        });
        assert!(!session.xyzzy_active());
        assert!(!session.invincible());
        assert!(!session.auto_fire());
    }

    #[test]
    fn secret_invincibility_requires_xyzzy_mode() {
        let mut session = SessionState::new();

        session.tick(SessionInput {
            typed_chars: vec!['g'],
            ..SessionInput::default()
        });

        assert!(!session.invincible());
    }

    #[test]
    fn secret_auto_fire_requires_xyzzy_mode() {
        let mut session = SessionState::new();

        session.tick(SessionInput {
            typed_chars: vec!['f'],
            ..SessionInput::default()
        });

        assert!(!session.auto_fire());
    }
}
