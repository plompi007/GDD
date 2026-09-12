import { runReplay } from '../../../src/core/level/ReplayRunner.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a02_bridge: unsolved without the bridge plank', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a02_bridge');
  const result = runReplay(level, registry);
  if (result.solved) {
    throw new Error('expected the level to be unsolvable without placing the bridge plank');
  }
});

registerTest('lvl_a02_bridge: solved once the solution plank bridges the gap', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a02_bridge');
  const solution = level.solutions[0];
  if (!solution) throw new Error('level has no solutions[] entry');
  const result = runReplay(level, registry, { parts: solution.parts, connections: solution.connections });
  if (!result.solved) {
    throw new Error(`expected SOLVED with the bridge plank placed, got failed=${result.failed} reason=${result.failReason}`);
  }
});

registerTest('lvl_a02_bridge: also solved with a flat (rotation 0) bridge plank', () => {
  // Proves the level tolerates the drag-drop UI's default (unrotated) placement,
  // not just the exact authored solution angle.
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a02_bridge');
  const result = runReplay(level, registry, {
    parts: [{ id: 's_flat', partType: 'plank_wood', x: 432, y: 711, rotation: 0, flipX: false, flipY: false, scale: 1, tags: [], params: {} }],
  });
  if (!result.solved) {
    throw new Error(`flat bridge FAILED: reason=${result.failReason} ticksRun=${result.ticksRun}`);
  }
});
