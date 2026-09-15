//! M9/Sandbox editor tool row (docs/GDD.md's Sandbox spec, §3.4/§3.5's
//! long-deferred UI chrome): the bevy_ui buttons that drive the
//! resource-based mechanics `input.rs` already implements and tests —
//! [`chainworks::input::EditorActionRequest`] (delete/rotate),
//! [`chainworks::input::ConnectToolRequest`] (rope/belt/wire), and
//! [`chainworks::save_load`] (save/load a real file). Sits directly above
//! `parts_bin`'s row, same styling tokens as `hud.rs`/`parts_bin.rs`.
//!
//! Scope note: Save/Load use a single fixed slot per level (named after
//! `EditorState.level.id`, in `save_load::default_saves_dir()`) — there's
//! no text-input widget in this UI stack yet to let a player name
//! multiple saves, and browsing several saved levels side by side is
//! `LevelSelect`'s job (still M6 debt, not this module's). One working
//! save slot is enough to prove the whole save → reload → same-map loop
//! end to end; a real "My Levels" list is a follow-up once `LevelSelect`
//! exists.

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
                    tool_button(parent, &font, SaveButton, "Save");
                    tool_button(parent, &font, LoadButton, "Load");
                    tool_button(parent, &font, SandboxButton, "Sandbox");
                });
        });
}

fn despawn_editor_tools(mut commands: Commands, roots: Query<Entity, With<ToolsRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
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

fn save_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    editor_state: Res<EditorState>,
    mut status: ResMut<StatusMessage>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let dir = save_load::default_saves_dir();
        status.0 = Some(
            match save_load::save_level(&editor_state.level, &dir, &editor_state.level.id) {
                Ok(path) => format!("Saved to {}", path.display()),
                Err(e) => format!("Save failed: {e}"),
            },
        );
    }
}

/// Loads back whatever [`save_button_interaction`] last wrote for this
/// same `EditorState.level.id` — see this module's own doc comment on
/// why that's one fixed slot rather than a chosen file for now.
fn load_button_interaction(
    interactions: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut editor_state: ResMut<EditorState>,
    mut rebuild: ResMut<ForceRebuildRequest>,
    mut status: ResMut<StatusMessage>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let dir = save_load::default_saves_dir();
        let path = dir.join(&editor_state.level.id).with_extension("json");
        match save_load::load_level(&path) {
            Ok(level) => {
                editor_state.level = level;
                rebuild.0 = true;
                status.0 = Some(format!("Loaded {}", path.display()));
            }
            Err(e) => status.0 = Some(format!("Load failed: {e}")),
        }
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
        app.init_resource::<StatusMessage>().add_systems(
            Update,
            (
                delete_button_interaction,
                rotate_button_interaction,
                connect_button_interaction,
                sync_connect_button_visuals,
                save_button_interaction,
                load_button_interaction,
                sandbox_button_interaction,
                sync_status_text,
            ),
        );
    }
}

/// [`EditorToolsLogicPlugin`] plus the real bevy_ui chrome (spawn/despawn
/// on every `OnEnter(GameState::Edit)`, same as `hud`/`parts_bin`) — what
/// the real app actually runs; see [`EditorToolsLogicPlugin`]'s own docs
/// for why tests use that one directly instead.
pub struct EditorToolsPlugin;

impl Plugin for EditorToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorToolsLogicPlugin).add_systems(
            OnEnter(crate::game_state::GameState::Edit),
            (despawn_editor_tools, spawn_editor_tools).chain(),
        );
    }
}
