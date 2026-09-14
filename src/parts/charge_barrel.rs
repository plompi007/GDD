//! `charge_barrel` (docs/GDD.md §1.3-ו/A08): `THERMAL in → radial impulse +
//! DESTRUCTIBLE demolition`. Detonation logic lives in `atmosphere.rs`'s
//! `ChargeBarrel` component/system — this module just resolves a placed
//! instance's `power` param into it.

use bevy::prelude::*;

use crate::atmosphere::ChargeBarrel;
use crate::level_file_format::PlacedPart;
use crate::part::PartDef;

fn default_power(def: &PartDef) -> f32 {
    match def.params.get("power") {
        Some(crate::part::ParamSpec::Number { default, .. }) => *default,
        _ => 900.0,
    }
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    let blast_power = def
        .blast_power
        .unwrap_or_else(|| placed.params.power.unwrap_or_else(|| default_power(def)));

    commands.entity(entity).insert(ChargeBarrel {
        blast_power,
        blast_radius: def.blast_radius.unwrap_or(150.0),
        detonated: false,
    });
}
