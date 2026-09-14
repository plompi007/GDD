//! `cutter_shears` (docs/GDD.md §1.3-ח/A09): `IMPACT/ELECTRIC → cuts a
//! nearby rope and pops nearby POPPABLE parts`. Fires once (like
//! `charge_barrel`'s single detonation) the first tick it's triggered —
//! either something touches its sensor collider (IMPACT) or its wired
//! `ELECTRIC in` port goes live.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::energy_graph::{EnergyGraph, EnergySignal};
use crate::level_file_format::PlacedPart;
use crate::level_load::{PartTags, PlacedId};
use crate::part::PartDef;
use crate::rope_network::{rope_world_points, RopeConnection};
use crate::sim::SimSet;

#[derive(Component)]
pub struct CutterShears {
    pub cut_radius: f32,
    pub electric_in_port: String,
    pub triggered: bool,
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, _placed: &PlacedPart) {
    let electric_in_port = def
        .ports
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "in".to_string());

    commands.entity(entity).insert(CutterShears {
        cut_radius: def.cut_radius.unwrap_or(70.0),
        electric_in_port,
        triggered: false,
    });
}

fn point_segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-6 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

#[allow(clippy::too_many_arguments)]
fn cutter_shears_system(
    mut commands: Commands,
    energy: Res<EnergyGraph>,
    mut cutters: Query<(&PlacedId, &Transform, &CollidingEntities, &mut CutterShears)>,
    ropes: Query<(Entity, &RopeConnection)>,
    transforms: Query<&Transform>,
    poppable: Query<(Entity, &Transform, &PartTags)>,
) {
    for (id, transform, colliding, mut cutter) in &mut cutters {
        if cutter.triggered {
            continue;
        }
        let impacted = !colliding.is_empty();
        let powered = matches!(
            energy.read(&id.0, &cutter.electric_in_port),
            Some(EnergySignal::Electric(true))
        );
        if !impacted && !powered {
            continue;
        }
        cutter.triggered = true;

        let origin = transform.translation.truncate();
        for (rope_entity, rope) in &ropes {
            let Some(points) = rope_world_points(rope, &transforms) else {
                continue;
            };
            let within_range = points
                .windows(2)
                .any(|seg| point_segment_distance(origin, seg[0], seg[1]) <= cutter.cut_radius);
            if within_range {
                commands.entity(rope_entity).despawn();
            }
        }
        for (entity, other_transform, tags) in &poppable {
            if tags.has("POPPABLE")
                && other_transform.translation.truncate().distance(origin) <= cutter.cut_radius
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub struct CutterShearsPlugin;

impl Plugin for CutterShearsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            cutter_shears_system.in_set(SimSet::CollisionRouting),
        );
    }
}
