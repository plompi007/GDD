//! `fan_blower` (docs/GDD.md §1.3-e/A05): `ELECTRIC in → PNEUMATIC` field.
//! The field/force math itself lives in `atmosphere.rs`'s `FanBlower`
//! component/system — this module just resolves a placed instance's
//! params into it, matching `motor_electric.rs`'s split.

use bevy::prelude::*;

use crate::atmosphere::FanBlower;
use crate::level_file_format::PlacedPart;
use crate::part::PartDef;

fn default_power(def: &PartDef) -> f32 {
    match def.params.get("power") {
        Some(crate::part::ParamSpec::Number { default, .. }) => *default,
        _ => 2.0,
    }
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    let electric_in_port = def
        .ports
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "in".to_string());
    let power = placed.params.power.unwrap_or_else(|| default_power(def));

    commands.entity(entity).insert(FanBlower {
        electric_in_port,
        power,
        range: def.field_range.unwrap_or(220.0),
        half_width: def.field_width.unwrap_or(50.0),
    });
}
