//! `PartDef` → `RigidBody` + `Collider` (docs/GDD.md §5.5, mirroring
//! OpenTIM's `part.rs` responsibility): the one place that turns part data
//! into an actual Bevy entity.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::atmosphere::WindReceiver;
use crate::part::{BodyKind, PartCategory, PartDef, ShapeSpec};

/// Temporary flat-color scheme so parts are visually distinguishable before
/// real art exists. Replaced by the Prism Foundry sprite/shader system in
/// M6.5 (docs/GDD.md §3.9) — this function should not survive past that.
pub fn placeholder_color(def: &PartDef) -> Color {
    if def.has_tag("BOUNCY") {
        return Color::srgb_u8(0xe2, 0x3d, 0x9c);
    }
    if def.has_tag("METALLIC") {
        return Color::srgb_u8(0x9a, 0xa3, 0xb8);
    }
    if def.has_tag("FRAGILE") {
        return Color::srgb_u8(0xd7, 0xe8, 0xf5);
    }
    if def.has_tag("WIND") {
        return Color::srgb_u8(0xff, 0x8f, 0x6b);
    }
    if def.has_tag("FLAMMABLE") {
        return Color::srgb_u8(0x8a, 0x5a, 0x2e);
    }
    match def.category {
        PartCategory::Static => Color::srgb_u8(0x2a, 0x30, 0x40),
        _ => Color::srgb_u8(0x6c, 0x8c, 0xff),
    }
}

/// Spawns one entity from a [`PartDef`]. `translation` is in pixels;
/// `rotation_degrees` follows Bevy's +z-out-of-screen convention (positive
/// = counter-clockwise). `meshes`/`materials` back the base shape for
/// `ShapeSpec::Ball` (a real circle mesh, replacing a square `Sprite` that
/// only ever matched the collider's bounding box) and `render::decorate`'s
/// cosmetic child shapes (gear teeth, a candle flame, ...) — `ShapeSpec::Box`
/// keeps using a plain `Sprite`, since a flat rectangle already matches its
/// own collider exactly.
pub fn spawn_part(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    def: &PartDef,
    translation: Vec2,
    rotation_degrees: f32,
) -> Entity {
    let color = placeholder_color(def);
    let transform = Transform::from_xyz(translation.x, translation.y, 0.0)
        .with_rotation(Quat::from_rotation_z(rotation_degrees.to_radians()));

    let collider = match def.body.shape {
        ShapeSpec::Ball { radius } => Collider::ball(radius),
        ShapeSpec::Box { w, h } => Collider::cuboid(w / 2.0, h / 2.0),
    };
    let rigid_body = match def.body.kind {
        BodyKind::Fixed => RigidBody::Fixed,
        BodyKind::Dynamic => RigidBody::Dynamic,
    };

    let mut entity = commands.spawn((
        rigid_body,
        collider,
        Friction::new(def.body.friction),
        Restitution::new(def.body.restitution),
        transform,
        // Every collider gets contact events, not just sensors: tag-rule
        // reactions (docs/GDD.md §1.4.2 — SHARP popping POPPABLE, etc.) and
        // IMPACT-triggered parts (switch_plate, punch_arm, cutter_shears)
        // all need `CollisionEvent`/`CollidingEntities` on ordinary solid
        // bodies too, not just Sensor-marked ones.
        ActiveEvents::COLLISION_EVENTS,
    ));

    match def.body.shape {
        ShapeSpec::Ball { radius } => {
            entity.insert((
                Mesh2d(meshes.add(Circle::new(radius))),
                MeshMaterial2d(materials.add(color)),
            ));
        }
        ShapeSpec::Box { w, h } => {
            entity.insert(Sprite::from_color(color, Vec2::new(w, h)));
        }
    }

    if def.body.sensor {
        // CollidingEntities only updates from CollisionEvent (docs: bevy_rapier2d
        // plugin/systems/collider.rs `update_colliding_entities`), which is only
        // emitted for colliders with ActiveEvents::COLLISION_EVENTS — win_conditions.rs
        // (M3) reads CollidingEntities on GOAL-category entities to evaluate CONTAINED.
        entity.insert((Sensor, CollidingEntities::default()));
    } else {
        // Non-sensor colliders still benefit from `CollidingEntities` for
        // the same tag-rule/IMPACT-trigger systems above (e.g. `spike_pin`
        // detecting a `POPPABLE` balloon it's physically touching).
        entity.insert(CollidingEntities::default());
    }

    if def.body.kind == BodyKind::Dynamic {
        entity.insert((
            AdditionalMassProperties::Mass(def.body.mass),
            GravityScale(def.body.gravity_scale),
            Damping {
                linear_damping: def.body.linear_damping,
                angular_damping: def.body.angular_damping,
            },
            // Without an explicit `Velocity`, bevy_rapier2d still simulates
            // the body fine (it tracks velocity internally regardless) but
            // no Bevy-side system can read or correct it — both
            // `rope_network`'s tension impulse and `conveyor`'s belt nudge
            // need read/write access to every dynamic body's velocity.
            Velocity::zero(),
            // docs/GDD.md §1.4.3: every dynamic body is a wind receiver;
            // `windFactor` defaults to 1.0 for parts that don't override it.
            WindReceiver {
                factor: def.wind_factor.unwrap_or(1.0),
                inv_mass: 1.0 / def.body.mass.max(0.01),
            },
        ));
        if def.body.ccd {
            entity.insert(Ccd::enabled());
        }
    }

    let id = entity.id();
    crate::render::decorate(commands, meshes, materials, id, def);
    id
}
