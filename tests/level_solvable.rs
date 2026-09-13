//! docs/GDD.md M3 acceptance: level 1 must be solvable and resettable
//! infinitely. Loads the real demo level JSON (not a synthetic scene) and
//! drives it exactly like the game would: enter Running, step until
//! GameState::Solved, reset back to Edit, and confirm it solves again.

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

const DEMO_LEVEL_JSON: &str = include_str!("../levels/A/lvl_a01_first_roll.json");
const MAX_TICKS_TO_SOLVE: u32 = 2000; // ~16.7s of simulated time at 120Hz

fn build_app() -> App {
    let level: LevelFile =
        serde_json::from_str(DEMO_LEVEL_JSON).expect("demo level JSON must be valid");

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
    // `Edit` is the default GameState, but its `OnEnter` (which spawns the
    // level via `reset_level`) only fires once the state-transition machinery
    // actually runs — one `update()` in. Without this, transitioning straight
    // to `Running` before the level ever spawned leaves nothing to simulate.
    app.update();
    app
}

fn set_state(app: &mut App, state: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(state);
    app.update();
}

fn run_until_solved(app: &mut App, max_ticks: u32) -> bool {
    for _ in 0..max_ticks {
        app.update();
        if *app.world().resource::<State<GameState>>().get() == GameState::Solved {
            return true;
        }
    }
    false
}

#[test]
fn demo_level_is_solvable_by_gravity_alone() {
    let mut app = build_app();

    // Starts in Edit (the default state) — nothing should be simulating yet.
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Edit
    );

    set_state(&mut app, GameState::Running);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "demo level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
    );
}

#[test]
fn demo_level_is_resettable_and_solvable_again() {
    let mut app = build_app();

    for attempt in 1..=3 {
        set_state(&mut app, GameState::Running);
        assert!(
            run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
            "attempt {attempt}: demo level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks"
        );

        // docs/GDD.md §2.5: reset rebuilds from EditorState, never rewinds
        // physics — going back to Edit must produce a fresh, solvable world.
        set_state(&mut app, GameState::Edit);
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Edit,
            "attempt {attempt}: reset did not return to Edit"
        );
    }
}
