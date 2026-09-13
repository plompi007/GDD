//! ChainWorks entry point. The sim core (fixed-step scheduling, physics
//! wiring — docs/GDD.md §2, §5.5) lives in `chainworks::sim`; this file
//! just builds the app and spawns the current bootstrap scene. See
//! `src/scenes.rs` for what M0/M1 actually renders.

use bevy::prelude::*;

use chainworks::scenes::{spawn_smoke_test_scene, FallingBox};
use chainworks::sim::SimPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ChainWorks — M0/M1 Bootstrap".into(),
                resolution: (900.0_f32, 700.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SimPlugin)
        .insert_resource(ClearColor(Color::srgb_u8(0x14, 0x17, 0x1f)))
        .add_systems(Startup, (spawn_camera, spawn_smoke_test_scene))
        .add_systems(Update, log_fps_and_box_height)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn log_fps_and_box_height(
    time: Res<Time>,
    box_query: Query<&Transform, With<FallingBox>>,
    mut seconds_since_log: Local<f32>,
) {
    *seconds_since_log += time.delta_secs();
    if *seconds_since_log < 1.0 {
        return;
    }
    *seconds_since_log = 0.0;

    let Ok(box_transform) = box_query.single() else {
        return;
    };
    println!(
        "fps: {:.0}  box.y: {:.1}px",
        1.0 / time.delta_secs().max(1e-6),
        box_transform.translation.y
    );
}
