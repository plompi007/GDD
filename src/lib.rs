//! ChainWorks library crate. `src/main.rs` is a thin binary on top of this;
//! `tests/*.rs` link against it directly so integration tests (determinism,
//! level-solvability, later golden-replay/solutions tests) exercise the
//! exact same code paths as the real game, not a reimplementation.

pub mod app;
pub mod game_state;
pub mod level_file_format;
pub mod level_load;
pub mod math;
pub mod part;
pub mod parts;
pub mod scenes;
pub mod sim;
pub mod win_conditions;
