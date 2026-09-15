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
use chainworks::input::{
    ConnectToolRequest, EditorActionRequest, EditorInputPlugin, PointerState, SelectedPart,
};
use chainworks::level_file_format::{ConnectionKind, LevelFile};
use chainworks::level_load::LevelPlugin;
use chainworks::parts::PartsPlugin;
use chainworks::rope_network::RopeConnection;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::ui::parts_bin::BinSlot;
use chainworks::win_conditions::WinConditionsPlugin;

const LEVEL_JSON: &str = include_str!("../levels/A/lvl_m5_drag_bridge.json");
const DROP_POS: Vec2 = Vec2::new(0.0, 96.0);
const MAX_TICKS_TO_SOLVE: u32 = 2000;

/// The real bin button is `ui::parts_bin`'s bevy_ui `Button` (M6) — driven
/// by bevy_ui's own picking backend, which needs a real window/camera this
/// headless test doesn't have. Spawning just the `BinSlot` + `Interaction`
/// pair `input::start_bin_drag_system` actually reads (mirroring exactly
/// what `parts_bin::spawn_parts_bin` would create for this level's single
/// bin entry) exercises that same consumption code without needing the
/// full UI/asset/rendering stack.
fn spawn_bin_slot_for_test(app: &mut App, part_type: &str, bin_index: usize) -> Entity {
    app.world_mut()
        .spawn((
            BinSlot {
                part_type: part_type.to_string(),
                bin_index,
            },
            Button,
            Interaction::default(),
        ))
        .id()
}

fn build_app() -> App {
    build_app_with(LEVEL_JSON)
}

fn build_app_with(level_json: &str) -> App {
    let level: LevelFile = serde_json::from_str(level_json).expect("demo level JSON must be valid");

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
    // Lets the initial `OnEnter(Edit)` spawn (level parts) fire before any
    // pointer/state changes are requested — same reasoning as
    // tests/level_solvable.rs's `build_app`.
    app.update();
    app
}

fn set_pointer(app: &mut App, world_pos: Vec2, pressed: bool, just_released: bool) {
    let mut pointer = app.world_mut().resource_mut::<PointerState>();
    pointer.world_pos = Some(world_pos);
    pointer.pressed = pressed;
    pointer.just_released = just_released;
}

/// Simulates: press the (only) bin button, drag to the drop point, release
/// — exactly the gesture a real mouse-drag or touch-drag performs. The
/// button press itself goes through a real `Interaction::Pressed` (see
/// [`spawn_bin_slot_for_test`]); everything from there on replays through
/// `PointerState`, same as a real mouse-drag or touch-drag would.
fn drag_plank_from_bin_to(app: &mut App, bin_slot: Entity, drop_pos: Vec2) {
    set_pointer(app, drop_pos, true, false);
    app.world_mut()
        .entity_mut(bin_slot)
        .insert(Interaction::Pressed);
    app.update();
    app.world_mut().entity_mut(bin_slot).insert(Interaction::None);
    app.update();
    set_pointer(app, drop_pos, false, true);
    app.update();
    set_pointer(app, drop_pos, false, false);
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
    let bin_slot = spawn_bin_slot_for_test(&mut app, "plank_wood", 0);
    drag_plank_from_bin_to(&mut app, bin_slot, DROP_POS);
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
    let bin_slot = spawn_bin_slot_for_test(&mut app, "plank_wood", 0);
    drag_plank_from_bin_to(&mut app, bin_slot, DROP_POS);
    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    assert_eq!(editor_state.level.parts_bin[0].count, 0);
    assert_eq!(editor_state.level.preplaced_parts.len(), 2); // subject + the placed plank
}

/// A tap-and-release on an already-placed part's own position, with no
/// movement in between — selects it (`start_existing_part_drag_system`'s
/// hit-test) without relocating it, mirroring the real click/tap gesture
/// M9's delete/rotate actions are meant to apply to.
fn tap_to_select(app: &mut App, world_pos: Vec2) {
    set_pointer(app, world_pos, true, false);
    app.world_mut().resource_mut::<PointerState>().just_pressed = true;
    app.update();
    app.world_mut().resource_mut::<PointerState>().just_pressed = false;
    app.update();
    set_pointer(app, world_pos, false, true);
    app.update();
    set_pointer(app, world_pos, false, false);
    app.update();
}

#[test]
fn deleting_the_selected_plank_removes_it_and_the_level_can_no_longer_solve() {
    let mut app = build_app();
    let bin_slot = spawn_bin_slot_for_test(&mut app, "plank_wood", 0);
    drag_plank_from_bin_to(&mut app, bin_slot, DROP_POS);
    tap_to_select(&mut app, DROP_POS);
    assert!(
        app.world().resource::<SelectedPart>().0.is_some(),
        "tapping the placed plank should select it"
    );

    app.world_mut()
        .resource_mut::<EditorActionRequest>()
        .delete_selected = true;
    app.update();

    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    assert_eq!(
        editor_state.level.preplaced_parts.len(),
        1,
        "only the subject should remain once the plank is deleted"
    );
    assert!(app.world().resource::<SelectedPart>().0.is_none());
    assert!(
        !run_until_solved(&mut app, MAX_TICKS_TO_SOLVE),
        "level solved itself after the only plank was deleted — nothing should catch the ball"
    );
}

/// A single tap (press-and-release with no movement) at a world position
/// — enough to drive [`chainworks::input::connect_tool_system`], which
/// only reacts to `just_pressed`. Unlike [`tap_to_select`] this doesn't
/// go through `start_existing_part_drag_system` at all (connect mode
/// disables that system while active), so it works on fixed parts too.
fn tap(app: &mut App, world_pos: Vec2) {
    let mut pointer = app.world_mut().resource_mut::<PointerState>();
    pointer.world_pos = Some(world_pos);
    pointer.just_pressed = true;
    pointer.pressed = true;
    pointer.just_released = false;
    drop(pointer);
    app.update();
    let mut pointer = app.world_mut().resource_mut::<PointerState>();
    pointer.just_pressed = false;
    pointer.pressed = false;
    pointer.just_released = true;
    drop(pointer);
    app.update();
}

const CONNECT_FIXTURE_JSON: &str = include_str!("../levels/A/lvl_m9_connect_tool.json");

#[test]
fn connecting_a_falling_ball_to_a_fixed_anchor_with_a_rope_catches_its_fall() {
    let mut app = build_app_with(CONNECT_FIXTURE_JSON);

    let (anchor_entity, ball_entity) = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &chainworks::level_load::PlacedId)>();
        let mut anchor = None;
        let mut ball = None;
        for (entity, id) in query.iter(app.world()) {
            match id.0.as_str() {
                "anchor" => anchor = Some(entity),
                "ball" => ball = Some(entity),
                _ => {}
            }
        }
        (anchor.expect("anchor must exist"), ball.expect("ball must exist"))
    };

    app.world_mut()
        .resource_mut::<ConnectToolRequest>()
        .kind = Some(ConnectionKind::Rope);
    tap(&mut app, Vec2::new(0.0, 300.0)); // anchor's position
    tap(&mut app, Vec2::new(0.0, 100.0)); // ball's position

    assert_eq!(
        app.world_mut()
            .query::<&RopeConnection>()
            .iter(app.world())
            .count(),
        1,
        "connecting rope-mode should have spawned exactly one RopeConnection"
    );
    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    assert_eq!(editor_state.level.connections.len(), 1);
    assert_eq!(editor_state.level.connections[0].kind, ConnectionKind::Rope);
    assert!(
        app.world()
            .resource::<ConnectToolRequest>()
            .kind
            .is_some(),
        "connect mode should stay active for further connections until explicitly turned off"
    );

    // The ball starts 200 units below the anchor; a rope with maxLength =
    // 200 * 1.05 = 210 should let it fall only a little further (~10
    // units) before tension catches it, nowhere near unconstrained
    // free-fall over the same time (hundreds of units, per every other
    // free-fall level in this project).
    for _ in 0..300 {
        app.update();
    }
    let anchor_y = app.world().get::<Transform>(anchor_entity).unwrap().translation.y;
    let ball_y = app.world().get::<Transform>(ball_entity).unwrap().translation.y;
    let fallen = anchor_y - ball_y - 200.0;
    assert!(
        (0.0..60.0).contains(&fallen),
        "ball should be caught by the rope's tension near its 210-unit max length, \
         but it fell {fallen:.1} units past its 200-unit starting offset (anchor_y={anchor_y}, ball_y={ball_y})"
    );
}

#[test]
fn connecting_a_switch_to_a_motor_with_a_wire_starts_it_spinning() {
    let mut app = build_app_with(CONNECT_FIXTURE_JSON);

    let motor_entity = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &chainworks::level_load::PlacedId)>();
        query
            .iter(app.world())
            .find(|(_, id)| id.0 == "motor")
            .map(|(entity, _)| entity)
            .expect("motor must exist")
    };

    app.world_mut()
        .resource_mut::<ConnectToolRequest>()
        .kind = Some(ConnectionKind::Wire);
    tap(&mut app, Vec2::new(400.0, 300.0)); // switch's position
    tap(&mut app, Vec2::new(400.0, 100.0)); // motor's position

    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    assert_eq!(editor_state.level.connections.len(), 1);
    assert_eq!(editor_state.level.connections[0].kind, ConnectionKind::Wire);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Running);
    for _ in 0..20 {
        app.update();
    }

    let rotary = app
        .world()
        .get::<chainworks::gear_train::RotaryState>(motor_entity)
        .expect("motor must have a RotaryState");
    assert_ne!(
        rotary.angular_velocity, 0.0,
        "motor should be spinning once wired to the switch a ball is already resting on"
    );
}

#[test]
fn rotating_the_selected_plank_updates_both_editor_state_and_the_live_transform() {
    let mut app = build_app();
    let bin_slot = spawn_bin_slot_for_test(&mut app, "plank_wood", 0);
    drag_plank_from_bin_to(&mut app, bin_slot, DROP_POS);
    tap_to_select(&mut app, DROP_POS);

    app.world_mut()
        .resource_mut::<EditorActionRequest>()
        .rotate_selected = true;
    app.update();

    // plank_wood's own data/parts/plank_wood.json declares rotationSnap: 15.
    let editor_state = app
        .world()
        .resource::<chainworks::level_load::EditorState>();
    let plank = editor_state
        .level
        .preplaced_parts
        .iter()
        .find(|p| p.part_type == "plank_wood")
        .expect("plank must still be placed");
    assert_eq!(plank.rotation, 15.0);

    let selected = app
        .world()
        .resource::<SelectedPart>()
        .0
        .expect("plank should still be selected after rotating");
    let transform = app.world().get::<Transform>(selected).unwrap();
    let live_rotation_deg = transform.rotation.to_euler(EulerRot::XYZ).2.to_degrees();
    assert!(
        (live_rotation_deg - 15.0).abs() < 0.01,
        "live Transform rotation should match the 15-degree snap, got {live_rotation_deg}"
    );
}
