use crate::game::World;

pub fn render(world: &World) -> String {
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
            "ENEMIES {}  HUMANS {}  THREAT {}",
            world.enemy_count(),
            world.human_count(),
            world.threat_score()
        ),
    ];

    lines.extend(
        buffer
            .into_iter()
            .map(|row| format!("|{}|", row.into_iter().collect::<String>())),
    );

    lines.push(String::from(
        "Use `cargo run -- --rom-report assets/roms/defender` to inspect local ROM references.",
    ));
    lines.join("\n")
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
}
