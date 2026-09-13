//! `PartDef` → `RigidBody` + `Collider` (docs/GDD.md §5.5, mirroring
//! OpenTIM's `part.rs` responsibility). M1 only provides generic
//! primitive-shape helpers — M2's `PartRegistry` will build data-driven
//! parts on top of these once `data/parts/*.json` exists.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Spawns a fixed (immovable) box collider with a flat-color sprite.
/// `half_extents` and `translation` are in pixels (docs/GDD.md §2.2).
pub fn spawn_static_box(
    commands: &mut Commands,
    translation: Vec2,
    half_extents: Vec2,
    friction: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            RigidBody::Fixed,
            Collider::cuboid(half_extents.x, half_extents.y),
            Friction::new(friction),
            Transform::from_xyz(translation.x, translation.y, 0.0),
            Sprite::from_color(color, half_extents * 2.0),
        ))
        .id()
}

/// Spawns a dynamic box collider with a flat-color sprite.
pub fn spawn_dynamic_box(
    commands: &mut Commands,
    translation: Vec2,
    half_extents: Vec2,
    restitution: f32,
    friction: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::cuboid(half_extents.x, half_extents.y),
            Restitution::new(restitution),
            Friction::new(friction),
            Transform::from_xyz(translation.x, translation.y, 0.0),
            Sprite::from_color(color, half_extents * 2.0),
        ))
        .id()
}
