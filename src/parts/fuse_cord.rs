//! `fuse_cord` (docs/GDD.md §1.3-ו/A08): burns from whichever end ignites
//! first toward the other at a fixed rate. The burn/ignition logic lives
//! in `atmosphere.rs`'s `FuseCord` component/system — this module just
//! derives its two end offsets from the part's own body width.

use bevy::prelude::*;

use crate::atmosphere::FuseCord;
use crate::part::{PartDef, ShapeSpec};

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef) {
    let length = match def.body.shape {
        ShapeSpec::Box { w, .. } => w,
        ShapeSpec::Ball { radius } => radius * 2.0,
    };
    let half = length / 2.0;

    commands.entity(entity).insert(FuseCord {
        length,
        in_offset: Vec2::new(-half, 0.0),
        out_offset: Vec2::new(half, 0.0),
        ignite_radius: def.thermal_radius.unwrap_or(8.0),
        burn_progress: 0.0,
        is_burning: false,
        burning_from_in: true,
    });
}
