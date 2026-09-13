//! `UI_TOKENS` (docs/GDD.md §3.9.1/§3.9.2/§3.9.4): the single source of
//! truth for spacing/radius/motion/color, so no UI code invents its own
//! duration/easing/radius/color inline. Every value here is a direct
//! transcription of the GDD's own tables — if a number changes, it changes
//! here, not at each call site.
//!
//! Scope note: `MOTION`'s easing curves and §3.9.3's shader-based "deep
//! flat" rendering (per-pixel lighting/AO, bloom, parallax) aren't
//! implemented yet — that's custom WGSL/post-processing work tracked
//! separately from this milestone's UI *shell*. What's here is genuinely
//! used: `RADIUS`/`SPACE` size this milestone's HUD/bin/button layout,
//! and `ENERGY_COLORS`/`PALETTE` recolor placeholder sprites correctly by
//! energy type instead of the old ad-hoc tag-based guessing.

use bevy::prelude::*;

use crate::part::EnergyType;

/// docs/GDD.md §3.9.1 `RADIUS`.
pub mod radius {
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 14.0;
    pub const LG: f32 = 22.0;
    pub const PILL: f32 = 999.0;
}

/// docs/GDD.md §3.9.1 `SPACE` — a scale, not free-floating numbers.
pub const SPACE: [f32; 8] = [0.0, 4.0, 8.0, 12.0, 16.0, 24.0, 32.0, 48.0];

/// docs/GDD.md §3.9.1 `MOTION` durations in milliseconds (easing curves
/// aren't wired into a tweening system yet — see module docs).
pub mod motion_ms {
    pub const INSTANT: f32 = 90.0;
    pub const SNAPPY: f32 = 160.0;
    pub const SMOOTH: f32 = 240.0;
    pub const CELEBRATE: f32 = 600.0;
}

/// docs/GDD.md §3.9.4 base palette (dark mode only for now — the light
/// variant is an accessibility option, not required for this milestone).
pub mod palette {
    use bevy::prelude::Color;

    pub const BG_CANVAS: Color = Color::srgb(0x14 as f32 / 255.0, 0x17 as f32 / 255.0, 0x1f as f32 / 255.0);
    pub const BG_PANEL: Color = Color::srgb(0x1d as f32 / 255.0, 0x21 as f32 / 255.0, 0x2c as f32 / 255.0);
    pub const INK_PRIMARY: Color = Color::srgb(0xf4 as f32 / 255.0, 0xf6 as f32 / 255.0, 0xfb as f32 / 255.0);
    pub const INK_SECONDARY: Color = Color::srgb(0x9a as f32 / 255.0, 0xa3 as f32 / 255.0, 0xb8 as f32 / 255.0);
    pub const ACCENT_BRAND: Color = Color::srgb(0x6c as f32 / 255.0, 0x8c as f32 / 255.0, 1.0);
    pub const SUCCESS: Color = Color::srgb(0x35 as f32 / 255.0, 0xd3 as f32 / 255.0, 0x99 as f32 / 255.0);
    pub const DANGER: Color = Color::srgb(1.0, 0x5c as f32 / 255.0, 0x6c as f32 / 255.0);
}

/// docs/GDD.md §3.9.2 — one signature color per `EnergyType`.
pub fn energy_color(energy: EnergyType) -> Color {
    match energy {
        EnergyType::Rotary => Color::srgb_u8(0xf2, 0xa3, 0x3d),
        EnergyType::Tension => Color::srgb_u8(0x3d, 0xdb, 0xd9),
        EnergyType::Electric => Color::srgb_u8(0xf4, 0xe0, 0x4d),
        EnergyType::Thermal => Color::srgb_u8(0xf2, 0x54, 0x2d),
        EnergyType::Pneumatic => Color::srgb_u8(0x7f, 0xe0, 0xc9),
        EnergyType::Light => Color::srgb_u8(0xff, 0xf3, 0xd6),
        EnergyType::Impact => Color::srgb_u8(0xe2, 0x3d, 0x9c),
    }
}

/// docs/GDD.md §3.9.7 UI font — Rubik (OFL-licensed, see
/// `assets/fonts/OFL.txt`), standard full-Latin build. All game content
/// (level/part JSON, UI labels) is English.
pub const UI_FONT_MEDIUM: &str = "fonts/Rubik-Medium.ttf";
pub const UI_FONT_REGULAR: &str = "fonts/Rubik-Regular.ttf";
