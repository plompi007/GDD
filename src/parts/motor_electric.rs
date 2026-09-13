//! `motor_electric` (docs/GDD.md §1.3-d): `ELECTRIC in → ROTARY out`. The
//! actual `ELECTRIC in → spin` behavior lives in `gear_train.rs`'s
//! `MotorSource`/`motor_source_system` (it needs to sit next to
//! `RotaryState`/`GearNode` to feed the same-tick meshing pass) — this
//! module is just where a placed motor's `rpm`/`direction` params get
//! resolved into that component at spawn time.

use bevy::prelude::*;

use crate::gear_train::{GearNode, MotorSource, RotaryState};
use crate::level_file_format::{PlacedPart, RotaryDirection};
use crate::part::{PartDef, ShapeSpec};

fn default_rpm(def: &PartDef) -> f32 {
    match def.params.get("rpm") {
        Some(crate::part::ParamSpec::Number { default, .. }) => *default,
        _ => 120.0,
    }
}

fn default_direction(def: &PartDef) -> RotaryDirection {
    match def.params.get("direction") {
        Some(crate::part::ParamSpec::Enum { default, .. }) if default == "CCW" => {
            RotaryDirection::Ccw
        }
        _ => RotaryDirection::Cw,
    }
}

/// docs/GDD.md's rotation convention: Bevy's +z-out-of-screen means a
/// positive angular velocity spins counter-clockwise — so CW maps to -1.
fn direction_sign(direction: RotaryDirection) -> f32 {
    match direction {
        RotaryDirection::Cw => -1.0,
        RotaryDirection::Ccw => 1.0,
    }
}

pub fn attach(commands: &mut Commands, entity: Entity, def: &PartDef, placed: &PlacedPart) {
    let rpm = placed.params.rpm.unwrap_or_else(|| default_rpm(def));
    let direction = placed
        .params
        .direction
        .unwrap_or_else(|| default_direction(def));
    let electric_in_port = def
        .ports
        .first()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "in".to_string());
    let radius = match def.body.shape {
        ShapeSpec::Ball { radius } => radius,
        ShapeSpec::Box { w, h } => w.min(h) / 2.0,
    };

    commands.entity(entity).insert((
        MotorSource {
            rpm,
            direction: direction_sign(direction),
            electric_in_port,
        },
        GearNode { radius },
        RotaryState::default(),
    ));
}
