//! M8 content acceptance for chapter C ("C_TIMING" — docs/GDD.md's chapter
//! enum, §appendix "פתילים, השהיות, סדר אירועים"): levels whose solution
//! depends on *when* things happen, not just *whether* they do. Same
//! pattern as tests/content_levels.rs / tests/content_levels_b.rs.

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
fn lvl_c01_burn_through_is_solvable() {
    let json = include_str!("../levels/C/lvl_c01_burn_through.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c01_burn_through did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c01_burn_through_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c01_burn_through.json");
    let first = subject_final_position(json, 500);
    let second = subject_final_position(json, 500);
    let third = subject_final_position(json, 500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c02_late_arrival_is_solvable() {
    let json = include_str!("../levels/C/lvl_c02_late_arrival.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c02_late_arrival did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c02_late_arrival_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c02_late_arrival.json");
    let first = subject_final_position(json, 500);
    let second = subject_final_position(json, 500);
    let third = subject_final_position(json, 500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c03_clear_the_way_is_solvable() {
    let json = include_str!("../levels/C/lvl_c03_clear_the_way.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c03_clear_the_way did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c03_clear_the_way_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c03_clear_the_way.json");
    let first = subject_final_position(json, 700);
    let second = subject_final_position(json, 700);
    let third = subject_final_position(json, 700);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c04_just_in_time_is_solvable() {
    let json = include_str!("../levels/C/lvl_c04_just_in_time.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c04_just_in_time did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c04_just_in_time_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c04_just_in_time.json");
    let first = subject_final_position(json, 450);
    let second = subject_final_position(json, 450);
    let third = subject_final_position(json, 450);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c05_needle_point_is_solvable() {
    let json = include_str!("../levels/C/lvl_c05_needle_point.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c05_needle_point did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c05_needle_point_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c05_needle_point.json");
    let first = subject_final_position(json, 460);
    let second = subject_final_position(json, 460);
    let third = subject_final_position(json, 460);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c06_twin_candles_is_solvable() {
    let json = include_str!("../levels/C/lvl_c06_twin_candles.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c06_twin_candles did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c06_twin_candles_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c06_twin_candles.json");
    let first = subject_final_position(json, 1300);
    let second = subject_final_position(json, 1300);
    let third = subject_final_position(json, 1300);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_c07_race_the_belt_is_solvable() {
    let json = include_str!("../levels/C/lvl_c07_race_the_belt.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_c07_race_the_belt did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_c07_race_the_belt_is_deterministic() {
    let json = include_str!("../levels/C/lvl_c07_race_the_belt.json");
    let first = subject_final_position(json, 1100);
    let second = subject_final_position(json, 1100);
    let third = subject_final_position(json, 1100);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
