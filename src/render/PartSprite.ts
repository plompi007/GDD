import { Graphics } from 'pixi.js';
import type { PartDef } from '../core/parts/PartDef.ts';
import { SIM } from '../core/sim/constants.ts';
import { THEME } from './theme.ts';

const { color } = THEME;
const OUTLINE = { width: 2.5, color: color.outline, alignment: 1 as const };
const OUTLINE_THIN = { width: 1.5, color: color.outline, alignment: 1 as const };

function glossHighlight(g: Graphics, radius: number): void {
  g.ellipse(-radius * 0.32, -radius * 0.35, radius * 0.4, radius * 0.28).fill({ color: 0xffffff, alpha: 0.55 });
}

function drawBall(g: Graphics, radius: number, fill: number): void {
  g.circle(0, 0, radius).fill(fill).stroke(OUTLINE);
  glossHighlight(g, radius);
}

function drawWoodPlank(g: Graphics, w: number, h: number): void {
  const r = Math.min(6, h / 3);
  g.roundRect(-w / 2, -h / 2, w, h, r).fill(color.wood).stroke(OUTLINE);
  const stripeCount = Math.max(1, Math.floor(w / 44));
  for (let i = 1; i <= stripeCount; i++) {
    const x = -w / 2 + (w / (stripeCount + 1)) * i;
    g.moveTo(x, -h / 2 + 3).lineTo(x, h / 2 - 3).stroke({ width: 2, color: color.woodDark, alpha: 0.6 });
  }
}

function drawFloor(g: Graphics, w: number, h: number): void {
  g.rect(-w / 2, -h / 2, w, h).fill(color.wood).stroke(OUTLINE);
  const plankWidth = 90;
  const count = Math.ceil(w / plankWidth);
  for (let i = 1; i < count; i++) {
    const x = -w / 2 + i * plankWidth;
    g.moveTo(x, -h / 2).lineTo(x, h / 2).stroke({ width: 2, color: color.woodDark, alpha: 0.55 });
  }
  g.rect(-w / 2, -h / 2, w, 5).fill({ color: color.brass, alpha: 0.9 });
}

function drawMetalBeam(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, 3).fill(color.steel).stroke(OUTLINE);
  g.rect(-w / 2, -h / 2, w, Math.max(2, h * 0.25)).fill({ color: 0xffffff, alpha: 0.35 });
  for (const bx of [-w / 2 + 9, w / 2 - 9]) g.circle(bx, 0, Math.min(3.5, h / 3)).fill(color.steelDark).stroke(OUTLINE_THIN);
}

function drawBrick(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, 3).fill(0xd97455).stroke(OUTLINE);
  const rows = Math.max(1, Math.round(h / 20));
  const rowH = h / rows;
  for (let r = 0; r < rows; r++) {
    const y = -h / 2 + r * rowH;
    g.moveTo(-w / 2, y).lineTo(w / 2, y).stroke({ width: 1.5, color: color.ink, alpha: 0.35 });
    const offset = r % 2 === 0 ? 0 : w / 3;
    g.moveTo(-w / 2 + offset, y).lineTo(-w / 2 + offset, y + rowH).stroke({ width: 1.5, color: color.ink, alpha: 0.3 });
  }
}

function drawGear(g: Graphics, radius: number): void {
  const teeth = radius > 0.45 * SIM.PIXELS_PER_METER ? 12 : 8;
  const toothLen = radius * 0.24;
  const dTheta = (Math.PI / teeth) * 0.4;
  const outer = radius + toothLen;

  g.circle(0, 0, radius).fill(color.brass);
  for (let i = 0; i < teeth; i++) {
    const a = (i / teeth) * Math.PI * 2;
    g.poly([
      Math.cos(a - dTheta) * radius,
      Math.sin(a - dTheta) * radius,
      Math.cos(a - dTheta) * outer,
      Math.sin(a - dTheta) * outer,
      Math.cos(a + dTheta) * outer,
      Math.sin(a + dTheta) * outer,
      Math.cos(a + dTheta) * radius,
      Math.sin(a + dTheta) * radius,
    ]).fill(color.brass);
  }
  g.circle(0, 0, radius + toothLen).stroke(OUTLINE);
  g.circle(0, 0, radius * 0.55).fill(color.brassDark).stroke(OUTLINE_THIN);
  g.circle(0, 0, radius * 0.16).fill(color.ink);
}

function drawPulley(g: Graphics, radius: number): void {
  g.circle(0, 0, radius).fill(color.steel).stroke(OUTLINE);
  g.circle(0, 0, radius * 0.85).stroke({ width: 2.5, color: color.brass });
  for (let i = 0; i < 4; i++) {
    const a = (i / 4) * Math.PI * 2 + Math.PI / 4;
    g.moveTo(0, 0).lineTo(Math.cos(a) * radius * 0.75, Math.sin(a) * radius * 0.75).stroke({ width: 2.5, color: color.steelDark });
  }
  g.circle(0, 0, radius * 0.22).fill(color.brass).stroke(OUTLINE_THIN);
}

function drawPowerPanel(g: Graphics, w: number, h: number, accent: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, 5).fill(color.inkSoft).stroke(OUTLINE);
  g.roundRect(-w / 2 + 4, -h / 2 + 4, w - 8, h - 8, 4).fill(color.steel);
  g.circle(0, 0, Math.min(w, h) * 0.22).fill(accent).stroke(OUTLINE_THIN);
  const corners: [number, number][] = [
    [-w / 2 + 6, -h / 2 + 6],
    [w / 2 - 6, -h / 2 + 6],
    [-w / 2 + 6, h / 2 - 6],
    [w / 2 - 6, h / 2 - 6],
  ];
  for (const [sx, sy] of corners) g.circle(sx, sy, 1.8).fill(color.brass);
}

function drawSensorZone(g: Graphics, w: number, hgt: number, kind: 'ball' | 'box', accent: number): void {
  if (kind === 'ball') g.circle(0, 0, w).fill({ color: accent, alpha: 0.4 });
  else g.roundRect(-w / 2, -hgt / 2, w, hgt, 8).fill({ color: accent, alpha: 0.4 });

  const r = kind === 'ball' ? w : Math.max(w, hgt) / 2;
  const dashCount = 16;
  for (let i = 0; i < dashCount; i++) {
    if (i % 2 === 0) continue;
    const a0 = (i / dashCount) * Math.PI * 2;
    const a1 = ((i + 0.7) / dashCount) * Math.PI * 2;
    g.moveTo(Math.cos(a0) * r, Math.sin(a0) * r).arc(0, 0, r, a0, a1).stroke({ width: 3, color: accent });
  }
  // Simple flag mark so a goal reads instantly as "target", not just a shape.
  g.moveTo(0, r * 0.05).lineTo(0, -r * 0.55).stroke({ width: 2, color: color.ink });
  g.poly([0, -r * 0.55, r * 0.35, -r * 0.4, 0, -r * 0.25]).fill(accent).stroke(OUTLINE_THIN);
}

function drawFlammableStick(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, h / 2).fill(0xc99a6f).stroke(OUTLINE);
  const segs = Math.max(4, Math.floor(w / 10));
  for (let i = 0; i < segs; i++) {
    if (i % 2 === 0) continue;
    const x = -w / 2 + (w / segs) * i;
    g.rect(x, -h / 2, w / segs, h).fill({ color: color.ink, alpha: 0.3 });
  }
}

function drawCandle(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2 + 4, w, h - 4, 2).fill(0xfdf3e0).stroke(OUTLINE);
  g.poly([0, -h / 2 + 4, -w * 0.35, -h / 2 - 10, w * 0.35, -h / 2 - 10]).fill(color.electric).stroke(OUTLINE_THIN);
  g.poly([0, -h / 2, -w * 0.15, -h / 2 - 5, w * 0.15, -h / 2 - 5]).fill(color.danger);
}

function drawBarrel(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, w * 0.2).fill(color.dangerDark).stroke(OUTLINE);
  for (const frac of [-0.28, 0.02, 0.32]) {
    g.rect(-w / 2, h * frac - 2.5, w, 5).fill(0xfdf3e0);
  }
}

const CATEGORY_FALLBACK: Record<PartDef['category'], number> = {
  STATIC: color.wood,
  DYNAMIC: color.rubber,
  MECHANISM: color.brass,
  POWER: color.electric,
  PNEUMATIC: color.pneumatic,
  THERMAL: color.danger,
  LIGHT: color.electric,
  ACTUATOR: color.brass,
  GOAL: color.success,
};

/** Draws a themed sprite sized from the part's own BodySpec — bold, flat, high-contrast shapes. */
export function spriteForPart(def: PartDef): Graphics {
  const g = new Graphics();
  const shape = def.body.shape;
  const isBall = shape.kind === 'ball';
  const w = isBall ? shape.radius * 2 * SIM.PIXELS_PER_METER : shape.w * SIM.PIXELS_PER_METER;
  const h = isBall ? w : shape.h * SIM.PIXELS_PER_METER;

  if (def.body.isSensor) {
    const accent = def.category === 'GOAL' ? color.success : def.category === 'POWER' ? color.electric : color.pneumatic;
    drawSensorZone(g, isBall ? shape.radius * SIM.PIXELS_PER_METER : w, h, isBall ? 'ball' : 'box', accent);
    return g;
  }

  switch (def.partType) {
    case 'gear_small':
    case 'gear_large':
      drawGear(g, shape.kind === 'ball' ? shape.radius * SIM.PIXELS_PER_METER : w / 2);
      return g;
    case 'pulley_wheel':
      drawPulley(g, shape.kind === 'ball' ? shape.radius * SIM.PIXELS_PER_METER : w / 2);
      return g;
    case 'floor_ground':
      drawFloor(g, w, h);
      return g;
    case 'plank_wood':
    case 'lever_seesaw':
      drawWoodPlank(g, w, h);
      return g;
    case 'wall_brick':
      drawBrick(g, w, h);
      return g;
    case 'fuse_cord':
      drawFlammableStick(g, w, h);
      return g;
    case 'candle':
      drawCandle(g, w, h);
      return g;
    case 'charge_barrel':
      drawBarrel(g, w, h);
      return g;
    default:
      break;
  }

  if (def.tags.includes('METALLIC') && !isBall) {
    drawMetalBeam(g, w, h);
    return g;
  }

  if (def.category === 'POWER' || def.category === 'MECHANISM' || def.category === 'PNEUMATIC') {
    const accent = def.category === 'PNEUMATIC' ? color.pneumatic : color.electric;
    drawPowerPanel(g, w, h, accent);
    return g;
  }

  if (isBall) {
    const fill = def.tags.includes('METALLIC')
      ? color.lead
      : def.tags.includes('FRAGILE')
        ? color.glass
        : def.tags.includes('BOUNCY')
          ? color.rubber
          : def.tags.includes('WIND')
            ? 0xfdf3e0
            : CATEGORY_FALLBACK[def.category];
    drawBall(g, w / 2, fill);
    return g;
  }

  const base = CATEGORY_FALLBACK[def.category];
  g.roundRect(-w / 2, -h / 2, w, h, 4).fill(base).stroke(OUTLINE);
  return g;
}
