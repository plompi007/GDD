//! Desktop mouse + Android touch input (docs/GDD.md §3.2/§3.3/§3.6),
//! unified into one [`PointerState`] so the actual drag/drop/place logic
//! is a single code path for both platforms — matching docs/GDD.md's "one
//! input layer, mapped differently" framing.
//!
//! Editing only runs in `GameState::Edit` (CLAUDE.md rule 4: `EditorState`
//! is read-only once `Running`). Scope note: rotate/flip/inspector/delete
//! (§3.4), the rope/belt connect tool (§3.5), and camera pan/zoom (§3.6)
//! are UI chrome or separate polish, not implemented here — placing a new
//! part from the bin (`crate::ui::parts_bin::BinSlot`, M6's real bevy_ui
//! chrome) and repositioning an already-placed one is the mechanic this
//! module proves works identically on mouse and touch.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game_state::GameState;
use crate::level_file_format::PlacedPart;
use crate::level_load::{spawn_placed_part, EditorState, LevelEntity, PlacedId};
use crate::parts::PartRegistry;
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

/// A press on a real `ui::parts_bin::BinSlot` button starts a from-bin
/// drag — read from bevy_ui's own `Interaction` (driven by its picking
/// backend against the real cursor/touch), not `PointerState`: once the
/// drag leaves the button's screen bounds, `Interaction` stops tracking
/// it, which is exactly why the rest of the drag (below) runs on
/// `PointerState` instead.
fn start_bin_drag_system(
    mut drag: ResMut<DragState>,
    pointer: Res<PointerState>,
    editor_state: Res<EditorState>,
    registry: Res<PartRegistry>,
    mut commands: Commands,
    bin_slots: Query<(&BinSlot, &Interaction), Changed<Interaction>>,
) {
    if drag.0.is_some() {
        return;
    }
    let Some((slot, _)) = bin_slots
        .iter()
        .find(|(_, interaction)| **interaction == Interaction::Pressed)
    else {
        return;
    };
    let available = editor_state
        .level
        .parts_bin
        .get(slot.bin_index)
        .map(|entry| entry.count)
        .unwrap_or(0);
    if available == 0 {
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
    editor_state: Res<EditorState>,
    placed: Query<(Entity, &PlacedId, &Transform), (With<LevelEntity>, Without<PlacedGhost>)>,
) {
    if !pointer.just_pressed || drag.0.is_some() {
        return;
    }
    let Some(world_pos) = pointer.world_pos else {
        return;
    };
    if let Some(entity) = hit_test_placed_part(world_pos, &editor_state, &placed) {
        drag.0 = Some(DragSource::ExistingPart { entity });
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

fn end_drag_system(
    pointer: Res<PointerState>,
    mut drag: ResMut<DragState>,
    mut editor_state: ResMut<EditorState>,
    registry: Res<PartRegistry>,
    mut commands: Commands,
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
                entry.count = entry.count.saturating_sub(1);
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
            spawn_placed_part(&mut commands, &registry, &new_part);
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
            .add_systems(
                Update,
                (
                    start_bin_drag_system,
                    start_existing_part_drag_system,
                    update_drag_system,
                    end_drag_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Edit)),
            );
    }
}
