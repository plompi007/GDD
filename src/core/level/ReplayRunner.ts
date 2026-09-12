import type { PartRegistry } from '../parts/PartRegistry.ts';
import { SIM } from '../sim/constants.ts';
import type { EditorPart } from './EditorState.ts';
import { GameSession } from './GameSession.ts';
import type { ConnectionDef, LevelDef, PlacedPartData } from './LevelSchema.ts';

export interface ReplayResult {
  solved: boolean;
  failed: boolean;
  failReason: string | null;
  solvedAtTick: number | null;
  ticksRun: number;
}

/**
 * Runs a level headlessly for up to `maxTicks` fixed steps, optionally with
 * extra parts/connections merged in (a solution's own placements). Used to
 * verify a level is actually solvable before shipping it (GDD §4.5's
 * "run both solutions headless, they must both pass" rule) and, from M8 on,
 * as the golden-replay CI check for every level's solutions[].
 */
export function runReplay(
  level: LevelDef,
  registry: PartRegistry,
  extra: { parts?: PlacedPartData[]; connections?: ConnectionDef[] } = {},
  maxTicks = Math.ceil(SIM.MAX_SIM_SECONDS / SIM.FIXED_DT),
): ReplayResult {
  const session = new GameSession(level, registry);
  const extraParts: EditorPart[] = (extra.parts ?? []).map((p) => ({ ...p, locked: false }));
  session.editorState.parts.push(...extraParts);
  session.editorState.connections.push(...(extra.connections ?? []));
  session.play();

  let solvedAtTick: number | null = null;
  for (let i = 0; i < maxTicks; i++) {
    session.frame(SIM.FIXED_DT);
    if (session.state === 'SOLVED') {
      solvedAtTick = session.simWorld?.tickIndex ?? i;
      break;
    }
    if (session.state === 'FAILED') break;
  }

  const result: ReplayResult = {
    solved: session.state === 'SOLVED',
    failed: session.state === 'FAILED',
    failReason: session.failReason,
    solvedAtTick,
    ticksRun: session.simWorld?.tickIndex ?? 0,
  };
  session.reset();
  return result;
}
