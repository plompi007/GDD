//! docs/GDD.md §3.9.6's "Juice — רשימת תגובות חובה": concrete game-feel
//! reactions a player action gets *before* any text confirms it (that
//! section's own "עיקרון-על"). Headless-safe (`Mesh2d`/`ColorMaterial`/
//! `bevy_ui::Node` only — both `Assets<Mesh>`/`Assets<ColorMaterial>` are
//! already `init_resource`'d in `PartsPlugin` for exactly this reason, see
//! that plugin's own docs), so this is tested the same way as everything
//! else in this codebase — unlike `render::PartArtPlugin`/`render_fx`'s
//! Gizmos-based connection lines, which genuinely need the real render
//! pipeline and can't be.
//!
//! Scope for this pass (docs/GDD.md's own status note has the full
//! breakdown): placement bounce+flash, the win-celebration pulse, and a
//! fail fade — the table's other rows (confetti, per-material
//! desaturation, squash-stretch on breakage, HUD focus-mode dimming) are
//! out of scope here, either because they need a dependency this project
//! doesn't have yet (`rand`, sanctioned for render-layer use by CLAUDE.md
//! rule 2 but not worth pulling in for one confetti burst) or because
//! doing them well needs per-material color grading this pass doesn't
//! attempt. The fail reaction here is a dark fade-in overlay, not true
//! per-pixel desaturation — a deliberate, cheaper stand-in.

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::level_load::LevelEntity;
use crate::sim::FIXED_DT;
use crate::ui::tokens::palette;

fn ticks_for_ms(ms: f32) -> u32 {
    ((ms / 1000.0) / FIXED_DT).max(1.0) as u32
}

/// docs/GDD.md §3.9.1's `MOTION.snappy` (160ms) applied to a placed part's
/// own scale — a single 1.0→1.08→1.0 hump (a `sin(t·π)` curve), a cheap
/// stand-in for the spec's spring/overshoot easing without pulling in a
/// full tweening crate for one curve.
#[derive(Component)]
pub struct JuiceBounce {
    ticks: u32,
    total_ticks: u32,
}

const BOUNCE_MS: f32 = 160.0;
const BOUNCE_PEAK: f32 = 0.08;

impl Default for JuiceBounce {
    fn default() -> Self {
        JuiceBounce { ticks: 0, total_ticks: ticks_for_ms(BOUNCE_MS) }
    }
}

fn advance_bounce(mut commands: Commands, mut bounces: Query<(Entity, &mut JuiceBounce, &mut Transform)>) {
    for (entity, mut bounce, mut transform) in &mut bounces {
        bounce.ticks += 1;
        if bounce.ticks >= bounce.total_ticks {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<JuiceBounce>();
            continue;
        }
        let t = bounce.ticks as f32 / bounce.total_ticks as f32;
        let scale = 1.0 + BOUNCE_PEAK * (t * std::f32::consts::PI).sin();
        transform.scale = Vec3::splat(scale);
    }
}

/// A short-lived expanding, fading flash — §3.9.6's "טבעת אור קצרה" on
/// placement, and the win-celebration pulse, both reuse this exact
/// component/system with different starting radius/color/duration. A
/// filled circle rather than a literal ring outline (simpler mesh, same
/// "flash of light" read at the sizes this renders at). `pub` so
/// integration tests can assert one exists/has cleared, same reasoning as
/// every other test-visible marker component in this codebase.
#[derive(Component)]
pub struct Flash {
    ticks: u32,
    total_ticks: u32,
    start_radius: f32,
    end_radius: f32,
    material: Handle<ColorMaterial>,
    base_alpha: f32,
}

/// Groups [`spawn_flash`]'s shape/timing parameters — keeps that function
/// under clippy's argument-count lint without hiding real complexity
/// behind an `#[allow]` (unlike the Bevy systems elsewhere in this
/// codebase that legitimately can't shed ECS params, this is a plain
/// function of my own design, so reducing it for real is the right fix).
pub struct FlashSpec {
    pub color: Color,
    pub start_radius: f32,
    pub end_radius: f32,
    pub duration_ms: f32,
}

pub fn spawn_flash(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    spec: FlashSpec,
) {
    let material = materials.add(ColorMaterial::from(spec.color));
    commands.spawn((
        Flash {
            ticks: 0,
            total_ticks: ticks_for_ms(spec.duration_ms),
            start_radius: spec.start_radius,
            end_radius: spec.end_radius,
            material: material.clone(),
            base_alpha: spec.color.alpha(),
        },
        Mesh2d(meshes.add(Circle::new(spec.start_radius.max(0.01)))),
        MeshMaterial2d(material),
        Transform::from_translation(position.extend(5.0)),
    ));
}

fn advance_flash(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut flashes: Query<(Entity, &mut Flash, &mut Transform)>,
) {
    for (entity, mut flash, mut transform) in &mut flashes {
        flash.ticks += 1;
        if flash.ticks >= flash.total_ticks {
            commands.entity(entity).despawn();
            continue;
        }
        let t = flash.ticks as f32 / flash.total_ticks as f32;
        let radius = flash.start_radius + (flash.end_radius - flash.start_radius) * t;
        transform.scale = Vec3::splat(radius / flash.start_radius.max(0.01));
        if let Some(mat) = materials.get_mut(&flash.material) {
            mat.color.set_alpha(flash.base_alpha * (1.0 - t));
        }
    }
}

/// docs/GDD.md §3.9.6: "כל הרכיבים שבפתרון מהבהבים פעימת אור אחת
/// מסונכרנת" — simplified here to every currently-spawned `LevelEntity`
/// (not just the ones in the specific solution path actually used), all
/// flashing the same success-colored pulse at the same instant.
const CELEBRATE_MS: f32 = 600.0;

fn on_solved(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    parts: Query<&Transform, With<LevelEntity>>,
) {
    for transform in &parts {
        let position = transform.translation.truncate();
        spawn_flash(
            &mut commands,
            &mut meshes,
            &mut materials,
            position,
            FlashSpec { color: palette::SUCCESS, start_radius: 10.0, end_radius: 40.0, duration_ms: CELEBRATE_MS },
        );
    }
}

/// docs/GDD.md §3.9.6: "fade-to-desaturate של הסצנה (400ms)" — approximated
/// here as a dark overlay fading in over the whole viewport (a `bevy_ui`
/// `Node`, so it sits above every world-space sprite regardless of camera
/// position) rather than true per-material desaturation. Cleared the next
/// time `Edit` is entered (a reset or loading a new level).
const FAIL_FADE_MS: f32 = 400.0;
const FAIL_OVERLAY_MAX_ALPHA: f32 = 0.55;

/// `pub` for the same reason as [`Flash`] — integration tests assert this
/// appears on `Failed` and clears on the next `Edit`.
#[derive(Component)]
pub struct FailOverlay {
    ticks: u32,
    total_ticks: u32,
}

fn spawn_fail_overlay(mut commands: Commands) {
    commands.spawn((
        FailOverlay { ticks: 0, total_ticks: ticks_for_ms(FAIL_FADE_MS) },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            right: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        BackgroundColor(palette::BG_CANVAS.with_alpha(0.0)),
        GlobalZIndex(1000),
    ));
}

fn advance_fail_overlay(mut overlays: Query<(&mut FailOverlay, &mut BackgroundColor)>) {
    for (mut overlay, mut background) in &mut overlays {
        if overlay.ticks >= overlay.total_ticks {
            continue;
        }
        overlay.ticks += 1;
        let t = overlay.ticks as f32 / overlay.total_ticks as f32;
        background.0.set_alpha(FAIL_OVERLAY_MAX_ALPHA * t);
    }
}

fn clear_fail_overlay(mut commands: Commands, overlays: Query<Entity, With<FailOverlay>>) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}

pub struct JuicePlugin;

impl Plugin for JuicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (advance_bounce, advance_flash, advance_fail_overlay))
            .add_systems(OnEnter(GameState::Solved), on_solved)
            .add_systems(OnEnter(GameState::Failed), spawn_fail_overlay)
            .add_systems(OnEnter(GameState::Edit), clear_fail_overlay);
    }
}
