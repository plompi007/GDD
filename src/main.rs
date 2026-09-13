//! ChainWorks entry point. The sim core (docs/GDD.md §2, §5.5) lives in
//! `chainworks::sim`; part data in `chainworks::parts`; level loading and
//! win conditions in `chainworks::level_load`/`chainworks::win_conditions`.
//! This file just builds the app, loads the current demo level, and adds
//! temporary keyboard controls standing in for the real Play/Reset UI
//! (M6) — Space to run/pause, R to reset.

use bevy::prelude::*;

use chainworks::game_state::GameState;
use chainworks::level_file_format::LevelFile;
use chainworks::level_load::LevelPlugin;
use chainworks::parts::PartsPlugin;
use chainworks::sim::SimPlugin;
use chainworks::win_conditions::WinConditionsPlugin;

const DEMO_LEVEL_JSON: &str = include_str!("../levels/A/lvl_a01_first_roll.json");

fn main() {
    let level: LevelFile =
        serde_json::from_str(DEMO_LEVEL_JSON).expect("demo level JSON must be valid");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ChainWorks — M3 Bootstrap".into(),
                resolution: (900.0_f32, 700.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(WinConditionsPlugin)
        .insert_resource(ClearColor(Color::srgb_u8(0x14, 0x17, 0x1f)))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (dev_controls, log_state_and_fps))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Temporary stand-in for the real Play/Reset UI (M6). Space toggles
/// Running/Edit; R forces a reset back to Edit from any state.
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
