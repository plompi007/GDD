//! Top HUD bar (title + goal text + reset) and the main Play/Stop/Reset
//! button (docs/GDD.md §3.1): "the main button is always in the same
//! place and swaps its label: Play → Stop → Reset. No popping buttons."
//!
//! Rebuilt on every `OnEnter(GameState::Edit)` (the level title/goal text
//! come from `EditorState`, which only ever changes alongside a
//! transition into `Edit` — a reset, or `app.rs`'s level-cycle stand-in
//! for the `LevelSelect` screen) — cheap enough at this scale, and keeps
//! this module from having to special-case "did the level change" vs.
//! "just re-entered Edit".

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::level_load::EditorState;
use crate::ui::tokens::{palette, radius, SPACE, UI_FONT_MEDIUM, UI_FONT_REGULAR};

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct GoalText;

#[derive(Component)]
struct MainButton;

#[derive(Component)]
struct MainButtonLabel;

/// Plain text, not icon+text (docs/GDD.md §3.1's "▶ Play" mockup uses a
/// glyph for compactness in ASCII art) — real icons are stroke-based SVG
/// assets per §3.9.7, not arbitrary Unicode symbols outside the bundled
/// font's standard Latin coverage.
fn main_button_label(state: GameState) -> &'static str {
    match state {
        GameState::Edit => "Play",
        GameState::Running => "Stop",
        GameState::Paused | GameState::Solved | GameState::Failed => "Reset",
    }
}

/// docs/GDD.md §3.1: the main button cycles Edit → Running → Paused →
/// (back to) Edit — "Play → Stop → Reset" — regardless of Solved/Failed,
/// which also reset back to Edit on the next press (same as the old
/// `dev_controls` keyboard shortcut this replaces).
fn next_state_for_main_button(state: GameState) -> GameState {
    match state {
        GameState::Edit => GameState::Running,
        GameState::Running => GameState::Paused,
        GameState::Paused | GameState::Solved | GameState::Failed => GameState::Edit,
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>, state: Res<EditorState>) {
    let title_font = TextFont {
        font: asset_server.load(UI_FONT_MEDIUM),
        font_size: 22.0,
        ..default()
    };
    let goal_font = TextFont {
        font: asset_server.load(UI_FONT_REGULAR),
        font_size: 16.0,
        ..default()
    };
    let button_font = TextFont {
        font: asset_server.load(UI_FONT_MEDIUM),
        font_size: 20.0,
        ..default()
    };

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                padding: UiRect::all(Val::Px(SPACE[3])),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(SPACE[1]),
                ..default()
            },
            BackgroundColor(palette::BG_PANEL.with_alpha(0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(state.level.title.clone()),
                title_font,
                TextColor(palette::INK_PRIMARY),
            ));
            parent.spawn((
                GoalText,
                Text::new(state.level.goal_text.clone().unwrap_or_default()),
                goal_font,
                TextColor(palette::INK_SECONDARY),
            ));
        });

    // docs/GDD.md §3.1: main button, always the same place (bottom,
    // centered) — 64pt tall.
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    MainButton,
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(palette::ACCENT_BRAND),
                    BorderRadius::all(Val::Px(radius::PILL)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        MainButtonLabel,
                        Text::new(main_button_label(GameState::Edit)),
                        button_font,
                        TextColor(palette::INK_PRIMARY),
                    ));
                });
        });
}

/// Both the top bar and the bottom button row are tagged `HudRoot` (Bevy
/// 0.16's `despawn()` recursively despawns descendants, so despawning
/// each root takes its children — title text, the button, its label —
/// with it).
fn despawn_hud(mut commands: Commands, roots: Query<Entity, With<HudRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

fn sync_main_button_label(
    state: Res<State<GameState>>,
    mut labels: Query<&mut Text, With<MainButtonLabel>>,
) {
    if !state.is_changed() {
        return;
    }
    for mut text in &mut labels {
        text.0 = main_button_label(*state.get()).to_string();
    }
}

fn main_button_interaction(
    mut interactions: Query<&Interaction, (Changed<Interaction>, With<MainButton>)>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &mut interactions {
        if *interaction == Interaction::Pressed {
            next_state.set(next_state_for_main_button(*state.get()));
        }
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Edit), (despawn_hud, spawn_hud).chain())
            .add_systems(Update, (sync_main_button_label, main_button_interaction));
    }
}
