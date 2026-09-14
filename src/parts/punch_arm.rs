//! `punch_arm` (docs/GDD.md §1.3-ח/A10): `IMPACT → fixed forward impulse`.
//! A sensor collider (same IMPACT-detection pathway as `switch_plate`);
//! whatever's touching it on a rising edge gets shoved along the arm's own
//! +local-x axis.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::part::PartDef;
use crate::sim::SimSet;

#[derive(Component)]
pub struct PunchArm {
    pub impulse_power: f32,
    pub was_touching: bool,
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef) {
    commands.entity(entity).insert(PunchArm {
        impulse_power: def.impulse_power.unwrap_or(700.0),
        was_touching: false,
    });
}

fn punch_arm_system(
    mut arms: Query<(&Transform, &CollidingEntities, &mut PunchArm)>,
    mut bodies: Query<&mut Velocity>,
) {
    for (transform, colliding, mut arm) in &mut arms {
        let touching = !colliding.is_empty();
        let rising_edge = touching && !arm.was_touching;
        arm.was_touching = touching;
        if !rising_edge {
            continue;
        }
        let direction = (transform.rotation * Vec3::X).truncate().normalize_or_zero();
        for target in colliding.iter() {
            if let Ok(mut velocity) = bodies.get_mut(target) {
                velocity.linvel += direction * arm.impulse_power;
            }
        }
    }
}

pub struct PunchArmPlugin;

impl Plugin for PunchArmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, punch_arm_system.in_set(SimSet::CollisionRouting));
    }
}
