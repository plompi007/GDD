import { describe, expect, it } from 'vitest';
import { SIM } from '../src/core/sim/constants.ts';

describe('SIM constants', () => {
  it('matches the fixed-timestep contract from the GDD (§2.2)', () => {
    expect(SIM.FIXED_DT).toBeCloseTo(1 / 120);
    expect(SIM.MAX_SUBSTEPS_PER_FRAME).toBe(4);
    expect(SIM.SLEEP_ENABLED).toBe(false);
  });
});
