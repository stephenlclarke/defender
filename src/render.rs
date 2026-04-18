use crate::game::World;
use crate::high_scores::HighScoreTable;

pub struct InitialsEntryView<'a> {
    pub high_score: u32,
    pub high_scores: &'a HighScoreTable,
    pub entry_score: u32,
    pub entry_rank: usize,
    pub initials: &'a str,
    pub xyzzy_active: bool,
    pub invincible: bool,
    pub auto_fire: bool,
}

pub fn render_grid(world: &World) -> Vec<String> {
    let mut buffer = vec![vec![' '; world.width()]; world.height()];
    if !world.planet_destroyed() {
        for screen_x in 0..world.width() {
            let terrain_row = world.terrain_row_at_screen_x(screen_x);
            if terrain_row < buffer.len() {
                buffer[terrain_row][screen_x] = '_';
            }
        }
    }

    for entity in world.entities() {
        let y = entity.position.y;
        if let Some(screen_x) = world.screen_x_for_world_x(entity.position.x)
            && y >= 0
            && (y as usize) < world.height()
        {
            buffer[y as usize][screen_x] = if entity.kind == crate::game::EntityKind::PlayerShip {
                world.player_facing().glyph()
            } else {
                entity.kind.glyph()
            };
        }
    }

    let status = world.status();
    let mut lines = vec![
        format!(
            "DEFENDER  SCORE {:06}  LIVES {}  WAVE {}  TICK {:03}",
            status.score,
            status.lives,
            status.wave,
            world.tick()
        ),
        format!(
            "ENEMIES {}  HUMANS {}  THREAT {}  BOMBS {}  CAM {:03}  {}",
            world.enemy_count(),
            world.human_count(),
            world.threat_score(),
            world.smart_bombs(),
            world.camera_x(),
            if world.planet_destroyed() {
                "DEEP SPACE"
            } else {
                "PLANET ACTIVE"
            },
        ),
        render_scanner(world),
    ];

    lines.extend(
        buffer
            .into_iter()
            .map(|row| format!("|{}|", row.into_iter().collect::<String>())),
    );
    lines
}

fn render_scanner(world: &World) -> String {
    let mut scanner = vec!['.'; world.width()];

    for entity in world.entities() {
        let index = ((entity.position.x.rem_euclid(world.world_span()) as usize) * world.width())
            / world.world_span() as usize;
        let glyph = if entity.kind == crate::game::EntityKind::PlayerShip {
            world.player_facing().glyph()
        } else {
            scanner_glyph(entity.kind)
        };
        plot_scanner_glyph(&mut scanner[index.min(world.width() - 1)], glyph);
    }

    format!("SCANNER |{}|", scanner.into_iter().collect::<String>())
}

fn scanner_glyph(kind: crate::game::EntityKind) -> char {
    match kind {
        crate::game::EntityKind::PlayerShip => '^',
        crate::game::EntityKind::PlayerShot | crate::game::EntityKind::EnemyShot => '!',
        crate::game::EntityKind::Human => 'h',
        crate::game::EntityKind::Lander => 'L',
        crate::game::EntityKind::Mutant => 'M',
        crate::game::EntityKind::Baiter => 'B',
        crate::game::EntityKind::Bomber => 'V',
        crate::game::EntityKind::Pod => 'P',
        crate::game::EntityKind::Swarmer => 'S',
        crate::game::EntityKind::Mine => 'x',
    }
}

fn plot_scanner_glyph(cell: &mut char, glyph: char) {
    let priority = |value: char| match value {
        '^' | '<' | '>' => 5,
        'h' => 4,
        'L' | 'M' | 'B' | 'V' | 'P' | 'S' => 3,
        'x' => 2,
        '!' => 1,
        _ => 0,
    };

    if priority(glyph) >= priority(*cell) {
        *cell = glyph;
    }
}

pub fn render(world: &World) -> String {
    render_with_flags(world, false, false, false)
}

pub fn render_with_flags(
    world: &World,
    xyzzy_active: bool,
    invincible: bool,
    auto_fire: bool,
) -> String {
    let mut lines = render_grid(world);
    if world.is_game_over() {
        lines.push(String::from(
            "GAME OVER. Press `q` or `Esc` to leave the live session.",
        ));
    }
    lines.extend(secret_mode_lines(xyzzy_active, invincible, auto_fire));
    lines.push(String::from(
        "Controls: `A` up, `Z` down, `Shift` thrust, `Space` flip direction, `Enter` fire, `Tab` smart bomb, `H` hyperspace, `q` quits.",
    ));
    lines.push(String::from(
        "Use `cargo run -- --rom-report assets/roms/defender` to inspect local ROM references.",
    ));
    lines.join("\n")
}

pub fn render_title_screen(high_score: u32) -> String {
    render_title_screen_with_flags(high_score, false, false, false)
}

pub fn render_title_screen_with_flags(
    high_score: u32,
    xyzzy_active: bool,
    invincible: bool,
    auto_fire: bool,
) -> String {
    [
        r" ____  _____ _____ _____ _   _ ____  _____ ____  ".to_string(),
        r"|  _ \| ____|  ___| ____| \ | |  _ \| ____|  _ \ ".to_string(),
        r"| | | |  _| | |_  |  _| |  \| | | | |  _| | |_) |".to_string(),
        r"| |_| | |___|  _| | |___| |\  | |_| | |___|  _ < ".to_string(),
        r"|____/|_____|_|   |_____|_| \_|____/|_____|_| \_\".to_string(),
        String::new(),
        String::from("LIVE TERMINAL PROTOTYPE"),
        String::new(),
        format!("HIGH SCORE {:06}", high_score),
        String::new(),
        String::from("PRESS `ENTER` OR `1` TO START"),
        String::from("PRESS `q` OR `Esc` TO QUIT"),
        String::new(),
        String::from("CONTROLS"),
        String::from("VERTICAL: `A` UP / `Z` DOWN"),
        String::from("DRIVE: `Shift` THRUST / `Space` FLIP DIRECTION"),
        String::from("LASER: `Enter`"),
        String::from("SMART BOMB: `Tab`"),
        String::from("HYPERSPACE: `H`"),
        secret_mode_lines(xyzzy_active, invincible, auto_fire).join("\n"),
    ]
    .join("\n")
}

pub fn render_game_over_screen(world: &World, high_score: u32) -> String {
    render_game_over_screen_with_flags(world, high_score, false, false, false)
}

pub fn render_game_over_screen_with_flags(
    world: &World,
    high_score: u32,
    xyzzy_active: bool,
    invincible: bool,
    auto_fire: bool,
) -> String {
    let mut lines = render_grid(world);
    lines.push(String::new());
    lines.push(format!(
        "GAME OVER  SCORE {:06}  HIGH SCORE {:06}",
        world.status().score,
        high_score
    ));
    lines.extend(secret_mode_lines(xyzzy_active, invincible, auto_fire));
    lines.push(String::from("PRESS `ENTER` OR `1` TO RESTART"));
    lines.push(String::from("PRESS `q` OR `Esc` TO QUIT"));
    lines.join("\n")
}

pub fn render_initials_entry_screen(world: &World, view: &InitialsEntryView<'_>) -> String {
    let mut lines = render_grid(world);
    lines.push(String::new());
    lines.push(format!(
        "GREAT SCORE {:06}  HIGH SCORE {:06}",
        view.entry_score, view.high_score
    ));
    lines.push(format!("QUALIFIES FOR RANK {:>2}", view.entry_rank));
    lines.push(String::from("ENTER INITIALS"));
    lines.push(format!("  [{}]", view.initials));
    lines.push(String::from(
        "TYPE LETTERS A-Z, `Backspace` DELETES, `Enter` SAVES",
    ));
    lines.push(String::new());
    lines.push(String::from("HIGH SCORES"));
    lines.push(String::from(" RANK  INITIALS   SCORE"));
    lines.push(String::from(" ----  --------  -------"));
    lines.extend(view.high_scores.rows());
    lines.extend(secret_mode_lines(
        view.xyzzy_active,
        view.invincible,
        view.auto_fire,
    ));
    lines.push(String::from("PRESS `q` OR `Esc` TO QUIT"));
    lines.join("\n")
}

fn secret_mode_lines(xyzzy_active: bool, invincible: bool, auto_fire: bool) -> Vec<String> {
    if !xyzzy_active {
        return Vec::new();
    }

    vec![
        String::from("XYZZY MODE ENABLED"),
        if invincible {
            String::from("SMART BOMBS INF  GOD MODE ON  INVINCIBLE")
        } else {
            String::from("SMART BOMBS INF  GOD MODE OFF  PRESS `G` TO TOGGLE INVINCIBILITY")
        },
        if auto_fire {
            String::from("AUTO FIRE ON  PRESS `F` TO TOGGLE")
        } else {
            String::from("AUTO FIRE OFF  PRESS `F` TO TOGGLE")
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, Status, World};
    use crate::high_scores::HighScoreTable;

    #[test]
    fn render_includes_status_header() {
        let output = super::render(&World::bootstrap());

        assert!(output.contains("DEFENDER"));
        assert!(output.contains("LIVES 3"));
        assert!(output.contains("THREAT"));
        assert!(output.contains("BOMBS 3"));
        assert!(output.contains("CAM"));
        assert!(output.contains("SCANNER |"));
    }

    #[test]
    fn render_places_entities_and_ground_line() {
        let world = World::with_entities(
            8,
            6,
            Status {
                score: 1200,
                lives: 2,
                wave: 4,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 1, 1, 0, 0),
                Entity::new(EntityKind::Human, 3, 3, 0, 0),
            ],
        );

        let output = super::render(&world);
        let rows: Vec<&str> = output.lines().collect();

        assert_eq!(rows[3], "|        |");
        assert_eq!(rows[4], "| >      |");
        assert_eq!(rows[6], "|   h    |");
        assert_eq!(rows[7], "|________|");
    }

    #[test]
    fn render_clips_entities_outside_the_frame() {
        let world = World::with_entities(
            4,
            4,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![Entity::new(EntityKind::Mutant, 9, 9, 0, 0)],
        );

        let output = super::render(&world);
        let frame_rows: Vec<&str> = output.lines().skip(3).take(world.height()).collect();

        assert!(frame_rows.iter().all(|row| !row.contains('M')));
    }

    #[test]
    fn render_appends_live_controls_and_game_over_notice() {
        let mut world = World::with_entities(
            8,
            6,
            Status {
                score: 0,
                lives: 1,
                wave: 1,
            },
            vec![
                Entity::new(EntityKind::PlayerShip, 2, 2, 0, 0),
                Entity::new(EntityKind::Lander, 2, 2, 0, 0),
            ],
        );
        world.step_live(crate::game::UpdateInput::default());

        let output = super::render(&world);
        assert!(output.contains("Controls:"));
        assert!(output.contains("GAME OVER"));
    }

    #[test]
    fn render_with_secret_mode_shows_xyzzy_indicator() {
        let output = super::render_with_flags(&World::bootstrap(), true, true, true);

        assert!(output.contains("XYZZY MODE ENABLED"));
        assert!(output.contains("SMART BOMBS INF"));
        assert!(output.contains("AUTO FIRE ON"));
    }

    #[test]
    fn bootstrap_render_shows_stepped_terrain_profile() {
        let output = super::render(&World::bootstrap());
        let frame_rows: Vec<&str> = output
            .lines()
            .skip(3)
            .take(World::bootstrap().height())
            .collect();
        let terrain_rows = frame_rows.iter().filter(|row| row.contains('_')).count();

        assert!(terrain_rows > 1);
    }

    #[test]
    fn title_screen_mentions_start_and_high_score() {
        let output = super::render_title_screen(1234);
        assert!(output.contains("LIVE TERMINAL PROTOTYPE"));
        assert!(output.contains("HIGH SCORE 001234"));
        assert!(output.contains("PRESS `ENTER` OR `1` TO START"));
        assert!(output.contains("LASER: `Enter`"));
    }

    #[test]
    fn game_over_screen_includes_restart_hint() {
        let world = World::bootstrap();
        let output = super::render_game_over_screen(&world, 4321);
        assert!(output.contains("GAME OVER"));
        assert!(output.contains("HIGH SCORE 004321"));
        assert!(output.contains("PRESS `ENTER` OR `1` TO RESTART"));
    }

    #[test]
    fn initials_entry_screen_mentions_rank_and_entry_controls() {
        let world = World::bootstrap();
        let output = super::render_initials_entry_screen(
            &world,
            &super::InitialsEntryView {
                high_score: 250_000,
                high_scores: &HighScoreTable::default(),
                entry_score: 60_000,
                entry_rank: 5,
                initials: "RO_",
                xyzzy_active: false,
                invincible: false,
                auto_fire: false,
            },
        );

        assert!(output.contains("GREAT SCORE 060000"));
        assert!(output.contains("QUALIFIES FOR RANK  5"));
        assert!(output.contains("[RO_]"));
        assert!(output.contains("TYPE LETTERS A-Z"));
        assert!(output.contains("HIGH SCORES"));
    }
}
