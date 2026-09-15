//! `PartDef`: the data schema for `data/parts/*.json` (docs/GDD.md §5.3,
//! §5.6). Mirrors OpenTIM's `part.rs` responsibility — this is the type
//! every part's data is deserialized into; per-part *behavior* (M4+) is
//! Systems querying marker components, not methods on this struct (see
//! docs/GDD.md §5.6 for why: ECS, not trait objects).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PartCategory {
    Static,
    Dynamic,
    Mechanism,
    Power,
    Pneumatic,
    Thermal,
    Light,
    Actuator,
    Goal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Tier {
    P0,
    P1,
    P2,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ShapeSpec {
    Ball { radius: f32 },
    Box { w: f32, h: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyKind {
    Fixed,
    Dynamic,
}

fn default_mass() -> f32 {
    1.0
}
fn default_friction() -> f32 {
    0.5
}
fn default_gravity_scale() -> f32 {
    1.0
}

/// Docs/GDD.md §5.3. Physical properties for one part's rigid body. Field
/// defaults match `bevy_rapier2d`'s own component defaults so a part JSON
/// only needs to specify what's unusual about it.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodySpec {
    #[serde(rename = "type")]
    pub kind: BodyKind,
    pub shape: ShapeSpec,
    #[serde(default = "default_mass")]
    pub mass: f32,
    #[serde(default)]
    pub restitution: f32,
    #[serde(default = "default_friction")]
    pub friction: f32,
    #[serde(default)]
    pub linear_damping: f32,
    #[serde(default)]
    pub angular_damping: f32,
    #[serde(default = "default_gravity_scale")]
    pub gravity_scale: f32,
    #[serde(default)]
    pub ccd: bool,
    /// Detects overlap without physical collision response (docs/GDD.md
    /// §1.3 ט: `bin_target`, `goal_zone`). Not in the original v1.1 sketch
    /// of this struct — added in M3 because win conditions need it.
    #[serde(default)]
    pub sensor: bool,
}

/// `Serialize` is for M9's sandbox save/load round-trip
/// (`level_file_format.rs`'s `Condition::EnergyState` embeds this) — not
/// needed for `PartDef` loading itself, which stays read-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EnergyType {
    Rotary,
    Tension,
    Electric,
    Thermal,
    Pneumatic,
    Light,
    Impact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PortDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

/// A named energy connection point on a part (docs/GDD.md §1.2/§1.3). Empty
/// for every M2 part — populated starting M4 (motors, sensors, etc.).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Port {
    pub id: String,
    pub dir: PortDirection,
    pub energy: EnergyType,
    pub offset: Offset,
}

/// A mechanical attachment point (rope anchors, gear axles). Empty for
/// every M2 part.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Anchor {
    pub idx: u32,
    pub kind: EnergyType,
    pub offset: Offset,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSpec {
    #[serde(default)]
    pub rotatable: bool,
    #[serde(default)]
    pub rotation_snap: f32,
    #[serde(default)]
    pub flippable: bool,
    #[serde(default)]
    pub sprite: String,
    #[serde(default)]
    pub bin_icon: String,
    #[serde(default)]
    pub hitbox_padding: f32,
}

/// A tunable parameter exposed in the level editor's part inspector
/// (docs/GDD.md §3.4). Not used by any M2 part — first real use is
/// `motor_electric`'s `rpm`/`direction` in M4.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParamSpec {
    Number {
        min: f32,
        max: f32,
        #[serde(default)]
        step: Option<f32>,
        default: f32,
    },
    Enum {
        values: Vec<String>,
        default: String,
    },
    Boolean {
        default: bool,
    },
}

/// One entry of `data/parts/*.json`, deserialized as-is (docs/GDD.md §5.3).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartDef {
    pub part_type: String,
    pub display_key: String,
    pub category: PartCategory,
    pub tier: Tier,
    pub body: BodySpec,
    #[serde(default)]
    pub tags: Vec<String>,
    pub wind_factor: Option<f32>,
    /// `fan_blower` (docs/GDD.md §1.3-e/A05): length of its rectangular
    /// PNEUMATIC field in front of it, in pixels.
    pub field_range: Option<f32>,
    /// `fan_blower`: half-width of that same field, in pixels.
    pub field_width: Option<f32>,
    /// `candle`/`fuse_cord`/`charge_barrel` (docs/GDD.md §1.3-ו/A08):
    /// THERMAL ignition/detection radius, in pixels.
    pub thermal_radius: Option<f32>,
    /// `charge_barrel`: radial-impulse magnitude on detonation.
    pub blast_power: Option<f32>,
    /// `charge_barrel`: radius (pixels) within which a detonation applies
    /// impulse and destroys `DESTRUCTIBLE`-tagged parts.
    pub blast_radius: Option<f32>,
    /// `punch_arm` (docs/GDD.md §1.3-ח/A10): forward-impulse magnitude
    /// applied to whatever triggers it.
    pub impulse_power: Option<f32>,
    /// `cutter_shears` (docs/GDD.md §1.3-ח/A09): radius (pixels) within
    /// which a trigger severs `rope`s and pops `POPPABLE` parts.
    pub cut_radius: Option<f32>,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    #[serde(default)]
    pub editor: EditorSpec,
    #[serde(default)]
    pub params: BTreeMap<String, ParamSpec>,
}

impl PartDef {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}
