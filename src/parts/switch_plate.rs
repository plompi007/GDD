//! `switch_plate` (docs/GDD.md §1.3-d/A06): `IMPACT in → toggle/pulse of
//! ELECTRIC out`. A sensor collider (docs/GDD.md §1.4.1's IMPACT/ELECTRIC
//! row) — `CollidingEntities` already exists for exactly this via
//! `body_factory::spawn_part`'s `sensor` handling, so this part just reads
//! it and emits into the [`EnergyGraph`], no new physics needed.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::energy_graph::{EnergyGraph, EnergySignal};
use crate::level_file_format::{PlacedPart, SwitchMode};
use crate::level_load::PlacedId;
use crate::part::PartDef;
use crate::sim::SimSet;

#[derive(Component)]
pub struct SwitchPlate {
    pub out_port: String,
    pub mode: SwitchMode,
    pub is_on: bool,
    /// Last tick's contact state, so `Toggle` mode can detect a rising
    /// edge instead of flipping every tick contact holds.
    pub was_touching: bool,
}

fn default_mode(def: &PartDef) -> SwitchMode {
    match def.params.get("mode") {
        Some(crate::part::ParamSpec::Enum { default, .. }) if default == "toggle" => {
            SwitchMode::Toggle
        }
        _ => SwitchMode::Momentary,
    }
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    let out_port = def
        .ports
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "out".to_string());
    let mode = placed.params.mode.unwrap_or_else(|| default_mode(def));

    commands.entity(entity).insert(SwitchPlate {
        out_port,
        mode,
        is_on: false,
        was_touching: false,
    });
}

/// docs/GDD.md §2.4 step 7: this is exactly what `SimSet::CollisionRouting`
/// is for — turning this tick's physical contact into this tick's IMPACT
/// (here: ELECTRIC) signal.
fn switch_plate_system(
    mut energy: ResMut<EnergyGraph>,
    mut switches: Query<(&PlacedId, &CollidingEntities, &mut SwitchPlate)>,
) {
    for (id, colliding, mut switch) in &mut switches {
        let touching = !colliding.is_empty();
        switch.is_on = match switch.mode {
            SwitchMode::Momentary => touching,
            SwitchMode::Toggle => {
                let rising_edge = touching && !switch.was_touching;
                if rising_edge {
                    !switch.is_on
                } else {
                    switch.is_on
                }
            }
        };
        switch.was_touching = touching;
        energy.emit(&id.0, &switch.out_port, EnergySignal::Electric(switch.is_on));
    }
}

pub struct SwitchPlatePlugin;

impl Plugin for SwitchPlatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            switch_plate_system.in_set(SimSet::CollisionRouting),
        );
    }
}
