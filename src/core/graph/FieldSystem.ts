import type { LevelRuntime } from '../level/LevelRuntime.ts';
import { SIM } from '../sim/constants.ts';

const FAN_FORCE_SCALE = 6; // N per power unit, tuned so a balloon visibly drifts
const FAN_BEAM_HALF_WIDTH_M = 1.0;

/**
 * A powered fan_blower pushes anything inside its forward beam, scaled by
 * the body's own windFactor (GDD §1.4.3 — "אותו שדה גם דוחף גופים פיזית
 * וגם מזין את הפורט הלוגי של הכנף"; the logical PNEUMATIC-port side of
 * that pseudocode is for wind_vane, deferred to a later content pass).
 */
export function applyFields(runtime: LevelRuntime): void {
  for (const fan of runtime.getByPartType('fan_blower')) {
    if (!fan.state.powered) continue;
    const power = typeof fan.params.power === 'number' ? fan.params.power : 2;
    const range = typeof fan.params.range === 'number' ? fan.params.range : 6;
    const pos = fan.rigidBody.translation();
    const rot = fan.rigidBody.rotation();
    const dir = { x: Math.cos(rot), y: Math.sin(rot) };

    for (const part of runtime.all()) {
      if (part.rigidBody.isFixed()) continue;
      const p = part.rigidBody.translation();
      const rel = { x: p.x - pos.x, y: p.y - pos.y };
      const along = rel.x * dir.x + rel.y * dir.y;
      if (along < 0 || along > range) continue;
      const perp = Math.abs(-rel.x * dir.y + rel.y * dir.x);
      if (perp > FAN_BEAM_HALF_WIDTH_M) continue;

      const falloff = 1 - along / range;
      const partWindFactor = part.def.windFactor ?? 1;
      const forceMag = FAN_FORCE_SCALE * power * partWindFactor * falloff;
      part.rigidBody.applyImpulse({ x: dir.x * forceMag * SIM.FIXED_DT, y: dir.y * forceMag * SIM.FIXED_DT }, true);
    }
  }
}
