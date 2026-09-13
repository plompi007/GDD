//! ChainWorks entry point. The sim core (fixed-step scheduling, physics
//! wiring — docs/GDD.md §2, §5.5) lives in `chainworks::sim`; part data
//! loading lives in `chainworks::parts`. This file just builds the app and
//! spawns the current bootstrap scene. See `src/scenes.rs` for what M0-M2
//! actually renders.

use bevy::prelude::*;

use chainworks::parts::PartsPlugin;
use chainworks::scenes::{spawn_smoke_test_scene, Tracked};
use chainworks::sim::SimPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ChainWorks — M0-M2 Bootstrap".into(),
                resolution: (900.0_f32, 700.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .insert_resource(ClearColor(Color::srgb_u8(0x14, 0x17, 0x1f)))
        .add_systems(Startup, (spawn_camera, spawn_smoke_test_scene))
        .add_systems(Update, log_fps_and_tracked_height)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn log_fps_and_tracked_height(
    time: Res<Time>,
    tracked_query: Query<&Transform, With<Tracked>>,
    mut seconds_since_log: Local<f32>,
) {
    *seconds_since_log += time.delta_secs();
    if *seconds_since_log < 1.0 {
        return;
    }
    *seconds_since_log = 0.0;

    let Ok(tracked_transform) = tracked_query.single() else {
        return;
    };
    println!(
        "fps: {:.0}  tracked: x={:.1}px y={:.1}px",
        1.0 / time.delta_secs().max(1e-6),
        tracked_transform.translation.x,
        tracked_transform.translation.y,
    );
}
