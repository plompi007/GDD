import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { LevelRuntime, RuntimePart } from '../level/LevelRuntime.ts';
import { SIM } from '../sim/constants.ts';

const CANDLE_RADIUS_M = 20 / SIM.PIXELS_PER_METER; // GDD §1.3.ו: "ברדיוס 20 יח'"
const FUSE_IGNITE_RADIUS_M = 8 / SIM.PIXELS_PER_METER; // GDD §1.4.3: "ברדיוס 8"
const FUSE_BURN_RATE_MPS = 45 / SIM.PIXELS_PER_METER; // GDD §1.3.ו: "45 יח'/שנייה"
const FLAMMABLE_BURN_MS = 800; // GDD §1.4.2

interface ThermalSource {
  x: number;
  y: number;
  radius: number;
}

function distance(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

/** An unlit fuse can only be ignited at its unburned "start" end, not its center. */
function ignitionPoint(part: RuntimePart): { x: number; y: number } {
  if (part.def.partType !== 'fuse_cord') return part.rigidBody.translation();
  const lengthM = typeof part.params.length === 'number' ? part.params.length : 5;
  const pos = part.rigidBody.translation();
  const rot = part.rigidBody.rotation();
  return { x: pos.x - Math.cos(rot) * (lengthM / 2), y: pos.y - Math.sin(rot) * (lengthM / 2) };
}

function explodeChargeBarrel(world: RAPIER.World, runtime: LevelRuntime, barrel: RuntimePart): void {
  const center = barrel.rigidBody.translation();
  const radius = typeof barrel.params.radius === 'number' ? barrel.params.radius : 3;
  const power = typeof barrel.params.power === 'number' ? barrel.params.power : 3;

  for (const part of runtime.all()) {
    if (part.id === barrel.id) continue;
    const p = part.rigidBody.translation();
    const d = distance(p, center);
    if (d > radius) continue;

    if (part.tags.includes('DESTRUCTIBLE')) {
      runtime.remove(world, part.id);
      continue;
    }
    if (!part.rigidBody.isFixed()) {
      const dir = d > 1e-6 ? { x: (p.x - center.x) / d, y: (p.y - center.y) / d } : { x: 0, y: -1 };
      const falloff = 1 - d / radius;
      const impulseMag = power * 3 * falloff;
      part.rigidBody.applyImpulse({ x: dir.x * impulseMag, y: dir.y * impulseMag }, true);
    }
  }
  runtime.remove(world, barrel.id);
}

/**
 * Candles emit heat continuously; fuse cords catch fire from any thermal
 * source, burn along their length at a fixed rate, and relight at their far
 * end (GDD §1.4.3); anything FLAMMABLE burns away after 800ms of exposure,
 * anything POPPABLE pops immediately, and charge_barrel detonates once —
 * destroying nearby DESTRUCTIBLE parts and flinging everything else.
 */
export function updateThermalSystem(
  world: RAPIER.World,
  runtime: LevelRuntime,
  dtSeconds: number,
  nowMs: number,
): void {
  const sources: ThermalSource[] = [];

  for (const candle of runtime.getByPartType('candle')) {
    if (candle.state.lit === undefined) candle.state.lit = candle.params.startsLit !== false;
    if (candle.state.lit) {
      const p = candle.rigidBody.translation();
      sources.push({ x: p.x, y: p.y, radius: CANDLE_RADIUS_M });
    }
  }

  for (const fuse of runtime.getByPartType('fuse_cord')) {
    if (!fuse.state.burning) continue;

    const lengthM = typeof fuse.params.length === 'number' ? fuse.params.length : 5;
    const progress = (Number(fuse.state.burnProgress) || 0) + FUSE_BURN_RATE_MPS * dtSeconds;
    const pos = fuse.rigidBody.translation();
    const rot = fuse.rigidBody.rotation();
    const dir = { x: Math.cos(rot), y: Math.sin(rot) };
    const half = lengthM / 2;
    const travelled = Math.min(progress, lengthM);
    const frontPos = { x: pos.x + dir.x * (-half + travelled), y: pos.y + dir.y * (-half + travelled) };
    sources.push({ x: frontPos.x, y: frontPos.y, radius: FUSE_IGNITE_RADIUS_M });

    if (progress >= lengthM) {
      runtime.remove(world, fuse.id);
    } else {
      fuse.state.burnProgress = progress;
    }
  }

  for (const source of sources) {
    for (const part of runtime.all()) {
      if (!part.tags.includes('FLAMMABLE')) continue;
      if (part.state.burning || part.state.lit) continue;
      const p = ignitionPoint(part);
      if (distance(p, source) > source.radius) continue;

      if (part.def.partType === 'fuse_cord') {
        part.state.burning = true;
        part.state.burnProgress = 0;
      } else if (part.tags.includes('POPPABLE')) {
        runtime.remove(world, part.id);
      } else {
        if (part.state.burnStartMs === undefined) {
          part.state.burnStartMs = nowMs;
        } else if (nowMs - Number(part.state.burnStartMs) >= FLAMMABLE_BURN_MS) {
          runtime.remove(world, part.id);
        }
      }
    }
  }

  for (const barrel of runtime.getByPartType('charge_barrel')) {
    if (barrel.state.triggered) continue;
    const p = barrel.rigidBody.translation();
    const touched = sources.some((s) => distance(p, s) <= s.radius + 0.4);
    if (!touched) continue;
    barrel.state.triggered = true;
    explodeChargeBarrel(world, runtime, barrel);
  }
}
