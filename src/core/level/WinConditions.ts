import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { LevelDef } from './LevelSchema.ts';
import type { Condition } from './LevelSchema.ts';
import type { LevelRuntime } from './LevelRuntime.ts';

/** A body counts as "settled" once its speed drops below this (m/s). */
const SETTLE_SPEED = 0.5;

export interface ConditionContext {
  world: RAPIER.World;
  runtime: LevelRuntime;
  nowMs: number;
  worldWidth: number;
  worldHeight: number;
}

/**
 * Per-session, per-condition-node timers for holdMs-gated conditions
 * (CONTAINED, ENERGY_STATE). Keyed by node identity, so a fresh Map per
 * GameSession/reset() is enough — no need to touch the level JSON itself.
 */
export class ConditionTimers {
  private readonly since = new Map<Condition, number>();

  holdFor(node: Condition, isTrueNow: boolean, nowMs: number, holdMs: number): boolean {
    if (!isTrueNow) {
      this.since.delete(node);
      return false;
    }
    const startedAt = this.since.get(node);
    if (startedAt === undefined) {
      this.since.set(node, nowMs);
      return holdMs <= 0;
    }
    return nowMs - startedAt >= holdMs;
  }
}

function speedOf(body: RAPIER.RigidBody): number {
  const v = body.linvel();
  return Math.hypot(v.x, v.y);
}

export function evaluateCondition(
  node: Condition,
  ctx: ConditionContext,
  timers: ConditionTimers,
): boolean {
  switch (node.type) {
    case 'CONTAINED': {
      const container = node.containerId ? ctx.runtime.get(node.containerId) : undefined;
      if (!container) return false;
      const subjects = node.subjectTag ? ctx.runtime.getByTag(node.subjectTag) : [];
      const isTrueNow = subjects.some(
        (s) =>
          !ctx.runtime.isRemoved(s.id) &&
          ctx.world.intersectionPair(s.collider, container.collider) &&
          speedOf(s.rigidBody) < SETTLE_SPEED,
      );
      return timers.holdFor(node, isTrueNow, ctx.nowMs, node.holdMs);
    }
    case 'REACHED_ZONE': {
      const zone = node.zoneId ? ctx.runtime.get(node.zoneId) : undefined;
      if (!zone) return false;
      const subjects = node.subjectTag ? ctx.runtime.getByTag(node.subjectTag) : [];
      const isTrueNow = subjects.some(
        (s) => !ctx.runtime.isRemoved(s.id) && ctx.world.intersectionPair(s.collider, zone.collider),
      );
      return timers.holdFor(node, isTrueNow, ctx.nowMs, node.holdMs);
    }
    case 'DESTROYED': {
      return node.targetId ? ctx.runtime.isRemoved(node.targetId) : false;
    }
    case 'SUBJECT_DESTROYED': {
      return node.subjectTag ? ctx.runtime.isTagFullyRemoved(node.subjectTag) : false;
    }
    case 'LEFT_BOUNDS': {
      const subjects = node.subjectTag ? ctx.runtime.getByTag(node.subjectTag) : [];
      return subjects.some((s) => {
        if (ctx.runtime.isRemoved(s.id)) return false;
        const t = s.rigidBody.translation();
        return t.x < 0 || t.x > ctx.worldWidth || t.y < 0 || t.y > ctx.worldHeight;
      });
    }
    case 'TIMEOUT': {
      // Compared against the level's own time limit by the caller (GameSession),
      // which knows SIM.MAX_SIM_SECONDS and level.world.timeLimitSec together.
      return false;
    }
    case 'ENERGY_STATE': {
      // Requires the energy graph (M4). Never true until then.
      return false;
    }
    case 'ALL_OF': {
      return (node.conditions ?? []).every((c) => evaluateCondition(c, ctx, timers));
    }
    case 'ANY_OF': {
      return (node.conditions ?? []).some((c) => evaluateCondition(c, ctx, timers));
    }
  }
}

export function evaluateAny(nodes: Condition[], ctx: ConditionContext, timers: ConditionTimers): boolean {
  return nodes.some((n) => evaluateCondition(n, ctx, timers));
}

export function isTimedOut(level: LevelDef, nowMs: number): boolean {
  return nowMs >= level.world.timeLimitSec * 1000;
}
