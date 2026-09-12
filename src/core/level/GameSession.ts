import type { PartRegistry } from '../parts/PartRegistry.ts';
import { updateElectricBus } from '../graph/ElectricBus.ts';
import { applyFields } from '../graph/FieldSystem.ts';
import { applyConveyorSurfaces, applyGearTrain } from '../graph/GearTrain.ts';
import { solveRopeTension } from '../graph/RopeNetwork.ts';
import { updateThermalSystem } from '../graph/ThermalSystem.ts';
import { SIM } from '../sim/constants.ts';
import { FixedStepLoop } from '../sim/FixedStepLoop.ts';
import { SimWorld } from '../sim/SimWorld.ts';
import { editorStateFromLevel, buildSimFromEditorState } from './LevelLoader.ts';
import type { EditorState } from './EditorState.ts';
import type { LevelRuntime } from './LevelRuntime.ts';
import type { LevelDef } from './LevelSchema.ts';
import { type ConditionContext, ConditionTimers, evaluateCondition, isTimedOut } from './WinConditions.ts';

export type GameState = 'EDIT' | 'RUNNING' | 'PAUSED' | 'SOLVED' | 'FAILED';

/**
 * Owns one level's full lifecycle: the editable source of truth
 * (editorState), the derived, disposable simulation (simWorld + runtime),
 * and the state machine gating what input each state allows (GDD §2.3).
 *
 * reset() never rewinds physics — it frees the Rapier world and rebuilds
 * a fresh one from editorState (GDD §2.5), the only supported way back to EDIT.
 */
export class GameSession {
  readonly level: LevelDef;
  editorState: EditorState;
  state: GameState = 'EDIT';
  simWorld: SimWorld | null = null;
  runtime: LevelRuntime | null = null;
  failReason: string | null = null;

  private readonly registry: PartRegistry;
  private timers = new ConditionTimers();
  private readonly loop: FixedStepLoop;

  constructor(level: LevelDef, registry: PartRegistry) {
    this.level = level;
    this.registry = registry;
    this.editorState = editorStateFromLevel(level);
    this.loop = new FixedStepLoop({
      isRunning: () => this.state === 'RUNNING' || this.state === 'SOLVED',
      onTick: () => this.tick(),
    });
  }

  play(): void {
    if (this.state !== 'EDIT') return;
    this.simWorld = new SimWorld({ x: 0, y: this.level.world.gravityY });
    this.runtime = buildSimFromEditorState(this.simWorld.rapier, this.editorState, this.registry);
    this.timers = new ConditionTimers();
    this.failReason = null;
    this.state = 'RUNNING';
  }

  pause(): void {
    if (this.state === 'RUNNING') this.state = 'PAUSED';
  }

  resume(): void {
    if (this.state === 'PAUSED') this.state = 'RUNNING';
  }

  /** The only way out of RUNNING/PAUSED/SOLVED/FAILED — always a full rebuild from editorState. */
  reset(): void {
    this.simWorld?.free();
    this.simWorld = null;
    this.runtime = null;
    this.timers = new ConditionTimers();
    this.failReason = null;
    this.state = 'EDIT';
    this.loop.reset();
  }

  frame(realDtSeconds: number): void {
    this.loop.frame(realDtSeconds);
  }

  private tick(): void {
    if (!this.simWorld || !this.runtime) return;

    // GDD §2.4's tick order is load-bearing for determinism — don't reorder.
    updateElectricBus(this.runtime, this.editorState.connections);
    updateThermalSystem(this.simWorld.rapier, this.runtime, SIM.FIXED_DT, this.simWorld.simTime * 1000);
    applyFields(this.runtime);
    solveRopeTension(this.runtime, this.editorState.connections);
    applyGearTrain(this.runtime, this.editorState.connections);
    this.simWorld.step();
    applyConveyorSurfaces(this.simWorld.rapier, this.runtime);

    // Once SOLVED the simulation keeps running briefly for the win celebration
    // (GDD §2.3) but win/fail conditions are no longer re-evaluated.
    if (this.state !== 'RUNNING') return;

    if (this.simWorld.simTime > SIM.MAX_SIM_SECONDS) {
      this.state = 'FAILED';
      this.failReason = 'TIMEOUT';
      return;
    }

    const nowMs = this.simWorld.simTime * 1000;
    const ctx: ConditionContext = {
      world: this.simWorld.rapier,
      runtime: this.runtime,
      nowMs,
      worldWidth: this.level.world.width / SIM.PIXELS_PER_METER,
      worldHeight: this.level.world.height / SIM.PIXELS_PER_METER,
    };

    for (const cond of this.level.failConditions) {
      const failed = cond.type === 'TIMEOUT' ? isTimedOut(this.level, nowMs) : evaluateCondition(cond, ctx, this.timers);
      if (failed) {
        this.state = 'FAILED';
        this.failReason = cond.type;
        return;
      }
    }

    // A level's win conditions are an implicit ALL_OF: every entry must hold
    // at once (use the explicit ANY_OF node type for "either works" logic).
    if (this.level.winConditions.every((c) => evaluateCondition(c, ctx, this.timers))) {
      this.state = 'SOLVED';
    }
  }
}
