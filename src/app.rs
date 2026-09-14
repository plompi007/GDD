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
use crate::level_load::{EditorState, LevelPlugin};
use crate::parts::PartsPlugin;
use crate::sim::SimPlugin;
use crate::ui::UiShellPlugin;
use crate::win_conditions::WinConditionsPlugin;

const DEMO_LEVEL_JSON: &str = include_str!("../levels/A/lvl_a01_first_roll.json");

/// Every level built so far (docs/GDD.md §5.8's chapters A-D), in playing
/// order — cyclable in-app via [`cycle_level_system`] since there's no
/// `LevelSelect` UI yet (M6's own to-do list). Keep this in sync with
/// `levels/**/*.json` as new ones are added; `parts::tests` already
/// guards the part registry the same way, this is content's equivalent.
const ALL_LEVELS: &[&str] = &[
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
    include_str!("../levels/B/lvl_b01_chain_cut.json"),
    include_str!("../levels/B/lvl_b02_double_fuse.json"),
    include_str!("../levels/B/lvl_b03_break_through.json"),
    include_str!("../levels/B/lvl_b04_launch_switch.json"),
    include_str!("../levels/B/lvl_b05_windswept.json"),
    include_str!("../levels/B/lvl_b06_triple_mesh.json"),
    include_str!("../levels/C/lvl_c01_burn_through.json"),
    include_str!("../levels/C/lvl_c02_late_arrival.json"),
    include_str!("../levels/C/lvl_c03_clear_the_way.json"),
];

/// Index into [`ALL_LEVELS`] of the level currently loaded into
/// [`EditorState`] — advanced by [`cycle_level_system`].
#[derive(Resource, Default)]
struct LevelIndex(usize);

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
        .init_resource::<LevelIndex>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, (dev_controls, cycle_level_system, log_state_and_fps))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    println!(
        "ChainWorks desktop controls: Space = play/pause, R = reset, N/P = next/previous level ({} loaded)",
        ALL_LEVELS.len()
    );
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

/// N/P cycle forward/back through every level in [`ALL_LEVELS`] — a
/// stand-in for the `LevelSelect` screen docs/GDD.md §3.9 still calls for
/// (M6's own to-do list), so every level built for M8 is actually
/// reachable in the running app/APK, not just from `cargo test`.
fn cycle_level_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut index: ResMut<LevelIndex>,
    mut editor_state: ResMut<EditorState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let delta: i32 = if keys.just_pressed(KeyCode::KeyN) {
        1
    } else if keys.just_pressed(KeyCode::KeyP) {
        -1
    } else {
        return;
    };
    let len = ALL_LEVELS.len() as i32;
    index.0 = ((index.0 as i32 + delta).rem_euclid(len)) as usize;
    let level: LevelFile = serde_json::from_str(ALL_LEVELS[index.0])
        .unwrap_or_else(|e| panic!("invalid level JSON in ALL_LEVELS[{}]: {e}", index.0));
    println!("level -> {} ({})", level.id, level.title);
    editor_state.level = level;
    next_state.set(GameState::Edit);
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
