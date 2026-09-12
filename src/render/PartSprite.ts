import { Graphics } from 'pixi.js';
import type { PartDef } from '../core/parts/PartDef.ts';
import { SIM } from '../core/sim/constants.ts';

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
export function spriteForPart(def: PartDef): Graphics {
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
