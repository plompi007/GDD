import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { runReplay } from '../../../src/core/level/ReplayRunner.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a01_free_fall: preplaced ball reaches the goal zone on its own', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a01_free_fall');
  const result = runReplay(level, registry);

  if (!result.solved) {
    throw new Error(`expected SOLVED, got failed=${result.failed} reason=${result.failReason} ticksRun=${result.ticksRun}`);
  }
});

registerTest('lvl_a01_free_fall: resetting and replaying reaches the same outcome', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a01_free_fall');

  const first = runReplay(level, registry);
  const second = runReplay(level, registry);
  const third = runReplay(level, registry);

  if (!(first.solved && second.solved && third.solved)) {
    throw new Error('expected all 3 independent replays to solve');
  }
  if (!(first.solvedAtTick === second.solvedAtTick && second.solvedAtTick === third.solvedAtTick)) {
    throw new Error(
      `expected identical solve tick across resets, got ${first.solvedAtTick}/${second.solvedAtTick}/${third.solvedAtTick}`,
    );
  }
});
