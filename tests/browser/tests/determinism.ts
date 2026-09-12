import { buildBody } from '../../../src/core/sim/BodyFactory.ts';
import { hashBytes } from '../../../src/core/sim/Determinism.ts';
import { SimWorld } from '../../../src/core/sim/SimWorld.ts';
import { registerTest } from '../registry.ts';

const TICKS = 600;

/**
 * A small scene exercising gravity, restitution, and friction across several
 * bodies — enough to catch any nondeterminism introduced by iteration order,
 * sleeping, or a stray Math.random()/Date.now() in the sim layer.
 */
function runScenario(): string {
  const world = new SimWorld();

  buildBody(world.rapier, { type: 'fixed', shape: { kind: 'box', w: 40, h: 1 } }, { x: 20, y: 30 });

  const drops = [
    { x: 5, y: 2, radius: 0.5 },
    { x: 10, y: 4, radius: 0.7 },
    { x: 15, y: 1, radius: 0.4 },
    { x: 25, y: 6, radius: 0.6 },
    { x: 30, y: 3, radius: 0.5 },
  ];
  for (const drop of drops) {
    buildBody(
      world.rapier,
      {
        type: 'dynamic',
        shape: { kind: 'ball', radius: drop.radius },
        mass: 1,
        restitution: 0.5,
        friction: 0.4,
      },
      { x: drop.x, y: drop.y },
    );
  }

  for (let i = 0; i < TICKS; i++) world.step();

  const hash = hashBytes(world.rapier.takeSnapshot());
  world.free();
  return hash;
}

registerTest('determinism: identical snapshot hash across 3 runs (GDD §2.6.7)', () => {
  const first = runScenario();
  const second = runScenario();
  const third = runScenario();

  if (second !== first || third !== first) {
    throw new Error(`hash mismatch: ${first} / ${second} / ${third}`);
  }
});
