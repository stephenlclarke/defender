use crate::attract_rom::{DEFENDER_LOGO_CHUNK_COUNT, WILLIAMS_TRACE_POINT_COUNT};
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
    pub palette_phase: usize,
    pub logo_trace_points: usize,
    pub logo_show_title_text: bool,
    pub logo_visible_defender_chunks: usize,
    pub logo_show_copyright: bool,
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

pub fn attract_cycle() -> Vec<AttractBeat> {
    vec![
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT / 8,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT / 3,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT * 2 / 3,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 700,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 350,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: 3,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 350,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: 6,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 350,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 1,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: 9,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 350,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 2,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: 12,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 350,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 3,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: DEFENDER_LOGO_CHUNK_COUNT,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 1_500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: DEFENDER_LOGO_CHUNK_COUNT,
            logo_show_copyright: true,
        },
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 1_500,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 1,
            logo_trace_points: WILLIAMS_TRACE_POINT_COUNT,
            logo_show_title_text: true,
            logo_visible_defender_chunks: DEFENDER_LOGO_CHUNK_COUNT,
            logo_show_copyright: true,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 2_000,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 2_000,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 1,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 2_000,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 2,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 2_000,
            world_steps: 0,
            revealed_score_entries: 0,
            palette_phase: 3,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 24,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 28,
            revealed_score_entries: 0,
            palette_phase: 1,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 32,
            revealed_score_entries: 0,
            palette_phase: 2,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 36,
            revealed_score_entries: 0,
            palette_phase: 3,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 40,
            revealed_score_entries: 0,
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 44,
            revealed_score_entries: 0,
            palette_phase: 1,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 2_000,
            world_steps: 48,
            revealed_score_entries: 0,
            palette_phase: 2,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 4_000,
            world_steps: 52,
            revealed_score_entries: 1,
            palette_phase: 1,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::PlayerShot),
            hold_ms: 4_000,
            world_steps: 56,
            revealed_score_entries: 2,
            palette_phase: 0,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 4_000,
            world_steps: 60,
            revealed_score_entries: 3,
            palette_phase: 1,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::Explosion),
            hold_ms: 2_000,
            world_steps: 64,
            revealed_score_entries: 4,
            palette_phase: 2,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 4_000,
            world_steps: 68,
            revealed_score_entries: 5,
            palette_phase: 3,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::HumanSaved),
            hold_ms: 6_000,
            world_steps: 72,
            revealed_score_entries: 6,
            palette_phase: 2,
            logo_trace_points: 0,
            logo_show_title_text: false,
            logo_visible_defender_chunks: 0,
            logo_show_copyright: false,
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

    for beat in cycle.iter().copied() {
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
            // `AMODES` builds this first page as one Williams / Electronics Inc. /
            // Presents / Defender / copyright composition.
            String::from("                 WILLIAMS"),
            String::new(),
            String::from("             ELECTRONICS INC."),
            String::new(),
            String::from("                  PRESENTS"),
            String::new(),
            String::from("                  DEFENDER"),
            String::new(),
            String::from("               COPYRIGHT 1980"),
        ],
    }
}

pub fn attract_scene(world: &World, revealed_score_entries: usize) -> Scene {
    let mut lines = vec![String::new()];
    lines.extend(crate::render::render_grid(world));
    lines.push(String::new());
    // `TEXTAB` / `TENT` in `amode1.src` rotate the instruction legend in this
    // order: SCANNER, LANDER, MUTANT, BAITER, BOMBER, POD, SWARMER.
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
        assert!(cycle.iter().any(|beat| beat.kind == SceneKind::HighScore));
        assert!(cycle.iter().any(|beat| beat.kind == SceneKind::Attract));
        assert_eq!(
            cycle
                .last()
                .expect("attract cycle should not be empty")
                .kind,
            SceneKind::Attract
        );
    }

    #[test]
    fn attract_beat_scene_renders_the_expected_variant() {
        let cycle = attract_cycle();

        assert!(cycle[0].scene().text().contains("WILLIAMS"));
        assert!(
            cycle
                .iter()
                .find(|beat| beat.kind == SceneKind::HighScore)
                .expect("attract cycle should include the hall of fame page")
                .scene()
                .text()
                .contains("HALL OF FAME")
        );
        assert!(
            cycle
                .iter()
                .find(|beat| beat.kind == SceneKind::Attract)
                .expect("attract cycle should include the score-card demo")
                .scene()
                .text()
                .contains("SCANNER")
        );
        assert!(
            cycle
                .last()
                .expect("attract cycle should not be empty")
                .scene()
                .text()
                .contains("SWARMER")
        );
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
            text.contains("SCANNER") || text.contains("HALL OF FAME") || text.contains("WILLIAMS")
        );
    }
}
