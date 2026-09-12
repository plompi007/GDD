import { buildBody } from '../../../src/core/sim/BodyFactory.ts';
import { SimWorld } from '../../../src/core/sim/SimWorld.ts';
import { registerTest } from '../registry.ts';

/**
 * spring_bouncer (data/parts/spring_bouncer.json) uses an energy-adding
 * restitution above 1.0 to launch objects, rather than a real spring force —
 * that's an unusual value for a physics engine, so this confirms Rapier
 * actually applies it (a ball rebounds higher than its drop height) instead
 * of silently clamping to 1.0 or going unstable.
 */
registerTest('spring bouncer: restitution above 1.0 launches a ball higher than its drop height', () => {
  const world = new SimWorld();

  const startY = 10;
  buildBody(world.rapier, { type: 'fixed', shape: { kind: 'box', w: 1.0, h: 0.5 }, restitution: 1.15, friction: 0.1 }, { x: 10, y: 12 });
  const { rigidBody: ball } = buildBody(world.rapier, { type: 'dynamic', shape: { kind: 'ball', radius: 0.2 }, mass: 1.0, restitution: 1.15 }, { x: 10, y: startY });

  // y increases downward here, so the landing point is the max y reached
  // while falling, and the post-bounce peak is the min y reached afterward.
  let landingY = -Infinity;
  let peakAfterBounceY = Infinity;
  let bounced = false;
  for (let i = 0; i < 600; i++) {
    world.step();
    const y = ball.translation().y;
    if (!bounced) {
      landingY = Math.max(landingY, y);
      if (ball.linvel().y < -0.01) bounced = true;
    } else {
      peakAfterBounceY = Math.min(peakAfterBounceY, y);
    }
  }

  if (!bounced) throw new Error('ball never rebounded off the spring');
  const dropDistance = landingY - startY;
  const reboundDistance = landingY - peakAfterBounceY;
  if (reboundDistance <= dropDistance) {
    throw new Error(`expected an energy-adding bounce higher than the ${dropDistance.toFixed(2)}m drop, got ${reboundDistance.toFixed(2)}m`);
  }

  world.free();
});
