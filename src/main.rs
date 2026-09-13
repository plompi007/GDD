//! M0 Bootstrap smoke test (docs/GDD.md §5.8): prove that Bevy (ECS/rendering)
//! and bevy_rapier2d (physics) load and run together, at 60fps, with a box
//! falling onto a floor. The real Sim / EnergyGraph / Part architecture
//! (mirroring OpenTIM's `part.rs` / `level_file_format.rs` / `atmosphere.rs`
//! shape as Bevy components + systems — see CLAUDE.md) arrives in M1+. This
//! file is intentionally a throwaway wiring check, not the sim core.
//!
//! Engine: Bevy (not `nannou`, the v1.2 choice) — chosen specifically because
//! Bevy natively targets Android (and iOS/desktop/web) from this same
//! codebase, which `nannou` cannot do. See docs/GDD.md §2.1a.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// How many pixels on screen equal one meter in the physics solver. With this
/// set, `Collider` sizes and `Transform` positions are specified directly in
/// pixels — bevy_rapier2d handles the meter conversion internally.
const PIXELS_PER_METER: f32 = 32.0;
const FIXED_DT: f32 = 1.0 / 120.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ChainWorks — M0 Bootstrap".into(),
                resolution: (900.0_f32, 700.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        // Fixed timestep, not Bevy's default Variable mode — required for the
        // determinism guarantees in docs/GDD.md §2.6 (golden replay tests).
        .insert_resource(TimestepMode::Fixed {
            dt: FIXED_DT,
            substeps: 1,
        })
        .insert_resource(ClearColor(Color::srgb_u8(0x14, 0x17, 0x1f)))
        .add_systems(Startup, setup)
        .add_systems(Update, log_fps_and_box_height)
        .run();
}

#[derive(Component)]
struct FallingBox;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Ground: a wide fixed slab. Bevy 2D uses standard +y-up, matching the
    // nannou-era convention note in docs/GDD.md §2.2 (unlike the original
    // Pixi.js-based v1.1 draft, which was +y-down).
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(640.0, 16.0),
        Transform::from_xyz(0.0, -192.0, 0.0),
        Sprite::from_color(Color::srgb_u8(0x2a, 0x30, 0x40), Vec2::new(1280.0, 32.0)),
    ));

    // Falling box.
    commands.spawn((
        FallingBox,
        RigidBody::Dynamic,
        Collider::cuboid(16.0, 16.0),
        Restitution::new(0.25),
        Friction::new(0.5),
        Transform::from_xyz(0.0, 192.0, 0.0),
        Sprite::from_color(Color::srgb_u8(0x6c, 0x8c, 0xff), Vec2::new(32.0, 32.0)),
    ));
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
