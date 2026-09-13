//! `GearTrain` (docs/GDD.md §5.5, §1.3-c): couples `ROTARY` power between
//! parts. Two mechanisms, matching the catalog's own distinction:
//!
//! - **Gear meshing** — geometric, not topological: any two [`GearNode`]s
//!   within `r1 + r2 + MESH_EPSILON` of each other couple every tick, with
//!   direction inverted and the `r1/r2` ratio applied (docs/GDD.md §1.3-c
//!   `gear_small`/`gear_large`). No JSON wiring needed — just place them
//!   close together, same as the real thing.
//! - **Belts** — the opposite case: two distant `ROTARY` nodes linked
//!   explicitly via a `connections: [{kind: BELT}]` entry (`drive_belt`),
//!   1:1, no inversion.
//!
//! Runs in `SimSet::GearTorques` (docs/GDD.md §2.4 step 5), after
//! `EnergyPropagate` has already told each [`MotorSource`] whether it's
//! powered this tick.

use bevy::prelude::*;

use crate::level_load::PlacedId;
use crate::sim::SimSet;

/// A meshing point for rotary power transfer (docs/GDD.md's "Mesh snap —
/// teeth adjacent" note): `motor_electric`'s output shaft and every
/// `gear_small`/`gear_large`. `radius` is the part's own physical radius
/// (its `Collider::ball` radius) — reused as-is for the mesh-distance and
/// gear-ratio math, since a gear's teeth radius *is* its body radius here.
#[derive(Component, Clone, Copy)]
pub struct GearNode {
    pub radius: f32,
}

/// Extra clearance added to `r1 + r2` before two [`GearNode`]s are
/// considered meshed, so placement doesn't need to be pixel-perfect.
/// Matches the editor snap distance docs/GDD.md documents for gears.
pub const MESH_EPSILON: f32 = 6.0;

/// Marks a [`GearNode`] as an independent rotary source rather than a
/// passive follower — docs/GDD.md §1.3-d `motor_electric`: `ELECTRIC in →
/// ROTARY out`. `rpm`/`direction` are resolved once at spawn time from the
/// placed part's params (docs/GDD.md §5.1), falling back to the part
/// type's own declared default — see `parts::motor_electric::attach`.
#[derive(Component)]
pub struct MotorSource {
    pub rpm: f32,
    pub direction: f32,
    pub electric_in_port: String,
}

/// This tick's angular velocity (rad/s, signed — positive matches Bevy's
/// +z-out-of-screen/counter-clockwise convention), written by
/// [`gear_mesh_system`]/[`motor_source_system`]/[`belt_transfer_system`],
/// read by anything downstream (`conveyor`'s belt effect; later, render
/// spin — docs/GDD.md §3.9/M6.5).
#[derive(Component, Default, Clone, Copy)]
pub struct RotaryState {
    pub angular_velocity: f32,
}

/// A `drive_belt` (docs/GDD.md §1.3-c): connects two distant `ROTARY`
/// nodes 1:1, no inversion. Not a physical body — like `rope`, it's built
/// once at level load from a `connections: [{kind: BELT}]` entry and
/// tracked as its own lightweight (component-only) entity.
#[derive(Component)]
pub struct BeltConnection {
    pub from: Entity,
    pub to: Entity,
}

fn rpm_to_rad_per_sec(rpm: f32) -> f32 {
    rpm * std::f32::consts::TAU / 60.0
}

/// docs/GDD.md §1.3-d `motor_electric`: reads its own `ELECTRIC in` port
/// from the [`EnergyGraph`](crate::energy_graph::EnergyGraph); powered =
/// spin at its configured rpm/direction, unpowered = stopped.
fn motor_source_system(
    energy: Res<crate::energy_graph::EnergyGraph>,
    mut motors: Query<(&PlacedId, &MotorSource, &mut RotaryState)>,
) {
    use crate::energy_graph::EnergySignal;

    for (id, motor, mut state) in &mut motors {
        let powered = matches!(
            energy.read(&id.0, &motor.electric_in_port),
            Some(EnergySignal::Electric(true))
        );
        state.angular_velocity = if powered {
            rpm_to_rad_per_sec(motor.rpm) * motor.direction
        } else {
            0.0
        };
    }
}

/// Proximity-based gear meshing. Collects every [`GearNode`] into a `Vec`
/// sorted by placed-part `id` (CLAUDE.md rule 2: never rely on ECS/HashMap
/// iteration order for anything simulation-visible) and relaxes it a bounded
/// number of passes — enough for any P0-sized gear chain to fully settle
/// within one tick, same idea as `EnergyGraph::propagate`'s 2-pass rule but
/// sized to the actual node count since a gear chain can be longer than 2.
fn gear_mesh_system(mut nodes: Query<(Entity, &PlacedId, &Transform, &GearNode, &mut RotaryState)>) {
    let mut items: Vec<(Entity, String, Vec2, f32, f32, bool)> = nodes
        .iter()
        .map(|(entity, id, transform, gear, state)| {
            (
                entity,
                id.0.clone(),
                transform.translation.truncate(),
                gear.radius,
                state.angular_velocity,
                false, // "driven this pass" flag, set for pre-existing motor sources below
            )
        })
        .collect();
    items.sort_by(|a, b| a.1.cmp(&b.1));

    // A node already carrying a nonzero angular velocity (from
    // `motor_source_system`, which ran earlier this tick) is this tick's
    // drive source; everything else starts as an undriven follower.
    for item in &mut items {
        item.5 = item.4 != 0.0;
    }

    let passes = items.len().max(1);
    for _ in 0..passes {
        let mut changed = false;
        for i in 0..items.len() {
            if !items[i].5 {
                continue;
            }
            let (pos_i, radius_i, angvel_i) = (items[i].2, items[i].3, items[i].4);
            for j in 0..items.len() {
                if i == j || items[j].5 {
                    continue;
                }
                let (pos_j, radius_j) = (items[j].2, items[j].3);
                if pos_i.distance(pos_j) <= radius_i + radius_j + MESH_EPSILON {
                    items[j].4 = -angvel_i * (radius_i / radius_j);
                    items[j].5 = true;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }

    for (entity, _, _, _, angvel, _) in items {
        if let Ok((_, _, _, _, mut state)) = nodes.get_mut(entity) {
            state.angular_velocity = angvel;
        }
    }
}

/// `drive_belt`: 1:1, no inversion, no meshing distance check — an explicit
/// long-distance link (docs/GDD.md §1.3-c). Runs after [`gear_mesh_system`]
/// so a belt fed from the far end of a gear chain sees this tick's settled
/// value, not last tick's.
fn belt_transfer_system(
    belts: Query<&BeltConnection>,
    mut states: Query<&mut RotaryState>,
    sources: Query<(), With<MotorSource>>,
) {
    for belt in &belts {
        let Ok(from_state) = states.get(belt.from).copied() else {
            continue;
        };
        if sources.get(belt.to).is_ok() {
            continue; // never override an independently-driven node
        }
        if let Ok(mut to_state) = states.get_mut(belt.to) {
            to_state.angular_velocity = from_state.angular_velocity;
        }
    }
}

pub struct GearTrainPlugin;

impl Plugin for GearTrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (motor_source_system, gear_mesh_system, belt_transfer_system)
                .chain()
                .in_set(SimSet::GearTorques),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_gears(positions: &[(f32, f32)], radii: &[f32]) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::transform::TransformPlugin)
            .init_resource::<crate::energy_graph::EnergyGraph>()
            .add_systems(Update, (motor_source_system, gear_mesh_system).chain());
        for (i, (pos, radius)) in positions.iter().zip(radii).enumerate() {
            let mut entity = app.world_mut().spawn((
                PlacedId(format!("g{i}")),
                Transform::from_xyz(pos.0, pos.1, 0.0),
                GearNode { radius: *radius },
                RotaryState::default(),
            ));
            if i == 0 {
                entity.insert(MotorSource {
                    rpm: 60.0,
                    direction: 1.0,
                    electric_in_port: "in".into(),
                });
            }
        }
        app.world_mut()
            .resource_mut::<crate::energy_graph::EnergyGraph>()
            .emit("g0", "in", crate::energy_graph::EnergySignal::Electric(true));
        app
    }

    #[test]
    fn meshed_gears_invert_direction_and_scale_by_radius_ratio() {
        let mut app = app_with_gears(&[(0.0, 0.0), (30.0, 0.0)], &[10.0, 20.0]);
        app.update();

        let mut query = app.world_mut().query::<(&PlacedId, &RotaryState)>();
        let mut by_id: std::collections::HashMap<String, f32> = query
            .iter(app.world())
            .map(|(id, state)| (id.0.clone(), state.angular_velocity))
            .collect();
        let motor_av = by_id.remove("g0").unwrap();
        let follower_av = by_id.remove("g1").unwrap();

        assert!(motor_av > 0.0);
        assert!((follower_av - (-motor_av * 0.5)).abs() < 1e-4);
    }

    #[test]
    fn gears_too_far_apart_do_not_mesh() {
        let mut app = app_with_gears(&[(0.0, 0.0), (1000.0, 0.0)], &[10.0, 20.0]);
        app.update();

        let mut query = app.world_mut().query::<(&PlacedId, &RotaryState)>();
        let follower_av = query
            .iter(app.world())
            .find(|(id, _)| id.0 == "g1")
            .unwrap()
            .1
            .angular_velocity;
        assert_eq!(follower_av, 0.0);
    }
}
