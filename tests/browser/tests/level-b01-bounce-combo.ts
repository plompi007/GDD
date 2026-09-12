import { runReplay } from '../../../src/core/level/ReplayRunner.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_b01_bounce_combo: unsolved without a bridge over the landing gap', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_b01_bounce_combo');
  const result = runReplay(level, registry);
  if (result.solved) {
    throw new Error('expected the level to be unsolvable without bridging the landing gap');
  }
});

registerTest('lvl_b01_bounce_combo: solved (spring launches the ball, the fan pushes it, the flat bridge finishes the path)', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_b01_bounce_combo');
  const solution = level.solutions[0];
  if (!solution) throw new Error('level has no solutions[] entry');
  const result = runReplay(level, registry, { parts: solution.parts, connections: solution.connections });
  if (!result.solved) {
    throw new Error(`expected SOLVED with the flat bridge, got failed=${result.failed} reason=${result.failReason} ticks=${result.ticksRun}`);
  }
});
