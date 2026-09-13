//! The parts bin (docs/GDD.md §3.1: "ארגז החלקים — 96pt", a horizontally
//! scrolling row at the bottom of the screen) as real screen-space
//! `bevy_ui`, replacing `input.rs`'s M5 placeholder (flat rectangles
//! positioned in *world* space). [`BinSlot`] + bevy_ui's own `Interaction`
//! is what `input::start_drag_system` reads to detect "the player pressed
//! this bin item" — see that module's docs for why the rest of the drag
//! (following the pointer, placing in the world) still runs on
//! `PointerState` rather than UI events: once the drag leaves the bin
//! button's bounds, `Interaction` no longer tracks it, but a raw pointer
//! press/release does.
//!
//! Scope note: no scrolling/expanded-grid mode, no real icons (a flat
//! placeholder color standing in for `editor.binIcon`), no rotate/flip/
//! inspector chrome — docs/GDD.md §3.1's "drag up for 4-column grid" and
//! §3.4's inline menu are separate, larger UI features than this
//! milestone's job of proving the drag mechanic works.

use bevy::prelude::*;

use crate::level_load::EditorState;
use crate::parts::PartRegistry;
use crate::sim::body_factory::placeholder_color;
use crate::ui::tokens::{palette, radius, SPACE, UI_FONT_REGULAR};

const SLOT_SIZE: f32 = 64.0;

/// Read by `input::start_drag_system` — the *only* thing that module still
/// needs from this one. `bin_index` matches `EditorState.level.partsBin`'s
/// index, so both modules stay in sync against the same source of truth.
#[derive(Component, Clone)]
pub struct BinSlot {
    pub part_type: String,
    pub bin_index: usize,
}

#[derive(Component)]
struct BinRoot;

#[derive(Component)]
struct BinCountLabel {
    bin_index: usize,
}

fn spawn_parts_bin(
    mut commands: Commands,
    state: Res<EditorState>,
    registry: Res<PartRegistry>,
    asset_server: Res<AssetServer>,
) {
    if state.level.parts_bin.is_empty() {
        return;
    }

    let count_font = TextFont {
        font: asset_server.load(UI_FONT_REGULAR),
        font_size: 14.0,
        ..default()
    };

    commands
        .spawn((
            BinRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(64.0), // above the main button bar
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(96.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(SPACE[2]),
                padding: UiRect::all(Val::Px(SPACE[2])),
                ..default()
            },
            BackgroundColor(palette::BG_PANEL.with_alpha(0.85)),
        ))
        .with_children(|parent| {
            for (index, entry) in state.level.parts_bin.iter().enumerate() {
                let color = registry
                    .get(&entry.part_type)
                    .map(placeholder_color)
                    .unwrap_or(palette::INK_SECONDARY);
                parent
                    .spawn((
                        BinSlot {
                            part_type: entry.part_type.clone(),
                            bin_index: index,
                        },
                        Button,
                        Node {
                            width: Val::Px(SLOT_SIZE),
                            height: Val::Px(SLOT_SIZE),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        BackgroundColor(color),
                        BorderRadius::all(Val::Px(radius::SM)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            BinCountLabel { bin_index: index },
                            Text::new(entry.count.to_string()),
                            count_font.clone(),
                            TextColor(palette::INK_PRIMARY),
                        ));
                    });
            }
        });
}

fn despawn_parts_bin(mut commands: Commands, roots: Query<Entity, With<BinRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Updates each slot's remaining-count label and dims it once empty —
/// `EditorState.level.partsBin[i].count` is the single source of truth
/// (mutated by `input::end_drag_system` on a successful placement).
fn sync_bin_visuals(
    state: Res<EditorState>,
    mut labels: Query<(&BinCountLabel, &mut Text)>,
    mut slots: Query<(&BinSlot, &mut BackgroundColor)>,
    registry: Res<PartRegistry>,
) {
    for (label, mut text) in &mut labels {
        let remaining = state
            .level
            .parts_bin
            .get(label.bin_index)
            .map(|e| e.count)
            .unwrap_or(0);
        text.0 = remaining.to_string();
    }
    for (slot, mut background) in &mut slots {
        let remaining = state
            .level
            .parts_bin
            .get(slot.bin_index)
            .map(|e| e.count)
            .unwrap_or(0);
        let base = registry
            .get(&slot.part_type)
            .map(placeholder_color)
            .unwrap_or(palette::INK_SECONDARY);
        background.0 = if remaining == 0 {
            base.with_alpha(0.35)
        } else {
            base
        };
    }
}

pub struct PartsBinPlugin;

impl Plugin for PartsBinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::game_state::GameState::Edit),
            (despawn_parts_bin, spawn_parts_bin).chain(),
        )
        .add_systems(Update, sync_bin_visuals);
    }
}
