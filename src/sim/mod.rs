//! The simulation core (docs/GDD.md §2, §5.5): wires `bevy_rapier2d`'s
//! physics step into Bevy's `FixedUpdate` schedule (its built-in
//! accumulator already implements the fixed-timestep-with-catch-up pattern
//! docs/GDD.md §2.4 describes) and fixes the system ordering determinism
//! depends on.
//!
//! The pre/post-physics [`SimSet`] variants are wired into the schedule now
//! but mostly empty until M2+ adds real systems to them — establishing the
//! *order* is the actual "sim core" deliverable of M1. Never add a system
//! that touches physics state without putting it in the right `SimSet`.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::energy_graph::EnergyGraphPlugin;
use crate::gear_train::GearTrainPlugin;
use crate::math::PIXELS_PER_METER;
use crate::rope_network::RopeNetworkPlugin;

pub mod body_factory;

/// Fixed physics timestep (docs/GDD.md §2.2). Simulation/graph code must
/// never read a real frame delta instead of this — see CLAUDE.md rule 2.
pub const FIXED_DT: f32 = 1.0 / 120.0;

/// The exact system ordering docs/GDD.md §2.4 requires, every tick:
/// 1. energy → 2. thermal → 3. fields → 4. ropes → 5. gears →
/// 6. the physics step itself (`PhysicsSet`, from `bevy_rapier2d`) →
/// 7. collision routing → 8. win/fail conditions.
/// Future milestones attach systems to these sets instead of ordering ad
/// hoc — that discipline is what keeps the simulation deterministic as
/// content grows.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// 1. חשמל → אור/חום/סיבוב (טופולוגי) — `energy_graph.rs` (M4)
    EnergyPropagate,
    /// 2. התקדמות פתילים, הצתות — `atmosphere.rs` (M4+)
    ThermalUpdate,
    /// 3. רוח / ואקום / מגנט — `atmosphere.rs` (M4+)
    FieldForces,
    /// 4. אילוצי חבלים — `rope_network.rs` (M4)
    RopeTension,
    /// 5. מומנטים למפרקים מנועיים — `gear_train.rs` (M4)
    GearTorques,
    /// 7. תרגום אירועי מגע ל-IMPACT — `sim/collision_router.rs` (M2+)
    CollisionRouting,
    /// 8. בדיקת ניצחון/כישלון — `win_conditions.rs` (M3)
    WinConditions,
}

/// Adds `bevy_rapier2d` (in the fixed schedule, for determinism) and locks
/// in the [`SimSet`] ordering. Both `src/main.rs` and integration tests add
/// this plugin so they get an identically-configured deterministic sim.
pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        // Order matters: RapierPhysicsPlugin::build() calls `init_resource::<TimestepMode>()`
        // (a no-op if already present) and immediately warns if it's not `Fixed` — so our
        // insert must happen *before* `add_plugins`, not after, or the plugin logs a false
        // "not Fixed" warning against its own just-inserted default before we override it.
        app.insert_resource(Time::<Fixed>::from_seconds(FIXED_DT as f64))
            .insert_resource(TimestepMode::Fixed {
                dt: FIXED_DT,
                substeps: 1,
            })
            .add_plugins(
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER)
                    .in_fixed_schedule(),
            )
            .add_plugins((EnergyGraphPlugin, GearTrainPlugin, RopeNetworkPlugin))
            .configure_sets(
                FixedUpdate,
                (
                    SimSet::EnergyPropagate,
                    SimSet::ThermalUpdate,
                    SimSet::FieldForces,
                    SimSet::RopeTension,
                    SimSet::GearTorques,
                    PhysicsSet::SyncBackend,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Writeback,
                    SimSet::CollisionRouting,
                    SimSet::WinConditions,
                )
                    .chain(),
            );
    }
}
