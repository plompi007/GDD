//! Every level built so far (docs/GDD.md §5.8's chapters A-D), and the
//! shared logic for switching `EditorState` to one of them — used by both
//! `app.rs`'s keyboard shortcut (N/P) and `ui::hud`'s touch buttons, since
//! there's no `LevelSelect` screen yet (M6's own to-do list) and this is
//! how the growing M8 content stays actually playable, on desktop and
//! Android alike, in the meantime.
//!
//! Keep this in sync with `levels/**/*.json` as new ones are added;
//! `parts::tests` already guards the part registry the same way, this is
//! content's equivalent.

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::level_file_format::LevelFile;
use crate::level_load::EditorState;

pub const ALL_LEVELS: &[&str] = &[
    include_str!("../levels/A/lvl_a01_first_roll.json"),
    include_str!("../levels/A/lvl_a02_bounce.json"),
    include_str!("../levels/A/lvl_a03_pulley_lift.json"),
    include_str!("../levels/A/lvl_a04_catapult.json"),
    include_str!("../levels/A/lvl_a05_tailwind.json"),
    include_str!("../levels/A/lvl_a06_switch.json"),
    include_str!("../levels/A/lvl_a07_gear_train.json"),
    include_str!("../levels/A/lvl_a08_fuse.json"),
    include_str!("../levels/A/lvl_a09_cut.json"),
    include_str!("../levels/A/lvl_a10_punch.json"),
    include_str!("../levels/A/lvl_a11_handle_with_care.json"),
    include_str!("../levels/A/lvl_a12_clear_the_air.json"),
    include_str!("../levels/B/lvl_b01_chain_cut.json"),
    include_str!("../levels/B/lvl_b02_double_fuse.json"),
    include_str!("../levels/B/lvl_b03_break_through.json"),
    include_str!("../levels/B/lvl_b04_launch_switch.json"),
    include_str!("../levels/B/lvl_b05_windswept.json"),
    include_str!("../levels/B/lvl_b06_triple_mesh.json"),
    include_str!("../levels/B/lvl_b07_bounce_relay.json"),
    include_str!("../levels/B/lvl_b08_launch_lever.json"),
    include_str!("../levels/B/lvl_b09_blow_the_switch.json"),
    include_str!("../levels/B/lvl_b10_four_in_a_row.json"),
    include_str!("../levels/B/lvl_b11_snip_and_drop.json"),
    include_str!("../levels/B/lvl_b12_blast_the_glass.json"),
    include_str!("../levels/B/lvl_b13_chain_reaction.json"),
    include_str!("../levels/B/lvl_b14_fragile_cargo.json"),
    include_str!("../levels/B/lvl_b15_ride_to_the_zone.json"),
    include_str!("../levels/B/lvl_b16_rise_and_shine.json"),
    include_str!("../levels/C/lvl_c01_burn_through.json"),
    include_str!("../levels/C/lvl_c02_late_arrival.json"),
    include_str!("../levels/C/lvl_c03_clear_the_way.json"),
    include_str!("../levels/C/lvl_c04_just_in_time.json"),
    include_str!("../levels/C/lvl_c05_needle_point.json"),
    include_str!("../levels/C/lvl_c06_twin_candles.json"),
    include_str!("../levels/C/lvl_c07_race_the_belt.json"),
    include_str!("../levels/C/lvl_c08_burn_and_deliver.json"),
    include_str!("../levels/C/lvl_c09_punch_the_flame.json"),
    include_str!("../levels/C/lvl_c10_no_time_to_spare.json"),
    include_str!("../levels/C/lvl_c11_two_ways_to_cut.json"),
    include_str!("../levels/C/lvl_c12_wind_up.json"),
    include_str!("../levels/C/lvl_c13_hurry_gently.json"),
    include_str!("../levels/C/lvl_c14_slow_release.json"),
    include_str!("../levels/C/lvl_c15_careful_aim.json"),
    include_str!("../levels/C/lvl_c16_one_careful_run.json"),
    include_str!("../levels/D/lvl_d01_chain_of_command.json"),
    include_str!("../levels/D/lvl_d02_signal_chain.json"),
    include_str!("../levels/D/lvl_d03_bounce_and_cut.json"),
    include_str!("../levels/D/lvl_d04_wind_and_fire.json"),
    include_str!("../levels/D/lvl_d05_fan_the_flame.json"),
    include_str!("../levels/D/lvl_d06_bounce_and_blow.json"),
    include_str!("../levels/D/lvl_d07_gentle_command.json"),
    include_str!("../levels/D/lvl_d08_punch_the_line.json"),
    include_str!("../levels/D/lvl_d09_high_release.json"),
    include_str!("../levels/D/lvl_d10_twin_flame_relay.json"),
    include_str!("../levels/D/lvl_d11_two_cuts_one_belt.json"),
    include_str!("../levels/D/lvl_d12_clear_and_carry.json"),
    include_str!("../levels/D/lvl_d13_wind_signal.json"),
    include_str!("../levels/D/lvl_d14_blast_and_belt.json"),
    include_str!("../levels/D/lvl_d15_four_gears_gently.json"),
    include_str!("../levels/D/lvl_d16_grand_finale.json"),
];

/// M9's free-build mode (docs/GDD.md's Sandbox spec) — deliberately *not*
/// part of [`ALL_LEVELS`]: it isn't one of the 60 story levels N/P cycles
/// through, it's a separate entry point (see [`enter_sandbox`]). Empty
/// canvas, no win/fail conditions, and one `unlimited: true` `partsBin`
/// entry per registered part type (`parts::tests` guards that list
/// against drift the same way it already guards `PartRegistry` itself).
pub const SANDBOX_LEVEL: &str = include_str!("../levels/sandbox.json");

/// Loads [`SANDBOX_LEVEL`] into `EditorState` and enters `GameState::Edit`
/// — same shape as [`cycle_level`], just a fixed destination instead of
/// stepping through an index, since Sandbox is one level, not sixty.
pub fn enter_sandbox(editor_state: &mut EditorState, next_state: &mut NextState<GameState>) {
    let level: LevelFile = serde_json::from_str(SANDBOX_LEVEL)
        .unwrap_or_else(|e| panic!("invalid SANDBOX_LEVEL JSON: {e}"));
    println!("level -> {} ({})", level.id, level.title);
    editor_state.level = level;
    next_state.set(GameState::Edit);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::parts::build_registry;

    /// Guards `levels/sandbox.json`'s `partsBin` against drift the same
    /// way `parts::tests` already guards `PartRegistry` itself against
    /// `PART_JSON` — every registered part type must have exactly one
    /// `unlimited: true` entry, and there must be no leftover entries
    /// for a part type that no longer exists.
    #[test]
    fn sandbox_parts_bin_has_exactly_one_unlimited_entry_per_registered_part_type() {
        let registry = build_registry();
        let level: LevelFile =
            serde_json::from_str(SANDBOX_LEVEL).expect("SANDBOX_LEVEL must parse");

        let registered: BTreeSet<&str> = registry.part_types().collect();
        let in_bin: BTreeSet<&str> = level
            .parts_bin
            .iter()
            .map(|entry| entry.part_type.as_str())
            .collect();
        assert_eq!(
            registered, in_bin,
            "sandbox.json's partsBin must list exactly the registered part types, no more, no less"
        );
        assert_eq!(
            level.parts_bin.len(),
            in_bin.len(),
            "no part type should appear twice in sandbox.json's partsBin"
        );
        assert!(
            level.parts_bin.iter().all(|entry| entry.unlimited),
            "every sandbox.json partsBin entry must be unlimited"
        );
    }

    #[test]
    fn sandbox_level_has_no_win_or_fail_conditions() {
        let level: LevelFile =
            serde_json::from_str(SANDBOX_LEVEL).expect("SANDBOX_LEVEL must parse");
        assert!(level.win_conditions.is_empty());
        assert!(level.fail_conditions.is_empty());
    }
}

/// Index into [`ALL_LEVELS`] of the level currently loaded into
/// [`EditorState`] — advanced by [`cycle_level`].
#[derive(Resource, Default)]
pub struct LevelIndex(pub usize);

/// Swaps `EditorState.level` to `ALL_LEVELS[index + delta]` (wrapping) and
/// re-enters `GameState::Edit`, exactly like a reset — the level itself is
/// the only thing that changed, not how loading/resetting works.
pub fn cycle_level(
    delta: i32,
    index: &mut LevelIndex,
    editor_state: &mut EditorState,
    next_state: &mut NextState<GameState>,
) {
    let len = ALL_LEVELS.len() as i32;
    index.0 = ((index.0 as i32 + delta).rem_euclid(len)) as usize;
    let level: LevelFile = serde_json::from_str(ALL_LEVELS[index.0])
        .unwrap_or_else(|e| panic!("invalid level JSON in ALL_LEVELS[{}]: {e}", index.0));
    println!("level -> {} ({})", level.id, level.title);
    editor_state.level = level;
    next_state.set(GameState::Edit);
}
