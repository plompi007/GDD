//! M8 content acceptance for chapter D ("D_MASTER" — docs/GDD.md's chapter
//! enum, §appendix "שרשראות ארוכות"): long causal chains through several
//! mechanics at once. Same pattern as tests/content_levels*.rs.

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
fn lvl_d01_chain_of_command_is_solvable() {
    let json = include_str!("../levels/D/lvl_d01_chain_of_command.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d01_chain_of_command did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d01_chain_of_command_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d01_chain_of_command.json");
    let first = subject_final_position(json, 1500);
    let second = subject_final_position(json, 1500);
    let third = subject_final_position(json, 1500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d02_signal_chain_is_solvable() {
    let json = include_str!("../levels/D/lvl_d02_signal_chain.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d02_signal_chain did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d02_signal_chain_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d02_signal_chain.json");
    let first = subject_final_position(json, 1100);
    let second = subject_final_position(json, 1100);
    let third = subject_final_position(json, 1100);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d03_bounce_and_cut_is_solvable() {
    let json = include_str!("../levels/D/lvl_d03_bounce_and_cut.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d03_bounce_and_cut did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d03_bounce_and_cut_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d03_bounce_and_cut.json");
    let first = subject_final_position(json, 650);
    let second = subject_final_position(json, 650);
    let third = subject_final_position(json, 650);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d04_wind_and_fire_is_solvable() {
    let json = include_str!("../levels/D/lvl_d04_wind_and_fire.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d04_wind_and_fire did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d04_wind_and_fire_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d04_wind_and_fire.json");
    let first = subject_final_position(json, 1500);
    let second = subject_final_position(json, 1500);
    let third = subject_final_position(json, 1500);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d05_fan_the_flame_is_solvable() {
    let json = include_str!("../levels/D/lvl_d05_fan_the_flame.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d05_fan_the_flame did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d05_fan_the_flame_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d05_fan_the_flame.json");
    let first = subject_final_position(json, 1750);
    let second = subject_final_position(json, 1750);
    let third = subject_final_position(json, 1750);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d06_bounce_and_blow_is_solvable() {
    let json = include_str!("../levels/D/lvl_d06_bounce_and_blow.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d06_bounce_and_blow did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d06_bounce_and_blow_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d06_bounce_and_blow.json");
    let first = subject_final_position(json, 870);
    let second = subject_final_position(json, 870);
    let third = subject_final_position(json, 870);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d07_gentle_command_is_solvable() {
    let json = include_str!("../levels/D/lvl_d07_gentle_command.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d07_gentle_command did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d07_gentle_command_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d07_gentle_command.json");
    let first = subject_final_position(json, 930);
    let second = subject_final_position(json, 930);
    let third = subject_final_position(json, 930);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d08_punch_the_line_is_solvable() {
    let json = include_str!("../levels/D/lvl_d08_punch_the_line.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d08_punch_the_line did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d08_punch_the_line_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d08_punch_the_line.json");
    let first = subject_final_position(json, 935);
    let second = subject_final_position(json, 935);
    let third = subject_final_position(json, 935);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d09_high_release_is_solvable() {
    let json = include_str!("../levels/D/lvl_d09_high_release.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d09_high_release did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d09_high_release_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d09_high_release.json");
    let first = subject_final_position(json, 760);
    let second = subject_final_position(json, 760);
    let third = subject_final_position(json, 760);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d10_twin_flame_relay_is_solvable() {
    let json = include_str!("../levels/D/lvl_d10_twin_flame_relay.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d10_twin_flame_relay did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d10_twin_flame_relay_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d10_twin_flame_relay.json");
    let first = subject_final_position(json, 1350);
    let second = subject_final_position(json, 1350);
    let third = subject_final_position(json, 1350);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d11_two_cuts_one_belt_is_solvable() {
    let json = include_str!("../levels/D/lvl_d11_two_cuts_one_belt.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d11_two_cuts_one_belt did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d11_two_cuts_one_belt_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d11_two_cuts_one_belt.json");
    let first = subject_final_position(json, 660);
    let second = subject_final_position(json, 660);
    let third = subject_final_position(json, 660);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d12_clear_and_carry_is_solvable() {
    let json = include_str!("../levels/D/lvl_d12_clear_and_carry.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d12_clear_and_carry did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d12_clear_and_carry_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d12_clear_and_carry.json");
    let first = subject_final_position(json, 870);
    let second = subject_final_position(json, 870);
    let third = subject_final_position(json, 870);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d13_wind_signal_is_solvable() {
    let json = include_str!("../levels/D/lvl_d13_wind_signal.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d13_wind_signal did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d13_wind_signal_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d13_wind_signal.json");
    let first = subject_final_position(json, 770);
    let second = subject_final_position(json, 770);
    let third = subject_final_position(json, 770);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d14_blast_and_belt_is_solvable() {
    let json = include_str!("../levels/D/lvl_d14_blast_and_belt.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d14_blast_and_belt did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d14_blast_and_belt_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d14_blast_and_belt.json");
    let first = subject_final_position(json, 970);
    let second = subject_final_position(json, 970);
    let third = subject_final_position(json, 970);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d15_four_gears_gently_is_solvable() {
    let json = include_str!("../levels/D/lvl_d15_four_gears_gently.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d15_four_gears_gently did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d15_four_gears_gently_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d15_four_gears_gently.json");
    let first = subject_final_position(json, 910);
    let second = subject_final_position(json, 910);
    let third = subject_final_position(json, 910);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}

#[test]
fn lvl_d16_grand_finale_is_solvable() {
    let json = include_str!("../levels/D/lvl_d16_grand_finale.json");
    let mut app = build_app(json);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "lvl_d16_grand_finale did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn lvl_d16_grand_finale_is_deterministic() {
    let json = include_str!("../levels/D/lvl_d16_grand_finale.json");
    let first = subject_final_position(json, 1520);
    let second = subject_final_position(json, 1520);
    let third = subject_final_position(json, 1520);
    assert_eq!(first, second, "run 1 and run 2 diverged");
    assert_eq!(second, third, "run 2 and run 3 diverged");
}
