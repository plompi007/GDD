import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Graphics } from 'pixi.js';
import { buildBody } from './core/sim/BodyFactory.ts';
import { SIM } from './core/sim/constants.ts';
import { FixedStepLoop } from './core/sim/FixedStepLoop.ts';
import { SimWorld } from './core/sim/SimWorld.ts';
import type { PartDef } from './core/parts/PartDef.ts';
import { loadPartRegistry } from './core/parts/PartRegistry.ts';

const CATEGORY_COLOR: Record<PartDef['category'], number> = {
  STATIC: 0xc9973f,
  DYNAMIC: 0xd9584b,
  MECHANISM: 0xfa7a13,
  POWER: 0xfa7a13,
  PNEUMATIC: 0x8fb8de,
  THERMAL: 0xd9584b,
  LIGHT: 0xf3e39a,
  ACTUATOR: 0xfa7a13,
  GOAL: 0x3fa77a,
};

/** Draws a shape sized straight from the part's own BodySpec — no hand-authored geometry to keep in sync. */
function spriteForPart(def: PartDef): Graphics {
  const g = new Graphics();
  const color = CATEGORY_COLOR[def.category];
  if (def.body.shape.kind === 'ball') {
    g.circle(0, 0, def.body.shape.radius * SIM.PIXELS_PER_METER).fill(color);
  } else {
    const w = def.body.shape.w * SIM.PIXELS_PER_METER;
    const h = def.body.shape.h * SIM.PIXELS_PER_METER;
    g.rect(-w / 2, -h / 2, w, h).fill(color);
  }
  return g;
}

function syncSprite(sprite: Graphics, body: RAPIER.RigidBody): void {
  const t = body.translation();
  sprite.position.set(t.x * SIM.PIXELS_PER_METER, t.y * SIM.PIXELS_PER_METER);
  sprite.rotation = body.rotation();
}

// PixiJS's async Application.init() never resolves when awaited at module top
// level in a Vite production build (github.com/pixijs/pixijs/issues/10456).
// Wrapping in an async IIFE avoids top-level await entirely.
async function main(): Promise<void> {
  const app = new Application();
  await app.init({ background: '#2a2420', resizeTo: window, preference: 'webgl' });
  document.getElementById('app')!.appendChild(app.canvas);

  const parts = loadPartRegistry();
  const simWorld = new SimWorld();

  // Temporary hard-coded scene (M2 acceptance: "balls roll on planks"). The
  // real level format arrives in M3's LevelLoader.
  const screenWidthM = app.screen.width / SIM.PIXELS_PER_METER;
  const screenHeightM = app.screen.height / SIM.PIXELS_PER_METER;
  const centerXM = screenWidthM / 2;

  interface Placed {
    def: PartDef;
    body: RAPIER.RigidBody;
    sprite: Graphics;
  }
  const placed: Placed[] = [];

  function place(partType: string, x: number, y: number, rotationDeg = 0): Placed {
    const def = parts.get(partType);
    const { rigidBody } = buildBody(simWorld.rapier, def.body, { x, y, rotation: (rotationDeg * Math.PI) / 180 });
    const sprite = spriteForPart(def);
    app.stage.addChild(sprite);
    syncSprite(sprite, rigidBody);
    const entry: Placed = { def, body: rigidBody, sprite };
    placed.push(entry);
    return entry;
  }

  place('floor_ground', centerXM, screenHeightM - 1);
  place('plank_wood', centerXM - 3, 3, 20);
  place('plank_wood', centerXM + 3, 6, -20);
  place('ball_wood', centerXM - 3, 1.5);
  place('ball_rubber', centerXM + 3, 4.5);

  const dynamicBodies = placed.filter((p) => p.def.body.type === 'dynamic');

  const loop = new FixedStepLoop({
    isRunning: () => true,
    onTick: () => simWorld.step(),
    onRender: () => {
      for (const p of dynamicBodies) syncSprite(p.sprite, p.body);
    },
  });

  app.ticker.add((ticker) => loop.frame(ticker.deltaMS / 1000));
}

void main();
