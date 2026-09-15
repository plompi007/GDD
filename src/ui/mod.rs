//! The UI shell (docs/GDD.md §3.1/§5.5, M6): HUD, parts bin, and the
//! design tokens both draw from. See each submodule's docs for exact
//! scope — inspector/rotate/flip/rope-tool chrome (§3.4/§3.5) and the
//! shader-based "deep flat" visual system (§3.9.3) are tracked
//! separately, not part of this milestone.

pub mod editor_tools;
pub mod hud;
pub mod parts_bin;
pub mod tokens;

use bevy::prelude::*;

pub struct UiShellPlugin;

impl Plugin for UiShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            hud::HudPlugin,
            parts_bin::PartsBinPlugin,
            editor_tools::EditorToolsPlugin,
        ));
    }
}
