use crate::game::World;

pub fn render_grid(world: &World) -> Vec<String> {
    let mut buffer = vec![vec![' '; world.width()]; world.height()];
    let ground_row = world.ground_row();

    if ground_row < buffer.len() {
        buffer[ground_row].fill('_');
    }

    for entity in world.entities() {
        let x = entity.position.x;
        let y = entity.position.y;
        if x >= 0 && (x as usize) < world.width() && y >= 0 && (y as usize) < world.height() {
            buffer[y as usize][x as usize] = entity.kind.glyph();
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
            "ENEMIES {}  HUMANS {}  THREAT {}  BOMBS {}",
            world.enemy_count(),
            world.human_count(),
            world.threat_score(),
            world.smart_bombs()
        ),
    ];

    lines.extend(
        buffer
            .into_iter()
            .map(|row| format!("|{}|", row.into_iter().collect::<String>())),
    );
    lines
}

pub fn render(world: &World) -> String {
    render_with_flags(world, false, false)
}

pub fn render_with_flags(world: &World, xyzzy_active: bool, invincible: bool) -> String {
    let mut lines = render_grid(world);
    if world.is_game_over() {
        lines.push(String::from(
            "GAME OVER. Press `q` or `Esc` to leave the live session.",
        ));
    }
    lines.extend(secret_mode_lines(xyzzy_active, invincible));
    lines.push(String::from(
        "Controls: `A` up, `Z` down, `Shift` thrust, `Space` reverse, `Enter` fire, `Tab` smart bomb, `H` hyperspace, `q` quits.",
    ));
    lines.push(String::from(
        "Use `cargo run -- --rom-report assets/roms/defender` to inspect local ROM references.",
    ));
    lines.join("\n")
}

pub fn render_title_screen(high_score: u32) -> String {
    render_title_screen_with_flags(high_score, false, false)
}

pub fn render_title_screen_with_flags(
    high_score: u32,
    xyzzy_active: bool,
    invincible: bool,
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
        format!("SESSION HIGH SCORE {:06}", high_score),
        String::new(),
        String::from("PRESS `ENTER` OR `1` TO START"),
        String::from("PRESS `q` OR `Esc` TO QUIT"),
        String::new(),
        String::from("CONTROLS"),
        String::from("VERTICAL: `A` UP / `Z` DOWN"),
        String::from("DRIVE: `Shift` THRUST / `Space` REVERSE"),
        String::from("LASER: `Enter`"),
        String::from("SMART BOMB: `Tab`"),
        String::from("HYPERSPACE: `H`"),
        secret_mode_lines(xyzzy_active, invincible).join("\n"),
    ]
    .join("\n")
}

pub fn render_game_over_screen(world: &World, high_score: u32) -> String {
    render_game_over_screen_with_flags(world, high_score, false, false)
}

pub fn render_game_over_screen_with_flags(
    world: &World,
    high_score: u32,
    xyzzy_active: bool,
    invincible: bool,
) -> String {
    let mut lines = render_grid(world);
    lines.push(String::new());
    lines.push(format!(
        "GAME OVER  SCORE {:06}  HIGH SCORE {:06}",
        world.status().score,
        high_score
    ));
    lines.extend(secret_mode_lines(xyzzy_active, invincible));
    lines.push(String::from("PRESS `ENTER` OR `1` TO RESTART"));
    lines.push(String::from("PRESS `q` OR `Esc` TO QUIT"));
    lines.join("\n")
}

fn secret_mode_lines(xyzzy_active: bool, invincible: bool) -> Vec<String> {
    if !xyzzy_active {
        return Vec::new();
    }

    vec![
        String::from("XYZZY MODE ENABLED"),
        if invincible {
            String::from("GOD MODE ON  INVINCIBLE  SMART BOMBS INF")
        } else {
            String::from("GOD MODE OFF  PRESS `i` TO TOGGLE INVINCIBILITY")
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::game::{Entity, EntityKind, Status, World};

    #[test]
    fn render_includes_status_header() {
        let output = super::render(&World::bootstrap());

        assert!(output.contains("DEFENDER"));
        assert!(output.contains("LIVES 3"));
        assert!(output.contains("THREAT"));
        assert!(output.contains("BOMBS 3"));
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

        assert_eq!(rows[2], "|        |");
        assert_eq!(rows[3], "| ^      |");
        assert_eq!(rows[5], "|   h    |");
        assert_eq!(rows[6], "|________|");
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
        let frame_rows: Vec<&str> = output.lines().skip(2).take(world.height()).collect();

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
        let output = super::render_with_flags(&World::bootstrap(), true, true);

        assert!(output.contains("XYZZY MODE ENABLED"));
        assert!(output.contains("SMART BOMBS INF"));
    }

    #[test]
    fn title_screen_mentions_start_and_high_score() {
        let output = super::render_title_screen(1234);
        assert!(output.contains("LIVE TERMINAL PROTOTYPE"));
        assert!(output.contains("SESSION HIGH SCORE 001234"));
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
}
