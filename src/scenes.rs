//! Throwaway scenes for bootstrap/determinism smoke tests (M0-M2). Real
//! levels arrive in M3 via `level_file_format.rs` + `level_load.rs`. Kept
//! as a shared function (not duplicated between `main.rs` and
//! `tests/determinism.rs`) so both exercise the exact same scene.

use bevy::prelude::*;

use crate::parts::PartRegistry;
use crate::sim::body_factory::spawn_part;

/// Marker for the one entity of interest in the smoke-test scene (currently
/// the rolling ball), so systems can query its transform without caring
/// about entity IDs.
#[derive(Component)]
pub struct Tracked;

/// M2 acceptance scene (docs/GDD.md §5.8): a ground floor, an angled plank,
/// and a rubber ball that rolls down the plank — built entirely from
/// `data/parts/*.json` via [`PartRegistry`], not hardcoded shapes.
pub fn spawn_smoke_test_scene(mut commands: Commands, parts: Res<PartRegistry>) {
    let floor = parts
        .get("floor_ground")
        .expect("floor_ground must be a registered P0 part");
    spawn_part(&mut commands, floor, Vec2::new(0.0, -300.0), 0.0);

    let plank = parts
        .get("plank_wood")
        .expect("plank_wood must be a registered P0 part");
    spawn_part(&mut commands, plank, Vec2::new(0.0, -120.0), -20.0);

    let ball = parts
        .get("ball_rubber")
        .expect("ball_rubber must be a registered P0 part");
    // Plank center (0,-120) rotated -20° puts its high (local -x) end at
    // roughly (-60,-98) and its low (local +x) end at roughly (60,-142) —
    // drop the ball just above the high end so it lands on the slope and
    // rolls downhill toward +x, instead of missing the plank entirely.
    let ball_entity = spawn_part(&mut commands, ball, Vec2::new(-45.0, 60.0), 0.0);
    commands.entity(ball_entity).insert(Tracked);
}
