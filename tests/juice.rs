//! docs/GDD.md §3.9.6's Juice table (`src/juice.rs`): placement gets a
//! scale-bounce + a color flash, solving flashes every part, failing fades
//! in an overlay. All headless-safe by design (see `juice.rs`'s own
//! docs), so — unlike `render_fx.rs`'s Gizmos-based rope/belt lines, which
//! genuinely need the real renderer — this is real automated coverage,
//! not just a visual spot-check.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::game_state::GameState;
use chainworks::input::{EditorInputPlugin, PointerState};
use chainworks::juice::{FailOverlay, Flash, JuiceBounce, JuicePlugin};
use chainworks::level_file_format::LevelFile;
use chainworks::level_load::LevelPlugin;
use chainworks::parts::PartsPlugin;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::ui::parts_bin::BinSlot;
use chainworks::win_conditions::WinConditionsPlugin;

const DRAG_BRIDGE_LEVEL: &str = include_str!("../levels/A/lvl_m5_drag_bridge.json");
const FIRST_ROLL_LEVEL: &str = include_str!("../levels/A/lvl_a01_first_roll.json");
const DROP_POS: Vec2 = Vec2::new(0.0, 96.0);
const MAX_TICKS_TO_SOLVE: u32 = 2000;

fn build_app(level_json: &str) -> App {
    let level: LevelFile = serde_json::from_str(level_json).expect("fixture level JSON must be valid");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(EditorInputPlugin)
        .add_plugins(WinConditionsPlugin)
        .add_plugins(JuicePlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    app.update();
    app
}

fn set_state(app: &mut App, state: GameState) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(state);
    app.update();
}

/// Same drag-from-bin gesture `tests/input.rs` already drives for real —
/// see that file's own docs on why a bare `(BinSlot, Button, Interaction)`
/// pair stands in for the real bevy_ui picking backend headlessly.
fn drag_plank_from_bin_to(app: &mut App, drop_pos: Vec2) {
    let bin_slot = app
        .world_mut()
        .spawn((
            BinSlot { part_type: "plank_wood".to_string(), bin_index: 0 },
            Button,
            Interaction::default(),
        ))
        .id();
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.world_pos = Some(drop_pos);
        pointer.pressed = true;
    }
    app.world_mut().entity_mut(bin_slot).insert(Interaction::Pressed);
    app.update();
    app.world_mut().entity_mut(bin_slot).insert(Interaction::None);
    app.update();
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.pressed = false;
        pointer.just_released = true;
    }
    app.update();
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.just_released = false;
    }
    app.update();
}

#[test]
fn placing_a_part_bounces_and_flashes_then_settles() {
    let mut app = build_app(DRAG_BRIDGE_LEVEL);

    drag_plank_from_bin_to(&mut app, DROP_POS);

    let mut bounce_query = app.world_mut().query::<&JuiceBounce>();
    assert!(
        bounce_query.iter(app.world()).next().is_some(),
        "placing a part should attach a JuiceBounce"
    );
    let mut flash_query = app.world_mut().query::<&Flash>();
    assert!(
        flash_query.iter(app.world()).next().is_some(),
        "placing a part should spawn a Flash"
    );

    // Both are short (160ms ≈ 19 ticks at 120Hz) — well within budget.
    for _ in 0..60 {
        app.update();
    }
    let mut bounce_query = app.world_mut().query::<&JuiceBounce>();
    assert!(
        bounce_query.iter(app.world()).next().is_none(),
        "JuiceBounce should have expired and been removed by now"
    );
    let mut flash_query = app.world_mut().query::<&Flash>();
    assert!(
        flash_query.iter(app.world()).next().is_none(),
        "Flash should have despawned itself by now"
    );
}

#[test]
fn solving_a_level_flashes_its_parts() {
    let mut app = build_app(FIRST_ROLL_LEVEL);

    set_state(&mut app, GameState::Running);
    let mut solved = false;
    for _ in 0..MAX_TICKS_TO_SOLVE {
        app.update();
        if *app.world().resource::<State<GameState>>().get() == GameState::Solved {
            solved = true;
            break;
        }
    }
    assert!(solved, "demo level did not reach Solved within {MAX_TICKS_TO_SOLVE} ticks");

    let mut flash_query = app.world_mut().query::<&Flash>();
    assert!(
        flash_query.iter(app.world()).next().is_some(),
        "reaching Solved should have spawned a celebration Flash on at least one part"
    );
}

#[test]
fn failing_shows_a_fail_overlay_that_clears_on_the_next_edit() {
    let mut app = build_app(FIRST_ROLL_LEVEL);

    set_state(&mut app, GameState::Failed);
    let mut overlay_query = app.world_mut().query::<&FailOverlay>();
    assert!(
        overlay_query.iter(app.world()).next().is_some(),
        "entering Failed should spawn a FailOverlay"
    );

    set_state(&mut app, GameState::Edit);
    let mut overlay_query = app.world_mut().query::<&FailOverlay>();
    assert!(
        overlay_query.iter(app.world()).next().is_none(),
        "returning to Edit should clear the FailOverlay"
    );
}
