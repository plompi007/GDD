//! `conveyor` (docs/GDD.md §1.3-c): `ROTARY in → surfaceVelocity`. Bodies
//! resting on it get pushed along its own local +X axis (rotated with the
//! part, so an angled conveyor pushes at an angle) at a speed proportional
//! to its current [`RotaryState`] — fed by a `drive_belt` from a motor/gear
//! upstream (docs/GDD.md's own "motor → gear → conveyor" M4 acceptance
//! case), not by direct gear meshing: a flat belt doesn't have a tooth
//! radius to mesh with.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::gear_train::RotaryState;
use crate::part::{PartDef, ShapeSpec};
use crate::sim::SimSet;

/// `speed_per_radian`: world units/sec of belt surface speed per rad/s of
/// rotary input. Modeled as `drum_radius` (half the conveyor's own height,
/// i.e. the radius of the roller a belt would wrap around) so a faster
/// motor or a bigger drum both plausibly mean a faster belt.
#[derive(Component)]
pub struct ConveyorBelt {
    pub speed_per_radian: f32,
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef) {
    let drum_radius = match def.body.shape {
        ShapeSpec::Box { h, .. } => h / 2.0,
        ShapeSpec::Ball { radius } => radius,
    };
    commands.entity(entity).insert((
        ConveyorBelt {
            speed_per_radian: drum_radius,
        },
        RotaryState::default(),
        CollidingEntities::default(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

fn conveyor_belt_effect_system(
    conveyors: Query<(&Transform, &ConveyorBelt, &RotaryState, &CollidingEntities)>,
    mut bodies: Query<&mut Velocity>,
) {
    for (transform, belt, state, touching) in &conveyors {
        let belt_speed = state.angular_velocity * belt.speed_per_radian;
        if belt_speed == 0.0 {
            continue;
        }
        let belt_dir = (transform.rotation * Vec3::X).truncate();
        for touched in touching.iter() {
            let Ok(mut velocity) = bodies.get_mut(touched) else {
                continue;
            };
            let current_along = velocity.linvel.dot(belt_dir);
            velocity.linvel += belt_dir * (belt_speed - current_along);
        }
    }
}

pub struct ConveyorPlugin;

impl Plugin for ConveyorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            conveyor_belt_effect_system.in_set(SimSet::GearTorques),
        );
    }
}
