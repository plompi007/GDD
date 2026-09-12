import { describe, expect, it } from 'vitest';
import { loadPartRegistry } from '../src/core/parts/PartRegistry.ts';

describe('PartRegistry', () => {
  it('loads and validates every data/parts/*.json file', () => {
    const registry = loadPartRegistry();
    const all = registry.all();
    expect(all.length).toBeGreaterThanOrEqual(25);
  });

  it('has no duplicate partType across files', () => {
    const registry = loadPartRegistry();
    const types = registry.all().map((p) => p.partType);
    expect(new Set(types).size).toBe(types.length);
  });

  it('exposes the P0 static and dynamic parts M2 requires', () => {
    const registry = loadPartRegistry();
    for (const partType of [
      'plank_wood',
      'beam_steel',
      'wall_brick',
      'spike_pin',
      'floor_ground',
      'ball_lead',
      'ball_iron',
      'ball_wood',
      'ball_rubber',
      'ball_glass',
      'crate_wood',
      'balloon_lift',
      'drifter_orb',
      'bin_target',
      'zone_goal',
    ]) {
      expect(registry.has(partType), `missing part: ${partType}`).toBe(true);
      expect(registry.get(partType).tier).toBe('P0');
    }
  });

  it('exposes the M4 mechanism/power parts', () => {
    const registry = loadPartRegistry();
    for (const [partType, tier] of [
      ['outlet_power', 'P0'],
      ['motor_electric', 'P0'],
      ['gear_small', 'P0'],
      ['gear_large', 'P0'],
      ['pulley_wheel', 'P0'],
      ['conveyor', 'P1'],
    ] as const) {
      expect(registry.has(partType), `missing part: ${partType}`).toBe(true);
      expect(registry.get(partType).tier).toBe(tier);
    }
  });

  it('exposes the M4 thermal and field parts', () => {
    const registry = loadPartRegistry();
    for (const partType of ['candle', 'fuse_cord', 'charge_barrel', 'fan_blower']) {
      expect(registry.has(partType), `missing part: ${partType}`).toBe(true);
      expect(registry.get(partType).tier).toBe('P0');
    }
  });

  it('throws a helpful error for an unknown partType', () => {
    const registry = loadPartRegistry();
    expect(() => registry.get('does_not_exist')).toThrow(/Unknown partType/);
  });
});
