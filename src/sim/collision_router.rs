//! `CollisionEvent` (`bevy_rapier2d`) → generic tag-rule reactions
//! (docs/GDD.md §1.4.2, §2.4 step 7, mirroring OpenTIM's
//! `collision_router.rs` responsibility). A part never checks another
//! part's specific type here — only tags (CLAUDE.md's "no
//! `partA === 'candle' && partB === 'fuse'`" rule) — so any future part
//! that carries `SHARP` or `POPPABLE` gets this behavior for free.
//!
//! IMPACT-*triggered* parts (`switch_plate`, `punch_arm`, `cutter_shears`)
//! read `CollidingEntities` themselves in their own per-part systems
//! (also placed in [`crate::sim::SimSet::CollisionRouting`]) rather than
//! going through this generic router — they need to know *which* specific
//! entity touched them, not just react to a tag combination.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level_load::PartTags;
use crate::sim::SimSet;

/// docs/GDD.md §1.4.2: `SHARP` touching `POPPABLE` pops it (e.g.
/// `spike_pin` vs. `balloon_lift`) — checked both ways since
/// `CollisionEvent::Started` doesn't guarantee entity order.
fn route_collisions(
    mut events: EventReader<CollisionEvent>,
    tags: Query<&PartTags>,
    mut commands: Commands,
) {
    for event in events.read() {
        let CollisionEvent::Started(a, b, _flags) = event else {
            continue;
        };
        let (Ok(tags_a), Ok(tags_b)) = (tags.get(*a), tags.get(*b)) else {
            continue;
        };
        if tags_a.has("SHARP") && tags_b.has("POPPABLE") {
            commands.entity(*b).despawn();
        } else if tags_b.has("SHARP") && tags_a.has("POPPABLE") {
            commands.entity(*a).despawn();
        }
    }
}

pub struct CollisionRouterPlugin;

impl Plugin for CollisionRouterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            route_collisions.in_set(SimSet::CollisionRouting),
        );
    }
}
