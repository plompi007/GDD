//! M4 acceptance (docs/GDD.md §5.8): "motor→gear→conveyor works, a pulley
//! lifts a weight" — driven exactly like `tests/level_solvable.rs` drives
//! M3's demo level: load the real level JSON, enter Running, step until
//! `GameState::Solved`.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::game_state::GameState;
use chainworks::level_file_format::LevelFile;
use chainworks::level_load::LevelPlugin;
use chainworks::parts::PartsPlugin;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::win_conditions::WinConditionsPlugin;

const MAX_TICKS_TO_SOLVE: u32 = 3000; // 25s of simulated time at 120Hz

fn build_app(level_json: &str) -> App {
    let level: LevelFile = serde_json::from_str(level_json).expect("demo level JSON must be valid");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(WinConditionsPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    // See tests/level_solvable.rs: the initial `OnEnter(Edit)` spawn needs
    // one `update()` to fire before any state transition is requested.
    app.update();
    app
}

fn run_until_solved(app: &mut App, max_ticks: u32) -> bool {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Running);
    for _ in 0..max_ticks {
        app.update();
        if *app.world().resource::<State<GameState>>().get() == GameState::Solved {
            return true;
        }
    }
    false
}

#[test]
fn motor_drives_gear_train_into_conveyor_which_delivers_the_ball() {
    let json = include_str!("../levels/A/lvl_a07_gear_train.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "gear-train level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn pulley_lifts_the_subject_via_a_falling_weight() {
    let json = include_str!("../levels/A/lvl_a03_pulley_lift.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "pulley-lift level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

/// CLAUDE.md's determinism checklist: after any change to the physics
/// engine or the energy graph, the same scene run 3 times must reach an
/// identical final state — `tests/determinism.rs` covers the M1 smoke
/// scene, this covers M4's new rope/gear/energy systems specifically.
fn subject_final_position(level_json: &str, ticks: u32) -> (f32, f32, f32) {
    let mut app = build_app(level_json);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Running);
    for _ in 0..ticks {
        app.update();
    }
    let mut query = app
        .world_mut()
        .query::<(&chainworks::level_load::PartTags, &Transform)>();
    let (_, transform) = query
        .iter(app.world())
        .find(|(tags, _)| tags.has("SUBJECT"))
        .expect("level must contain a SUBJECT-tagged entity");
    (
        transform.translation.x,
        transform.translation.y,
        transform.rotation.to_euler(EulerRot::XYZ).2,
    )
}

#[test]
fn gear_train_level_is_deterministic_across_independent_runs() {
    let json = include_str!("../levels/A/lvl_a07_gear_train.json");
    let first = subject_final_position(json, 400);
    let second = subject_final_position(json, 400);
    let third = subject_final_position(json, 400);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn pulley_lift_level_is_deterministic_across_independent_runs() {
    let json = include_str!("../levels/A/lvl_a03_pulley_lift.json");
    let first = subject_final_position(json, 100);
    let second = subject_final_position(json, 100);
    let third = subject_final_position(json, 100);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
