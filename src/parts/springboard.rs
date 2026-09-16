//! `springboard` (docs/GDD.md §1.3): pure `bevy_rapier2d` physics — a fixed
//! body with high `restitution`, no gameplay behavior component or `attach`
//! case at all before this module. That means there was no existing signal
//! to hook a "just launched something" animation to; this module adds
//! exactly that and nothing else — the bounce itself is still entirely
//! Rapier's own restitution response, unaffected by anything here.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::sim::SimSet;

#[derive(Component)]
pub struct Springboard;

/// Present only while the compressed-spring frame should show; removed by
/// [`tick_springboard_flash`] once its counter reaches zero. Headless-safe
/// (a plain tick counter, same style as `atmosphere::FuseFlicker`/
/// `ExplosionEffect`, no `AssetServer`) — `render::PartArtPlugin` is what
/// actually swaps in `assets/parts/springboard_action.png` while this is
/// present.
#[derive(Component)]
pub struct SpringboardFlash {
    ticks_remaining: u32,
}

/// Long enough to read as a compression, short enough to have fully
/// reverted well before a second bounce could plausibly land.
const FLASH_TICKS: u32 = (0.15 / crate::sim::FIXED_DT) as u32;

pub fn attach(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Springboard);
}

fn start_springboard_flash(
    mut events: EventReader<CollisionEvent>,
    springboards: Query<Entity, With<Springboard>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let CollisionEvent::Started(a, b, _flags) = event else {
            continue;
        };
        for entity in [*a, *b] {
            if springboards.contains(entity) {
                commands
                    .entity(entity)
                    .insert(SpringboardFlash { ticks_remaining: FLASH_TICKS });
            }
        }
    }
}

fn tick_springboard_flash(mut commands: Commands, mut flashes: Query<(Entity, &mut SpringboardFlash)>) {
    for (entity, mut flash) in &mut flashes {
        if flash.ticks_remaining == 0 {
            commands.entity(entity).remove::<SpringboardFlash>();
        } else {
            flash.ticks_remaining -= 1;
        }
    }
}

pub struct SpringboardPlugin;

impl Plugin for SpringboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (start_springboard_flash, tick_springboard_flash).in_set(SimSet::CollisionRouting),
        );
    }
}
