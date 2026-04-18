use crate::audio::SoundCue;
use crate::game::World;
use crate::high_scores::{HighScoreEntry, HighScoreTable};

const ATTRACT_SCORE_CARD: [(&str, u32); 6] = [
    ("LANDER", 150),
    ("MUTANT", 150),
    ("BAITER", 200),
    ("BOMBER", 250),
    ("POD", 1000),
    ("SWARMER", 150),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    Logo,
    Attract,
    HighScore,
}

impl SceneKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "logo" => Some(Self::Logo),
            "attract" => Some(Self::Attract),
            "high-score" => Some(Self::HighScore),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub kind: SceneKind,
    pub lines: Vec<String>,
}

impl Scene {
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttractBeat {
    pub kind: SceneKind,
    pub cue: Option<SoundCue>,
    pub hold_ms: u64,
    pub world_steps: usize,
    pub revealed_score_entries: usize,
}

impl AttractBeat {
    pub fn scene(self) -> Scene {
        self.scene_with_tables(&HighScoreTable::default(), &HighScoreTable::default())
    }

    pub fn scene_with_tables(self, todays: &HighScoreTable, all_time: &HighScoreTable) -> Scene {
        match self.kind {
            SceneKind::Logo => logo_scene(),
            SceneKind::Attract => {
                let mut world = World::bootstrap();
                for _ in 0..self.world_steps {
                    world.step();
                }
                attract_scene(&world, self.revealed_score_entries)
            }
            SceneKind::HighScore => high_score_scene_with_tables(todays, all_time),
        }
    }
}

pub fn attract_cycle() -> [AttractBeat; 9] {
    [
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 1_000,
            world_steps: 0,
            revealed_score_entries: 0,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 650,
            world_steps: 0,
            revealed_score_entries: 0,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 500,
            world_steps: 2,
            revealed_score_entries: 1,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::PlayerShot),
            hold_ms: 500,
            world_steps: 4,
            revealed_score_entries: 2,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 500,
            world_steps: 6,
            revealed_score_entries: 3,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::Explosion),
            hold_ms: 500,
            world_steps: 8,
            revealed_score_entries: 4,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 500,
            world_steps: 10,
            revealed_score_entries: 5,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::HumanSaved),
            hold_ms: 550,
            world_steps: 12,
            revealed_score_entries: 6,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 1_250,
            world_steps: 0,
            revealed_score_entries: 0,
        },
    ]
}

pub fn scene_for_elapsed_ms(
    elapsed_ms: u64,
    todays: &HighScoreTable,
    all_time: &HighScoreTable,
) -> Scene {
    beat_for_elapsed_ms(elapsed_ms).scene_with_tables(todays, all_time)
}

pub fn beat_for_elapsed_ms(elapsed_ms: u64) -> AttractBeat {
    let cycle = attract_cycle();
    let cycle_ms = cycle.iter().map(|beat| beat.hold_ms).sum::<u64>();
    let mut remaining = if cycle_ms == 0 {
        0
    } else {
        elapsed_ms % cycle_ms
    };

    for beat in cycle {
        if remaining < beat.hold_ms {
            return beat;
        }
        remaining -= beat.hold_ms;
    }

    cycle[0]
}

pub fn logo_scene() -> Scene {
    Scene {
        kind: SceneKind::Logo,
        lines: vec![
            // Red-label attract text follows the `WILLIAMS` / `ELECTRONICS INC.` /
            // `PRESENTS` / `DEFENDER` sequence from `mess0.src` and `amode1.src`.
            String::from("                 WILLIAMS"),
            String::from("             ELECTRONICS INC."),
            String::from("                  PRESENTS"),
            String::new(),
            String::from(r" ____  _____ _____ _____ _   _ ____  _____ ____  "),
            String::from(r"|  _ \| ____|  ___| ____| \ | |  _ \| ____|  _ \ "),
            String::from(r"| | | |  _| | |_  |  _| |  \| | | | |  _| | |_) |"),
            String::from(r"| |_| | |___|  _| | |___| |\  | |_| | |___|  _ < "),
            String::from(r"|____/|_____|_|   |_____|_| \_|____/|_____|_| \_\"),
            String::new(),
            String::from("      COPYRIGHT 1980 - WILLIAMS ELECTRONICS"),
        ],
    }
}

pub fn attract_scene(world: &World, revealed_score_entries: usize) -> Scene {
    let mut lines = vec![String::from("PRESS 1 OR 2 PLAYER START"), String::new()];
    lines.extend(crate::render::render_grid(world));
    lines.push(String::new());
    // `TEXTAB` in `amode1.src` rotates the attract score legend in this order.
    lines.push(String::from("SCANNER"));
    lines.extend(
        ATTRACT_SCORE_CARD
            .into_iter()
            .take(revealed_score_entries)
            .map(|(name, score)| format!("{name:<8}{score:>8}")),
    );

    Scene {
        kind: SceneKind::Attract,
        lines,
    }
}

pub fn high_score_scene() -> Scene {
    high_score_scene_with_tables(&HighScoreTable::default(), &HighScoreTable::default())
}

pub fn high_score_scene_with_tables(todays: &HighScoreTable, all_time: &HighScoreTable) -> Scene {
    let mut lines = vec![
        String::from("DEFENDER"),
        String::from("HALL OF FAME"),
        String::new(),
        format!("{:<24}{}", "TODAYS GREATEST", "ALL TIME GREATEST"),
        format!("{:<24}{}", " RANK  INITIALS SCORE", " RANK  INITIALS SCORE"),
    ];

    // Red-label `HALDIS` renders the volatile `THSTAB` "TODAYS GREATEST" table on
    // the left and the CMOS-backed `CRHSTD` "ALL TIME GREATEST" table on the right.
    let row_count = todays.entries().len().max(all_time.entries().len());
    for index in 0..row_count {
        let left = compact_score_row(index + 1, todays.entries().get(index));
        let right = compact_score_row(index + 1, all_time.entries().get(index));
        lines.push(format!("{left:<24}{right}"));
    }

    Scene {
        kind: SceneKind::HighScore,
        lines,
    }
}

fn compact_score_row(rank: usize, entry: Option<&HighScoreEntry>) -> String {
    match entry {
        Some(entry) => format!("{rank:>2}. {:<3} {:>6}", entry.initials, entry.score),
        None => format!("{rank:>2}. --- ------"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{audio::SoundCue, game::World, high_scores::HighScoreTable};

    use super::{
        SceneKind, attract_cycle, attract_scene, high_score_scene, logo_scene, scene_for_elapsed_ms,
    };

    #[test]
    fn parse_scene_kind_recognises_supported_values() {
        assert_eq!(SceneKind::parse("logo"), Some(SceneKind::Logo));
        assert_eq!(SceneKind::parse("attract"), Some(SceneKind::Attract));
        assert_eq!(SceneKind::parse("high-score"), Some(SceneKind::HighScore));
        assert_eq!(SceneKind::parse("unknown"), None);
    }

    #[test]
    fn logo_scene_contains_live_launch_hints() {
        let scene = logo_scene();
        let text = scene.text();

        assert!(text.contains("WILLIAMS"));
        assert!(text.contains("PRESENTS"));
    }

    #[test]
    fn attract_scene_wraps_rendered_world() {
        let scene = attract_scene(&World::bootstrap(), 6);
        let text = scene.text();

        assert!(text.contains("PRESS 1 OR 2 PLAYER START"));
        assert!(text.contains("SCANNER"));
        assert!(text.contains("LANDER"));
        assert!(text.contains("SWARMER"));
        assert!(text.contains("THREAT"));
    }

    #[test]
    fn high_score_scene_lists_ranked_scores() {
        let scene = high_score_scene();
        let text = scene.text();

        assert!(text.contains("HALL OF FAME"));
        assert!(text.contains("TODAYS GREATEST"));
        assert!(text.contains("ALL TIME GREATEST"));
        assert!(text.contains("1."));
        assert!(text.contains("21270"));
    }

    #[test]
    fn attract_cycle_covers_logo_attract_and_high_score() {
        let cycle = attract_cycle();

        assert_eq!(cycle[0].kind, SceneKind::Logo);
        assert_eq!(cycle[0].cue, Some(SoundCue::LogoFanfare));
        assert_eq!(cycle[1].kind, SceneKind::Attract);
        assert_eq!(cycle[8].kind, SceneKind::HighScore);
    }

    #[test]
    fn attract_beat_scene_renders_the_expected_variant() {
        let cycle = attract_cycle();

        assert!(cycle[0].scene().text().contains("WILLIAMS"));
        assert!(
            cycle[1]
                .scene()
                .text()
                .contains("PRESS 1 OR 2 PLAYER START")
        );
        assert!(cycle[7].scene().text().contains("POD"));
        assert!(cycle[8].scene().text().contains("HALL OF FAME"));
    }

    #[test]
    fn scene_for_elapsed_ms_wraps_across_the_attract_cycle() {
        let scene = scene_for_elapsed_ms(
            4_200,
            &HighScoreTable::default(),
            &HighScoreTable::default(),
        );
        let text = scene.text();

        assert!(
            text.contains("PRESS 1 OR 2 PLAYER START")
                || text.contains("HALL OF FAME")
                || text.contains("WILLIAMS")
        );
    }
}
