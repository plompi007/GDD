import type { LevelRuntime } from '../level/LevelRuntime.ts';
import type { ConnectionDef } from '../level/LevelSchema.ts';
import { SIM } from '../sim/constants.ts';

const TENSION_STIFFNESS = 40; // impulse per meter of overstretch, tuned for a snappy-but-stable pull
const MAX_TENSION_IMPULSE = 8;

function sub(a: { x: number; y: number }, b: { x: number; y: number }) {
  return { x: a.x - b.x, y: a.y - b.y };
}

function length(v: { x: number; y: number }): number {
  return Math.hypot(v.x, v.y);
}

function normalize(v: { x: number; y: number }): { x: number; y: number } {
  const len = length(v);
  return len > 1e-9 ? { x: v.x / len, y: v.y / len } : { x: 0, y: 0 };
}

/**
 * Ropes are not physical bodies (GDD §1.4.3) — a ROPE connection is a
 * distance constraint between its two endpoints, routed through any
 * intermediate pulley anchors purely to compute total length and each
 * endpoint's pull direction. Once taut, each endpoint gets an impulse
 * pulling it toward its own adjacent segment; fixed endpoints absorb none
 * of it, so all the correction goes to the free end (e.g. a weight hanging
 * off a pulley anchored to a wall).
 */
export function solveRopeTension(runtime: LevelRuntime, connections: ConnectionDef[]): void {
  for (const conn of connections) {
    if (conn.kind !== 'ROPE') continue;

    const fromPart = runtime.get(conn.from.partId);
    const toPart = runtime.get(conn.to.partId);
    if (!fromPart || !toPart) continue;

    const throughParts = conn.routedThrough.map((ref) => runtime.get(ref.partId)).filter((p) => p !== undefined);
    const chain = [fromPart, ...throughParts, toPart];
    const positions = chain.map((p) => p.rigidBody.translation());

    let totalLength = 0;
    for (let i = 0; i < positions.length - 1; i++) {
      const a = positions[i];
      const b = positions[i + 1];
      if (a && b) totalLength += length(sub(b, a));
    }

    // Level JSON expresses maxLength in world-space units, same as x/y (GDD §2.2); positions here are meters.
    const maxLength = conn.maxLength !== undefined ? conn.maxLength / SIM.PIXELS_PER_METER : totalLength;
    if (totalLength <= maxLength) continue; // slack rope pulls on nothing

    const overstretch = totalLength - maxLength;
    const impulseMag = Math.min(TENSION_STIFFNESS * overstretch, MAX_TENSION_IMPULSE);

    const p0 = positions[0];
    const p1 = positions[1];
    const pLast = positions[positions.length - 1];
    const pBeforeLast = positions[positions.length - 2];
    if (!p0 || !p1 || !pLast || !pBeforeLast) continue;

    if (!fromPart.rigidBody.isFixed()) {
      const dir = normalize(sub(p1, p0));
      fromPart.rigidBody.applyImpulse({ x: dir.x * impulseMag, y: dir.y * impulseMag }, true);
    }
    if (!toPart.rigidBody.isFixed()) {
      const dir = normalize(sub(pBeforeLast, pLast));
      toPart.rigidBody.applyImpulse({ x: dir.x * impulseMag, y: dir.y * impulseMag }, true);
    }
  }
}
