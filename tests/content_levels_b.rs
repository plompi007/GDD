//! M8 content acceptance for chapter B ("B_COMBOS" — docs/GDD.md's chapter
//! enum): levels that combine two or more already-shipped mechanics rather
//! than introducing new engine subsystems. Same pattern as
//! tests/content_levels.rs.

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
fn lvl_b01_chain_cut_is_solvable() {
    let json = include_str!("../levels/B/lvl_b01_chain_cut.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b01_chain_cut did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b01_chain_cut_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b01_chain_cut.json");
    let first = subject_final_position(json, 800);
    let second = subject_final_position(json, 800);
    let third = subject_final_position(json, 800);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b02_double_fuse_is_solvable() {
    let json = include_str!("../levels/B/lvl_b02_double_fuse.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b02_double_fuse did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b02_double_fuse_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b02_double_fuse.json");
    let first = subject_final_position(json, 1500);
    let second = subject_final_position(json, 1500);
    let third = subject_final_position(json, 1500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b03_break_through_is_solvable() {
    let json = include_str!("../levels/B/lvl_b03_break_through.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b03_break_through did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b03_break_through_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b03_break_through.json");
    let first = subject_final_position(json, 400);
    let second = subject_final_position(json, 400);
    let third = subject_final_position(json, 400);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b04_launch_switch_is_solvable() {
    let json = include_str!("../levels/B/lvl_b04_launch_switch.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b04_launch_switch did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b04_launch_switch_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b04_launch_switch.json");
    let first = subject_final_position(json, 1000);
    let second = subject_final_position(json, 1000);
    let third = subject_final_position(json, 1000);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b05_windswept_is_solvable() {
    let json = include_str!("../levels/B/lvl_b05_windswept.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b05_windswept did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b05_windswept_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b05_windswept.json");
    let first = subject_final_position(json, 1000);
    let second = subject_final_position(json, 1000);
    let third = subject_final_position(json, 1000);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b06_triple_mesh_is_solvable() {
    let json = include_str!("../levels/B/lvl_b06_triple_mesh.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b06_triple_mesh did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b06_triple_mesh_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b06_triple_mesh.json");
    let first = subject_final_position(json, 600);
    let second = subject_final_position(json, 600);
    let third = subject_final_position(json, 600);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b07_bounce_relay_is_solvable() {
    let json = include_str!("../levels/B/lvl_b07_bounce_relay.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b07_bounce_relay did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b07_bounce_relay_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b07_bounce_relay.json");
    let first = subject_final_position(json, 600);
    let second = subject_final_position(json, 600);
    let third = subject_final_position(json, 600);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b08_launch_lever_is_solvable() {
    let json = include_str!("../levels/B/lvl_b08_launch_lever.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b08_launch_lever did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b08_launch_lever_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b08_launch_lever.json");
    let first = subject_final_position(json, 900);
    let second = subject_final_position(json, 900);
    let third = subject_final_position(json, 900);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b09_blow_the_switch_is_solvable() {
    let json = include_str!("../levels/B/lvl_b09_blow_the_switch.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b09_blow_the_switch did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b09_blow_the_switch_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b09_blow_the_switch.json");
    let first = subject_final_position(json, 800);
    let second = subject_final_position(json, 800);
    let third = subject_final_position(json, 800);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b10_four_in_a_row_is_solvable() {
    let json = include_str!("../levels/B/lvl_b10_four_in_a_row.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b10_four_in_a_row did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b10_four_in_a_row_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b10_four_in_a_row.json");
    let first = subject_final_position(json, 400);
    let second = subject_final_position(json, 400);
    let third = subject_final_position(json, 400);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b11_snip_and_drop_is_solvable() {
    let json = include_str!("../levels/B/lvl_b11_snip_and_drop.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b11_snip_and_drop did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b11_snip_and_drop_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b11_snip_and_drop.json");
    let first = subject_final_position(json, 390);
    let second = subject_final_position(json, 390);
    let third = subject_final_position(json, 390);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b12_blast_the_glass_is_solvable() {
    let json = include_str!("../levels/B/lvl_b12_blast_the_glass.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b12_blast_the_glass did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b12_blast_the_glass_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b12_blast_the_glass.json");
    let first = subject_final_position(json, 1000);
    let second = subject_final_position(json, 1000);
    let third = subject_final_position(json, 1000);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b13_chain_reaction_is_solvable() {
    let json = include_str!("../levels/B/lvl_b13_chain_reaction.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b13_chain_reaction did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b13_chain_reaction_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b13_chain_reaction.json");
    let first = subject_final_position(json, 1200);
    let second = subject_final_position(json, 1200);
    let third = subject_final_position(json, 1200);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b14_fragile_cargo_is_solvable() {
    let json = include_str!("../levels/B/lvl_b14_fragile_cargo.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b14_fragile_cargo did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b15_ride_to_the_zone_is_solvable() {
    let json = include_str!("../levels/B/lvl_b15_ride_to_the_zone.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b15_ride_to_the_zone did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b15_ride_to_the_zone_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b15_ride_to_the_zone.json");
    let first = subject_final_position(json, 580);
    let second = subject_final_position(json, 580);
    let third = subject_final_position(json, 580);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b16_rise_and_shine_is_solvable() {
    let json = include_str!("../levels/B/lvl_b16_rise_and_shine.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_b16_rise_and_shine did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_b16_rise_and_shine_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b16_rise_and_shine.json");
    let first = subject_final_position(json, 540);
    let second = subject_final_position(json, 540);
    let third = subject_final_position(json, 540);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_b14_fragile_cargo_is_deterministic() {
    let json = include_str!("../levels/B/lvl_b14_fragile_cargo.json");
    let first = subject_final_position(json, 660);
    let second = subject_final_position(json, 660);
    let third = subject_final_position(json, 660);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
