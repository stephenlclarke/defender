//! Actor smoke runner.
//!
//! This command path exercises the actor driver through `ActorRuntimeAdapter`
//! and the native draw planner while actor frames also expose a clean
//! `GameFrame` handoff for runtime preflights.

include!("actor_smoke_reports.rs");
include!("actor_smoke_observers.rs");
include!("actor_smoke_tests.rs");
