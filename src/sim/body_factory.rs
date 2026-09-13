//! `PartDef` → `RigidBody` + `Collider` (docs/GDD.md §5.5, mirroring
//! OpenTIM's `part.rs` responsibility): the one place that turns part data
//! into an actual Bevy entity.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

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
/// = counter-clockwise).
pub fn spawn_part(
    commands: &mut Commands,
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
    let sprite_size = match def.body.shape {
        ShapeSpec::Ball { radius } => Vec2::splat(radius * 2.0),
        ShapeSpec::Box { w, h } => Vec2::new(w, h),
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
        Sprite::from_color(color, sprite_size),
    ));

    if def.body.kind == BodyKind::Dynamic {
        entity.insert((
            AdditionalMassProperties::Mass(def.body.mass),
            GravityScale(def.body.gravity_scale),
            Damping {
                linear_damping: def.body.linear_damping,
                angular_damping: def.body.angular_damping,
            },
        ));
        if def.body.ccd {
            entity.insert(Ccd::enabled());
        }
    }

    entity.id()
}
