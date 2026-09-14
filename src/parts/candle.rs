//! `candle` (docs/GDD.md §1.3-ו/A08): a constant THERMAL point source. The
//! actual ignition/propagation logic lives in `atmosphere.rs`'s
//! `ThermalEmitter` component/system — this module just resolves a placed
//! instance's `startsLit` param into it.

use bevy::prelude::*;

use crate::atmosphere::ThermalEmitter;
use crate::level_file_format::PlacedPart;
use crate::part::PartDef;

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    commands.entity(entity).insert(ThermalEmitter {
        radius: def.thermal_radius.unwrap_or(20.0),
        active: placed.params.starts_lit.unwrap_or(true),
    });
}
