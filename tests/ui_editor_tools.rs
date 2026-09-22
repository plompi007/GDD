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

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
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
    ConnectButton, DeleteButton, EditorToolsLogicPlugin, LoadButton, LoadMenuOpen, RotateButton,
    SaveButton, SaveNameBuffer, SaveNameField, SavedLevelRow, StatusMessage, TextInputFocus,
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

/// Sends one `KeyboardInput` "pressed" event carrying a typed character,
/// then runs one `app.update()` so `save_name_keyboard_system` sees it —
/// mirrors `press_button`'s one-shot shape but for the keyboard path
/// instead of `Interaction`.
///
/// Real winit sends the space bar as the distinct named `Key::Space`,
/// not `Key::Character(" ")` — a live Xvfb run of the actual app is what
/// caught `save_name_keyboard_system` originally missing that case, so
/// this helper reproduces it faithfully rather than the easier-but-wrong
/// `Key::Character(" ")` a naive fake would send.
fn type_char(app: &mut App, c: char) {
    let logical_key = if c == ' ' {
        Key::Space
    } else {
        let mut buf = [0u8; 4];
        Key::Character(c.encode_utf8(&mut buf).into())
    };
    app.world_mut().send_event(KeyboardInput {
        key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
        logical_key,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

fn press_key(app: &mut App, logical_key: Key) {
    app.world_mut().send_event(KeyboardInput {
        key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
        logical_key,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
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

/// Spawns a bare `(SavedLevelRow(path), Button, Interaction::Pressed)`
/// entity, mirroring what `ui/editor_tools.rs`'s `sync_load_menu` would
/// have produced for that path — that system needs `AssetServer` for its
/// rows' text so it can't run under this headless app (see this module's
/// top doc comment), but `saved_level_row_interaction` itself doesn't
/// care how the entity got there, only that it exists.
fn press_saved_level_row(app: &mut App, path: std::path::PathBuf) {
    let entity = app.world_mut().spawn((SavedLevelRow(path), Button, Interaction::Pressed)).id();
    app.update();
    app.world_mut().entity_mut(entity).despawn();
}

#[test]
fn save_then_load_round_trips_through_a_real_file_via_the_my_levels_menu() {
    let saves_dir = save_load::default_saves_dir();
    let saved_path = saves_dir.join("lvl_m9_connect_tool").with_extension("json");
    let _ = fs::remove_file(&saved_path); // clean slate, in case a previous run left one

    let mut app = build_app();

    // No name typed — Save falls back to the level's own id, same
    // filename the old single-slot behavior always used.
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

    // Load no longer loads a fixed slot directly — it opens the "My
    // Levels" menu, and a specific row (found by `list_saved_levels`,
    // faked here the same way the rest of this file fakes bevy_ui's
    // picking output) is what actually loads a file.
    press_button(&mut app, LoadButton);
    assert!(app.world().resource::<LoadMenuOpen>().0, "Load should have opened the menu");

    press_saved_level_row(&mut app, saved_path.clone());
    assert!(!app.world().resource::<LoadMenuOpen>().0, "picking a save should close the menu");
    {
        let editor_state = app.world().resource::<EditorState>();
        assert_eq!(
            editor_state.level.title, "Connect Tool Fixture",
            "picking the save should have restored the on-disk title, not kept the in-memory mutation"
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

#[test]
fn load_button_toggles_the_my_levels_menu_open_and_closed() {
    let mut app = build_app();
    assert!(!app.world().resource::<LoadMenuOpen>().0);

    press_button(&mut app, LoadButton);
    assert!(app.world().resource::<LoadMenuOpen>().0);

    press_button(&mut app, LoadButton);
    assert!(!app.world().resource::<LoadMenuOpen>().0, "pressing Load again should close it");
}

#[test]
fn typing_into_the_save_name_field_updates_the_buffer_and_supports_backspace() {
    let mut app = build_app();
    assert!(!app.world().resource::<TextInputFocus>().0, "unfocused until clicked");

    // Typing before the field has focus must be a no-op.
    type_char(&mut app, 'X');
    assert_eq!(app.world().resource::<SaveNameBuffer>().0, "");

    press_button(&mut app, SaveNameField);
    assert!(app.world().resource::<TextInputFocus>().0);

    for c in "Bridge Run".chars() {
        type_char(&mut app, c);
    }
    assert_eq!(app.world().resource::<SaveNameBuffer>().0, "Bridge Run");

    press_key(&mut app, Key::Backspace);
    press_key(&mut app, Key::Backspace);
    assert_eq!(app.world().resource::<SaveNameBuffer>().0, "Bridge R");

    // A character outside the safe filename charset never reaches the
    // buffer at all, not even filtered later.
    type_char(&mut app, '/');
    assert_eq!(app.world().resource::<SaveNameBuffer>().0, "Bridge R");

    press_key(&mut app, Key::Enter);
    assert!(!app.world().resource::<TextInputFocus>().0, "Enter should unfocus the field");
}

#[test]
fn save_button_uses_the_typed_name_instead_of_the_level_id() {
    let saves_dir = save_load::default_saves_dir();
    let named_path = saves_dir.join("My Custom Save").with_extension("json");
    let _ = fs::remove_file(&named_path);

    let mut app = build_app();
    press_button(&mut app, SaveNameField);
    for c in "My Custom Save".chars() {
        type_char(&mut app, c);
    }

    press_button(&mut app, SaveButton);
    assert!(named_path.exists(), "Save should have written to the typed name, {named_path:?}");

    let _ = fs::remove_file(&named_path);
}

#[test]
fn pressing_another_tool_button_defocuses_the_save_name_field() {
    let mut app = build_app();
    press_button(&mut app, SaveNameField);
    assert!(app.world().resource::<TextInputFocus>().0);

    press_button(&mut app, RotateButton);
    assert!(!app.world().resource::<TextInputFocus>().0, "clicking elsewhere should defocus it");
}
