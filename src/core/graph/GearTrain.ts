import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { LevelRuntime, RuntimePart } from '../level/LevelRuntime.ts';
import type { ConnectionDef } from '../level/LevelSchema.ts';

const MESH_EPSILON = 0.08; // meters of slack allowed when deciding two gears "mesh" (GDD §1.3.C)
const MESH_PASSES = 4;
const CONVEYOR_SURFACE_RADIUS = 0.3; // m — converts an incoming angularVelocity (rad/s) to a belt surface speed (m/s)
const GEAR_TYPES = new Set(['gear_small', 'gear_large']);

function rotate(v: { x: number; y: number }, angle: number): { x: number; y: number } {
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  return { x: v.x * cos - v.y * sin, y: v.x * sin + v.y * cos };
}

function anchorWorldPos(part: RuntimePart, anchorIdx = 0): { x: number; y: number } {
  const anchor = part.def.anchors[anchorIdx];
  const t = part.rigidBody.translation();
  if (!anchor) return t;
  const offset = rotate(anchor.offset, part.rigidBody.rotation());
  return { x: t.x + offset.x, y: t.y + offset.y };
}

function radiusOf(part: RuntimePart): number {
  return part.def.body.shape.kind === 'ball' ? part.def.body.shape.radius : 0;
}

function angularVelocityFromParams(part: RuntimePart): number {
  const rpm = typeof part.params.rpm === 'number' ? part.params.rpm : 0;
  const direction = part.params.direction === 'CCW' ? -1 : 1;
  return ((rpm * 2 * Math.PI) / 60) * direction;
}

/**
 * Spins motors from ElectricBus power, meshes gears by proximity (inverting
 * direction, scaling by radius ratio — GDD §1.4.1), links BELT connections
 * 1:1 with no inversion, and drives conveyor surface velocity from whatever
 * feeds its ROTARY-in port.
 */
export function applyGearTrain(runtime: LevelRuntime, connections: ConnectionDef[]): void {
  const motors = runtime.getByPartType('motor_electric');
  const gears = [...runtime.getByPartType('gear_small'), ...runtime.getByPartType('gear_large')];
  const conveyors = runtime.getByPartType('conveyor');
  const belts = connections.filter((c) => c.kind === 'BELT');

  for (const motor of motors) {
    motor.state.angularVelocity = motor.state.powered ? angularVelocityFromParams(motor) : 0;
  }
  for (const part of [...gears, ...conveyors]) part.state.angularVelocity = 0;

  const drivers = [...motors, ...gears];

  for (let pass = 0; pass < MESH_PASSES; pass++) {
    for (const gear of gears) {
      const gearPos = anchorWorldPos(gear);
      const gearRadius = radiusOf(gear);
      for (const other of drivers) {
        if (other === gear) continue;
        const otherVel = Number(other.state.angularVelocity) || 0;
        if (otherVel === 0) continue;
        const otherPos = anchorWorldPos(other);
        const otherRadius = radiusOf(other);
        const dist = Math.hypot(gearPos.x - otherPos.x, gearPos.y - otherPos.y);
        if (dist > gearRadius + otherRadius + MESH_EPSILON) continue;

        const isOtherGear = GEAR_TYPES.has(other.def.partType);
        const ratio = isOtherGear && otherRadius > 0 ? otherRadius / gearRadius : 1;
        // Two meshed gears spin opposite ways; a motor drives its first gear the same way its shaft turns.
        const sign = isOtherGear ? -1 : 1;
        gear.state.angularVelocity = otherVel * ratio * sign;
        break;
      }
    }

    for (const belt of belts) {
      const from = runtime.get(belt.from.partId);
      const to = runtime.get(belt.to.partId);
      if (!from || !to) continue;
      const fromVel = Number(from.state.angularVelocity) || 0;
      const toVel = Number(to.state.angularVelocity) || 0;
      if (fromVel !== 0 && toVel === 0) to.state.angularVelocity = fromVel;
      else if (toVel !== 0 && fromVel === 0) from.state.angularVelocity = toVel;
    }
  }
}

/** Nudges any body currently touching a conveyor toward its belt's surface speed. */
export function applyConveyorSurfaces(world: RAPIER.World, runtime: LevelRuntime): void {
  for (const conveyor of runtime.getByPartType('conveyor')) {
    const surfaceSpeed = (Number(conveyor.state.angularVelocity) || 0) * CONVEYOR_SURFACE_RADIUS;
    if (surfaceSpeed === 0) continue;
    world.contactPairsWith(conveyor.collider, (otherCollider) => {
      const body = otherCollider.parent();
      if (!body || body.isFixed()) return;
      const v = body.linvel();
      body.setLinvel({ x: v.x + (surfaceSpeed - v.x) * 0.3, y: v.y }, true);
    });
  }
}
