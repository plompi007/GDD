import { SIM } from './constants.ts';

export interface FixedStepLoopOptions {
  /** Called once per fixed tick, in the exact order the caller sets up. */
  onTick: () => void;
  /** Called once per rendered frame with an interpolation alpha in [0, 1). */
  onRender?: (alpha: number) => void;
  /** Whether ticks should currently advance (mirrors the RUNNING/SOLVED states of GDD §2.3). */
  isRunning: () => boolean;
}

/**
 * Fixed-timestep accumulator loop (GDD §2.4). Real frame time never leaks
 * into simulation logic — only whole `SIM.FIXED_DT` steps do, which is what
 * makes replays deterministic regardless of the device's frame rate.
 */
export class FixedStepLoop {
  private accumulator = 0;

  constructor(private readonly opts: FixedStepLoopOptions) {}

  frame(realDtSeconds: number): void {
    if (!this.opts.isRunning()) {
      this.opts.onRender?.(1.0);
      return;
    }

    // Clamp to avoid a spiral of death after the app is backgrounded and resumed.
    this.accumulator += Math.min(realDtSeconds, 0.1);

    let steps = 0;
    while (this.accumulator >= SIM.FIXED_DT && steps < SIM.MAX_SUBSTEPS_PER_FRAME) {
      this.opts.onTick();
      this.accumulator -= SIM.FIXED_DT;
      steps++;
    }
    // Falling behind: drop the remainder rather than ever running extra
    // catch-up steps, which would make ticks depend on device speed.
    if (steps === SIM.MAX_SUBSTEPS_PER_FRAME) this.accumulator = 0;

    this.opts.onRender?.(this.accumulator / SIM.FIXED_DT);
  }

  reset(): void {
    this.accumulator = 0;
  }
}
