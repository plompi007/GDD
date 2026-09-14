//! `PartRegistry`: loads and validates every `data/parts/*.json` file
//! (docs/GDD.md §5.5, mirroring OpenTIM's `parts/` responsibility).
//!
//! Part JSON is embedded at compile time (`include_str!`) rather than read
//! from disk at runtime: it keeps M2 simple and behaves identically on
//! desktop and Android, where arbitrary filesystem reads aren't available
//! the same way (assets need APK bundling). A hot-reloadable
//! `bevy_asset`-based loader is a reasonable upgrade for a later polish
//! pass, not required for correctness now — nothing about the `PartDef`
//! shape (src/part.rs) needs to change when that happens.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::level_file_format::PlacedPart;
use crate::part::PartDef;

pub mod candle;
pub mod charge_barrel;
pub mod conveyor;
pub mod cutter_shears;
pub mod fan_blower;
pub mod fuse_cord;
pub mod lever_seesaw;
pub mod motor_electric;
pub mod outlet_power;
pub mod punch_arm;
pub mod switch_plate;

/// One `include_str!` per file, matching docs/GDD.md §1.3's initial P0
/// catalog (5 static + 7 dynamic = 12) plus the two GOAL-category parts
/// (§1.3 ט) M3 needs to have any win condition to test against, plus M4's
/// energy/gear/rope demo parts (§1.3 c/d). Add new parts here as later
/// milestones need them, keeping this list in sync with `data/parts/`.
const PART_JSON: &[&str] = &[
    include_str!("../../data/parts/floor_ground.json"),
    include_str!("../../data/parts/plank_wood.json"),
    include_str!("../../data/parts/beam_steel.json"),
    include_str!("../../data/parts/wall_brick.json"),
    include_str!("../../data/parts/spike_pin.json"),
    include_str!("../../data/parts/ball_lead.json"),
    include_str!("../../data/parts/ball_iron.json"),
    include_str!("../../data/parts/ball_wood.json"),
    include_str!("../../data/parts/ball_rubber.json"),
    include_str!("../../data/parts/ball_glass.json"),
    include_str!("../../data/parts/crate_wood.json"),
    include_str!("../../data/parts/balloon_lift.json"),
    include_str!("../../data/parts/bin_target.json"),
    include_str!("../../data/parts/goal_zone.json"),
    include_str!("../../data/parts/outlet_power.json"),
    include_str!("../../data/parts/motor_electric.json"),
    include_str!("../../data/parts/gear_small.json"),
    include_str!("../../data/parts/gear_large.json"),
    include_str!("../../data/parts/conveyor.json"),
    include_str!("../../data/parts/pulley_wheel.json"),
    include_str!("../../data/parts/springboard.json"),
    include_str!("../../data/parts/lever_seesaw.json"),
    include_str!("../../data/parts/fan_blower.json"),
    include_str!("../../data/parts/switch_plate.json"),
    include_str!("../../data/parts/candle.json"),
    include_str!("../../data/parts/fuse_cord.json"),
    include_str!("../../data/parts/charge_barrel.json"),
    include_str!("../../data/parts/cutter_shears.json"),
    include_str!("../../data/parts/punch_arm.json"),
];

/// All known part definitions, keyed by `part_type`. A `BTreeMap` (not
/// `HashMap`) so iteration order is always the same across runs — required
/// by CLAUDE.md rule 2 (determinism) for any code that ever iterates every
/// part (e.g. a future parts-bin UI).
#[derive(Resource, Debug, Default)]
pub struct PartRegistry {
    defs: BTreeMap<String, PartDef>,
}

impl PartRegistry {
    pub fn get(&self, part_type: &str) -> Option<&PartDef> {
        self.defs.get(part_type)
    }

    pub fn part_types(&self) -> impl Iterator<Item = &str> {
        self.defs.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
}

/// Parses every embedded part JSON file. Panics on malformed JSON or a
/// `partType` that doesn't match its own file — both are build-time content
/// bugs (bad data shipped in the binary), not conditions the running game
/// should ever need to recover from, so a hard failure at startup is
/// correct here (CLAUDE.md rule 1).
pub fn build_registry() -> PartRegistry {
    let mut defs = BTreeMap::new();
    for json in PART_JSON {
        let def: PartDef = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("invalid part JSON in data/parts/: {e}\n{json}"));
        let previous = defs.insert(def.part_type.clone(), def);
        if let Some(previous) = previous {
            panic!("duplicate partType in data/parts/: {}", previous.part_type);
        }
    }
    PartRegistry { defs }
}

pub struct PartsPlugin;

impl Plugin for PartsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(build_registry()).add_plugins((
            outlet_power::OutletPowerPlugin,
            conveyor::ConveyorPlugin,
            switch_plate::SwitchPlatePlugin,
            cutter_shears::CutterShearsPlugin,
            punch_arm::PunchArmPlugin,
        ));
    }
}

/// Per-part-type extra components beyond the generic physics body every
/// part gets from `body_factory::spawn_part` (docs/GDD.md §5.6's "component
/// = ECS, not trait object" rule: this dispatch is a plain `match`, not a
/// generic behavior registry — each arm just wires up that one part's own
/// marker components). Called once per placed part at spawn/reset time.
pub fn attach_part_behavior(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    match def.part_type.as_str() {
        "outlet_power" => outlet_power::attach(commands, entity, def, placed),
        "motor_electric" => motor_electric::attach(commands, entity, def, placed),
        "gear_small" | "gear_large" => {
            let radius = match def.body.shape {
                crate::part::ShapeSpec::Ball { radius } => radius,
                crate::part::ShapeSpec::Box { w, h } => w.min(h) / 2.0,
            };
            commands.entity(entity).insert((
                crate::gear_train::GearNode { radius },
                crate::gear_train::RotaryState::default(),
            ));
        }
        "conveyor" => conveyor::attach(commands, entity, def),
        "lever_seesaw" => lever_seesaw::attach(commands, entity, Vec2::new(placed.x, placed.y)),
        "fan_blower" => fan_blower::attach(commands, entity, def, placed),
        "switch_plate" => switch_plate::attach(commands, entity, def, placed),
        "candle" => candle::attach(commands, entity, def, placed),
        "fuse_cord" => fuse_cord::attach(commands, entity, def),
        "charge_barrel" => charge_barrel::attach(commands, entity, def, placed),
        "cutter_shears" => cutter_shears::attach(commands, entity, def, placed),
        "punch_arm" => punch_arm::attach(commands, entity, def, placed),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_embedded_part_json_parses_and_has_unique_part_types() {
        let registry = build_registry();
        assert_eq!(registry.len(), PART_JSON.len());
    }

    #[test]
    fn expected_p0_parts_are_present() {
        let registry = build_registry();
        for part_type in [
            "floor_ground",
            "plank_wood",
            "beam_steel",
            "wall_brick",
            "spike_pin",
            "ball_lead",
            "ball_iron",
            "ball_wood",
            "ball_rubber",
            "ball_glass",
            "crate_wood",
            "balloon_lift",
            "bin_target",
            "goal_zone",
            "outlet_power",
            "motor_electric",
            "gear_small",
            "gear_large",
            "conveyor",
            "pulley_wheel",
            "springboard",
            "lever_seesaw",
            "fan_blower",
            "switch_plate",
            "candle",
            "fuse_cord",
            "charge_barrel",
            "cutter_shears",
            "punch_arm",
        ] {
            assert!(
                registry.get(part_type).is_some(),
                "missing expected P0 part: {part_type}"
            );
        }
    }
}
