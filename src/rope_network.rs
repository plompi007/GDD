//! `RopeNetwork` (docs/GDD.md §5.5, §1.3-c `rope`/`pulley_wheel`): a rope is
//! explicitly *not* a physical body — no collider, no rigid body — so it
//! can't be built from `bevy_rapier2d`'s own joints without inventing a
//! fake mass-less body per segment. Instead this models the whole
//! (possibly pulley-routed) rope as one inextensible-when-taut constraint,
//! applied directly as a velocity correction to its two end bodies.
//!
//! Runs in `SimSet::RopeTension` (docs/GDD.md §2.4 step 4), before the
//! physics step integrates the corrected velocities into position.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::part::Offset;
use crate::sim::SimSet;

/// One rope, built once at level load from a `connections: [{kind: ROPE}]`
/// entry (docs/GDD.md §5.1) — never mutated afterward; reset despawns and
/// rebuilds it like every other `LevelEntity`. `pulleys` lists routing
/// points in order from `end_a` to `end_b`; empty for a direct rope.
#[derive(Component)]
pub struct RopeConnection {
    pub end_a: (Entity, Offset),
    pub end_b: (Entity, Offset),
    pub pulleys: Vec<(Entity, Offset)>,
    pub max_length: f32,
}

pub(crate) fn world_anchor(transforms: &Query<&Transform>, anchor: (Entity, Offset)) -> Option<Vec2> {
    let transform = transforms.get(anchor.0).ok()?;
    Some(
        (transform.translation + transform.rotation * Vec3::new(anchor.1.x, anchor.1.y, 0.0))
            .truncate(),
    )
}

/// The full sequence of world-space points along a rope (both ends plus
/// any pulley routing, in order) — shared by `rope_tension_system` and
/// anything else that needs a rope's current geometry (e.g.
/// `cutter_shears`'s and `atmosphere`'s proximity checks).
pub(crate) fn rope_world_points(
    rope: &RopeConnection,
    transforms: &Query<&Transform>,
) -> Option<Vec<Vec2>> {
    let mut points = Vec::with_capacity(rope.pulleys.len() + 2);
    points.push(world_anchor(transforms, rope.end_a)?);
    for pulley in &rope.pulleys {
        points.push(world_anchor(transforms, *pulley)?);
    }
    points.push(world_anchor(transforms, rope.end_b)?);
    Some(points)
}

/// Shortest distance from a point to a line segment — shared by
/// `cutter_shears` (does a trigger's radius reach a rope segment?) and
/// `atmosphere`'s thermal system (docs/GDD.md §1.4.1: THERMAL x TENSION —
/// "a rope burns through and parts").
pub(crate) fn point_segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-6 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// docs/GDD.md §1.3-c: "changes direction, 1:1 force ratio" — a real rope
/// only pulls (never pushes, i.e. never resists going slack) and transmits
/// the same tension through every pulley redirect. Modeled as a single
/// sequential-impulse correction: once the rope is at/over `max_length`,
/// cancel exactly the combined "stretching" component of both end
/// bodies' velocities (their motion *away* from their nearest routing
/// point), split by inverse mass — the same math a two-body distance-joint
/// solver uses, generalized to a bent (pulley-routed) path since the
/// pulleys themselves are fixed and contribute no length-rate of their own.
fn rope_tension_system(
    ropes: Query<&RopeConnection>,
    transforms: Query<&Transform>,
    mut bodies: Query<(&mut Velocity, &AdditionalMassProperties)>,
) {
    for rope in &ropes {
        let Some(points) = rope_world_points(rope, &transforms) else {
            continue;
        };

        let total_length: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
        if total_length <= rope.max_length {
            continue; // slack: nothing to do, a rope never pushes
        }

        // Unit vector pointing *away* from each end's nearest routing
        // point — the direction that end moving further increases length.
        let dir_a = (points[0] - points[1]).normalize_or_zero();
        let dir_b = (points[points.len() - 1] - points[points.len() - 2]).normalize_or_zero();

        let Ok([(mut vel_a, mass_a), (mut vel_b, mass_b)]) =
            bodies.get_many_mut([rope.end_a.0, rope.end_b.0])
        else {
            continue;
        };
        let inv_mass_a = mass_of(mass_a).recip();
        let inv_mass_b = mass_of(mass_b).recip();

        let stretching_rate = vel_a.linvel.dot(dir_a) + vel_b.linvel.dot(dir_b);
        if stretching_rate <= 0.0 {
            continue; // already easing off, not actively stretching further
        }
        let impulse = stretching_rate / (inv_mass_a + inv_mass_b);
        vel_a.linvel -= dir_a * impulse * inv_mass_a;
        vel_b.linvel -= dir_b * impulse * inv_mass_b;
    }
}

fn mass_of(additional: &AdditionalMassProperties) -> f32 {
    match additional {
        AdditionalMassProperties::Mass(m) => *m,
        // Every M4 rope endpoint is spawned via `body_factory::spawn_part`,
        // which always uses the `Mass(f32)` variant (docs/GDD.md §5.3's
        // `BodySpec::mass`) — the `MassProperties` variant is never
        // constructed anywhere in this codebase.
        AdditionalMassProperties::MassProperties(props) => props.mass,
    }
}

pub struct RopeNetworkPlugin;

impl Plugin for RopeNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            rope_tension_system.in_set(SimSet::RopeTension),
        );
    }
}
