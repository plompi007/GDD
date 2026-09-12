/**
 * The energy taxonomy every part's ports are built from (GDD §1.2). A part
 * never references another part by name — it only emits/consumes one of
 * these, so a new interaction is just a new pairing in InteractionRules.
 */
export const ENERGY_TYPES = [
  'ROTARY',
  'TENSION',
  'ELECTRIC',
  'THERMAL',
  'PNEUMATIC',
  'LIGHT',
  'IMPACT',
] as const;

export type EnergyType = (typeof ENERGY_TYPES)[number];
