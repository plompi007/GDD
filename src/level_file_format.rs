//! Serde structs matching `schemas/level.schema.json` (docs/GDD.md §5.1),
//! mirroring OpenTIM's `level_file_format.rs` responsibility. This is the
//! *only* place that knows what a level JSON file looks like — loading
//! (`level_load.rs`) and win-condition evaluation (`win_conditions.rs`)
//! both build on these types instead of touching JSON directly.
//!
//! `Serialize` (docs/GDD.md's M9/Sandbox spec) exists so a `LevelFile`
//! built live in Sandbox mode can be written back out through the exact
//! same schema every official level already parses — not a second format.

use serde::{Deserialize, Serialize};

use crate::part::EnergyType;

fn default_world_width() -> f32 {
    1600.0
}
fn default_world_height() -> f32 {
    1200.0
}
fn default_gravity_y() -> f32 {
    9.81
}
fn default_time_limit_sec() -> f32 {
    90.0
}
fn default_theme() -> String {
    "prism_foundry".to_string()
}
fn default_hold_ms() -> f32 {
    500.0
}
fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chapter {
    #[serde(rename = "A_FOUNDATIONS")]
    AFoundations,
    #[serde(rename = "B_COMBOS")]
    BCombos,
    #[serde(rename = "C_TIMING")]
    CTiming,
    #[serde(rename = "D_MASTER")]
    DMaster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldConfig {
    #[serde(default = "default_world_width")]
    pub width: f32,
    #[serde(default = "default_world_height")]
    pub height: f32,
    #[serde(default = "default_gravity_y")]
    pub gravity_y: f32,
    #[serde(default = "default_time_limit_sec")]
    pub time_limit_sec: f32,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RotaryDirection {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwitchMode {
    Toggle,
    Momentary,
}

/// Per-instance overrides for a part's tunable parameters (docs/GDD.md
/// §5.1 `placedPart.params`). Every field is optional: which ones apply
/// depends on the part type, validated against `PartDef.params` at load
/// time (M4+, once parts with real params exist) rather than here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedPartParams {
    pub rpm: Option<f32>,
    pub direction: Option<RotaryDirection>,
    pub power: Option<f32>,
    pub angle: Option<f32>,
    pub starts_on: Option<bool>,
    pub starts_lit: Option<bool>,
    pub length: Option<f32>,
    pub charge: Option<i32>,
    pub mode: Option<SwitchMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedPart {
    pub id: String,
    pub part_type: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub params: PlacedPartParams,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConnectionKind {
    Rope,
    Belt,
    Wire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRef {
    pub part_id: String,
    #[serde(default)]
    pub anchor_idx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub kind: ConnectionKind,
    pub from: AnchorRef,
    pub to: AnchorRef,
    pub max_length: Option<f32>,
    #[serde(default)]
    pub routed_through: Vec<AnchorRef>,
}

/// One win/fail condition (docs/GDD.md §5.1 `definitions.condition` — the
/// schema uses a single flat type for both `winConditions` and
/// `failConditions`, so this enum covers every variant from both).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum Condition {
    Contained {
        subject_tag: String,
        container_id: String,
        #[serde(default = "default_hold_ms")]
        hold_ms: f32,
    },
    ReachedZone {
        subject_tag: String,
        zone_id: String,
    },
    EnergyState {
        node_id: String,
        energy: EnergyType,
        active: bool,
        #[serde(default = "default_hold_ms")]
        hold_ms: f32,
    },
    Destroyed {
        target_id: String,
    },
    Timeout,
    SubjectDestroyed {
        subject_tag: String,
    },
    LeftBounds {
        subject_tag: String,
    },
    AllOf {
        conditions: Vec<Condition>,
    },
    AnyOf {
        conditions: Vec<Condition>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartsBinEntry {
    pub part_type: String,
    pub count: u32,
    #[serde(default)]
    pub locked_params: Vec<String>,
    /// M9/Sandbox: when `true`, `count` is never checked or decremented —
    /// `input.rs`'s bin-drag systems place from this entry regardless of
    /// how many times it's been used. `false` (the default, so every
    /// existing story level's `partsBin` entries are unaffected) keeps
    /// the ordinary "consumed on placement" behavior.
    #[serde(default)]
    pub unlimited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Solution {
    pub label: String,
    pub parts: Vec<PlacedPart>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    pub expected_solve_tick: Option<u32>,
}

/// One `levels/**/*.json` file, deserialized as-is (docs/GDD.md §5.1/§5.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelFile {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub chapter: Option<Chapter>,
    pub order: Option<u32>,
    pub difficulty: Option<u8>,
    pub goal_text: Option<String>,
    #[serde(default)]
    pub hints: Vec<String>,
    pub world: WorldConfig,
    #[serde(default)]
    pub fixed_parts: Vec<PlacedPart>,
    #[serde(default)]
    pub preplaced_parts: Vec<PlacedPart>,
    #[serde(default)]
    pub parts_bin: Vec<PartsBinEntry>,
    #[serde(default)]
    pub connections: Vec<Connection>,
    pub win_conditions: Vec<Condition>,
    #[serde(default)]
    pub fail_conditions: Vec<Condition>,
    #[serde(default)]
    pub solutions: Vec<Solution>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_example_level() {
        // docs/GDD.md §5.2's own worked example — if this ever fails to
        // parse, either this struct or the GDD's schema doc has drifted.
        let json = include_str!("../levels/A/lvl_a03_pulley_lift_example.json");
        let level: LevelFile = serde_json::from_str(json).expect("must parse");
        assert_eq!(level.id, "lvl_a03_pulley_lift");
        assert_eq!(level.win_conditions.len(), 1);
        assert_eq!(level.solutions.len(), 2);
    }

    /// M9/Sandbox's foundation: a level round-tripped through
    /// `serde_json::to_string`/`from_str` must parse back into a
    /// `LevelFile` that produces the *same* JSON again — proving `Serialize`
    /// really does write the same schema `Deserialize` already reads,
    /// not a drifted lookalike. Runs this against every one of the 60
    /// built-in levels (`ALL_LEVELS`), not just one hand-picked example,
    /// since a save bug might only show up on a level using a field the
    /// simple example doesn't (e.g. `Condition::AllOf`, `routedThrough`).
    #[test]
    fn every_official_level_round_trips_through_save_and_load() {
        for (index, json) in crate::level_catalog::ALL_LEVELS.iter().enumerate() {
            let original: LevelFile = serde_json::from_str(json)
                .unwrap_or_else(|e| panic!("ALL_LEVELS[{index}] failed to parse: {e}"));
            let saved = serde_json::to_string(&original)
                .unwrap_or_else(|e| panic!("ALL_LEVELS[{index}] failed to serialize: {e}"));
            let reloaded: LevelFile = serde_json::from_str(&saved)
                .unwrap_or_else(|e| panic!("ALL_LEVELS[{index}] failed to re-parse: {e}"));
            let saved_again = serde_json::to_string(&reloaded)
                .unwrap_or_else(|e| panic!("ALL_LEVELS[{index}] failed to re-serialize: {e}"));
            assert_eq!(
                saved, saved_again,
                "ALL_LEVELS[{index}] ({}) changed on a second save/load round trip",
                original.id
            );
        }
    }
}
