/**
 * "Modern Workshop" visual identity — bright, high-contrast, flat shapes
 * with bold dark outlines on a light ground, so parts read clearly at a
 * glance instead of blending into a moody background. Vivid saturated
 * accent colors per material/category, not muted earth tones. One source
 * of truth shared by the Pixi renderer and the HTML UI overlay.
 */
export const THEME = {
  color: {
    // Ground
    paperLight: 0xf2ead8,
    paperMid: 0xe4d7ba,
    ink: 0x24211d,
    inkSoft: 0x4a443c,
    playfield: 0xdcebf2,

    // Materials
    wood: 0xe3a857,
    woodDark: 0xb9822f,
    brass: 0xffc94a,
    brassDark: 0xd99a1f,
    steel: 0x8fa3b8,
    steelDark: 0x5b7086,

    // Balls / dynamics
    lead: 0x3d4a5c,
    iron: 0x5b7c99,
    rubber: 0xff5d5d,
    glass: 0x6fe0e6,
    bounceHighlight: 0xffffff,

    // Semantic accents
    danger: 0xff5449,
    dangerDark: 0xc23a31,
    success: 0x2ec4b6,
    successDark: 0x1f9c90,
    electric: 0xffd23f,
    electricDark: 0xd9a80e,
    pneumatic: 0x5aa9e6,

    outline: 0x24211d,
  },
  css: {
    paperLight: '#f2ead8',
    paperMid: '#e4d7ba',
    ink: '#24211d',
    inkSoft: '#4a443c',
    wood: '#e3a857',
    brass: '#ffc94a',
    brassDark: '#d99a1f',
    danger: '#ff5449',
    dangerDark: '#c23a31',
    success: '#2ec4b6',
    successDark: '#1f9c90',
    electric: '#ffd23f',
    pneumatic: '#5aa9e6',
  },
  font: {
    display: '"Rubik", "Segoe UI", sans-serif',
    body: '"Heebo", "Segoe UI", sans-serif',
    mono: '"JetBrains Mono", ui-monospace, monospace',
  },
} as const;
