//! `Condition` evaluation (docs/GDD.md §2.7): checks world *state*, never
//! which parts the player placed — that's what makes multiple solutions
//! possible. Runs in `SimSet::WinConditions`, last in the tick order
//! (docs/GDD.md §2.4), and only while `GameState::Running`.
//!
//! Known limitation carried from M3: hold-duration (`holdMs`) tracking is
//! implemented for *top-level* conditions only. A `CONTAINED` nested
//! inside `ALL_OF`/`ANY_OF` is evaluated instantaneously (no memory of how
//! long it's been true) — fine for every level shipped so far (none nest
//! conditions), but worth fixing before a level actually needs it.
//! `ENERGY_STATE` still always evaluates false: M4 added a real
//! `EnergyGraph` (`crate::energy_graph`), but no shipped level's win/fail
//! condition needs to read it yet, and doing so requires a part-registry
//! lookup this system doesn't otherwise need — wire it up when a level
//! actually requires it.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game_state::GameState;
use crate::level_file_format::Condition;
use crate::level_load::{EditorState, PartTags, PlacedId};
use crate::sim::{SimSet, FIXED_DT};

/// Wall-clock-independent simulation time (docs/GDD.md §2.6: never read
/// real elapsed time in sim logic). Only advances while `Running`.
#[derive(Resource, Default)]
pub struct SimClock {
    pub elapsed_seconds: f32,
}

/// Hold-duration accumulators, parallel to a level's `winConditions`/
/// `failConditions` arrays (top-level entries only — see module docs).
#[derive(Resource, Default)]
pub struct ConditionHoldState {
    win: Vec<f32>,
    fail: Vec<f32>,
}

pub struct WinConditionsPlugin;

impl Plugin for WinConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimClock>()
            .init_resource::<ConditionHoldState>()
            .add_systems(
                FixedUpdate,
                evaluate_win_conditions
                    .in_set(SimSet::WinConditions)
                    .run_if(in_state(GameState::Running)),
            );
    }
}

fn find_by_tag(entities: &[(Entity, &PartTags, &Transform)], tag: &str) -> Option<Entity> {
    entities
        .iter()
        .find(|(_, tags, _)| tags.has(tag))
        .map(|(e, _, _)| *e)
}

fn find_by_id(ids: &[(String, Entity)], id: &str) -> Option<Entity> {
    ids.iter().find(|(pid, _)| pid == id).map(|(_, e)| *e)
}

/// `Some(hold_ms)` for conditions the schema gives a `holdMs` field to,
/// `None` for conditions that resolve the instant they're true.
fn condition_hold_ms(cond: &Condition) -> Option<f32> {
    match cond {
        Condition::Contained { hold_ms, .. } => Some(*hold_ms),
        Condition::EnergyState { hold_ms, .. } => Some(*hold_ms),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn eval_condition(
    cond: &Condition,
    entities: &[(Entity, &PartTags, &Transform)],
    colliding: &Query<&CollidingEntities>,
    ids: &[(String, Entity)],
    elapsed_seconds: f32,
    time_limit_sec: f32,
    world_half_extent: Vec2,
) -> bool {
    match cond {
        Condition::Contained {
            subject_tag,
            container_id,
            ..
        } => {
            let Some(subject) = find_by_tag(entities, subject_tag) else {
                return false;
            };
            let Some(container) = find_by_id(ids, container_id) else {
                return false;
            };
            colliding
                .get(container)
                .map(|c| c.contains(subject))
                .unwrap_or(false)
        }
        Condition::ReachedZone {
            subject_tag,
            zone_id,
        } => {
            let Some(subject) = find_by_tag(entities, subject_tag) else {
                return false;
            };
            let Some(zone) = find_by_id(ids, zone_id) else {
                return false;
            };
            colliding
                .get(zone)
                .map(|c| c.contains(subject))
                .unwrap_or(false)
        }
        // No energy graph until M4 — always false, never silently "solved".
        Condition::EnergyState { .. } => false,
        Condition::Destroyed { target_id } => find_by_id(ids, target_id).is_none(),
        Condition::Timeout => elapsed_seconds >= time_limit_sec,
        Condition::SubjectDestroyed { subject_tag } => find_by_tag(entities, subject_tag).is_none(),
        Condition::LeftBounds { subject_tag } => {
            let Some(subject) = find_by_tag(entities, subject_tag) else {
                return false;
            };
            let Some((_, _, transform)) = entities.iter().find(|(e, _, _)| *e == subject) else {
                return false;
            };
            transform.translation.x.abs() > world_half_extent.x
                || transform.translation.y.abs() > world_half_extent.y
        }
        Condition::AllOf { conditions } => conditions.iter().all(|c| {
            eval_condition(
                c,
                entities,
                colliding,
                ids,
                elapsed_seconds,
                time_limit_sec,
                world_half_extent,
            )
        }),
        Condition::AnyOf { conditions } => conditions.iter().any(|c| {
            eval_condition(
                c,
                entities,
                colliding,
                ids,
                elapsed_seconds,
                time_limit_sec,
                world_half_extent,
            )
        }),
    }
}

fn apply_hold(cond: &Condition, instant: bool, hold_elapsed: &mut f32, dt_ms: f32) -> bool {
    match condition_hold_ms(cond) {
        Some(hold_ms) => {
            if instant {
                *hold_elapsed += dt_ms;
            } else {
                *hold_elapsed = 0.0;
            }
            *hold_elapsed >= hold_ms
        }
        None => instant,
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_win_conditions(
    state: Res<EditorState>,
    mut hold_state: ResMut<ConditionHoldState>,
    mut sim_clock: ResMut<SimClock>,
    mut next_state: ResMut<NextState<GameState>>,
    tagged: Query<(Entity, &PartTags, &Transform)>,
    placed_ids: Query<(&PlacedId, Entity)>,
    colliding: Query<&CollidingEntities>,
) {
    sim_clock.elapsed_seconds += FIXED_DT;

    let entities: Vec<(Entity, &PartTags, &Transform)> = tagged.iter().collect();
    let ids: Vec<(String, Entity)> = placed_ids
        .iter()
        .map(|(pid, e)| (pid.0.clone(), e))
        .collect();
    let half_extent = Vec2::new(state.level.world.width / 2.0, state.level.world.height / 2.0);
    let dt_ms = FIXED_DT * 1000.0;

    hold_state
        .win
        .resize(state.level.win_conditions.len(), 0.0);
    let mut all_win = !state.level.win_conditions.is_empty();
    for (i, cond) in state.level.win_conditions.iter().enumerate() {
        let instant = eval_condition(
            cond,
            &entities,
            &colliding,
            &ids,
            sim_clock.elapsed_seconds,
            state.level.world.time_limit_sec,
            half_extent,
        );
        if !apply_hold(cond, instant, &mut hold_state.win[i], dt_ms) {
            all_win = false;
        }
    }
    if all_win {
        next_state.set(GameState::Solved);
        return;
    }

    hold_state
        .fail
        .resize(state.level.fail_conditions.len(), 0.0);
    for (i, cond) in state.level.fail_conditions.iter().enumerate() {
        let instant = eval_condition(
            cond,
            &entities,
            &colliding,
            &ids,
            sim_clock.elapsed_seconds,
            state.level.world.time_limit_sec,
            half_extent,
        );
        if apply_hold(cond, instant, &mut hold_state.fail[i], dt_ms) {
            next_state.set(GameState::Failed);
            return;
        }
    }
}
