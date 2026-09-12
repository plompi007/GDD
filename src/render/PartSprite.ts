import { Graphics, FillGradient } from 'pixi.js';
import type { PartDef } from '../core/parts/PartDef.ts';
import { SIM } from '../core/sim/constants.ts';
import { THEME } from './theme.ts';

const { color } = THEME;

/** A soft top-left highlight over a base color — the "polished object" look used throughout. */
function sphereGradient(base: number, highlight: number): FillGradient {
  return new FillGradient({
    type: 'radial',
    center: { x: 0.32, y: 0.3 },
    innerRadius: 0,
    outerCenter: { x: 0.5, y: 0.5 },
    outerRadius: 0.75,
    colorStops: [
      { offset: 0, color: highlight },
      { offset: 1, color: base },
    ],
    textureSpace: 'local',
  });
}

function metalGradient(dark: number, light: number, angleDeg = 45): FillGradient {
  const rad = (angleDeg * Math.PI) / 180;
  return new FillGradient({
    type: 'linear',
    start: { x: 0.5 - Math.cos(rad) * 0.6, y: 0.5 - Math.sin(rad) * 0.6 },
    end: { x: 0.5 + Math.cos(rad) * 0.6, y: 0.5 + Math.sin(rad) * 0.6 },
    colorStops: [
      { offset: 0, color: light },
      { offset: 0.5, color: dark },
      { offset: 1, color: light },
    ],
    textureSpace: 'local',
  });
}

function drawWoodPlank(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, Math.min(4, h / 4)).fill(color.oakMid);
  const grainCount = Math.max(2, Math.floor(w / 26));
  for (let i = 1; i < grainCount; i++) {
    const gx = -w / 2 + (w / grainCount) * i;
    g.moveTo(gx, -h / 2 + 1).lineTo(gx + h * 0.4, h / 2 - 1).stroke({ width: 1, color: color.walnutDeep, alpha: 0.35 });
  }
  g.roundRect(-w / 2, -h / 2, w, h, Math.min(4, h / 4)).stroke({ width: 1.5, color: color.walnutDeep, alpha: 0.6 });
}

function drawFloor(g: Graphics, w: number, h: number): void {
  g.rect(-w / 2, -h / 2, w, h).fill(color.oakMid);
  const plankWidth = 90;
  const count = Math.ceil(w / plankWidth);
  for (let i = 0; i <= count; i++) {
    const x = -w / 2 + i * plankWidth;
    g.moveTo(x, -h / 2).lineTo(x, h / 2).stroke({ width: 1.5, color: color.walnutDeep, alpha: 0.4 });
  }
  g.rect(-w / 2, -h / 2, w, 3).fill({ color: color.brass, alpha: 0.5 });
}

function drawMetalBeam(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, 2).fill(metalGradient(color.steelDark, color.steel, 90));
  for (const bx of [-w / 2 + 8, w / 2 - 8]) {
    g.circle(bx, 0, Math.min(3, h / 3)).fill(color.walnutDeep);
  }
  g.roundRect(-w / 2, -h / 2, w, h, 2).stroke({ width: 1, color: color.walnutDeep, alpha: 0.5 });
}

function drawBrick(g: Graphics, w: number, h: number): void {
  g.rect(-w / 2, -h / 2, w, h).fill(color.danger);
  const rows = Math.max(1, Math.round(h / 20));
  const rowH = h / rows;
  for (let r = 0; r < rows; r++) {
    const y = -h / 2 + r * rowH;
    g.moveTo(-w / 2, y).lineTo(w / 2, y).stroke({ width: 1, color: color.walnutDeep, alpha: 0.4 });
    const offset = r % 2 === 0 ? 0 : (w / 2) % (w / 3 || 1);
    g.moveTo(-w / 2 + offset, y).lineTo(-w / 2 + offset, y + rowH).stroke({ width: 1, color: color.walnutDeep, alpha: 0.3 });
  }
  g.rect(-w / 2, -h / 2, w, h).stroke({ width: 1.5, color: color.walnutDeep, alpha: 0.6 });
}

function drawGear(g: Graphics, radius: number): void {
  const teeth = radius > 0.45 * SIM.PIXELS_PER_METER ? 12 : 8;
  const toothLen = radius * 0.22;
  const dTheta = (Math.PI / teeth) * 0.4;
  const outer = radius + toothLen;

  g.circle(0, 0, radius).fill(metalGradient(color.brassDeep, color.brassBright, 45));
  for (let i = 0; i < teeth; i++) {
    const a = (i / teeth) * Math.PI * 2;
    const points = [
      Math.cos(a - dTheta) * radius,
      Math.sin(a - dTheta) * radius,
      Math.cos(a - dTheta) * outer,
      Math.sin(a - dTheta) * outer,
      Math.cos(a + dTheta) * outer,
      Math.sin(a + dTheta) * outer,
      Math.cos(a + dTheta) * radius,
      Math.sin(a + dTheta) * radius,
    ];
    g.poly(points).fill(color.brass);
  }
  g.circle(0, 0, radius * 0.62).fill(metalGradient(color.brassDeep, color.brassBright, 45));
  g.circle(0, 0, radius * 0.62).stroke({ width: 1, color: color.walnutDeep, alpha: 0.5 });
  g.circle(0, 0, radius * 0.18).fill(color.walnutDeep);
}

function drawPulley(g: Graphics, radius: number): void {
  g.circle(0, 0, radius).fill(metalGradient(color.steelDark, color.steel, 45));
  g.circle(0, 0, radius).stroke({ width: 2, color: color.brass });
  for (let i = 0; i < 5; i++) {
    const a = (i / 5) * Math.PI * 2;
    g.moveTo(0, 0).lineTo(Math.cos(a) * radius * 0.8, Math.sin(a) * radius * 0.8).stroke({ width: 1.5, color: color.walnutDeep, alpha: 0.5 });
  }
  g.circle(0, 0, radius * 0.2).fill(color.brassBright);
}

function drawPowerPanel(g: Graphics, w: number, h: number, accent: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, 4).fill(color.walnutDeep);
  g.roundRect(-w / 2 + 3, -h / 2 + 3, w - 6, h - 6, 3).fill(metalGradient(color.steelDark, color.steel, 60));
  g.rect(-w / 2 + 6, -h / 2 + 6, w - 12, 4).fill({ color: accent, alpha: 0.85 });
  const corners: [number, number][] = [
    [-w / 2 + 5, -h / 2 + 5],
    [w / 2 - 5, -h / 2 + 5],
    [-w / 2 + 5, h / 2 - 5],
    [w / 2 - 5, h / 2 - 5],
  ];
  for (const [sx, sy] of corners) {
    g.circle(sx, sy, 1.6).fill(color.brass);
  }
}

function drawSensorZone(g: Graphics, w: number, hgt: number, kind: 'ball' | 'box', accent: number): void {
  const alpha = 0.28;
  if (kind === 'ball') g.circle(0, 0, w).fill({ color: accent, alpha });
  else g.roundRect(-w / 2, -hgt / 2, w, hgt, 6).fill({ color: accent, alpha });
  const dashCount = 14;
  const r = kind === 'ball' ? w : Math.max(w, hgt) / 2;
  for (let i = 0; i < dashCount; i++) {
    if (i % 2 === 0) continue;
    const a0 = (i / dashCount) * Math.PI * 2;
    const a1 = ((i + 0.7) / dashCount) * Math.PI * 2;
    g.moveTo(Math.cos(a0) * r, Math.sin(a0) * r).arc(0, 0, r, a0, a1).stroke({ width: 2, color: accent, alpha: 0.85 });
  }
}

function drawFlammableStick(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, h / 2).fill(color.brassDeep);
  const segs = Math.max(4, Math.floor(w / 10));
  for (let i = 0; i < segs; i++) {
    if (i % 2 === 0) continue;
    const x = -w / 2 + (w / segs) * i;
    g.rect(x, -h / 2, w / segs, h).fill({ color: color.walnutDeep, alpha: 0.35 });
  }
}

function drawCandle(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2 + 4, w, h - 4, 2).fill(color.oakLight);
  g.moveTo(0, -h / 2 + 4).lineTo(-w * 0.35, -h / 2 - 8).lineTo(w * 0.35, -h / 2 - 8).closePath().fill(sphereGradient(color.danger, 0xffd27a));
}

function drawBarrel(g: Graphics, w: number, h: number): void {
  g.roundRect(-w / 2, -h / 2, w, h, w * 0.2).fill(metalGradient(color.walnutDeep, color.oakMid, 90));
  for (const frac of [-0.3, 0, 0.3]) {
    g.rect(-w / 2, h * frac - 2, w, 4).fill({ color: color.danger, alpha: 0.85 });
  }
}

const CATEGORY_FALLBACK: Record<PartDef['category'], number> = {
  STATIC: color.oakMid,
  DYNAMIC: color.danger,
  MECHANISM: color.brass,
  POWER: color.brass,
  PNEUMATIC: color.turquoise,
  THERMAL: color.danger,
  LIGHT: color.brassBright,
  ACTUATOR: color.brass,
  GOAL: color.success,
};

/** Draws a themed sprite sized from the part's own BodySpec — shape stays authoritative, look is themed on top. */
export function spriteForPart(def: PartDef): Graphics {
  const g = new Graphics();
  const shape = def.body.shape;
  const isBall = shape.kind === 'ball';
  const w = isBall ? shape.radius * 2 * SIM.PIXELS_PER_METER : shape.w * SIM.PIXELS_PER_METER;
  const h = isBall ? w : shape.h * SIM.PIXELS_PER_METER;

  if (def.body.isSensor) {
    const accent = def.category === 'GOAL' ? color.success : def.category === 'POWER' ? color.brassBright : color.turquoise;
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
    const accent = def.category === 'PNEUMATIC' ? color.turquoise : color.brassBright;
    drawPowerPanel(g, w, h, accent);
    return g;
  }

  if (isBall) {
    const base = def.tags.includes('METALLIC')
      ? color.steelDark
      : def.tags.includes('FRAGILE')
        ? 0xbfe3e6
        : def.tags.includes('BOUNCY')
          ? color.danger
          : def.tags.includes('WIND')
            ? color.paper
            : CATEGORY_FALLBACK[def.category];
    const highlight = def.tags.includes('METALLIC') ? color.steel : def.tags.includes('FRAGILE') ? 0xffffff : color.dangerBright;
    g.circle(0, 0, w / 2).fill(sphereGradient(base, highlight));
    g.circle(0, 0, w / 2).stroke({ width: 1, color: color.walnutDeep, alpha: 0.35 });
    return g;
  }

  // Generic box fallback for anything not covered above (crate_wood, etc.).
  const base = CATEGORY_FALLBACK[def.category];
  g.roundRect(-w / 2, -h / 2, w, h, 3).fill(base);
  g.roundRect(-w / 2, -h / 2, w, h, 3).stroke({ width: 1.5, color: color.walnutDeep, alpha: 0.5 });
  return g;
}
