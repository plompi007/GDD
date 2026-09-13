//! M8 content acceptance for the roadmap's A02/A04 levels (docs/GDD.md
//! appendix): each new level must be solvable by gravity/physics alone
//! (no editor input yet exists in these preplaced-only levels) and
//! deterministic across independent runs, same pattern as
//! tests/level_solvable.rs and tests/gear_and_pulley.rs.

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

const MAX_TICKS_TO_SOLVE: u32 = 3000;

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
fn lvl_a02_bounce_is_solvable() {
    let json = include_str!("../levels/A/lvl_a02_bounce.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_a02_bounce did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_a02_bounce_is_deterministic() {
    let json = include_str!("../levels/A/lvl_a02_bounce.json");
    let first = subject_final_position(json, 500);
    let second = subject_final_position(json, 500);
    let third = subject_final_position(json, 500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_a04_catapult_is_solvable() {
    let json = include_str!("../levels/A/lvl_a04_catapult.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_a04_catapult did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_a04_catapult_is_deterministic() {
    let json = include_str!("../levels/A/lvl_a04_catapult.json");
    let first = subject_final_position(json, 800);
    let second = subject_final_position(json, 800);
    let third = subject_final_position(json, 800);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
