//! Desktop mouse + Android touch input (docs/GDD.md §3.2/§3.3/§3.6),
//! unified into one [`PointerState`] so the actual drag/drop/place logic
//! is a single code path for both platforms — matching docs/GDD.md's "one
//! input layer, mapped differently" framing.
//!
//! Editing only runs in `GameState::Edit` (CLAUDE.md rule 4: `EditorState`
//! is read-only once `Running`). Scope note: placing a new part from the
//! bin (`crate::ui::parts_bin::BinSlot`, M6's real bevy_ui chrome) and
//! repositioning an already-placed one is the mechanic that proves this
//! works identically on mouse and touch. M9/Sandbox adds the delete and
//! rotate mechanics ([`SelectedPart`]/[`EditorActionRequest`]) the same
//! way — a testable resource-driven action, with the real button/gesture
//! that sets it (§3.4) left as bevy_ui chrome for later, same as
//! `PointerState` itself versus its two real OS adapters below. `flip_x`/
//! `flip_y` stay unimplemented here on purpose: no shipped part has an
//! asymmetric collider, so flipping one currently has zero physical or
//! visual effect to test against — wiring a no-op action would be dead
//! code. The rope/belt/wire connect tool (§3.5) is [`ConnectToolRequest`]/
//! [`connect_tool_system`] below, same testable-resource pattern again —
//! it builds the exact same `RopeConnection`/`BeltConnection`/
//! `EnergyGraph` edge `level_load.rs`'s `spawn_connections` would from a
//! level file, just live, from two taps instead of from JSON at load
//! time. Camera pan/zoom (§3.6) remains separate, not touched here.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::energy_graph::EnergyGraph;
use crate::game_state::GameState;
use crate::gear_train::BeltConnection;
use crate::level_file_format::{AnchorRef, Connection, ConnectionKind, PlacedPart};
use crate::level_load::{anchor_offset, spawn_placed_part, EditorState, LevelEntity, PlacedId};
use crate::parts::PartRegistry;
use crate::rope_network::{world_anchor, RopeConnection};
use crate::ui::parts_bin::BinSlot;

/// docs/GDD.md §3.3 priority 4 ("Grid snap, always") — the one snap tier
/// this milestone implements; anchor/mesh/surface snap priorities 1-3 are
/// UX polish layered on top later, not required for placement to work.
const GRID_SNAP: f32 = 16.0;

/// Good enough for M5's "can I grab this part" check across every P0
/// part's collider; exact per-shape picking is a later refinement.
const PICK_RADIUS: f32 = 28.0;

/// The single input source every editing system reads — written each
/// frame by [`update_pointer_from_mouse`]/[`update_pointer_from_touch`] in
/// the real app. Headless tests write this directly instead, exercising
/// the exact same placement logic without a real window or OS input.
#[derive(Resource, Default)]
pub struct PointerState {
    pub world_pos: Option<Vec2>,
    pub just_pressed: bool,
    pub pressed: bool,
    pub just_released: bool,
}

fn grid_snap(pos: Vec2) -> Vec2 {
    (pos / GRID_SNAP).round() * GRID_SNAP
}

fn touch_is_active(touches: Res<Touches>) -> bool {
    touches.iter().next().is_some() || touches.iter_just_released().next().is_some()
}

/// Single-touch drag only (docs/GDD.md §3.4's two-finger free-rotate is
/// out of scope this milestone) — the first active touch, if any.
fn update_pointer_from_touch(touches: Res<Touches>, mut pointer: ResMut<PointerState>) {
    if let Some(touch) = touches.iter().next() {
        pointer.world_pos = Some(touch.position());
        pointer.pressed = true;
        pointer.just_pressed = touches.iter_just_pressed().next().is_some();
        pointer.just_released = false;
        return;
    }
    pointer.pressed = false;
    pointer.just_pressed = false;
    pointer.just_released = touches.iter_just_released().next().is_some();
}

/// Gated to skip whenever a touch is active (see [`touch_is_active`]) so a
/// touch device's synthesized mouse events can't fight the real touch for
/// control of the drag.
fn update_pointer_from_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut pointer: ResMut<PointerState>,
) {
    pointer.pressed = buttons.pressed(MouseButton::Left);
    pointer.just_pressed = buttons.just_pressed(MouseButton::Left);
    pointer.just_released = buttons.just_released(MouseButton::Left);

    pointer.world_pos = windows.single().ok().and_then(|window| {
        let cursor = window.cursor_position()?;
        let (camera, transform) = cameras.single().ok()?;
        camera.viewport_to_world_2d(transform, cursor).ok()
    });
}

#[derive(Component)]
struct PlacedGhost;

enum DragSource {
    FromBin { part_type: String, bin_index: usize },
    ExistingPart { entity: Entity },
}

#[derive(Resource, Default)]
struct DragState(Option<DragSource>);

/// The part a delete/rotate action (below) applies to — set whenever a
/// press picks up an existing placed part (same hit-test as starting a
/// drag), cleared on a press that hits nothing. Never a fixed part:
/// `hit_test_placed_part` already excludes those.
#[derive(Resource, Default)]
pub struct SelectedPart(pub Option<Entity>);

/// One-shot flags for the M9/Sandbox editor actions that don't fit the
/// press/drag/release gesture above (docs/GDD.md §3.4, M6 debt) — a real
/// delete/rotate button is bevy_ui chrome for later; this is the testable
/// mechanic underneath it, following the exact pattern `PointerState`
/// already established for headless tests to drive directly.
#[derive(Resource, Default)]
pub struct EditorActionRequest {
    pub delete_selected: bool,
    pub rotate_selected: bool,
}

/// Drives the rope/belt/wire connect tool (docs/GDD.md §3.5, M6 debt): set
/// `kind` to start (e.g. from a future "connect: rope/belt/wire" button —
/// bevy_ui chrome for later, same as everything else in this file), then
/// tap two already-placed parts. `pending_first` is `connect_tool_system`'s
/// own bookkeeping between the two taps, not something a caller sets.
#[derive(Resource, Default)]
pub struct ConnectToolRequest {
    pub kind: Option<ConnectionKind>,
    pending_first: Option<Entity>,
}

fn hit_test_placed_part(
    pointer_pos: Vec2,
    state: &EditorState,
    placed: &Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
) -> Option<Entity> {
    placed
        .iter()
        .filter(|(_, id, _)| {
            // Fixed parts (docs/GDD.md §5.1 `fixedParts`) are level furniture,
            // never player-movable.
            !state.level.fixed_parts.iter().any(|p| p.id == id.0)
        })
        .find(|(_, _, transform)| {
            transform.translation.truncate().distance(pointer_pos) < PICK_RADIUS
        })
        .map(|(entity, _, _)| entity)
}

/// Like [`hit_test_placed_part`], but includes `fixedParts` too — the
/// connect tool wires ports/anchors, which fixed furniture (a level's own
/// `motor_electric`, say) has exactly as much as a movable part does,
/// unlike drag/delete/rotate which only make sense for something the
/// player actually placed.
fn hit_test_any_part(
    pointer_pos: Vec2,
    placed: &Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
) -> Option<Entity> {
    placed
        .iter()
        .find(|(_, _, transform)| {
            transform.translation.truncate().distance(pointer_pos) < PICK_RADIUS
        })
        .map(|(entity, _, _)| entity)
}

/// `id` + `partType` for an already-placed entity, looked up through
/// `EditorState` (the single source of truth) rather than any live
/// per-entity part-type component, since none exists — see this module's
/// own doc comment on why `EditorState.level.preplaced_parts` already has
/// to be kept in sync for every other editor action.
fn placed_id_and_type(
    entity: Entity,
    state: &EditorState,
    placed: &Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
) -> Option<(String, String)> {
    let (_, id, _) = placed.iter().find(|(e, _, _)| *e == entity)?;
    let part_type = state
        .level
        .fixed_parts
        .iter()
        .chain(state.level.preplaced_parts.iter())
        .find(|p| p.id == id.0)
        .map(|p| p.part_type.clone())?;
    Some((id.0.clone(), part_type))
}

/// A press on a real `ui::parts_bin::BinSlot` button starts a from-bin
/// drag — read from bevy_ui's own `Interaction` (driven by its picking
/// backend against the real cursor/touch), not `PointerState`: once the
/// drag leaves the button's screen bounds, `Interaction` stops tracking
/// it, which is exactly why the rest of the drag (below) runs on
/// `PointerState` instead.
fn start_bin_drag_system(
    mut drag: ResMut<DragState>,
    pointer: Res<PointerState>,
    connect: Res<ConnectToolRequest>,
    editor_state: Res<EditorState>,
    registry: Res<PartRegistry>,
    mut commands: Commands,
    bin_slots: Query<(&BinSlot, &Interaction), Changed<Interaction>>,
) {
    if drag.0.is_some() || connect.kind.is_some() {
        return;
    }
    let Some((slot, _)) = bin_slots
        .iter()
        .find(|(_, interaction)| **interaction == Interaction::Pressed)
    else {
        return;
    };
    let can_place = editor_state
        .level
        .parts_bin
        .get(slot.bin_index)
        .map(|entry| entry.unlimited || entry.count > 0)
        .unwrap_or(false);
    if !can_place {
        return;
    }
    if let (Some(def), Some(world_pos)) = (registry.get(&slot.part_type), pointer.world_pos) {
        commands.spawn((
            PlacedGhost,
            Transform::from_translation(world_pos.extend(60.0)),
            Sprite::from_color(
                crate::sim::body_factory::placeholder_color(def).with_alpha(0.6),
                Vec2::splat(32.0),
            ),
        ));
    }
    drag.0 = Some(DragSource::FromBin {
        part_type: slot.part_type.clone(),
        bin_index: slot.bin_index,
    });
}

fn start_existing_part_drag_system(
    pointer: Res<PointerState>,
    mut drag: ResMut<DragState>,
    mut selected: ResMut<SelectedPart>,
    connect: Res<ConnectToolRequest>,
    editor_state: Res<EditorState>,
    placed: Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
) {
    if !pointer.just_pressed || drag.0.is_some() || connect.kind.is_some() {
        return;
    }
    let Some(world_pos) = pointer.world_pos else {
        return;
    };
    let hit = hit_test_placed_part(world_pos, &editor_state, &placed);
    // A press on empty space deselects — same gesture a real editor uses,
    // and it means [`apply_editor_actions_system`] never acts on a stale
    // selection from a part the player has since clicked away from.
    selected.0 = hit;
    if let Some(entity) = hit {
        drag.0 = Some(DragSource::ExistingPart { entity });
    }
}

/// Applies a pending delete/rotate request (set on [`EditorActionRequest`],
/// e.g. by a future delete/rotate button — see that resource's own docs)
/// to whatever [`SelectedPart`] currently holds, then clears both the
/// request and (for delete) the selection. Runs after the drag systems so
/// a delete can't race an in-progress drag of the same entity.
fn apply_editor_actions_system(
    mut actions: ResMut<EditorActionRequest>,
    mut selected: ResMut<SelectedPart>,
    mut editor_state: ResMut<EditorState>,
    registry: Res<PartRegistry>,
    mut commands: Commands,
    mut placed: Query<(&PlacedId, &mut Transform), With<LevelEntity>>,
) {
    let delete_requested = std::mem::take(&mut actions.delete_selected);
    let rotate_requested = std::mem::take(&mut actions.rotate_selected);
    if !delete_requested && !rotate_requested {
        return;
    }
    let Some(entity) = selected.0 else {
        return;
    };
    let Ok((id, mut transform)) = placed.get_mut(entity) else {
        selected.0 = None;
        return;
    };

    if delete_requested {
        let id = id.0.clone();
        editor_state
            .level
            .preplaced_parts
            .retain(|part| part.id != id);
        commands.entity(entity).despawn();
        selected.0 = None;
        return;
    }

    if rotate_requested {
        let id = id.0.clone();
        let Some(part) = editor_state
            .level
            .preplaced_parts
            .iter_mut()
            .find(|p| p.id == id)
        else {
            return;
        };
        let Some(def) = registry.get(&part.part_type) else {
            return;
        };
        if !def.editor.rotatable || def.editor.rotation_snap <= 0.0 {
            return;
        }
        part.rotation = (part.rotation + def.editor.rotation_snap) % 360.0;
        transform.rotation = Quat::from_rotation_z(part.rotation.to_radians());
    }
}

/// The first half of a rope/belt/wire connection — anchor 0 on both ends
/// (docs/GDD.md §5.1's `anchorRef.anchorIdx` default), matching the
/// simplest case `level_load.rs`'s own JSON-driven `spawn_connections`
/// already handles the same way. Picking a specific anchor on a
/// multi-anchor part is future UI polish, not needed for a first
/// functioning connect tool.
#[allow(clippy::too_many_arguments)]
fn connect_tool_system(
    pointer: Res<PointerState>,
    mut connect: ResMut<ConnectToolRequest>,
    mut editor_state: ResMut<EditorState>,
    registry: Res<PartRegistry>,
    mut energy: ResMut<EnergyGraph>,
    mut commands: Commands,
    mut counter: Local<u64>,
    placed: Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
    transforms: Query<&Transform>,
) {
    let Some(kind) = connect.kind else {
        return;
    };
    if !pointer.just_pressed {
        return;
    }
    let Some(world_pos) = pointer.world_pos else {
        return;
    };
    let Some(entity) = hit_test_any_part(world_pos, &placed) else {
        return;
    };

    let Some(first) = connect.pending_first else {
        connect.pending_first = Some(entity);
        return;
    };
    if first == entity {
        return; // tapped the same part twice — not a connection to itself
    }
    connect.pending_first = None;

    let Some((id_a, type_a)) = placed_id_and_type(first, &editor_state, &placed) else {
        return;
    };
    let Some((id_b, type_b)) = placed_id_and_type(entity, &editor_state, &placed) else {
        return;
    };
    let (Some(def_a), Some(def_b)) = (registry.get(&type_a), registry.get(&type_b)) else {
        return;
    };

    *counter += 1;
    let connection = Connection {
        id: format!("connect_{}", *counter),
        kind,
        from: AnchorRef { part_id: id_a.clone(), anchor_idx: 0 },
        to: AnchorRef { part_id: id_b.clone(), anchor_idx: 0 },
        max_length: None,
        routed_through: Vec::new(),
    };

    match kind {
        ConnectionKind::Rope => {
            let end_a = (first, anchor_offset(def_a, 0));
            let end_b = (entity, anchor_offset(def_b, 0));
            let max_length = match (world_anchor(&transforms, end_a), world_anchor(&transforms, end_b)) {
                (Some(a), Some(b)) => a.distance(b) * 1.05,
                _ => return,
            };
            commands.spawn((
                RopeConnection {
                    end_a,
                    end_b,
                    pulleys: Vec::new(),
                    max_length,
                },
                LevelEntity,
            ));
            editor_state.level.connections.push(Connection {
                max_length: Some(max_length),
                ..connection
            });
        }
        ConnectionKind::Belt => {
            commands.spawn((BeltConnection { from: first, to: entity }, LevelEntity));
            editor_state.level.connections.push(connection);
        }
        ConnectionKind::Wire => {
            let from_port = def_a.ports.first().map(|p| p.id.as_str()).unwrap_or("out");
            let to_port = def_b.ports.first().map(|p| p.id.as_str()).unwrap_or("in");
            energy.connect(&id_a, from_port, &id_b, to_port);
            editor_state.level.connections.push(connection);
        }
    }
}

fn update_drag_system(
    pointer: Res<PointerState>,
    drag: Res<DragState>,
    mut placed: Query<&mut Transform, (With<LevelEntity>, Without<PlacedGhost>)>,
    mut ghosts: Query<&mut Transform, (With<PlacedGhost>, Without<LevelEntity>)>,
) {
    if !pointer.pressed {
        return;
    }
    let Some(world_pos) = pointer.world_pos else {
        return;
    };
    match &drag.0 {
        Some(DragSource::FromBin { .. }) => {
            for mut transform in &mut ghosts {
                transform.translation = grid_snap(world_pos).extend(60.0);
            }
        }
        Some(DragSource::ExistingPart { entity }) => {
            if let Ok(mut transform) = placed.get_mut(*entity) {
                let z = transform.translation.z;
                transform.translation = grid_snap(world_pos).extend(z);
            }
        }
        None => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn end_drag_system(
    pointer: Res<PointerState>,
    mut drag: ResMut<DragState>,
    mut editor_state: ResMut<EditorState>,
    registry: Res<PartRegistry>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut counter: Local<u64>,
    placed: Query<(&PlacedId, &Transform), With<LevelEntity>>,
    ghosts: Query<Entity, With<PlacedGhost>>,
) {
    if !pointer.just_released {
        return;
    }
    let Some(source) = drag.0.take() else {
        return;
    };

    for ghost in &ghosts {
        commands.entity(ghost).despawn();
    }

    match source {
        DragSource::FromBin { part_type, bin_index } => {
            let Some(world_pos) = pointer.world_pos else {
                return;
            };
            let snapped = grid_snap(world_pos);
            if let Some(entry) = editor_state.level.parts_bin.get_mut(bin_index) {
                if !entry.unlimited {
                    entry.count = entry.count.saturating_sub(1);
                }
            }
            *counter += 1;
            let new_part = PlacedPart {
                id: format!("placed_{}", *counter),
                part_type,
                x: snapped.x,
                y: snapped.y,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                scale: 1.0,
                tags: Vec::new(),
                params: Default::default(),
            };
            editor_state.level.preplaced_parts.push(new_part.clone());
            let entity =
                spawn_placed_part(&mut commands, &mut meshes, &mut materials, &registry, &new_part);
            // docs/GDD.md §3.9.6: placement gets a scale-bounce + a
            // category-colored flash, both before any text/UI confirms
            // the drop — see `juice`'s own docs on why this is
            // headless-safe rather than gated like the real-art swap.
            commands.entity(entity).insert(crate::juice::JuiceBounce::default());
            if let Some(def) = registry.get(&new_part.part_type) {
                crate::juice::spawn_flash(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    snapped,
                    crate::juice::FlashSpec {
                        color: crate::sim::body_factory::placeholder_color(def),
                        start_radius: 8.0,
                        end_radius: 36.0,
                        duration_ms: 160.0,
                    },
                );
            }
        }
        DragSource::ExistingPart { entity } => {
            // `update_drag_system` already grid-snapped this entity's live
            // Transform every frame while dragging — just mirror it back
            // into `EditorState` (CLAUDE.md rule 4's single source of truth).
            if let Ok((id, transform)) = placed.get(entity) {
                if let Some(part) = editor_state
                    .level
                    .preplaced_parts
                    .iter_mut()
                    .find(|p| p.id == id.0)
                {
                    part.x = transform.translation.x;
                    part.y = transform.translation.y;
                }
            }
        }
    }
}

/// Reads real OS mouse/touch input and writes [`PointerState`] from it.
/// Separate from [`EditorInputPlugin`] so headless tests can add just the
/// core plugin and write `PointerState` directly — with both plugins
/// combined (as the real app does, see `src/app.rs`), this would otherwise
/// overwrite a test's injected pointer with "nothing is pressed" every
/// frame before the core systems ever saw it.
pub struct PointerAdapterPlugin;

impl Plugin for PointerAdapterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerState>().add_systems(
            Update,
            (
                update_pointer_from_touch,
                update_pointer_from_mouse.run_if(not(touch_is_active)),
            )
                .chain(),
        );
    }
}

/// The core editing mechanic: bin buttons, drag/place/reposition — reads
/// only [`PointerState`], so it works identically whether that state came
/// from [`PointerAdapterPlugin`] (the real app) or was written directly
/// (a headless test exercising the exact same drag-drop logic).
pub struct EditorInputPlugin;

impl Plugin for EditorInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerState>()
            .init_resource::<DragState>()
            .init_resource::<SelectedPart>()
            .init_resource::<EditorActionRequest>()
            .init_resource::<ConnectToolRequest>()
            .add_systems(
                Update,
                (
                    start_bin_drag_system,
                    start_existing_part_drag_system,
                    update_drag_system,
                    end_drag_system,
                    apply_editor_actions_system,
                    connect_tool_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Edit)),
            );
    }
}
