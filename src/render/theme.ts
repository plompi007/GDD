/**
 * "Timber & Brass" visual identity (GDD §4.3) — a modern artisan workshop:
 * oak and walnut wood, polished brass fittings, on a pegboard tool-wall.
 * One source of truth shared by the Pixi renderer and the HTML UI overlay,
 * so both are guaranteed to agree.
 */
export const THEME = {
  color: {
    walnutDark: 0x2a2420,
    walnutDeep: 0x1c1813,
    oakLight: 0xd9c5a0,
    oakMid: 0xb99a6b,
    brass: 0xc9973f,
    brassBright: 0xecc879,
    brassDeep: 0x8a6423,
    steel: 0x9aa3ab,
    steelDark: 0x5c646b,
    danger: 0xd9584b,
    dangerBright: 0xf2776a,
    success: 0x4fae7f,
    successBright: 0x7ecda3,
    turquoise: 0x4f9da7,
    ink: 0x2a2420,
    paper: 0xf3ead8,
  },
  css: {
    walnutDark: '#2a2420',
    walnutDeep: '#1c1813',
    oakLight: '#d9c5a0',
    oakMid: '#b99a6b',
    brass: '#c9973f',
    brassBright: '#ecc879',
    brassDeep: '#8a6423',
    steel: '#9aa3ab',
    danger: '#d9584b',
    dangerBright: '#f2776a',
    success: '#4fae7f',
    successBright: '#7ecda3',
    turquoise: '#4f9da7',
    paper: '#f3ead8',
  },
  font: {
    display: '"Rubik", "Segoe UI", sans-serif',
    body: '"Heebo", "Segoe UI", sans-serif',
    mono: '"JetBrains Mono", ui-monospace, monospace',
  },
} as const;
