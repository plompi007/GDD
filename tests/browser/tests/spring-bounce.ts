import { buildBody } from '../../../src/core/sim/BodyFactory.ts';
import { SimWorld } from '../../../src/core/sim/SimWorld.ts';
import { registerTest } from '../registry.ts';

/**
 * spring_bouncer (data/parts/spring_bouncer.json) uses an energy-adding
 * restitution above 1.0 to launch objects, rather than a real spring force.
 * Rapier combines two colliders' restitution by averaging them, so the
 * spring's own value has to be declared well above 1.0 (2.0) to still net an
 * energy-adding bounce once averaged with a normal object's realistic
 * restitution (ball_baseball's 0.3) — this uses those exact real values to
 * confirm the combined bounce still clears the drop height, not a
 * matched-restitution pair that would hide the averaging effect.
 */
registerTest('spring bouncer: restitution above 1.0 launches a ball higher than its drop height', () => {
  const world = new SimWorld();

  const startY = 10;
  buildBody(world.rapier, { type: 'fixed', shape: { kind: 'box', w: 1.0, h: 0.5 }, restitution: 2.0, friction: 0.1 }, { x: 10, y: 12 });
  const { rigidBody: ball } = buildBody(world.rapier, { type: 'dynamic', shape: { kind: 'ball', radius: 0.35 }, mass: 1.0, restitution: 0.3 }, { x: 10, y: startY });

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
