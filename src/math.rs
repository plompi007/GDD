//! Shared physics/render constants (docs/GDD.md §2.2).
//!
//! With `bevy_rapier2d`'s `pixels_per_meter` set to this value, `Collider`
//! sizes and `Transform` positions are specified directly in pixels —
//! bevy_rapier2d handles the meter conversion for the solver internally. No
//! other code should ever multiply/divide by this constant.

pub const PIXELS_PER_METER: f32 = 32.0;
