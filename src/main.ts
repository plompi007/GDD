import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Container, Graphics, Text } from 'pixi.js';
import type { PartDef } from './core/parts/PartDef.ts';
import { loadPartRegistry } from './core/parts/PartRegistry.ts';
import { SIM } from './core/sim/constants.ts';
import { loadLevelById } from './core/level/LevelCatalog.ts';
import { GameSession } from './core/level/GameSession.ts';

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
  const alpha = def.body.isSensor ? 0.35 : 1;
  if (def.body.shape.kind === 'ball') {
    g.circle(0, 0, def.body.shape.radius * SIM.PIXELS_PER_METER).fill({ color, alpha });
  } else {
    const w = def.body.shape.w * SIM.PIXELS_PER_METER;
    const h = def.body.shape.h * SIM.PIXELS_PER_METER;
    g.rect(-w / 2, -h / 2, w, h).fill({ color, alpha });
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

  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a01_free_fall');
  const session = new GameSession(level, registry);

  const worldLayer = new Container();
  app.stage.addChild(worldLayer);
  // Fit the level's fixed WORLD_WIDTH/HEIGHT grid to whatever the screen is (GDD §3.7 CONTAIN fit).
  const fitScale = Math.min(app.screen.width / level.world.width, app.screen.height / level.world.height);
  worldLayer.scale.set(fitScale);

  const label = new Text({
    text: '',
    style: { fill: 0xffffff, fontSize: 18 },
  });
  label.position.set(12, 12);
  app.stage.addChild(label);

  let sprites: { sprite: Graphics; body: RAPIER.RigidBody }[] = [];

  function rebuildSpritesFromRuntime(): void {
    worldLayer.removeChildren();
    sprites = [];
    if (!session.runtime) return;
    for (const part of session.runtime.all()) {
      const sprite = spriteForPart(part.def);
      worldLayer.addChild(sprite);
      syncSprite(sprite, part.rigidBody);
      sprites.push({ sprite, body: part.rigidBody });
    }
  }

  session.play();
  rebuildSpritesFromRuntime();

  window.addEventListener('keydown', (e) => {
    if (e.key === 'r') {
      session.reset();
      session.play();
      rebuildSpritesFromRuntime();
    }
  });

  app.ticker.add((ticker) => {
    session.frame(ticker.deltaMS / 1000);
    for (const s of sprites) syncSprite(s.sprite, s.body);
    label.text = `${level.title} — ${session.state}${session.failReason ? ` (${session.failReason})` : ''} — press R to reset`;
  });
}

void main();
