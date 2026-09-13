//! The game state machine (docs/GDD.md §2.3). A thin wrapper around Bevy's
//! built-in `States` support — `OnEnter(GameState::Edit)` is what actually
//! does the "reset" (docs/GDD.md §2.5: rebuild from `EditorState`, never
//! rewind physics), wired in `level_load.rs`.

use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Edit,
    Running,
    Paused,
    Solved,
    Failed,
}
