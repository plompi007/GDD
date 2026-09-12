import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Graphics } from 'pixi.js';
import { SIM } from './core/sim/constants.ts';

// PixiJS's async Application.init() never resolves when awaited at module top
// level in a Vite production build (github.com/pixijs/pixijs/issues/10456).
// Wrapping in an async IIFE avoids top-level await entirely.
async function main(): Promise<void> {
  const app = new Application();
  await app.init({ background: '#2a2420', resizeTo: window, preference: 'webgl' });
  document.getElementById('app')!.appendChild(app.canvas);

  const world = new RAPIER.World(SIM.GRAVITY);

  // Frame a small demo scene against the actual screen size (the full
  // WORLD_WIDTH/HEIGHT game-world constants apply once levels exist, from M2 on).
  const screenWidthM = app.screen.width / SIM.PIXELS_PER_METER;
  const screenHeightM = app.screen.height / SIM.PIXELS_PER_METER;
  const floorHalfWidthM = screenWidthM / 2;
  const floorHalfHeightM = 0.5;
  const floorYM = screenHeightM - 2;

  const floorDesc = RAPIER.RigidBodyDesc.fixed().setTranslation(screenWidthM / 2, floorYM);
  const floorBody = world.createRigidBody(floorDesc);
  world.createCollider(RAPIER.ColliderDesc.cuboid(floorHalfWidthM, floorHalfHeightM), floorBody);

  const boxDesc = RAPIER.RigidBodyDesc.dynamic().setTranslation(screenWidthM / 2, 2);
  const boxBody = world.createRigidBody(boxDesc);
  world.createCollider(RAPIER.ColliderDesc.cuboid(0.5, 0.5), boxBody);

  const floorSprite = new Graphics()
    .rect(
      -floorHalfWidthM * SIM.PIXELS_PER_METER,
      -floorHalfHeightM * SIM.PIXELS_PER_METER,
      floorHalfWidthM * 2 * SIM.PIXELS_PER_METER,
      floorHalfHeightM * 2 * SIM.PIXELS_PER_METER,
    )
    .fill(0xc9973f);
  app.stage.addChild(floorSprite);

  const boxSprite = new Graphics()
    .rect(-0.5 * SIM.PIXELS_PER_METER, -0.5 * SIM.PIXELS_PER_METER, 1 * SIM.PIXELS_PER_METER, 1 * SIM.PIXELS_PER_METER)
    .fill(0xd9584b);
  app.stage.addChild(boxSprite);

  function syncSprite(sprite: Graphics, body: RAPIER.RigidBody): void {
    const t = body.translation();
    sprite.position.set(t.x * SIM.PIXELS_PER_METER, t.y * SIM.PIXELS_PER_METER);
    sprite.rotation = body.rotation();
  }

  syncSprite(floorSprite, floorBody);
  syncSprite(boxSprite, boxBody);

  let accumulator = 0;
  app.ticker.add((ticker) => {
    accumulator += Math.min(ticker.deltaMS / 1000, 0.1);
    let steps = 0;
    while (accumulator >= SIM.FIXED_DT && steps < SIM.MAX_SUBSTEPS_PER_FRAME) {
      world.step();
      accumulator -= SIM.FIXED_DT;
      steps++;
    }
    if (steps === SIM.MAX_SUBSTEPS_PER_FRAME) accumulator = 0;

    syncSprite(boxSprite, boxBody);
  });
}

void main();
