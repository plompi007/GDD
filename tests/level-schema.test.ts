import { describe, expect, it } from 'vitest';
import { editorStateFromLevel } from '../src/core/level/LevelLoader.ts';
import { loadAllLevels, loadLevelById } from '../src/core/level/LevelCatalog.ts';

describe('level schema + catalog', () => {
  it('loads and validates every levels/**/*.json file', () => {
    const levels = loadAllLevels();
    expect(levels.length).toBeGreaterThanOrEqual(1);
  });

  it('has no duplicate level ids', () => {
    const ids = loadAllLevels().map((l) => l.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it('throws a helpful error for an unknown level id', () => {
    expect(() => loadLevelById('does_not_exist')).toThrow(/Unknown level id/);
  });

  it('merges fixed + preplaced parts into editorState, marking fixed parts locked', () => {
    const level = loadLevelById('lvl_a01_free_fall');
    const editorState = editorStateFromLevel(level);

    const fixedIds = level.fixedParts.map((p) => p.id);
    const preplacedIds = level.preplacedParts.map((p) => p.id);
    expect(editorState.parts).toHaveLength(fixedIds.length + preplacedIds.length);

    for (const part of editorState.parts) {
      expect(part.locked).toBe(fixedIds.includes(part.id));
    }
  });
});
