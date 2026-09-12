import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { buildBody } from '../../../src/core/sim/BodyFactory.ts';
import { SimWorld } from '../../../src/core/sim/SimWorld.ts';
import { registerTest } from '../registry.ts';

/**
 * A focused correctness test for the PIVOT/revolute-joint mechanism used by
 * lever_seesaw (GDD §1.3.C), independent of any specific level's tuning:
 * a lever pinned at its center to a fixed point, loaded on one end, should
 * tip toward the loaded side and stay physically stable (no joint blow-up).
 */
registerTest('pivot joint: a loaded lever tips toward the weight without exploding', () => {
  const world = new SimWorld();

  const { rigidBody: pivotBody } = buildBody(
    world.rapier,
    { type: 'fixed', shape: { kind: 'ball', radius: 0.05 } },
    { x: 10, y: 10 },
  );
  const { rigidBody: leverBody } = buildBody(
    world.rapier,
    { type: 'dynamic', shape: { kind: 'box', w: 3, h: 0.3 }, mass: 2, friction: 0.4 },
    { x: 10, y: 10 },
  );
  world.rapier.createImpulseJoint(RAPIER.JointData.revolute({ x: 0, y: 0 }, { x: 0, y: 0 }), pivotBody, leverBody, true);

  // A weight resting on the right end (+x) should tip that side down (+y).
  buildBody(world.rapier, { type: 'dynamic', shape: { kind: 'ball', radius: 0.3 }, mass: 5 }, { x: 11.3, y: 9.6 });

  for (let i = 0; i < 300; i++) world.step();

  const pivotPos = pivotBody.translation();
  const leverPos = leverBody.translation();
  const drift = Math.hypot(leverPos.x - pivotPos.x, leverPos.y - pivotPos.y);
  if (drift > 0.5) {
    throw new Error(`joint drifted apart (distance ${drift.toFixed(3)}m) — likely unstable`);
  }

  const rotation = leverBody.rotation();
  if (rotation <= 0) {
    throw new Error(`expected the lever to tip positively (weighted +x side down), got rotation=${rotation}`);
  }

  world.free();
});
