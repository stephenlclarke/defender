use crate::{
    audio::SoundCue,
    game::{Entity, EntityKind, World},
};

#[derive(Debug, Clone)]
pub struct GameplayBeat {
    pub world: World,
    pub cue: Option<SoundCue>,
    pub hold_ms: u64,
}

pub fn gameplay_demo_cycle() -> [GameplayBeat; 6] {
    let mut world = World::bootstrap();
    let beat0 = GameplayBeat {
        world: world.clone(),
        cue: Some(SoundCue::EnemySweep),
        hold_ms: 550,
    };

    for _ in 0..2 {
        world.step();
    }
    let beat1 = GameplayBeat {
        world: world.clone(),
        cue: Some(SoundCue::PlayerShot),
        hold_ms: 520,
    };

    for _ in 0..2 {
        world.step();
    }
    world.remove_first_by_kind(EntityKind::Lander);
    world.add_score(150);
    let beat2 = GameplayBeat {
        world: world.clone(),
        cue: Some(SoundCue::Explosion),
        hold_ms: 560,
    };

    for _ in 0..2 {
        world.step();
    }
    world.add_score(250);
    world.spawn_entity(Entity::new(
        EntityKind::Human,
        54,
        world.ground_row() as i32 - 1,
        0,
        0,
    ));
    let beat3 = GameplayBeat {
        world: world.clone(),
        cue: Some(SoundCue::HumanSaved),
        hold_ms: 560,
    };

    for _ in 0..2 {
        world.step();
    }
    world.set_wave(2);
    world.spawn_entity(Entity::new(EntityKind::Lander, 56, 3, -1, 1));
    let beat4 = GameplayBeat {
        world: world.clone(),
        cue: Some(SoundCue::EnemySweep),
        hold_ms: 560,
    };

    for _ in 0..2 {
        world.step();
    }
    world.set_lives(2);
    world.remove_first_by_kind(EntityKind::Human);
    world.add_score(400);
    let beat5 = GameplayBeat {
        world,
        cue: Some(SoundCue::Explosion),
        hold_ms: 620,
    };

    [beat0, beat1, beat2, beat3, beat4, beat5]
}

#[cfg(test)]
mod tests {
    use crate::audio::SoundCue;

    use super::gameplay_demo_cycle;

    #[test]
    fn gameplay_demo_cycle_advances_score_and_wave() {
        let cycle = gameplay_demo_cycle();

        assert_eq!(cycle[0].world.status().score, 0);
        assert_eq!(cycle[2].world.status().score, 150);
        assert_eq!(cycle[3].world.status().score, 400);
        assert_eq!(cycle[4].world.status().wave, 2);
        assert_eq!(cycle[5].world.status().lives, 2);
    }

    #[test]
    fn gameplay_demo_cycle_uses_action_cues() {
        let cycle = gameplay_demo_cycle();

        assert_eq!(cycle[0].cue, Some(SoundCue::EnemySweep));
        assert_eq!(cycle[1].cue, Some(SoundCue::PlayerShot));
        assert_eq!(cycle[2].cue, Some(SoundCue::Explosion));
        assert_eq!(cycle[3].cue, Some(SoundCue::HumanSaved));
    }

    #[test]
    fn gameplay_demo_cycle_changes_entity_counts() {
        let cycle = gameplay_demo_cycle();

        assert_eq!(cycle[0].world.enemy_count(), 2);
        assert_eq!(cycle[2].world.enemy_count(), 1);
        assert_eq!(cycle[3].world.human_count(), 3);
        assert_eq!(cycle[5].world.human_count(), 2);
    }
}
