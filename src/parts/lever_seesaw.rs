//! `lever_seesaw` (docs/GDD.md §1.3-c): a beam pivoting around its own
//! center — "converts falling into throwing" (§ appendix A04). Physically
//! a `RevoluteJoint` between the beam and an invisible `RigidBody::Fixed`
//! anchor spawned at the same point (bevy_rapier2d has no "pivot at a
//! fixed world point with no second body" joint — the anchor entity is
//! the standard workaround, same idea as `rope_network`'s pulley points).

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level_load::LevelEntity;

pub fn attach(commands: &mut Commands, entity: Entity, translation: Vec2) {
    let pivot = commands
        .spawn((
            RigidBody::Fixed,
            Transform::from_translation(translation.extend(0.0)),
            LevelEntity,
        ))
        .id();
    commands.entity(entity).insert(ImpulseJoint::new(
        pivot,
        RevoluteJointBuilder::new(),
    ));
}
