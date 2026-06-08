#[cfg(test)]
mod tests {
    use super::*;

    include!("actor_game_tests_bridge.rs");
    include!("actor_game_tests_attract.rs");
    include!("actor_game_tests_session.rs");
    include!("actor_game_tests_wave.rs");
    include!("actor_game_tests_sources.rs");
    include!("actor_game_tests_combat.rs");
    include!("actor_game_tests_support.rs");
}
