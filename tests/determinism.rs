//! docs/GDD.md §2.6 / CLAUDE.md: after 600 ticks, the same scene must reach
//! identical state across independent runs. This is the load-bearing test
//! for every future physics/energy-graph change — never bypass it.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::scenes::{spawn_smoke_test_scene, FallingBox};
use chainworks::sim::{SimPlugin, FIXED_DT};

const TICKS: u32 = 600;

#[derive(Clone, Copy, Debug, PartialEq)]
struct BodyState {
    x: f32,
    y: f32,
    rotation_z: f32,
}

fn run_scene(ticks: u32) -> BodyState {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(SimPlugin)
        .add_systems(Startup, spawn_smoke_test_scene)
        // Deterministic time source: exactly one FixedUpdate tick per
        // app.update() call, independent of real wall-clock speed
        // (docs/GDD.md §2.6 checklist item 4 — no dependency on real dt).
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));

    for _ in 0..ticks {
        app.update();
    }

    let mut query = app
        .world_mut()
        .query_filtered::<&Transform, With<FallingBox>>();
    let transform = query
        .single(app.world())
        .expect("smoke-test scene must contain exactly one FallingBox");

    BodyState {
        x: transform.translation.x,
        y: transform.translation.y,
        rotation_z: transform.rotation.to_euler(EulerRot::XYZ).2,
    }
}

#[test]
fn same_scene_reaches_identical_state_after_600_ticks() {
    let first = run_scene(TICKS);
    let second = run_scene(TICKS);
    let third = run_scene(TICKS);

    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
