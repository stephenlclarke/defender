#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionBox {
    pub position: ScreenPosition,
    pub size: (u8, u8),
}

impl CollisionBox {
    pub const fn new(position: ScreenPosition, size: (u8, u8)) -> Self {
        Self { position, size }
    }

    pub fn overlaps(self, other: Self) -> bool {
        let left = i16::from(self.position.x);
        let right = left + i16::from(self.size.0);
        let top = i16::from(self.position.y);
        let bottom = top + i16::from(self.size.1);
        let other_left = i16::from(other.position.x);
        let other_right = other_left + i16::from(other.size.0);
        let other_top = i16::from(other.position.y);
        let other_bottom = other_top + i16::from(other.size.1);

        left < other_right && right > other_left && top < other_bottom && bottom > other_top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectileEnemyHit {
    pub projectile_index: usize,
    pub enemy_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerEnemyHit {
    pub enemy_index: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CollisionSystem;

impl CollisionSystem {
    pub fn first_projectile_enemy_hit(
        projectiles: &[CollisionBox],
        enemies: &[CollisionBox],
    ) -> Option<ProjectileEnemyHit> {
        for (projectile_index, projectile) in projectiles.iter().copied().enumerate() {
            for (enemy_index, enemy) in enemies.iter().copied().enumerate() {
                if projectile.overlaps(enemy) {
                    return Some(ProjectileEnemyHit {
                        projectile_index,
                        enemy_index,
                    });
                }
            }
        }
        None
    }

    pub fn first_player_enemy_hit(
        player: CollisionBox,
        enemies: &[CollisionBox],
    ) -> Option<PlayerEnemyHit> {
        enemies
            .iter()
            .copied()
            .position(|enemy| player.overlaps(enemy))
            .map(|enemy_index| PlayerEnemyHit { enemy_index })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveState {
    pub wave: u8,
    pub active_enemies: usize,
}

impl WaveState {
    pub const fn new(wave: u8, active_enemies: usize) -> Self {
        Self {
            wave,
            active_enemies,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveStatus {
    InProgress,
    Cleared { next_wave: u8 },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WaveSystem;

impl WaveSystem {
    pub fn evaluate(state: WaveState) -> WaveStatus {
        if state.active_enemies == 0 {
            WaveStatus::Cleared {
                next_wave: Self::next_wave(state.wave),
            }
        } else {
            WaveStatus::InProgress
        }
    }

    pub const fn next_wave(current_wave: u8) -> u8 {
        if current_wave == 0 {
            1
        } else {
            current_wave.saturating_add(1)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStock {
    pub lives: u8,
    pub smart_bombs: u8,
}

impl PlayerStock {
    pub const fn new(lives: u8, smart_bombs: u8) -> Self {
        Self { lives, smart_bombs }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerDamageFrame {
    pub stock: PlayerStock,
    pub game_over: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerDamageSystem;

impl PlayerDamageSystem {
    pub fn apply_hit(stock: PlayerStock) -> PlayerDamageFrame {
        let lives = stock.lives.saturating_sub(1);

        PlayerDamageFrame {
            stock: PlayerStock::new(lives, stock.smart_bombs),
            game_over: lives == 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreFrame {
    pub scores: ScoreSnapshot,
    pub stock: PlayerStock,
    pub bonus_awards: u8,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScoreSystem;

impl ScoreSystem {
    pub const BONUS_INTERVAL: u32 = 10_000;

    pub fn award_points(
        scores: ScoreSnapshot,
        stock: PlayerStock,
        current_player: u8,
        points: u32,
    ) -> ScoreFrame {
        let mut scores = scores;
        let mut stock = stock;
        let active_score = if current_player == 2 {
            &mut scores.player_two
        } else {
            &mut scores.player_one
        };
        *active_score = active_score.saturating_add(points);
        scores.high_score = scores.high_score.max(*active_score);

        let bonus_awards = bonus_awards(*active_score, scores.next_bonus);
        if bonus_awards > 0 {
            stock.lives = stock.lives.saturating_add(bonus_awards);
            stock.smart_bombs = stock.smart_bombs.saturating_add(bonus_awards);
            scores.next_bonus = advance_bonus_threshold(scores.next_bonus, bonus_awards);
        }

        ScoreFrame {
            scores,
            stock,
            bonus_awards,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreEntryFrame {
    pub qualifies: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HighScoreInitialsState {
    pub initials: [Option<char>; 3],
    pub cursor: u8,
}

impl HighScoreInitialsState {
    pub const EMPTY: Self = Self {
        initials: [None, None, None],
        cursor: 0,
    };

    pub const fn is_complete(self) -> bool {
        self.cursor >= 3
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HighScoreEntrySystem;

impl HighScoreEntrySystem {
    pub const fn evaluate(score: u32, high_score: u32) -> HighScoreEntryFrame {
        HighScoreEntryFrame {
            qualifies: score > 0 && score > high_score,
        }
    }

    pub fn enter_initial(
        state: HighScoreInitialsState,
        initial: Option<char>,
        backspace: bool,
    ) -> HighScoreInitialsFrame {
        let mut state = state;
        let mut accepted = false;

        if backspace && state.cursor > 0 {
            state.cursor -= 1;
            state.initials[usize::from(state.cursor)] = None;
            return HighScoreInitialsFrame {
                state,
                accepted,
                submitted: false,
            };
        }

        if let Some(initial) = initial.and_then(normalized_initial)
            && !state.is_complete()
        {
            state.initials[usize::from(state.cursor)] = Some(initial);
            state.cursor += 1;
            accepted = true;
        }

        HighScoreInitialsFrame {
            state,
            accepted,
            submitted: accepted && state.is_complete(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighScoreInitialsFrame {
    pub state: HighScoreInitialsState,
    pub accepted: bool,
    pub submitted: bool,
}

fn normalized_initial(initial: char) -> Option<char> {
    initial
        .is_ascii_alphabetic()
        .then(|| initial.to_ascii_uppercase())
}

fn bonus_awards(score: u32, next_bonus: u32) -> u8 {
    if next_bonus == u32::MAX || score < next_bonus {
        return 0;
    }

    let thresholds = 1 + (score - next_bonus) / ScoreSystem::BONUS_INTERVAL;
    thresholds.min(u32::from(u8::MAX)) as u8
}

fn advance_bonus_threshold(next_bonus: u32, bonus_awards: u8) -> u32 {
    next_bonus.saturating_add(ScoreSystem::BONUS_INTERVAL.saturating_mul(u32::from(bonus_awards)))
}
