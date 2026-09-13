//! `LevelFile` → spawned entities (docs/GDD.md §5.5, mirroring OpenTIM's
//! `level_load.rs` responsibility). `EditorState` is the single source of
//! truth (docs/GDD.md §2.5): reset never rewinds physics, it despawns
//! everything tagged [`LevelEntity`] and rebuilds from `EditorState` fresh.

use bevy::prelude::*;
use bevy_rapier2d::prelude::RapierConfiguration;

use crate::game_state::GameState;
use crate::level_file_format::{LevelFile, PlacedPart};
use crate::parts::PartRegistry;
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

fn spawn_placed_part(commands: &mut Commands, registry: &PartRegistry, placed: &PlacedPart) {
    let def = registry.get(&placed.part_type).unwrap_or_else(|| {
        panic!(
            "level {:?} references unknown partType: {}",
            placed.id, placed.part_type
        )
    });
    let entity = spawn_part(
        commands,
        def,
        Vec2::new(placed.x, placed.y),
        placed.rotation,
    );
    commands.entity(entity).insert((
        PlacedId(placed.id.clone()),
        PartTags(placed.tags.clone()),
        LevelEntity,
    ));
}

/// docs/GDD.md §2.5: reset = despawn every level entity and rebuild from
/// `EditorState` from scratch. Also used for the *initial* spawn (run via
/// `OnEnter(GameState::Edit)`, which fires once on startup since `Edit` is
/// the default state) — "load a level" and "reset a level" are the same
/// operation on purpose.
pub fn reset_level(
    mut commands: Commands,
    registry: Res<PartRegistry>,
    state: Res<EditorState>,
    existing: Query<Entity, With<LevelEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for placed in state
        .level
        .fixed_parts
        .iter()
        .chain(state.level.preplaced_parts.iter())
    {
        spawn_placed_part(&mut commands, &registry, placed);
    }
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
            .add_systems(OnEnter(GameState::Edit), reset_level)
            .add_systems(Update, sync_physics_active);
    }
}
