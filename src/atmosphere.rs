//! `FieldSystem` + `ThermalSystem` (docs/GDD.md §1.3-e/1.3-ו, §1.4.1,
//! §1.4.3, mirroring OpenTIM's `atmosphere.rs` responsibility): the two
//! *proximity-based* (not topological/`EnergyGraph`-wired) energy carriers.
//!
//! - **PNEUMATIC** (wind): `fan_blower` projects a rectangular field in
//!   front of itself; every `WindReceiver` body inside it gets pushed along
//!   the field's direction, scaled by its own `windFactor` and inverse mass
//!   (docs/GDD.md §1.4.3 "מה קורה כשהדף אוויר מזיז מניפה?"). Runs in
//!   [`SimSet::FieldForces`].
//! - **THERMAL** (fire): `candle` is a constant point source; `fuse_cord`
//!   ignites when either end enters another source's radius, then its own
//!   *current burn point* becomes a temporary source too (docs/GDD.md
//!   §1.4.3 "מה קורה כשלהבה נוגעת בפתיל?") — this is how fire propagates
//!   from a candle down a fuse to a `charge_barrel`, or from fuse to fuse.
//!   Runs in [`SimSet::ThermalUpdate`].

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::energy_graph::{EnergyGraph, EnergySignal};
use crate::level_load::{PartTags, PlacedId};
use crate::sim::{SimSet, FIXED_DT};

/// Every `DYNAMIC` body gets one of these at spawn (docs/GDD.md §1.3
/// footnote: wind sensitivity is metadata, not a special-cased force) —
/// `factor` defaults to `1.0` for parts that don't declare `windFactor` in
/// their JSON (docs/GDD.md §1.4.3's `body.windFactor`).
#[derive(Component, Clone, Copy)]
pub struct WindReceiver {
    pub factor: f32,
    pub inv_mass: f32,
}

/// `fan_blower` (docs/GDD.md §1.3-e): `ELECTRIC in` gates a rectangular
/// PNEUMATIC field projected along its own +local-x axis.
#[derive(Component, Clone)]
pub struct FanBlower {
    pub electric_in_port: String,
    pub power: f32,
    pub range: f32,
    pub half_width: f32,
}

/// A constant THERMAL point source (docs/GDD.md §1.3-ו `candle`): `active`
/// is fixed at spawn from the placed instance's `startsLit` param — no
/// in-editor toggle yet.
#[derive(Component, Clone, Copy)]
pub struct ThermalEmitter {
    pub radius: f32,
    pub active: bool,
}

/// `fuse_cord` (docs/GDD.md §1.3-ו): burns from whichever end ignites
/// first toward the other at a fixed rate, its own current burn point
/// acting as a temporary [`ThermalEmitter`] each tick it's lit.
#[derive(Component, Clone, Copy)]
pub struct FuseCord {
    pub length: f32,
    pub in_offset: Vec2,
    pub out_offset: Vec2,
    pub ignite_radius: f32,
    pub burn_progress: f32,
    pub is_burning: bool,
    /// `true` while burning from `in_offset` toward `out_offset`.
    pub burning_from_in: bool,
}

/// docs/GDD.md's fuse-burn rate (units/second) — the "central timing
/// element of the game" (§1.3-ו).
pub const FUSE_BURN_RATE: f32 = 45.0;

/// `charge_barrel` (docs/GDD.md §1.3-ו): detonates once, the first tick it
/// falls within *some other* THERMAL source's own broadcast radius
/// (docs/GDD.md §1.4.3 — proximity is always checked against the emitting
/// source's radius, e.g. a candle's `thermalRadius` or a burning fuse's
/// 8px flame point, never the receiver's own) — radial impulse to nearby
/// dynamic bodies, despawns nearby `DESTRUCTIBLE` parts, then despawns
/// itself.
#[derive(Component, Clone, Copy)]
pub struct ChargeBarrel {
    pub blast_power: f32,
    pub blast_radius: f32,
    pub detonated: bool,
}

fn world_point(transform: &Transform, local_offset: Vec2) -> Vec2 {
    (transform.translation + transform.rotation * local_offset.extend(0.0)).truncate()
}

/// docs/GDD.md §1.4.3's `FieldSystem`: every powered `fan_blower` pushes
/// every `WindReceiver` body inside its field along its own +local-x axis,
/// falling off linearly with distance and scaled by the body's own
/// `windFactor`/mass — a direct `Velocity` nudge (like `rope_network`'s and
/// `conveyor`'s own corrections), kept consistent with the rest of this
/// codebase's impulse-based style rather than `ExternalForce`.
fn wind_field_system(
    energy: Res<EnergyGraph>,
    fans: Query<(&PlacedId, &Transform, &FanBlower)>,
    mut bodies: Query<(&Transform, &mut Velocity, &WindReceiver)>,
) {
    for (id, fan_transform, fan) in &fans {
        let powered = matches!(
            energy.read(&id.0, &fan.electric_in_port),
            Some(EnergySignal::Electric(true))
        );
        if !powered {
            continue;
        }
        let origin = fan_transform.translation.truncate();
        let direction = (fan_transform.rotation * Vec3::X)
            .truncate()
            .normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }
        for (body_transform, mut velocity, receiver) in &mut bodies {
            let offset = body_transform.translation.truncate() - origin;
            let along = offset.dot(direction);
            if along < 0.0 || along > fan.range {
                continue;
            }
            let perp = (offset - direction * along).length();
            if perp > fan.half_width {
                continue;
            }
            let falloff = 1.0 - (along / fan.range) * 0.5;
            let accel = fan.power * receiver.factor * receiver.inv_mass * falloff;
            velocity.linvel += direction * accel * FIXED_DT;
        }
    }
}

/// A THERMAL source point active this tick: every lit `candle` plus every
/// currently-burning `fuse_cord`'s own advancing burn point (docs/GDD.md
/// §1.4.3) — computed once per tick and shared by ignition, tag-rule, and
/// detonation checks so fire propagates consistently through whatever's in
/// range this same tick.
fn collect_thermal_sources(
    candles: &Query<(&Transform, &ThermalEmitter)>,
    fuses: &Query<(Entity, &Transform, &mut FuseCord)>,
) -> Vec<(Vec2, f32)> {
    let mut sources = Vec::new();
    for (transform, emitter) in candles {
        if emitter.active {
            sources.push((transform.translation.truncate(), emitter.radius));
        }
    }
    for (_, transform, fuse) in fuses {
        if !fuse.is_burning {
            continue;
        }
        let (from, to) = if fuse.burning_from_in {
            (fuse.in_offset, fuse.out_offset)
        } else {
            (fuse.out_offset, fuse.in_offset)
        };
        let t = (fuse.burn_progress / fuse.length).clamp(0.0, 1.0);
        let point = world_point(transform, from.lerp(to, t));
        sources.push((point, fuse.ignite_radius));
    }
    sources
}

fn near_any_source(point: Vec2, sources: &[(Vec2, f32)]) -> bool {
    sources
        .iter()
        .any(|(source, radius)| point.distance(*source) <= *radius)
}

/// docs/GDD.md §2.4 step 2 / §1.4.3: fire ignition, propagation, tag-rule
/// reactions (`THERMAL` in range of `FLAMMABLE`/`POPPABLE`), and
/// `charge_barrel` detonation, all against this tick's shared source list
/// so a candle can light a fuse which lights a barrel which destroys a
/// wall in a single, deterministic pass.
fn thermal_update_system(
    mut commands: Commands,
    candles: Query<(&Transform, &ThermalEmitter)>,
    mut fuses: Query<(Entity, &Transform, &mut FuseCord)>,
    mut barrels: Query<(Entity, &Transform, &mut ChargeBarrel)>,
    mut dynamic_bodies: Query<(&Transform, &mut Velocity)>,
    tagged: Query<(Entity, &Transform, &PartTags)>,
) {
    let sources = collect_thermal_sources(&candles, &fuses);

    // 1. Ignite idle fuses whose either end is in range of a source.
    for (_, transform, mut fuse) in &mut fuses {
        if fuse.is_burning || fuse.burn_progress > 0.0 {
            continue;
        }
        let in_point = world_point(transform, fuse.in_offset);
        let out_point = world_point(transform, fuse.out_offset);
        if near_any_source(in_point, &sources) {
            fuse.is_burning = true;
            fuse.burning_from_in = true;
        } else if near_any_source(out_point, &sources) {
            fuse.is_burning = true;
            fuse.burning_from_in = false;
        }
    }

    // 2. Tag rules (docs/GDD.md §1.4.2): THERMAL near FLAMMABLE/POPPABLE.
    for (entity, transform, tags) in &tagged {
        if !(tags.has("FLAMMABLE") || tags.has("POPPABLE")) {
            continue;
        }
        if near_any_source(transform.translation.truncate(), &sources) {
            commands.entity(entity).despawn();
        }
    }

    // 3. `charge_barrel` detonation.
    for (entity, transform, mut barrel) in &mut barrels {
        if barrel.detonated {
            continue;
        }
        if !near_any_source(transform.translation.truncate(), &sources) {
            continue;
        }
        barrel.detonated = true;
        let center = transform.translation.truncate();
        for (body_transform, mut velocity) in &mut dynamic_bodies {
            let offset = body_transform.translation.truncate() - center;
            let dist = offset.length();
            if dist > barrel.blast_radius || dist < 1e-3 {
                continue;
            }
            let falloff = 1.0 - dist / barrel.blast_radius;
            velocity.linvel += offset.normalize() * barrel.blast_power * falloff;
        }
        for (other, other_transform, other_tags) in &tagged {
            if other == entity || !other_tags.has("DESTRUCTIBLE") {
                continue;
            }
            if other_transform.translation.truncate().distance(center) <= barrel.blast_radius {
                commands.entity(other).despawn();
            }
        }
        commands.entity(entity).despawn();
    }

    // 4. Advance burning fuses; one that reaches its far end disappears
    //    (docs/GDD.md §1.4.3: "כש-burnProgress >= length ... הפתיל נעלם").
    let mut burned_out = Vec::new();
    for (entity, _, mut fuse) in &mut fuses {
        if !fuse.is_burning {
            continue;
        }
        fuse.burn_progress += FUSE_BURN_RATE * FIXED_DT;
        if fuse.burn_progress >= fuse.length {
            burned_out.push(entity);
        }
    }
    for entity in burned_out {
        commands.entity(entity).despawn();
    }
}

pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            thermal_update_system.in_set(SimSet::ThermalUpdate),
        )
        .add_systems(FixedUpdate, wind_field_system.in_set(SimSet::FieldForces));
    }
}
