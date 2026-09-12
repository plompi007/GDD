import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Graphics } from 'pixi.js';
import { buildBody } from './core/sim/BodyFactory.ts';
import { SIM } from './core/sim/constants.ts';
import { FixedStepLoop } from './core/sim/FixedStepLoop.ts';
import { SimWorld } from './core/sim/SimWorld.ts';

// PixiJS's async Application.init() never resolves when awaited at module top
// level in a Vite production build (github.com/pixijs/pixijs/issues/10456).
// Wrapping in an async IIFE avoids top-level await entirely.
async function main(): Promise<void> {
  const app = new Application();
  await app.init({ background: '#2a2420', resizeTo: window, preference: 'webgl' });
  document.getElementById('app')!.appendChild(app.canvas);

  const simWorld = new SimWorld();

  // Frame a small demo scene against the actual screen size (the full
  // WORLD_WIDTH/HEIGHT game-world constants apply once levels exist, from M2 on).
  const screenWidthM = app.screen.width / SIM.PIXELS_PER_METER;
  const screenHeightM = app.screen.height / SIM.PIXELS_PER_METER;
  const floorHalfWidthM = screenWidthM / 2;
  const floorYM = screenHeightM - 2;

  const { rigidBody: floorBody } = buildBody(
    simWorld.rapier,
    { type: 'fixed', shape: { kind: 'box', w: floorHalfWidthM * 2, h: 1 } },
    { x: screenWidthM / 2, y: floorYM },
  );

  const { rigidBody: boxBody } = buildBody(
    simWorld.rapier,
    { type: 'dynamic', shape: { kind: 'box', w: 1, h: 1 }, mass: 1, restitution: 0.3, friction: 0.5 },
    { x: screenWidthM / 2, y: 2 },
  );

  const floorSprite = new Graphics()
    .rect(-floorHalfWidthM * SIM.PIXELS_PER_METER, -0.5 * SIM.PIXELS_PER_METER, floorHalfWidthM * 2 * SIM.PIXELS_PER_METER, 1 * SIM.PIXELS_PER_METER)
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

  const loop = new FixedStepLoop({
    isRunning: () => true,
    onTick: () => simWorld.step(),
    onRender: () => syncSprite(boxSprite, boxBody),
  });

  app.ticker.add((ticker) => loop.frame(ticker.deltaMS / 1000));
}

void main();
