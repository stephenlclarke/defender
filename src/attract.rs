use crate::audio::SoundCue;
use crate::game::World;
use crate::high_scores::HighScoreTable;

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
}

impl AttractBeat {
    pub fn scene(self) -> Scene {
        match self.kind {
            SceneKind::Logo => logo_scene(),
            SceneKind::Attract => {
                let mut world = World::bootstrap();
                for _ in 0..self.world_steps {
                    world.step();
                }
                attract_scene(&world)
            }
            SceneKind::HighScore => high_score_scene(),
        }
    }
}

pub fn attract_cycle() -> [AttractBeat; 7] {
    [
        AttractBeat {
            kind: SceneKind::Logo,
            cue: Some(SoundCue::LogoFanfare),
            hold_ms: 900,
            world_steps: 0,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::AttractHum),
            hold_ms: 750,
            world_steps: 0,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::EnemySweep),
            hold_ms: 550,
            world_steps: 2,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::PlayerShot),
            hold_ms: 500,
            world_steps: 4,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::HumanSaved),
            hold_ms: 550,
            world_steps: 6,
        },
        AttractBeat {
            kind: SceneKind::Attract,
            cue: Some(SoundCue::Explosion),
            hold_ms: 600,
            world_steps: 8,
        },
        AttractBeat {
            kind: SceneKind::HighScore,
            cue: Some(SoundCue::HighScoreChime),
            hold_ms: 950,
            world_steps: 0,
        },
    ]
}

pub fn logo_scene() -> Scene {
    Scene {
        kind: SceneKind::Logo,
        lines: vec![
            String::from(r" ____  _____ _____ _____ _   _ ____  _____ ____  "),
            String::from(r"|  _ \| ____|  ___| ____| \ | |  _ \| ____|  _ \ "),
            String::from(r"| | | |  _| | |_  |  _| |  \| | | | |  _| | |_) |"),
            String::from(r"| |_| | |___|  _| | |___| |\  | |_| | |___|  _ < "),
            String::from(r"|____/|_____|_|   |_____|_| \_|____/|_____|_| \_\"),
            String::new(),
            String::from("             NATIVE RUST PROTOTYPE"),
            String::new(),
            String::from("         LIVE TITLE / ATTRACT / HIGH SCORE"),
            String::new(),
            String::from("              RUN `cargo run` TO PLAY"),
            String::from("         RUN `cargo run -- --mute` FOR QUIET"),
        ],
    }
}

pub fn attract_scene(world: &World) -> Scene {
    let mut lines = vec![
        String::from("ATTRACT MODE"),
        String::from("Protect the last humans. Stop the landers before they abduct."),
        String::new(),
    ];
    lines.extend(crate::render::render_grid(world));
    lines.push(String::new());
    lines.push(String::from("LANDERS DIVE LOW. MUTANTS PATROL THE SKY."));
    lines.push(String::from(
        "CURRENT BUILD: WORLD MODEL, ATTRACT PANELS, AND ROM AUDIT.",
    ));

    Scene {
        kind: SceneKind::Attract,
        lines,
    }
}

pub fn high_score_scene() -> Scene {
    let mut lines = vec![
        String::from("HIGH SCORES"),
        String::new(),
        String::from(" RANK  INITIALS   SCORE"),
        String::from(" ----  --------  -------"),
    ];

    lines.extend(HighScoreTable::default().rows());

    lines.push(String::new());
    lines.push(String::from("BONUS SHIP EVERY 10000 POINTS"));
    lines.push(String::from("PROTECT HUMANS TO BUILD SCORE MULTIPLIERS"));

    Scene {
        kind: SceneKind::HighScore,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use crate::{audio::SoundCue, game::World};

    use super::{SceneKind, attract_cycle, attract_scene, high_score_scene, logo_scene};

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

        assert!(text.contains("NATIVE RUST PROTOTYPE"));
        assert!(text.contains("cargo run"));
    }

    #[test]
    fn attract_scene_wraps_rendered_world() {
        let scene = attract_scene(&World::bootstrap());
        let text = scene.text();

        assert!(text.contains("ATTRACT MODE"));
        assert!(text.contains("THREAT"));
    }

    #[test]
    fn high_score_scene_lists_ranked_scores() {
        let scene = high_score_scene();
        let text = scene.text();

        assert!(text.contains("HIGH SCORES"));
        assert!(text.contains("1."));
        assert!(text.contains("250000"));
    }

    #[test]
    fn attract_cycle_covers_logo_attract_and_high_score() {
        let cycle = attract_cycle();

        assert_eq!(cycle[0].kind, SceneKind::Logo);
        assert_eq!(cycle[0].cue, Some(SoundCue::LogoFanfare));
        assert_eq!(cycle[1].kind, SceneKind::Attract);
        assert_eq!(cycle[6].kind, SceneKind::HighScore);
    }

    #[test]
    fn attract_beat_scene_renders_the_expected_variant() {
        let cycle = attract_cycle();

        assert!(cycle[0].scene().text().contains("NATIVE RUST PROTOTYPE"));
        assert!(cycle[1].scene().text().contains("ATTRACT MODE"));
        assert!(cycle[6].scene().text().contains("HIGH SCORES"));
    }
}
