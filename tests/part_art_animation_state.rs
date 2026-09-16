//! The headless-safe half of the graphics animation pass (docs/GDD.md's
//! graphics status note): `atmosphere::FuseFlicker`/`ExplosionEffect` and
//! `parts::springboard::SpringboardFlash` are plain state, no `AssetServer`
//! involved — `render::PartArtPlugin` (real-app-only, untestable headlessly
//! by design, see its own docs) just reads them to pick a texture. This
//! file proves the state itself is correct, reusing real level fixtures
//! (`lvl_a08_fuse`'s candle→fuse→barrel chain, `lvl_a02_bounce`'s
//! springboard) rather than inventing new ones.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformPlugin;

use chainworks::atmosphere::{ExplosionEffect, FuseFlicker};
use chainworks::game_state::GameState;
use chainworks::level_file_format::LevelFile;
use chainworks::level_load::LevelPlugin;
use chainworks::parts::springboard::{Springboard, SpringboardFlash};
use chainworks::parts::PartsPlugin;
use chainworks::sim::{SimPlugin, FIXED_DT};
use chainworks::win_conditions::WinConditionsPlugin;

const FUSE_LEVEL: &str = include_str!("../levels/A/lvl_a08_fuse.json");
const SPRINGBOARD_LEVEL: &str = include_str!("../levels/A/lvl_a02_bounce.json");

fn build_app(level_json: &str) -> App {
    let level: LevelFile = serde_json::from_str(level_json).expect("fixture level JSON must be valid");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(PartsPlugin)
        .add_plugins(SimPlugin)
        .add_plugins(LevelPlugin { level })
        .add_plugins(WinConditionsPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FIXED_DT as f64,
        )));
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Running);
    app.update();
    app
}

/// `lvl_a08_fuse`'s candle starts lit and its fuse sits within ignite
/// range, so the fuse should start burning almost immediately — well
/// before the ball on the far side of the level could reach anything.
#[test]
fn a_burning_fuse_cycles_through_its_flicker_frames() {
    let mut app = build_app(FUSE_LEVEL);

    // Let ignition happen, then sample frames over a burn window.
    for _ in 0..30 {
        app.update();
    }
    let mut query = app.world_mut().query::<&FuseFlicker>();
    let flicker = query.single(app.world()).expect("fuse must have a FuseFlicker");
    assert!(
        flicker.frame < 3,
        "flicker frame must stay in 0..3, got {}",
        flicker.frame
    );

    let mut frames_seen = std::collections::BTreeSet::new();
    for _ in 0..200 {
        app.update();
        let mut query = app.world_mut().query::<&FuseFlicker>();
        if let Ok(flicker) = query.single(app.world()) {
            frames_seen.insert(flicker.frame);
        }
    }
    assert!(
        frames_seen.len() > 1,
        "a fuse burning for 200 ticks should have cycled through more than one flicker frame, saw {frames_seen:?}"
    );
}

/// The same level's `charge_barrel` sits right next to the fuse's far end
/// and is expected to detonate around tick 900 (its own `solutions[]`
/// entry) — well within this test's budget. Once it detonates, the barrel
/// entity itself is gone (docs/GDD.md: it despawns itself), but a
/// same-position `ExplosionEffect` should appear in its place and then
/// clean itself up.
#[test]
fn a_detonating_barrel_spawns_and_then_clears_a_short_lived_explosion_effect() {
    let mut app = build_app(FUSE_LEVEL);

    let mut saw_effect = false;
    for _ in 0..1200 {
        app.update();
        let mut query = app.world_mut().query::<&ExplosionEffect>();
        if query.iter(app.world()).next().is_some() {
            saw_effect = true;
            break;
        }
    }
    assert!(saw_effect, "expected an ExplosionEffect within 1200 ticks of the barrel level running");

    // It's a short cosmetic flash, not a permanent fixture — it must
    // eventually despawn on its own.
    let mut cleared = false;
    for _ in 0..200 {
        app.update();
        let mut query = app.world_mut().query::<&ExplosionEffect>();
        if query.iter(app.world()).next().is_none() {
            cleared = true;
            break;
        }
    }
    assert!(cleared, "ExplosionEffect should despawn itself well within 200 more ticks");
}

/// `lvl_a02_bounce`'s ball drops onto the angled springboard and bounces —
/// that real contact should flag `SpringboardFlash`, which should then
/// clear itself shortly after (it's a compression flash, not a toggle).
#[test]
fn bouncing_off_a_springboard_flashes_and_then_clears() {
    let mut app = build_app(SPRINGBOARD_LEVEL);

    let mut saw_flash = false;
    for _ in 0..300 {
        app.update();
        let mut query = app.world_mut().query::<(&Springboard, Option<&SpringboardFlash>)>();
        if query.iter(app.world()).any(|(_, flash)| flash.is_some()) {
            saw_flash = true;
            break;
        }
    }
    assert!(saw_flash, "expected the ball's real bounce to flag SpringboardFlash within 300 ticks");

    let mut cleared = false;
    for _ in 0..60 {
        app.update();
        let mut query = app.world_mut().query::<(&Springboard, Option<&SpringboardFlash>)>();
        if query.iter(app.world()).all(|(_, flash)| flash.is_none()) {
            cleared = true;
            break;
        }
    }
    assert!(cleared, "SpringboardFlash should clear itself well within 60 more ticks");
}
