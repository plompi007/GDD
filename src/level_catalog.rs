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
    include_str!("../levels/D/lvl_d01_chain_of_command.json"),
    include_str!("../levels/D/lvl_d02_signal_chain.json"),
    include_str!("../levels/D/lvl_d03_bounce_and_cut.json"),
    include_str!("../levels/D/lvl_d04_wind_and_fire.json"),
];

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
