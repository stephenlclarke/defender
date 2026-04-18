use crate::game::World;

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
            String::from("         START LOGO / ATTRACT / HIGH SCORE"),
            String::new(),
            String::from("         RUN `cargo run -- --scene attract`"),
            String::from("       RUN `cargo run -- --scene high-score`"),
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

    for (rank, initials, score) in [
        (1, "SLC", 250_000),
        (2, "ACE", 175_000),
        (3, "ROM", 125_000),
        (4, "ARC", 90_000),
        (5, "CPU", 50_000),
    ] {
        lines.push(format!("  {:>2}.  {:<8}  {:>7}", rank, initials, score));
    }

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
    use crate::game::World;

    use super::{SceneKind, attract_scene, high_score_scene, logo_scene};

    #[test]
    fn parse_scene_kind_recognises_supported_values() {
        assert_eq!(SceneKind::parse("logo"), Some(SceneKind::Logo));
        assert_eq!(SceneKind::parse("attract"), Some(SceneKind::Attract));
        assert_eq!(SceneKind::parse("high-score"), Some(SceneKind::HighScore));
        assert_eq!(SceneKind::parse("unknown"), None);
    }

    #[test]
    fn logo_scene_contains_title_and_scene_hints() {
        let scene = logo_scene();
        let text = scene.text();

        assert!(text.contains("NATIVE RUST PROTOTYPE"));
        assert!(text.contains("high-score"));
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
}
