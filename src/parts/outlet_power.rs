//! `outlet_power` (docs/GDD.md §1.3-d): a constant `ELECTRIC` source.
//! `startsOn` is a per-instance param (docs/GDD.md §5.1 `placedPart.params`)
//! — there's no in-editor toggle for it yet (that's `switch_plate`, not
//! implemented until a level actually needs it).

use bevy::prelude::*;

use crate::energy_graph::{EnergyGraph, EnergySignal};
use crate::level_file_format::PlacedPart;
use crate::level_load::PlacedId;
use crate::part::PartDef;
use crate::sim::SimSet;

#[derive(Component)]
pub struct OutletSource {
    pub starts_on: bool,
    pub out_port: String,
}

/// Reads `def.ports[0]` as the part's `ELECTRIC out` port and `placed`'s
/// `startsOn` override (falling back to `true` if the placed instance
/// doesn't specify one — matches docs/GDD.md §1.3-d's default expectation
/// that a wall outlet is simply live).
pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    let out_port = def
        .ports
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "out".to_string());
    commands.entity(entity).insert(OutletSource {
        starts_on: placed.params.starts_on.unwrap_or(true),
        out_port,
    });
}

fn outlet_source_system(mut energy: ResMut<EnergyGraph>, outlets: Query<(&PlacedId, &OutletSource)>) {
    for (id, outlet) in &outlets {
        energy.emit(
            &id.0,
            &outlet.out_port,
            EnergySignal::Electric(outlet.starts_on),
        );
    }
}

pub struct OutletPowerPlugin;

impl Plugin for OutletPowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            outlet_source_system.in_set(SimSet::EnergyPropagate),
        );
    }
}
