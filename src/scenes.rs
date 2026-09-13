//! Throwaway scenes for bootstrap/determinism smoke tests (M0/M1). Real
//! levels arrive in M3 via `level_file_format.rs` + `level_load.rs`. Kept
//! as a shared function (not duplicated between `main.rs` and
//! `tests/determinism.rs`) so both exercise the exact same scene.

use bevy::prelude::*;

use crate::sim::body_factory::{spawn_dynamic_box, spawn_static_box};

/// Marker for the one dynamic body in the smoke-test scene, so systems can
/// query its transform without caring about entity IDs.
#[derive(Component)]
pub struct FallingBox;

/// Ground slab + one falling box (docs/GDD.md M0 acceptance: the box must
/// fall and settle on the floor).
pub fn spawn_smoke_test_scene(mut commands: Commands) {
    spawn_static_box(
        &mut commands,
        Vec2::new(0.0, -192.0),
        Vec2::new(640.0, 16.0),
        0.6,
        Color::srgb_u8(0x2a, 0x30, 0x40),
    );

    let falling_box = spawn_dynamic_box(
        &mut commands,
        Vec2::new(0.0, 192.0),
        Vec2::new(16.0, 16.0),
        0.25,
        0.5,
        Color::srgb_u8(0x6c, 0x8c, 0xff),
    );
    commands.entity(falling_box).insert(FallingBox);
}
