//! Procedural part visuals — the "graphics" pass docs/GDD.md's §3.9 calls
//! for, built entirely from Bevy's own 2D primitive meshes (`Circle`,
//! `Rectangle`, `Triangle2d`), not bitmap/vector art assets. No such
//! assets exist in this project yet, and generating real sprite/SVG art
//! isn't something available in this session — this is code-only
//! geometry, one deliberate step up from `body_factory::placeholder_color`'s
//! flat-rectangle-for-everything baseline, not a replacement for real art
//! whenever that becomes available.
//!
//! [`decorate`] is called once, right after `body_factory::spawn_part`
//! builds a part's physics body + base sprite/mesh — every shape it adds
//! is a pure-cosmetic child entity (`Mesh2d` + `MeshMaterial2d<ColorMaterial>`,
//! no collider, no physics component of any kind), parented to the part's
//! own entity so it moves and rotates with it for free via Bevy's normal
//! `Transform` propagation. A part type with nothing listed in
//! `decorations_for` just keeps its plain flat base shape — decorating
//! every one of the 29 registered types is future polish, not required
//! for this pass to be a real improvement.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::part::PartDef;

const SHADE: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
const HIGHLIGHT: Color = Color::srgba(1.0, 1.0, 1.0, 0.55);
const FLAME: Color = Color::srgb(1.0, 0.65, 0.2);
const WICK: Color = Color::srgb(0.2, 0.13, 0.07);

struct Decoration {
    mesh: Mesh,
    color: Color,
    offset: Vec2,
    rotation_deg: f32,
    /// Stacking order among decorations on the *same* part — higher
    /// draws on top of lower. Independent of the part's own base-shape
    /// z (always 0.0, set by `body_factory::spawn_part`), since these
    /// are child entities with their own local z.
    z: f32,
}

impl Decoration {
    fn new(mesh: impl Into<Mesh>, color: Color, offset: Vec2) -> Self {
        Decoration { mesh: mesh.into(), color, offset, rotation_deg: 0.0, z: 1.0 }
    }

    fn rotated(mut self, degrees: f32) -> Self {
        self.rotation_deg = degrees;
        self
    }

    fn at_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }
}

/// `count` teeth as small outward-pointing rectangles around a circle of
/// `radius`, plus a darker hub circle on top — the shape every gear
/// (`gear_small`/`gear_large`) shares, just parametrized by size.
fn gear_teeth(radius: f32, count: u32, tooth_len: f32, tooth_width: f32, hub_radius: f32) -> Vec<Decoration> {
    let mut decorations: Vec<Decoration> = (0..count)
        .map(|i| {
            let angle = i as f32 * TAU / count as f32;
            let dir = Vec2::from_angle(angle);
            // Half in, half out of the rim so there's no visible gap
            // between tooth and gear body.
            let center_dist = radius + tooth_len / 2.0 - 2.0;
            Decoration::new(Rectangle::new(tooth_len, tooth_width), SHADE, dir * center_dist)
                .rotated(angle.to_degrees())
        })
        .collect();
    decorations.push(Decoration::new(Circle::new(hub_radius), SHADE, Vec2::ZERO).at_z(2.0));
    decorations
}

/// Three blades meeting at a center hub — `fan_blower`.
fn fan_blades() -> Vec<Decoration> {
    let blade_len = 14.0;
    let mut decorations: Vec<Decoration> = (0..3)
        .map(|i| {
            let angle = i as f32 * TAU / 3.0;
            let dir = Vec2::from_angle(angle);
            Decoration::new(Rectangle::new(blade_len, 4.0), SHADE, dir * (blade_len / 2.0))
                .rotated(angle.to_degrees())
        })
        .collect();
    decorations.push(Decoration::new(Circle::new(6.0), SHADE, Vec2::ZERO).at_z(2.0));
    decorations
}

/// A small upward flame + wick above the candle body — `candle`
/// (`data/parts/candle.json`'s own box is 16×24, so its top edge is at
/// local y = 12).
fn candle_flame() -> Vec<Decoration> {
    vec![
        Decoration::new(
            Rectangle::new(2.0, 6.0),
            WICK,
            Vec2::new(0.0, 13.0),
        ),
        Decoration::new(
            Triangle2d::new(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0), Vec2::new(0.0, 10.0)),
            FLAME,
            Vec2::new(0.0, 17.0),
        )
        .at_z(2.0),
    ]
}

/// A small raised button on `switch_plate`'s 64×12 plate.
fn switch_button() -> Vec<Decoration> {
    vec![Decoration::new(Rectangle::new(10.0, 10.0), HIGHLIGHT, Vec2::ZERO)]
}

/// Four short inward spokes + a hub — `motor_electric` (radius 18),
/// visually distinct from `pulley_wheel` (hub only, no spokes) and the
/// gears (teeth, not spokes).
fn motor_spokes(radius: f32) -> Vec<Decoration> {
    let spoke_len = radius - 4.0;
    let mut decorations: Vec<Decoration> = (0..4)
        .map(|i| {
            let angle = i as f32 * TAU / 4.0;
            let dir = Vec2::from_angle(angle);
            Decoration::new(Rectangle::new(spoke_len, 3.0), SHADE, dir * (spoke_len / 2.0))
                .rotated(angle.to_degrees())
        })
        .collect();
    decorations.push(Decoration::new(Circle::new(5.0), SHADE, Vec2::ZERO).at_z(2.0));
    decorations
}

/// Just a darker hub — `pulley_wheel` (radius 14).
fn pulley_hub() -> Vec<Decoration> {
    vec![Decoration::new(Circle::new(5.0), SHADE, Vec2::ZERO)]
}

/// Two crossed blades — `cutter_shears` (32×32 sensor box), read as a
/// pair of scissors rather than a plain square.
fn scissors_cross() -> Vec<Decoration> {
    vec![
        Decoration::new(Rectangle::new(22.0, 4.0), HIGHLIGHT, Vec2::ZERO).rotated(45.0),
        Decoration::new(Rectangle::new(22.0, 4.0), HIGHLIGHT, Vec2::ZERO).rotated(-45.0),
    ]
}

/// Two horizontal bands — `charge_barrel` (radius 20), read as barrel
/// hoops rather than a plain circle.
fn barrel_bands() -> Vec<Decoration> {
    vec![
        Decoration::new(Rectangle::new(30.0, 5.0), SHADE, Vec2::new(0.0, 8.0)),
        Decoration::new(Rectangle::new(30.0, 5.0), SHADE, Vec2::new(0.0, -8.0)),
    ]
}

/// Three rightward chevrons along the belt — `conveyor` (220×20 box) —
/// a fixed visual direction (the belt's actual driven direction is a
/// runtime `RotaryState` sign, not something this static decoration
/// needs to track for a first pass).
fn belt_chevrons() -> Vec<Decoration> {
    let arrow = Triangle2d::new(Vec2::new(-4.0, 5.0), Vec2::new(-4.0, -5.0), Vec2::new(5.0, 0.0));
    [-60.0, 0.0, 60.0]
        .into_iter()
        .map(|x| Decoration::new(arrow, HIGHLIGHT, Vec2::new(x, 0.0)))
        .collect()
}

/// A small "fist" at the forward (local +x) end — `punch_arm` (20×40
/// sensor box; its own system already treats local +x as the punch
/// direction, see `parts::punch_arm`).
fn punch_fist() -> Vec<Decoration> {
    vec![Decoration::new(Circle::new(6.0), SHADE, Vec2::new(10.0, 0.0))]
}

/// Two prong dots — `outlet_power` (24×24 box).
fn outlet_prongs() -> Vec<Decoration> {
    vec![
        Decoration::new(Circle::new(2.5), SHADE, Vec2::new(-4.0, 4.0)),
        Decoration::new(Circle::new(2.5), SHADE, Vec2::new(4.0, 4.0)),
    ]
}

fn decorations_for(def: &PartDef) -> Vec<Decoration> {
    match def.part_type.as_str() {
        "gear_small" => gear_teeth(16.0, 6, 5.0, 6.0, 5.0),
        "gear_large" => gear_teeth(32.0, 8, 6.0, 8.0, 8.0),
        "fan_blower" => fan_blades(),
        "candle" => candle_flame(),
        "switch_plate" => switch_button(),
        "motor_electric" => motor_spokes(18.0),
        "pulley_wheel" => pulley_hub(),
        "cutter_shears" => scissors_cross(),
        "charge_barrel" => barrel_bands(),
        "conveyor" => belt_chevrons(),
        "punch_arm" => punch_fist(),
        "outlet_power" => outlet_prongs(),
        _ => Vec::new(),
    }
}

/// Spawns every decoration `decorations_for(def)` returns as a child of
/// `parent`, if any — a no-op for the ~17 part types not listed there,
/// which just keep their plain base shape from `body_factory::spawn_part`.
pub fn decorate(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    parent: Entity,
    def: &PartDef,
) {
    let decorations = decorations_for(def);
    if decorations.is_empty() {
        return;
    }
    commands.entity(parent).with_children(|children| {
        for deco in decorations {
            children.spawn((
                Mesh2d(meshes.add(deco.mesh)),
                MeshMaterial2d(materials.add(deco.color)),
                Transform::from_translation(deco.offset.extend(deco.z))
                    .with_rotation(Quat::from_rotation_z(deco.rotation_deg.to_radians())),
            ));
        }
    });
}
