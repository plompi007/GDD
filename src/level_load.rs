//! `LevelFile` → spawned entities (docs/GDD.md §5.5, mirroring OpenTIM's
//! `level_load.rs` responsibility). `EditorState` is the single source of
//! truth (docs/GDD.md §2.5): reset never rewinds physics, it despawns
//! everything tagged [`LevelEntity`] and rebuilds from `EditorState` fresh.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_rapier2d::prelude::RapierConfiguration;

use crate::energy_graph::EnergyGraph;
use crate::game_state::GameState;
use crate::level_file_format::{Connection, ConnectionKind, LevelFile, PlacedPart};
use crate::part::{Offset, PartDef};
use crate::parts::{attach_part_behavior, PartRegistry};
use crate::rope_network::RopeConnection;
use crate::sim::body_factory::spawn_part;

/// The placed-part `id` from the level JSON, so win conditions can look up
/// "the entity called `bucket`" etc.
#[derive(Component, Debug, Clone)]
pub struct PlacedId(pub String);

/// The `tags` array from the level JSON (e.g. `["SUBJECT"]`).
#[derive(Component, Debug, Clone, Default)]
pub struct PartTags(pub Vec<String>);

impl PartTags {
    pub fn has(&self, tag: &str) -> bool {
        self.0.iter().any(|t| t == tag)
    }
}

/// Marks every entity spawned from a level, so reset can find and despawn
/// them all without needing to track handles anywhere else.
#[derive(Component)]
pub struct LevelEntity;

/// docs/GDD.md §2.5's `editorState` — the single source of truth for what's
/// placed where. Read-only once `GameState::Running` is entered (M5 will
/// enforce that when editing exists; for now nothing mutates it yet).
#[derive(Resource, Clone)]
pub struct EditorState {
    pub level: LevelFile,
}

pub(crate) fn spawn_placed_part(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    registry: &PartRegistry,
    placed: &PlacedPart,
) -> Entity {
    let def = registry.get(&placed.part_type).unwrap_or_else(|| {
        panic!(
            "level {:?} references unknown partType: {}",
            placed.id, placed.part_type
        )
    });
    let entity = spawn_part(
        commands,
        meshes,
        materials,
        def,
        Vec2::new(placed.x, placed.y),
        placed.rotation,
    );
    // `PartTags` is the union of this part *type*'s inherent tags
    // (data/parts/*.json's own "tags" — DESTRUCTIBLE, FLAMMABLE, POPPABLE,
    // SHARP, etc., used by generic tag-rule systems like
    // sim::collision_router and atmosphere's thermal tag rules) and this
    // *placed instance*'s own level-authored tags (SUBJECT, GOAL, etc.) —
    // the two are orthogonal, so callers never need to care which side a
    // tag came from.
    let tags = def
        .tags
        .iter()
        .cloned()
        .chain(placed.tags.iter().cloned())
        .collect();
    commands.entity(entity).insert((
        PlacedId(placed.id.clone()),
        PartTags(tags),
        LevelEntity,
    ));
    attach_part_behavior(commands, entity, def, placed);
    entity
}

/// The anchor offset a `Connection`'s `AnchorRef` points at (docs/GDD.md
/// §5.1 `anchorRef.anchorIdx` indexes into the part's own `anchors[]`).
/// Missing/out-of-range indexes fall back to the part's own origin rather
/// than panicking — a level author leaving `anchorIdx` at its schema
/// default (0) on a part with no declared anchors is a content mistake to
/// catch in level-authoring tooling (M8), not a reason to crash the sim.
pub(crate) fn anchor_offset(def: &PartDef, anchor_idx: u32) -> Offset {
    def.anchors
        .iter()
        .find(|a| a.idx == anchor_idx)
        .map(|a| a.offset)
        .unwrap_or(Offset { x: 0.0, y: 0.0 })
}

fn placed_world_anchor(placed: &PlacedPart, offset: Offset) -> Vec2 {
    let rotation = Quat::from_rotation_z(placed.rotation.to_radians());
    let world_offset = rotation * Vec3::new(offset.x, offset.y, 0.0);
    Vec2::new(placed.x, placed.y) + world_offset.truncate()
}

/// Resolves a `connections[].from`/`to`/`routedThrough` entry (docs/GDD.md
/// §5.1 `anchorRef`) against this level's already-spawned parts.
fn resolve_anchor_ref(
    id_to_entity: &BTreeMap<String, Entity>,
    placed_lookup: &BTreeMap<String, &PlacedPart>,
    registry: &PartRegistry,
    anchor_ref: &crate::level_file_format::AnchorRef,
) -> (Entity, Offset) {
    let entity = *id_to_entity.get(&anchor_ref.part_id).unwrap_or_else(|| {
        panic!(
            "connection references unknown placed part id: {}",
            anchor_ref.part_id
        )
    });
    let placed = placed_lookup[&anchor_ref.part_id];
    let def = registry.get(&placed.part_type).unwrap_or_else(|| {
        panic!(
            "connection references part {} of unknown partType: {}",
            anchor_ref.part_id, placed.part_type
        )
    });
    (entity, anchor_offset(def, anchor_ref.anchor_idx))
}

/// docs/GDD.md §5.1: a `ROPE`/`BELT` connection's `maxLength`, if omitted,
/// defaults to the initial distance × 1.05 — computed here straight from
/// placed-part data (before physics runs), never from live `Transform`s.
fn default_rope_length(
    placed_lookup: &BTreeMap<String, &PlacedPart>,
    end_a: &crate::level_file_format::AnchorRef,
    routed_through: &[crate::level_file_format::AnchorRef],
    end_b: &crate::level_file_format::AnchorRef,
    registry: &PartRegistry,
) -> f32 {
    let world = |anchor_ref: &crate::level_file_format::AnchorRef| -> Vec2 {
        let placed = placed_lookup[&anchor_ref.part_id];
        let def = registry.get(&placed.part_type).expect("resolved above");
        placed_world_anchor(placed, anchor_offset(def, anchor_ref.anchor_idx))
    };
    let mut points = Vec::with_capacity(routed_through.len() + 2);
    points.push(world(end_a));
    points.extend(routed_through.iter().map(world));
    points.push(world(end_b));
    let length: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    length * 1.05
}

fn spawn_connections(
    commands: &mut Commands,
    energy: &mut EnergyGraph,
    registry: &PartRegistry,
    id_to_entity: &BTreeMap<String, Entity>,
    placed_lookup: &BTreeMap<String, &PlacedPart>,
    connections: &[Connection],
) {
    for conn in connections {
        match conn.kind {
            ConnectionKind::Rope => {
                let end_a = resolve_anchor_ref(id_to_entity, placed_lookup, registry, &conn.from);
                let end_b = resolve_anchor_ref(id_to_entity, placed_lookup, registry, &conn.to);
                let pulleys: Vec<_> = conn
                    .routed_through
                    .iter()
                    .map(|r| resolve_anchor_ref(id_to_entity, placed_lookup, registry, r))
                    .collect();
                let max_length = conn.max_length.unwrap_or_else(|| {
                    default_rope_length(
                        placed_lookup,
                        &conn.from,
                        &conn.routed_through,
                        &conn.to,
                        registry,
                    )
                });
                commands.spawn((
                    RopeConnection {
                        end_a,
                        end_b,
                        pulleys,
                        max_length,
                    },
                    LevelEntity,
                ));
            }
            ConnectionKind::Wire => {
                let from_placed = placed_lookup[&conn.from.part_id];
                let from_def = registry.get(&from_placed.part_type).unwrap_or_else(|| {
                    panic!("unknown partType: {}", from_placed.part_type)
                });
                let to_placed = placed_lookup[&conn.to.part_id];
                let to_def = registry
                    .get(&to_placed.part_type)
                    .unwrap_or_else(|| panic!("unknown partType: {}", to_placed.part_type));
                let from_port = from_def
                    .ports
                    .get(conn.from.anchor_idx as usize)
                    .map(|p| p.id.as_str())
                    .unwrap_or("out");
                let to_port = to_def
                    .ports
                    .get(conn.to.anchor_idx as usize)
                    .map(|p| p.id.as_str())
                    .unwrap_or("in");
                energy.connect(&conn.from.part_id, from_port, &conn.to.part_id, to_port);
            }
            ConnectionKind::Belt => {
                let from = id_to_entity[&conn.from.part_id];
                let to = id_to_entity[&conn.to.part_id];
                commands.spawn((crate::gear_train::BeltConnection { from, to }, LevelEntity));
            }
        }
    }
}

/// docs/GDD.md §2.5: reset = despawn every level entity and rebuild from
/// `EditorState` from scratch. Also used for the *initial* spawn (run via
/// `OnEnter(GameState::Edit)`, which fires once on startup since `Edit` is
/// the default state) — "load a level" and "reset a level" are the same
/// operation on purpose.
pub fn reset_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    registry: Res<PartRegistry>,
    state: Res<EditorState>,
    mut energy: ResMut<EnergyGraph>,
    existing: Query<Entity, With<LevelEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    // docs/GDD.md §2.5: everything derived (physics, the energy graph's
    // wiring+signals) is torn down and rebuilt from `EditorState` on every
    // reset — never patched incrementally.
    energy.clear();

    let placed_parts: Vec<&PlacedPart> = state
        .level
        .fixed_parts
        .iter()
        .chain(state.level.preplaced_parts.iter())
        .collect();

    let mut id_to_entity = BTreeMap::new();
    let mut placed_lookup = BTreeMap::new();
    for placed in &placed_parts {
        let entity = spawn_placed_part(&mut commands, &mut meshes, &mut materials, &registry, placed);
        id_to_entity.insert(placed.id.clone(), entity);
        placed_lookup.insert(placed.id.clone(), *placed);
    }

    spawn_connections(
        &mut commands,
        &mut energy,
        &registry,
        &id_to_entity,
        &placed_lookup,
        &state.level.connections,
    );
}

/// Set by anything that replaces `EditorState.level` wholesale while
/// already in `GameState::Edit` (M9's Load button: reading a save back in
/// is the same "rebuild everything from `EditorState`" operation reset
/// already is, just not triggered by an actual state change) and needs
/// the same despawn+respawn `OnEnter(GameState::Edit)` already does on a
/// real Edit-to-Edit-via-something-else transition — which
/// `NextState::set(GameState::Edit)` would *not* trigger while already
/// in `Edit`, per Bevy's own "only fires on an actual value change"
/// state semantics.
#[derive(Resource, Default)]
pub struct ForceRebuildRequest(pub bool);

fn force_rebuild_requested(request: Res<ForceRebuildRequest>) -> bool {
    request.0
}

fn clear_force_rebuild_request(mut request: ResMut<ForceRebuildRequest>) {
    request.0 = false;
}

/// Keeps `bevy_rapier2d`'s own pause flag in sync with [`GameState`]
/// (docs/GDD.md §2.3): frozen in `Edit`/`Paused`/`Failed`, stepping in
/// `Running` (and `Solved`, which keeps running briefly for the win
/// celebration — see docs/GDD.md §2.3; the auto-freeze-after-2s timer
/// itself is a later polish-pass detail, not implemented yet).
fn sync_physics_active(
    state: Res<State<GameState>>,
    mut configs: Query<&mut RapierConfiguration>,
) {
    let active = matches!(state.get(), GameState::Running | GameState::Solved);
    for mut config in &mut configs {
        config.physics_pipeline_active = active;
    }
}

/// Registers [`GameState`], loads `level` into [`EditorState`], and wires
/// reset (`OnEnter(GameState::Edit)`, which fires once on startup too,
/// since `Edit` is the default state — "load" and "reset" are the same
/// operation on purpose, see docs/GDD.md §2.5).
///
/// Requires `StatesPlugin` (part of `DefaultPlugins`; add it explicitly
/// first if building a headless app on `MinimalPlugins`).
pub struct LevelPlugin {
    pub level: LevelFile,
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(EditorState {
                level: self.level.clone(),
            })
            .init_resource::<ForceRebuildRequest>()
            .add_systems(OnEnter(GameState::Edit), reset_level)
            .add_systems(
                Update,
                (
                    (reset_level, clear_force_rebuild_request)
                        .chain()
                        .run_if(force_rebuild_requested),
                    sync_physics_active,
                ),
            );
    }
}
