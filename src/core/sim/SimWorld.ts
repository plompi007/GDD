import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { SIM } from './constants.ts';

/**
 * Thin wrapper around a Rapier world configured for deterministic replay
 * (GDD §2.2, §2.6). Never mutate `.rapier` state outside of `step()` — the
 * only way to undo simulation is to `free()` and build a fresh world
 * (GDD §2.5: reset = rebuild from scratch, never rewind).
 */
export class SimWorld {
  readonly rapier: RAPIER.World;
  simTime = 0;
  tickIndex = 0;

  constructor(gravity: { x: number; y: number } = SIM.GRAVITY) {
    this.rapier = new RAPIER.World(gravity);
    this.rapier.integrationParameters.dt = SIM.FIXED_DT;
    this.rapier.integrationParameters.numSolverIterations = SIM.SOLVER_ITERATIONS;
  }

  /** Advances the simulation by exactly one fixed tick. */
  step(): void {
    this.rapier.step();
    this.simTime += SIM.FIXED_DT;
    this.tickIndex++;
  }

  free(): void {
    this.rapier.free();
  }
}
