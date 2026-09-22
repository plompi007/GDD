//! docs/GDD.md §3.9.2's "קידוד צבע לפי סוג אנרגיה" for the two connection
//! kinds that were, until this pass, drawn as **nothing at all**:
//! `RopeConnection`/`BeltConnection` have always been physics-only,
//! component-only entities (see their own doc comments in
//! `rope_network.rs`/`gear_train.rs`) — a real, previously-undiscovered
//! gap where a core mechanic (every rope, pulley, and belt in every level)
//! had zero visual representation whatsoever, not a style problem.
//!
//! Uses `Gizmos` (immediate-mode, redrawn fresh every frame from live
//! `Transform`s — a taut rope or a spinning belt just naturally looks
//! right with no extra reveal/interpolation state to keep in sync).
//! `Gizmos` needs `GizmoPlugin`, part of `DefaultPlugins`'s render stack —
//! not available under this project's `MinimalPlugins`-based headless
//! tests, so this whole module is real-app-only, registered only by
//! `app.rs`, exactly like `render::PartArtPlugin`. Verified visually
//! against the real binary under Xvfb, not by a headless test — see this
//! module's own verification note in docs/GDD.md.
//!
//! Scope: a solid core line plus one wider/fainter line behind it (a cheap
//! stand-in for real bloom/glow, which would need a custom shader or an
//! HDR + `Bloom` post-process pass this pass doesn't attempt) — not
//! §3.9.2's animated flowing-particle look, and not modulated by live
//! tension/RPM. Both are documented follow-ups, not silently dropped.

use bevy::prelude::*;

use crate::energy_graph::WireConnection;
use crate::gear_train::{BeltConnection, GearNode};
use crate::part::EnergyType;
use crate::rope_network::{rope_world_points, RopeConnection};
use crate::ui::tokens::energy_color;

/// Draws a soft-glow line: a wide, low-alpha line behind a thin bright
/// core — Gizmos has no per-call line-width control, so "width" here is
/// just two overlapping draws offset along the segment's own normal.
fn glow_line(gizmos: &mut Gizmos, a: Vec2, b: Vec2, color: Color) {
    let dir = (b - a).normalize_or_zero();
    let normal = Vec2::new(-dir.y, dir.x);
    let glow = color.with_alpha(0.22);
    for offset in [-2.0, -1.0, 1.0, 2.0] {
        gizmos.line_2d(a + normal * offset, b + normal * offset, glow);
    }
    gizmos.line_2d(a, b, color);
}

fn draw_ropes(mut gizmos: Gizmos, ropes: Query<&RopeConnection>, transforms: Query<&Transform>) {
    let color = energy_color(EnergyType::Tension);
    for rope in &ropes {
        let Some(points) = rope_world_points(rope, &transforms) else {
            continue;
        };
        for segment in points.windows(2) {
            glow_line(&mut gizmos, segment[0], segment[1], color);
        }
    }
}

fn draw_belts(
    mut gizmos: Gizmos,
    belts: Query<&BeltConnection>,
    nodes: Query<(&Transform, Option<&GearNode>)>,
) {
    let color = energy_color(EnergyType::Rotary);
    for belt in &belts {
        let (Ok((from_t, from_gear)), Ok((to_t, to_gear))) =
            (nodes.get(belt.from), nodes.get(belt.to))
        else {
            continue;
        };
        let from_pos = from_t.translation.truncate();
        let to_pos = to_t.translation.truncate();
        let dir = (to_pos - from_pos).normalize_or_zero();
        // Start/end the line at each pulley's own rim, not its center, so
        // it reads as a belt wrapped around the wheel rather than a line
        // drawn straight through it.
        let start = from_pos + dir * from_gear.map_or(0.0, |g| g.radius);
        let end = to_pos - dir * to_gear.map_or(0.0, |g| g.radius);
        glow_line(&mut gizmos, start, end, color);
    }
}

/// Straight line, center to center — unlike a belt, a wire doesn't wrap
/// around anything to start/end at a rim offset from, and unlike a rope
/// it has no pulleys to route through (`WireConnection` only ever has the
/// two endpoints `energy_graph.rs`'s own doc comment describes).
fn draw_wires(mut gizmos: Gizmos, wires: Query<&WireConnection>, transforms: Query<&Transform>) {
    let color = energy_color(EnergyType::Electric);
    for wire in &wires {
        let (Ok(from_t), Ok(to_t)) = (transforms.get(wire.from), transforms.get(wire.to)) else {
            continue;
        };
        glow_line(&mut gizmos, from_t.translation.truncate(), to_t.translation.truncate(), color);
    }
}

pub struct ConnectionVisualsPlugin;

impl Plugin for ConnectionVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_ropes, draw_belts, draw_wires));
    }
}
