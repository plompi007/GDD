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

use crate::part::{PartDef, ShapeSpec};
use crate::parts::PartRegistry;
use crate::sim::body_factory::PartType;

/// Real art asset path (under `assets/`) for a part's *base* sprite, if one
/// has been generated yet — see docs/GDD.md's graphics status note for how
/// these were produced (DALL·E, background-stripped via ImageMagick
/// floodfill, downscaled to 256px max). For a part with idle/action
/// variants (`switch_plate`, `springboard`, `cutter_shears`, `punch_arm`)
/// this is the *idle* frame only — the action-frame swap and the
/// `fuse_cord`/`charge_barrel` animation loops are not wired up yet, this
/// just gets the correct art on screen at rest. `gear_large` and `candle`
/// have no usable art yet (the gear_large generation came back wrong, and
/// candle was only ever a single non-transparent test image) so both still
/// fall through to the procedural placeholder below. `fuse_cord` is
/// deliberately excluded too, despite having generated frames on disk
/// (`assets/parts/fuse_cord_{0,1,2}.png`, still used by the flicker-*state*
/// machinery in `atmosphere.rs`/`sync_fuse_cord_sprite`): that art is a
/// diagonal squiggle on a near-square canvas (~1.3:1), but the part's own
/// collider is a 100×6 sliver (~16.7:1) — forcing `custom_size` to the
/// collider's exact box squashes the art ~13x vertically, past the point
/// where its thin curved lines survive at all (verified in-game: the fuse
/// rendered as literally nothing between the candle and the barrel). Fixing
/// this needs better-fitting source art, not a sizing formula — a uniform
/// "contain" fit would just make the fuse balloon to ~10x its actual
/// length instead. Every other elongated part's art (`conveyor`,
/// `plank_wood`, `beam_steel`, `lever_seesaw`) sits at a much milder
/// 1.6–2.2x mismatch on a *solid-fill* shape, which stays visible under the
/// same squash — confirmed by `plank_wood` rendering correctly in-game.
pub fn static_texture_for(part_type: &str) -> Option<&'static str> {
    match part_type {
        "ball_wood" => Some("parts/ball_wood.png"),
        "ball_rubber" => Some("parts/ball_rubber.png"),
        "ball_iron" => Some("parts/ball_iron.png"),
        "ball_lead" => Some("parts/ball_lead.png"),
        "ball_glass" => Some("parts/ball_glass.png"),
        "balloon_lift" => Some("parts/balloon_lift.png"),
        "beam_steel" => Some("parts/beam_steel.png"),
        "bin_target" => Some("parts/bin_target.png"),
        "charge_barrel" => Some("parts/charge_barrel.png"),
        "conveyor" => Some("parts/conveyor.png"),
        "crate_wood" => Some("parts/crate_wood.png"),
        "cutter_shears" => Some("parts/cutter_shears_idle.png"),
        "fan_blower" => Some("parts/fan_blower.png"),
        "floor_ground" => Some("parts/floor_ground.png"),
        "gear_small" => Some("parts/gear_small.png"),
        "goal_zone" => Some("parts/goal_zone.png"),
        "lever_seesaw" => Some("parts/lever_seesaw.png"),
        "motor_electric" => Some("parts/motor_electric.png"),
        "outlet_power" => Some("parts/outlet_power.png"),
        "plank_wood" => Some("parts/plank_wood.png"),
        "pulley_wheel" => Some("parts/pulley_wheel.png"),
        "punch_arm" => Some("parts/punch_arm_idle.png"),
        "spike_pin" => Some("parts/spike_pin.png"),
        "springboard" => Some("parts/springboard_idle.png"),
        "switch_plate" => Some("parts/switch_plate_idle.png"),
        "wall_brick" => Some("parts/wall_brick.png"),
        _ => None,
    }
}

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
    // Real art already carries this detail (gear teeth, a candle's flame,
    // ...) — layering the procedural version underneath/behind it too
    // would just double up, so skip it entirely once art exists.
    if static_texture_for(&def.part_type).is_some() {
        return;
    }
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

/// Swaps in the real texture from [`static_texture_for`] on every
/// newly-spawned part, replacing whatever procedural placeholder
/// `body_factory::spawn_part`/[`decorate`] gave it (a `Mesh2d` circle for a
/// `Ball` shape, or a flat-color `Sprite` for a `Box` shape — both get
/// overwritten by a real textured `Sprite` sized to the same collider
/// bounding box).
///
/// Deliberately **not** added to [`crate::parts::PartsPlugin`] or any other
/// plugin the headless test suite uses: it needs a real `AssetServer` (and,
/// via `asset_server.load`, the full `ImagePlugin`/`RenderAssetPlugin`
/// pipeline `DefaultPlugins` sets up) which `MinimalPlugins`-based tests
/// deliberately don't have, precisely so those tests stay renderer-free.
/// Only `app.rs`'s real `App::new()` registers this plugin.
fn apply_real_art_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<PartRegistry>,
    spawned: Query<(Entity, &PartType), Added<PartType>>,
) {
    for (entity, part_type) in &spawned {
        let Some(path) = static_texture_for(&part_type.0) else {
            continue;
        };
        let Some(def) = registry.get(&part_type.0) else {
            continue;
        };
        let size = match def.body.shape {
            ShapeSpec::Ball { radius } => Vec2::splat(radius * 2.0),
            ShapeSpec::Box { w, h } => Vec2::new(w, h),
        };
        commands
            .entity(entity)
            .remove::<Mesh2d>()
            .remove::<MeshMaterial2d<ColorMaterial>>()
            .insert(Sprite {
                image: asset_server.load(path),
                custom_size: Some(size),
                ..default()
            });
    }
}

/// `switch_plate_idle.png` while off, `switch_plate_action.png` while on —
/// reads the exact same `SwitchPlate.is_on` the energy graph itself already
/// drives, so this can never disagree with the actual electrical state.
fn sync_switch_plate_sprite(
    asset_server: Res<AssetServer>,
    mut switches: Query<(&crate::parts::switch_plate::SwitchPlate, &mut Sprite)>,
) {
    for (switch, mut sprite) in &mut switches {
        let path = if switch.is_on {
            "parts/switch_plate_action.png"
        } else {
            "parts/switch_plate_idle.png"
        };
        sprite.image = asset_server.load(path);
    }
}

/// `cutter_shears_action.png` once `CutterShears.triggered` flips — stays
/// that way, matching the part's own real one-shot-then-done behavior.
fn sync_cutter_shears_sprite(
    asset_server: Res<AssetServer>,
    mut cutters: Query<(&crate::parts::cutter_shears::CutterShears, &mut Sprite)>,
) {
    for (cutter, mut sprite) in &mut cutters {
        let path = if cutter.triggered {
            "parts/cutter_shears_action.png"
        } else {
            "parts/cutter_shears_idle.png"
        };
        sprite.image = asset_server.load(path);
    }
}

/// `punch_arm_action.png` for exactly as long as something is actually
/// touching it (`PunchArm.was_touching`, the same field the impulse system
/// itself uses) — extended while it's actually shoving something, retracted
/// otherwise.
fn sync_punch_arm_sprite(
    asset_server: Res<AssetServer>,
    mut arms: Query<(&crate::parts::punch_arm::PunchArm, &mut Sprite)>,
) {
    for (arm, mut sprite) in &mut arms {
        let path = if arm.was_touching {
            "parts/punch_arm_action.png"
        } else {
            "parts/punch_arm_idle.png"
        };
        sprite.image = asset_server.load(path);
    }
}

/// `springboard_action.png` for the short window `SpringboardFlash` marks
/// after a real bounce (see `parts::springboard`'s own docs on why this
/// needed a new component: unlike the other three, plain restitution gives
/// no existing signal to read).
fn sync_springboard_sprite(
    asset_server: Res<AssetServer>,
    mut boards: Query<
        (&mut Sprite, Has<crate::parts::springboard::SpringboardFlash>),
        With<crate::parts::springboard::Springboard>,
    >,
) {
    for (mut sprite, flashing) in &mut boards {
        let path = if flashing {
            "parts/springboard_action.png"
        } else {
            "parts/springboard_idle.png"
        };
        sprite.image = asset_server.load(path);
    }
}

/// Cycles through `fuse_cord`'s 3 flicker frames per `FuseFlicker.frame` —
/// **currently a no-op**, see `static_texture_for`'s own docs on why that
/// art isn't actually applied yet (a real aspect-ratio mismatch that made
/// the fuse invisible in-game, not just a style nitpick). Left in place
/// (rather than deleted) so turning this back on is a one-line change
/// (`static_texture_for`'s `fuse_cord` arm) once better-fitting art exists;
/// `FuseFlicker` itself keeps advancing and is tested regardless.
fn sync_fuse_cord_sprite(
    asset_server: Res<AssetServer>,
    mut fuses: Query<(&crate::atmosphere::FuseFlicker, &mut Sprite)>,
) {
    if static_texture_for("fuse_cord").is_none() {
        return;
    }
    for (flicker, mut sprite) in &mut fuses {
        let path = match flicker.frame {
            0 => "parts/fuse_cord_0.png",
            1 => "parts/fuse_cord_1.png",
            _ => "parts/fuse_cord_2.png",
        };
        sprite.image = asset_server.load(path);
    }
}

/// Puts the real explosion-puff art on every `atmosphere::ExplosionEffect`
/// the instant `thermal_update_system` spawns one (see that component's own
/// docs on why it exists as a separate entity rather than an animation
/// frame on the barrel itself). A fixed visual size, not derived from
/// `blast_radius` — this is a cosmetic puff, not a hitbox, and doesn't need
/// to be pixel-accurate to the actual blast range.
fn spawn_explosion_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    spawned: Query<Entity, Added<crate::atmosphere::ExplosionEffect>>,
) {
    for entity in &spawned {
        commands.entity(entity).insert(Sprite {
            image: asset_server.load("parts/charge_barrel_explosion.png"),
            custom_size: Some(Vec2::splat(60.0)),
            ..default()
        });
    }
}

/// See [`apply_real_art_system`]'s own docs for why this is never added to
/// a headless test app — only `app.rs`'s real `App::new()` uses it.
pub struct PartArtPlugin;

impl Plugin for PartArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_real_art_system,
                sync_switch_plate_sprite,
                sync_cutter_shears_sprite,
                sync_punch_arm_sprite,
                sync_springboard_sprite,
                sync_fuse_cord_sprite,
                spawn_explosion_sprite,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift guard: every path `static_texture_for` returns must actually
    /// exist under `assets/`, and for every *registered* part type (not
    /// just the ones in this list) — a typo'd path would otherwise fail
    /// silently at runtime (`asset_server.load` just logs a warning and
    /// renders nothing, per M9's font-loading bug).
    #[test]
    fn every_static_texture_path_exists_on_disk() {
        let registry = crate::parts::build_registry();
        for part_type in registry.part_types() {
            let Some(path) = static_texture_for(part_type) else {
                continue;
            };
            let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join(path);
            assert!(
                full.exists(),
                "static_texture_for({part_type:?}) points at {full:?}, which doesn't exist"
            );
        }
    }
}
