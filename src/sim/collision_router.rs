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

/// docs/GDD.md §1.4.2: "`FRAGILE` בפגיעה ב-`speed` > 18 → הגוף מתנפץ
/// ונעלם" — the spec's own number is a pre-Bevy (real-world m/s) figure;
/// `bevy_rapier2d`'s `pixels_per_meter` scaling (docs/GDD.md §2.2,
/// `math::PIXELS_PER_METER = 32`) means a `Velocity` read directly off a
/// body is already in that same scaled pixel/s space, so the equivalent
/// threshold here is `18.0 * PIXELS_PER_METER`.
const FRAGILE_SHATTER_SPEED: f32 = 18.0 * crate::math::PIXELS_PER_METER;

/// docs/GDD.md §1.4.2's tag rules: a part never checks another part's
/// specific type here, only tags (CLAUDE.md's "no `partA === 'candle' &&
/// partB === 'fuse'`" rule) — any future part carrying `SHARP`,
/// `POPPABLE`, or `FRAGILE` gets this behavior for free.
///
/// IMPACT-*triggered* parts (`switch_plate`, `punch_arm`, `cutter_shears`)
/// read `CollidingEntities` themselves in their own per-part systems
/// (also placed in [`crate::sim::SimSet::CollisionRouting`]) rather than
/// going through this generic router — they need to know *which* specific
/// entity touched them, not just react to a tag combination.
fn route_collisions(
    mut events: EventReader<CollisionEvent>,
    tags: Query<&PartTags>,
    velocities: Query<&Velocity>,
    mut commands: Commands,
) {
    for event in events.read() {
        let CollisionEvent::Started(a, b, _flags) = event else {
            continue;
        };
        let (Ok(tags_a), Ok(tags_b)) = (tags.get(*a), tags.get(*b)) else {
            continue;
        };
        let mut already_despawned = None;
        if tags_a.has("SHARP") && tags_b.has("POPPABLE") {
            commands.entity(*b).despawn();
            already_despawned = Some(*b);
        } else if tags_b.has("SHARP") && tags_a.has("POPPABLE") {
            commands.entity(*a).despawn();
            already_despawned = Some(*a);
        }
        for (entity, entity_tags) in [(*a, tags_a), (*b, tags_b)] {
            if already_despawned == Some(entity) {
                continue;
            }
            if entity_tags.has("FRAGILE")
                && velocities
                    .get(entity)
                    .is_ok_and(|v| v.linvel.length() > FRAGILE_SHATTER_SPEED)
            {
                commands.entity(entity).despawn();
            }
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
