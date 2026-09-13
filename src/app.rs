//! The actual app: builds the `App` and loads the current demo level.
//!
//! This lives in the library crate (not `src/main.rs`) because Android
//! needs `#[bevy_main]` on a function called `main` compiled into the
//! **cdylib** — see `Cargo.toml`'s `[lib]` section and docs/GDD.md §7.5.
//! `src/main.rs` (the desktop binary) just calls [`main`] normally.

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::input::{EditorInputPlugin, PointerAdapterPlugin};
use crate::level_file_format::LevelFile;
use crate::level_load::LevelPlugin;
use crate::parts::PartsPlugin;
use crate::sim::SimPlugin;
use crate::ui::UiShellPlugin;
use crate::win_conditions::WinConditionsPlugin;

const DEMO_LEVEL_JSON: &str = include_str!("../levels/A/lvl_a01_first_roll.json");

#[bevy_main]
pub fn main() {
    let level: LevelFile =
        serde_json::from_str(DEMO_LEVEL_JSON).expect("demo level JSON must be valid");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ChainWorks — M3.5 Bootstrap".into(),
                resolution: (900.0_f32, 700.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(PointerAdapterPlugin)
        .add_plugins(EditorInputPlugin)
        .add_plugins(UiShellPlugin)
        .add_plugins(WinConditionsPlugin)
        .insert_resource(ClearColor(Color::srgb_u8(0x14, 0x17, 0x1f)))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (dev_controls, log_state_and_fps))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Desktop power-user shortcuts alongside `ui::hud`'s real Play/Stop/Reset
/// button (M6) — Space mirrors the main button's own cycle, R force-resets
/// from any state. Not the only way to control the game anymore, just a
/// convenience; there's no touch equivalent (a phone has no keyboard),
/// which is fine now that the real button covers both.
fn dev_controls(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Edit);
        return;
    }
    if keys.just_pressed(KeyCode::Space) {
        match state.get() {
            GameState::Edit => next_state.set(GameState::Running),
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
            GameState::Solved | GameState::Failed => next_state.set(GameState::Edit),
        }
    }
}

fn log_state_and_fps(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut last_state: Local<Option<GameState>>,
    mut seconds_since_log: Local<f32>,
) {
    if last_state.as_ref() != Some(state.get()) {
        println!("state -> {:?}", state.get());
        *last_state = Some(*state.get());
    }

    *seconds_since_log += time.delta_secs();
    if *seconds_since_log < 1.0 {
        return;
    }
    *seconds_since_log = 0.0;
    println!(
        "fps: {:.0}  state: {:?}",
        1.0 / time.delta_secs().max(1e-6),
        state.get()
    );
}
