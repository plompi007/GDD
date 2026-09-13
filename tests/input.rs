//! M5 acceptance (docs/GDD.md §5.8): "can build a complete solution with
//! either mouse or finger". Drives the exact same `PointerState` that the
//! real mouse/touch adapters write (see `src/input.rs`) — a genuine
//! end-to-end exercise of the drag-from-bin placement mechanic, not a
//! shortcut around it: this level is unsolvable without it (the ball falls
//! straight through empty space unless a plank is placed to catch it).

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::game_state::GameState;
use chainworks::input::{EditorInputPlugin, PointerState};
use chainworks::level_file_format::LevelFile;
use chainworks::level_load::LevelPlugin;
use chainworks::parts::PartsPlugin;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::win_conditions::WinConditionsPlugin;

const LEVEL_JSON: &str = include_str!("../levels/A/lvl_m5_drag_bridge.json");
const BIN_BUTTON_POS: Vec2 = Vec2::new(0.0, -300.0);
const DROP_POS: Vec2 = Vec2::new(0.0, 96.0);
const MAX_TICKS_TO_SOLVE: u32 = 2000;

fn build_app() -> App {
    let level: LevelFile = serde_json::from_str(LEVEL_JSON).expect("demo level JSON must be valid");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(EditorInputPlugin)
        .add_plugins(WinConditionsPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    // Lets the initial `OnEnter(Edit)` spawn (level parts + bin buttons)
    // fire before any pointer/state changes are requested — same reasoning
    // as tests/level_solvable.rs's `build_app`.
    app.update();
    app
}

fn set_pointer(app: &mut App, world_pos: Vec2, just_pressed: bool, pressed: bool, just_released: bool) {
    let mut pointer = app.world_mut().resource_mut::<PointerState>();
    pointer.world_pos = Some(world_pos);
    pointer.just_pressed = just_pressed;
    pointer.pressed = pressed;
    pointer.just_released = just_released;
}

/// Simulates: press on the (only) bin button, drag to the drop point,
/// release — exactly the gesture a real mouse-drag or touch-drag performs,
/// just replayed through `PointerState` instead of a real window/OS input.
fn drag_plank_from_bin_to(app: &mut App, drop_pos: Vec2) {
    set_pointer(app, BIN_BUTTON_POS, true, true, false);
    app.update();
    set_pointer(app, drop_pos, false, true, false);
    app.update();
    set_pointer(app, drop_pos, false, false, true);
    app.update();
    set_pointer(app, drop_pos, false, false, false);
    app.update();
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
fn dragging_a_plank_from_the_bin_lets_the_level_solve() {
    let mut app = build_app();
    drag_plank_from_bin_to(&mut app, DROP_POS);
    assert!(
        run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks after placing the plank"
    );
}

#[test]
fn without_placing_anything_the_level_fails() {
    let mut app = build_app();
    // No drag at all — the ball must fall straight through and the level
    // must eventually fail (LEFT_BOUNDS or TIMEOUT), never solve itself.
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Running);
    for _ in 0..MAX_TICKS_TO_SOLVE {
        app.update();
        let state = *app.world().resource::<State<GameState>>().get();
        assert_ne!(
            state,
            GameState::Solved,
            "level solved itself with nothing placed — CONTAINED must be checking the wrong thing"
        );
        if state == GameState::Failed {
            return;
        }
    }
    panic!("level neither solved nor failed within {MAX_TICKS_TO_SOLVE} ticks with nothing placed");
}

#[test]
fn placing_a_plank_consumes_it_from_the_parts_bin() {
    let mut app = build_app();
    drag_plank_from_bin_to(&mut app, DROP_POS);
    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    assert_eq!(editor_state.level.parts_bin[0].count, 0);
    assert_eq!(editor_state.level.preplaced_parts.len(), 2); // subject + the placed plank
}
