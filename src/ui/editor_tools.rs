//! M9/Sandbox editor tool row (docs/GDD.md's Sandbox spec, §3.4/§3.5's
//! long-deferred UI chrome): the bevy_ui buttons that drive the
//! resource-based mechanics `input.rs` already implements and tests —
//! [`chainworks::input::EditorActionRequest`] (delete/rotate),
//! [`chainworks::input::ConnectToolRequest`] (rope/belt/wire), and
//! [`chainworks::save_load`] (save/load a real file, now to a
//! player-named slot rather than one fixed one — see [`SaveNameField`]).
//! Sits directly above `parts_bin`'s row, same styling tokens as
//! `hud.rs`/`parts_bin.rs`.
//!
//! **"My Levels" browsing:** [`LoadButton`] no longer loads one fixed
//! slot directly — it toggles [`LoadMenuOpen`], and `sync_load_menu`
//! (real-UI-only, needs `AssetServer` for its rows' text, so it's on
//! [`EditorToolsPlugin`] rather than [`EditorToolsLogicPlugin`]) spawns
//! a row per [`crate::save_load::list_saved_levels`] entry. This is a
//! flat list, not `LevelSelect`'s own screen (still separate, still M6
//! debt) — enough to pick among several of a player's own saves, not a
//! browsable catalog of the built-in 60.

use std::path::PathBuf;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

use crate::input::{ConnectToolRequest, EditorActionRequest};
use crate::level_catalog::enter_sandbox;
use crate::level_file_format::ConnectionKind;
use crate::level_load::{EditorState, ForceRebuildRequest};
use crate::save_load;
use crate::ui::tokens::{palette, radius, SPACE, UI_FONT_MEDIUM};

#[derive(Component)]
struct ToolsRoot;

/// `pub`: headless tests spawn this directly (paired with a real
/// `Interaction`) to drive the button-interaction systems below without
/// needing `AssetServer`/a real window — the same reasoning
/// `parts_bin::BinSlot` is `pub` for, see `tests/input.rs`.
#[derive(Component)]
pub struct DeleteButton;

#[derive(Component)]
pub struct RotateButton;

#[derive(Component)]
pub struct ConnectButton(pub ConnectionKind);

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct SandboxButton;

/// The text-entry box a player types a save name into — `pub` for the
/// same reason as `DeleteButton`/`SaveButton` above, but note that
/// *typing into it* isn't driven by `Interaction` the way a button press
/// is: see [`save_name_keyboard_system`] for the actual keystroke path,
/// and this module's own top doc comment for why the name it produces
/// goes through `save_load::sanitize_file_stem` rather than straight to
/// disk.
#[derive(Component)]
pub struct SaveNameField;

#[derive(Component)]
struct SaveNameText;

/// Whether [`SaveNameField`] currently has keyboard focus. While `true`,
/// `app.rs`'s desktop shortcuts (Space/R/N/P/B) must not act on the same
/// keystrokes — typing a save name like `"Bridge Run"` would otherwise
/// also reset the level (R), cycle it (N/P), or jump to Sandbox (B) as
/// an unwanted side effect. `pub` so `app.rs` can read (not write) it.
#[derive(Resource, Default)]
pub struct TextInputFocus(pub bool);

/// What's currently typed into [`SaveNameField`] — already filtered to a
/// safe filename charset on every keystroke by
/// [`save_name_keyboard_system`], not just when [`SaveButton`] is
/// finally pressed (a nicer typing experience: no character a player
/// types ever visibly appears only to silently vanish at save time).
#[derive(Resource, Default)]
pub struct SaveNameBuffer(pub String);

/// Whether the "My Levels" list (see this module's top doc comment) is
/// currently shown. `pub` so headless tests can assert on it directly
/// without needing the real spawned list.
#[derive(Resource, Default)]
pub struct LoadMenuOpen(pub bool);

#[derive(Component)]
struct LoadMenuRoot;

/// One row of the "My Levels" list, carrying the exact path
/// `save_load::list_saved_levels` found it at. `pub`: headless tests
/// spawn this directly (paired with a real `Interaction`), the same way
/// `DeleteButton`/`SaveButton` above are — see this module's top doc
/// comment on why the real spawn path (`sync_load_menu`) can't run
/// headless at all (it needs `AssetServer` for each row's text).
#[derive(Component)]
pub struct SavedLevelRow(pub PathBuf);

/// A one-line status message (e.g. "Saved." / "Nothing saved yet.") —
/// its own component so [`sync_status_text`] can find and update it
/// without needing to know this row's exact layout.
#[derive(Component)]
struct StatusText;

/// Set by the save/load button systems, read by [`sync_status_text`] —
/// same one-shot-request shape as everything else in this module, kept
/// separate from a direct `Text` mutation so those systems don't need a
/// `Query` for a UI detail that isn't really their job. `pub` so headless
/// tests can assert on the outcome ("Saved to ..." vs. "Save failed: ...")
/// without needing the real `StatusText` UI node to read it back from.
#[derive(Resource, Default)]
pub struct StatusMessage(pub Option<String>);

fn tool_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    font: &TextFont,
    marker: T,
    label: &str,
) {
    parent
        .spawn((
            marker,
            Button,
            Node {
                height: Val::Px(40.0),
                padding: UiRect::axes(Val::Px(SPACE[3]), Val::Px(SPACE[1])),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette::BG_PANEL),
            BorderRadius::all(Val::Px(radius::SM)),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new(label), font.clone(), TextColor(palette::INK_PRIMARY)));
        });
}

/// The save-name text-entry box — visually similar to [`tool_button`]
/// but not a `Button` a single press "does" anything to; its content
/// comes from [`save_name_keyboard_system`], its live text/highlight
/// from [`sync_save_name_text`].
fn save_name_field(parent: &mut ChildSpawnerCommands, font: &TextFont) {
    parent
        .spawn((
            SaveNameField,
            Button,
            Node {
                height: Val::Px(40.0),
                min_width: Val::Px(130.0),
                padding: UiRect::axes(Val::Px(SPACE[3]), Val::Px(SPACE[1])),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(palette::BG_PANEL),
            BorderRadius::all(Val::Px(radius::SM)),
        ))
        .with_children(|parent| {
            parent.spawn((
                SaveNameText,
                Text::new("(level name)"),
                font.clone(),
                TextColor(palette::INK_SECONDARY),
            ));
        });
}

fn spawn_editor_tools(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = TextFont {
        font: asset_server.load(UI_FONT_MEDIUM),
        font_size: 15.0,
        ..default()
    };
    let status_font = TextFont {
        font: asset_server.load(UI_FONT_MEDIUM),
        font_size: 13.0,
        ..default()
    };

    commands
        .spawn((
            ToolsRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(160.0), // above parts_bin's 96px row + hud's 64px bar
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(SPACE[1]),
                padding: UiRect::all(Val::Px(SPACE[2])),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                StatusText,
                Text::new(""),
                status_font,
                TextColor(palette::INK_SECONDARY),
            ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(SPACE[2]),
                    ..default()
                })
                .with_children(|parent| {
                    tool_button(parent, &font, DeleteButton, "Delete");
                    tool_button(parent, &font, RotateButton, "Rotate");
                    tool_button(parent, &font, ConnectButton(ConnectionKind::Rope), "Rope");
                    tool_button(parent, &font, ConnectButton(ConnectionKind::Belt), "Belt");
                    tool_button(parent, &font, ConnectButton(ConnectionKind::Wire), "Wire");
                    save_name_field(parent, &font);
                    tool_button(parent, &font, SaveButton, "Save");
                    tool_button(parent, &font, LoadButton, "Load");
                    tool_button(parent, &font, SandboxButton, "Sandbox");
                });
        });
}

fn despawn_editor_tools(
    mut commands: Commands,
    roots: Query<Entity, With<ToolsRoot>>,
    menus: Query<Entity, With<LoadMenuRoot>>,
    mut menu_open: ResMut<LoadMenuOpen>,
    mut focus: ResMut<TextInputFocus>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    for entity in &menus {
        commands.entity(entity).despawn();
    }
    menu_open.0 = false;
    focus.0 = false;
}

fn delete_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<DeleteButton>)>,
    mut actions: ResMut<EditorActionRequest>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            actions.delete_selected = true;
        }
    }
}

fn rotate_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<RotateButton>)>,
    mut actions: ResMut<EditorActionRequest>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            actions.rotate_selected = true;
        }
    }
}

/// Pressing the already-active connect mode's own button turns it back
/// off (a toggle, not a one-way switch) — otherwise the only way out of
/// connect mode would be picking a different kind, with no way back to
/// plain drag/place.
fn connect_button_interaction(
    interactions: Query<(&Interaction, &ConnectButton), Changed<Interaction>>,
    mut connect: ResMut<ConnectToolRequest>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            connect.kind = if connect.kind == Some(button.0) {
                None
            } else {
                Some(button.0)
            };
        }
    }
}

/// Highlights whichever `ConnectButton` matches the live
/// `ConnectToolRequest.kind`, if any — the only visual feedback this
/// first pass gives for "connect mode is on and waiting for two taps".
fn sync_connect_button_visuals(
    connect: Res<ConnectToolRequest>,
    mut buttons: Query<(&ConnectButton, &mut BackgroundColor)>,
) {
    for (button, mut background) in &mut buttons {
        background.0 = if connect.kind == Some(button.0) {
            palette::ACCENT_BRAND
        } else {
            palette::BG_PANEL
        };
    }
}

/// Uses whatever's typed into [`SaveNameField`] (sanitized — see this
/// module's top doc comment) as the file name, falling back to the
/// level's own id (the old fixed-slot behavior) if nothing valid was
/// typed — pressing Save with an empty name field still does something
/// sensible rather than failing.
fn save_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    editor_state: Res<EditorState>,
    name_buffer: Res<SaveNameBuffer>,
    mut status: ResMut<StatusMessage>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let dir = save_load::default_saves_dir();
        let file_stem = save_load::sanitize_file_stem(&name_buffer.0)
            .unwrap_or_else(|| editor_state.level.id.clone());
        status.0 = Some(
            match save_load::save_level(&editor_state.level, &dir, &file_stem) {
                Ok(path) => format!("Saved to {}", path.display()),
                Err(e) => format!("Save failed: {e}"),
            },
        );
    }
}

/// Opens/closes the "My Levels" list rather than loading one fixed slot
/// directly (see this module's top doc comment) — `sync_load_menu` does
/// the actual spawning, since it needs `AssetServer` for each row's text.
fn load_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut menu_open: ResMut<LoadMenuOpen>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            menu_open.0 = !menu_open.0;
        }
    }
}

/// Loads whichever save a player picked from the "My Levels" list —
/// `row.0` comes straight from `save_load::list_saved_levels`'s own
/// listing of real files already on disk, not further free-typed input,
/// so (unlike [`SaveNameField`]'s text) nothing here needs sanitizing on
/// the way in.
fn saved_level_row_interaction(
    interactions: Query<(&Interaction, &SavedLevelRow), Changed<Interaction>>,
    mut editor_state: ResMut<EditorState>,
    mut rebuild: ResMut<ForceRebuildRequest>,
    mut status: ResMut<StatusMessage>,
    mut menu_open: ResMut<LoadMenuOpen>,
) {
    for (interaction, row) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match save_load::load_level(&row.0) {
            Ok(level) => {
                editor_state.level = level;
                rebuild.0 = true;
                status.0 = Some(format!("Loaded {}", row.0.display()));
            }
            Err(e) => status.0 = Some(format!("Load failed: {e}")),
        }
        menu_open.0 = false;
    }
}

fn save_name_field_focus_system(
    interactions: Query<&Interaction, (Changed<Interaction>, With<SaveNameField>)>,
    mut focus: ResMut<TextInputFocus>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            focus.0 = true;
        }
    }
}

/// Clicking any of this row's other buttons while the name field is
/// focused unfocuses it first — the same "click away to dismiss" a
/// player already expects from any other app's text field, so a
/// forgotten Enter press doesn't leave Space/R/N/P/B silently dead.
fn defocus_save_name_on_other_button_press(
    interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            Without<SaveNameField>,
            Or<(
                With<DeleteButton>,
                With<RotateButton>,
                With<ConnectButton>,
                With<SandboxButton>,
            )>,
        ),
    >,
    mut focus: ResMut<TextInputFocus>,
) {
    if interactions.iter().any(|i| *i == Interaction::Pressed) {
        focus.0 = false;
    }
}

/// Only allow a safe filename charset to ever land in [`SaveNameBuffer`]
/// in the first place — see this module's top doc comment on why an
/// unfiltered typed name is a real path-traversal risk, not just messy
/// input.
fn is_allowed_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == ' ' || c == '_' || c == '-'
}

/// Drives [`SaveNameBuffer`] from real keystrokes while [`SaveNameField`]
/// is focused. `EventReader::clear()` while unfocused only resets *this
/// reader's* cursor, not the shared `Events<KeyboardInput>` queue — it
/// doesn't stop `app.rs`'s `ButtonInput<KeyCode>` (a separate reader)
/// from seeing the same keys, and it exists so that keystrokes typed
/// before this field gained focus (e.g. the "B" that just opened
/// Sandbox) don't get replayed into the buffer the moment it does.
fn save_name_keyboard_system(
    mut events: EventReader<KeyboardInput>,
    mut focus: ResMut<TextInputFocus>,
    mut buffer: ResMut<SaveNameBuffer>,
) {
    if !focus.0 {
        events.clear();
        return;
    }
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(s) => {
                for c in s.chars().filter(|c| is_allowed_name_char(*c)) {
                    if buffer.0.chars().count() < save_load::MAX_SAVE_NAME_LEN {
                        buffer.0.push(c);
                    }
                }
            }
            // The space bar comes through as this distinct named variant,
            // not `Key::Character(" ")` — easy to miss since every other
            // allowed character above is a `Character`.
            Key::Space => {
                if buffer.0.chars().count() < save_load::MAX_SAVE_NAME_LEN {
                    buffer.0.push(' ');
                }
            }
            Key::Backspace => {
                buffer.0.pop();
            }
            Key::Enter | Key::Escape => focus.0 = false,
            _ => {}
        }
    }
}

/// Headless-safe (mutates an existing `Text`/`BackgroundColor`, doesn't
/// spawn anything, so no `AssetServer` dependency) — shows the typed
/// name plus a trailing `_` cursor while focused, the placeholder text
/// while empty and unfocused, and highlights the field's background
/// while it has focus.
fn sync_save_name_text(
    buffer: Res<SaveNameBuffer>,
    focus: Res<TextInputFocus>,
    mut texts: Query<&mut Text, With<SaveNameText>>,
    mut fields: Query<&mut BackgroundColor, With<SaveNameField>>,
) {
    for mut text in &mut texts {
        text.0 = if focus.0 {
            format!("{}_", buffer.0)
        } else if buffer.0.is_empty() {
            "(level name)".to_string()
        } else {
            buffer.0.clone()
        };
    }
    for mut background in &mut fields {
        background.0 = if focus.0 { palette::ACCENT_BRAND } else { palette::BG_PANEL };
    }
}

fn sandbox_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<SandboxButton>)>,
    mut editor_state: ResMut<EditorState>,
    mut next_state: ResMut<NextState<crate::game_state::GameState>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            enter_sandbox(&mut editor_state, &mut next_state);
        }
    }
}

fn sync_status_text(mut status: ResMut<StatusMessage>, mut texts: Query<&mut Text, With<StatusText>>) {
    let Some(message) = status.0.take() else {
        return;
    };
    for mut text in &mut texts {
        text.0 = message.clone();
    }
}

/// Just the button-interaction/status systems, no UI spawn/despawn — no
/// `AssetServer` dependency, so headless tests (`tests/ui_editor_tools.rs`,
/// same reasoning as `input::EditorInputPlugin` vs. `PointerAdapterPlugin`)
/// can add this alone, spawn a bare `(DeleteButton, Interaction)` etc.
/// pair directly (mirroring what bevy_ui's real picking backend would
/// produce), and exercise the exact same logic the real button row uses.
pub struct EditorToolsLogicPlugin;

impl Plugin for EditorToolsLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusMessage>()
            .init_resource::<TextInputFocus>()
            .init_resource::<SaveNameBuffer>()
            .init_resource::<LoadMenuOpen>()
            // `save_name_keyboard_system`'s `EventReader<KeyboardInput>`
            // needs `Events<KeyboardInput>` to exist at all, which the
            // real app already gets from `DefaultPlugins`' `InputPlugin`
            // — registering it again here is a no-op there, but it's
            // what makes this plugin (and the headless tests that use it
            // directly, without `DefaultPlugins`) safe on its own.
            .add_event::<KeyboardInput>()
            .add_systems(
                Update,
                (
                    delete_button_interaction,
                    rotate_button_interaction,
                    connect_button_interaction,
                    sync_connect_button_visuals,
                    save_name_field_focus_system,
                    defocus_save_name_on_other_button_press,
                    save_name_keyboard_system,
                    sync_save_name_text,
                    save_button_interaction,
                    load_button_interaction,
                    saved_level_row_interaction,
                    sandbox_button_interaction,
                    sync_status_text,
                ),
            );
    }
}

/// Spawns/despawns the "My Levels" list (see this module's top doc
/// comment) to track [`LoadMenuOpen`] — real-UI-only (needs
/// `AssetServer` for each row's text), so it's added here rather than in
/// [`EditorToolsLogicPlugin`]; headless tests spawn `SavedLevelRow`
/// directly instead (mirroring `DeleteButton`/`SaveButton`) to exercise
/// [`saved_level_row_interaction`] without this system ever running.
/// Idempotent rather than change-detected: cheap to run every frame,
/// and sidesteps any question of whether it runs before or after
/// whatever system flipped `LoadMenuOpen` in the same tick.
fn sync_load_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_open: Res<LoadMenuOpen>,
    existing: Query<Entity, With<LoadMenuRoot>>,
) {
    let already_open = !existing.is_empty();
    if menu_open.0 == already_open {
        return;
    }
    if !menu_open.0 {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }

    let font = TextFont {
        font: asset_server.load(UI_FONT_MEDIUM),
        font_size: 14.0,
        ..default()
    };
    let saves = save_load::list_saved_levels(&save_load::default_saves_dir());

    commands
        .spawn((
            LoadMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(210.0), // just above the tools row itself
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(SPACE[1]),
                padding: UiRect::all(Val::Px(SPACE[2])),
                ..default()
            },
            BackgroundColor(palette::BG_CANVAS),
        ))
        .with_children(|parent| {
            if saves.is_empty() {
                parent.spawn((
                    Text::new("No saves yet."),
                    font.clone(),
                    TextColor(palette::INK_SECONDARY),
                ));
                return;
            }
            for path in saves {
                let label =
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("(unknown)").to_string();
                parent
                    .spawn((
                        SavedLevelRow(path),
                        Button,
                        Node {
                            min_width: Val::Px(160.0),
                            height: Val::Px(32.0),
                            padding: UiRect::axes(Val::Px(SPACE[3]), Val::Px(SPACE[1])),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(palette::BG_PANEL),
                        BorderRadius::all(Val::Px(radius::SM)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(label),
                            font.clone(),
                            TextColor(palette::INK_PRIMARY),
                        ));
                    });
            }
        });
}

/// [`EditorToolsLogicPlugin`] plus the real bevy_ui chrome (spawn/despawn
/// on every `OnEnter(GameState::Edit)`, same as `hud`/`parts_bin`, plus
/// [`sync_load_menu`]) — what the real app actually runs; see
/// [`EditorToolsLogicPlugin`]'s own docs for why tests use that one
/// directly instead.
pub struct EditorToolsPlugin;

impl Plugin for EditorToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorToolsLogicPlugin)
            .add_systems(
                OnEnter(crate::game_state::GameState::Edit),
                (despawn_editor_tools, spawn_editor_tools).chain(),
            )
            .add_systems(Update, sync_load_menu);
    }
}
