//! M9/Sandbox's bevy_ui button row (`src/ui/editor_tools.rs`) — drives the
//! exact same resources `tests/input.rs` already proved work
//! (`EditorActionRequest`, `ConnectToolRequest`) and `src/save_load.rs`'s
//! real file I/O, but through a button's `Interaction::Pressed` instead
//! of a direct resource write. Uses `EditorToolsLogicPlugin` (no
//! `AssetServer` dependency — see that plugin's own docs) and spawns bare
//! `(Marker, Interaction)` pairs the same way `tests/input.rs`'s
//! `spawn_bin_slot_for_test` fakes bevy_ui's real picking backend.

use std::fs;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::input::{ConnectToolRequest, EditorInputPlugin, PointerState};
use chainworks::level_file_format::{ConnectionKind, LevelFile};
use chainworks::level_load::{EditorState, LevelPlugin, PlacedId};
use chainworks::parts::PartsPlugin;
use chainworks::save_load;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::ui::editor_tools::{
    ConnectButton, DeleteButton, EditorToolsLogicPlugin, LoadButton, RotateButton, SaveButton,
    StatusMessage,
};
use chainworks::win_conditions::WinConditionsPlugin;

const LEVEL_JSON: &str = include_str!("../levels/A/lvl_m9_connect_tool.json");

fn build_app() -> App {
    let level: LevelFile = serde_json::from_str(LEVEL_JSON).expect("fixture level JSON must be valid");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(EditorInputPlugin)
        .add_plugins(WinConditionsPlugin)
        .add_plugins(EditorToolsLogicPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    app.update();
    app
}

/// Spawns a bare `(marker, Button, Interaction::Pressed)` entity, runs one
/// `app.update()` so the matching `*_button_interaction` system sees it,
/// then despawns it — a single simulated tap, not a held press.
fn press_button<T: Component>(app: &mut App, marker: T) {
    let entity = app.world_mut().spawn((marker, Button, Interaction::Pressed)).id();
    app.update();
    app.world_mut().entity_mut(entity).despawn();
}

#[test]
fn delete_button_removes_the_selected_part() {
    let mut app = build_app();
    let ball_entity = {
        let mut query = app.world_mut().query::<(Entity, &PlacedId)>();
        query
            .iter(app.world())
            .find(|(_, id)| id.0 == "ball")
            .map(|(e, _)| e)
            .expect("ball must exist")
    };
    app.world_mut().resource_mut::<chainworks::input::SelectedPart>().0 = Some(ball_entity);

    press_button(&mut app, DeleteButton);

    let editor_state = app.world().resource::<EditorState>();
    assert!(
        !editor_state.level.preplaced_parts.iter().any(|p| p.id == "ball"),
        "Delete button should have removed the selected ball from EditorState"
    );
    assert!(app.world().get_entity(ball_entity).is_err(), "ball entity should be despawned");
}

#[test]
fn rotate_button_rotates_the_selected_part() {
    // The connect-tool fixture's parts aren't rotatable per their own
    // data/parts JSON -- use a level that actually has a rotatable part
    // instead: the M5 drag-bridge plank, placed the same way
    // tests/input.rs does, then selected directly (this test only cares
    // about the Rotate button, not re-proving drag/place).
    let plank_level: LevelFile = serde_json::from_str(include_str!(
        "../levels/A/lvl_m5_drag_bridge.json"
    ))
    .unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level: plank_level })
        .add_plugins(EditorInputPlugin)
        .add_plugins(WinConditionsPlugin)
        .add_plugins(EditorToolsLogicPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    app.update();

    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.world_pos = Some(Vec2::new(0.0, 96.0));
        pointer.pressed = true;
        pointer.just_pressed = true;
    }
    let bin_slot = app
        .world_mut()
        .spawn((
            chainworks::ui::parts_bin::BinSlot { part_type: "plank_wood".to_string(), bin_index: 0 },
            Button,
            Interaction::Pressed,
        ))
        .id();
    app.update();
    app.world_mut().entity_mut(bin_slot).insert(Interaction::None);
    app.update();
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.pressed = false;
        pointer.just_pressed = false;
        pointer.just_released = true;
    }
    app.update();
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.just_released = false;
    }
    app.update();

    // Select the just-placed plank with a plain tap (no movement).
    let plank_entity = {
        let mut query = app.world_mut().query::<(Entity, &PlacedId)>();
        query
            .iter(app.world())
            .find(|(_, id)| id.0.starts_with("placed_"))
            .map(|(e, _)| e)
            .expect("plank must have been placed")
    };
    {
        let mut pointer = app.world_mut().resource_mut::<PointerState>();
        pointer.world_pos = Some(Vec2::new(0.0, 96.0));
        pointer.just_pressed = true;
        pointer.pressed = true;
    }
    app.update();

    press_button(&mut app, RotateButton);

    let editor_state = app.world().resource::<EditorState>();
    let plank = editor_state
        .level
        .preplaced_parts
        .iter()
        .find(|p| p.part_type == "plank_wood")
        .expect("plank must still be placed");
    assert_eq!(plank.rotation, 15.0, "plank_wood's own rotationSnap is 15");
    let transform = app.world().get::<Transform>(plank_entity).unwrap();
    assert!((transform.rotation.to_euler(EulerRot::XYZ).2.to_degrees() - 15.0).abs() < 0.01);
}

#[test]
fn connect_button_toggles_connect_mode_on_and_off() {
    let mut app = build_app();

    press_button(&mut app, ConnectButton(ConnectionKind::Rope));
    assert_eq!(
        app.world().resource::<ConnectToolRequest>().kind,
        Some(ConnectionKind::Rope)
    );

    // Pressing the same button again turns it back off.
    press_button(&mut app, ConnectButton(ConnectionKind::Rope));
    assert_eq!(app.world().resource::<ConnectToolRequest>().kind, None);

    press_button(&mut app, ConnectButton(ConnectionKind::Wire));
    assert_eq!(
        app.world().resource::<ConnectToolRequest>().kind,
        Some(ConnectionKind::Wire)
    );
}

#[test]
fn save_then_load_buttons_round_trip_through_a_real_file() {
    let saves_dir = save_load::default_saves_dir();
    let saved_path = saves_dir.join("lvl_m9_connect_tool").with_extension("json");
    let _ = fs::remove_file(&saved_path); // clean slate, in case a previous run left one

    let mut app = build_app();

    press_button(&mut app, SaveButton);
    assert!(saved_path.exists(), "Save button should have written {saved_path:?}");
    let status = app.world_mut().resource_mut::<StatusMessage>().0.take();
    assert!(
        status.as_deref().is_some_and(|s| s.starts_with("Saved to")),
        "expected a 'Saved to ...' status message, got {status:?}"
    );

    // Mutate EditorState in-memory (without touching the saved file) so
    // Load's effect is actually observable, not just "still the same
    // because nothing changed".
    app.world_mut().resource_mut::<EditorState>().level.title = "Mutated In Memory".to_string();

    press_button(&mut app, LoadButton);
    {
        let editor_state = app.world().resource::<EditorState>();
        assert_eq!(
            editor_state.level.title, "Connect Tool Fixture",
            "Load button should have restored the on-disk title, not kept the in-memory mutation"
        );
    }

    // One more tick lets the forced-rebuild system (level_load.rs) act on
    // the flag Load just set — the real, observable effect (a fresh
    // respawn from the reloaded EditorState) rather than the flag itself,
    // which is an implementation detail with no ordering guarantee
    // relative to *this* same update.
    app.update();
    let ball_still_present = app
        .world_mut()
        .query::<&PlacedId>()
        .iter(app.world())
        .any(|id| id.0 == "ball");
    assert!(ball_still_present, "rebuild after Load should have respawned the fixture's parts");

    let _ = fs::remove_file(&saved_path);
}
