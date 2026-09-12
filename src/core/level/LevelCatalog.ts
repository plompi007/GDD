import { parseLevel, type LevelDef } from './LevelSchema.ts';

/** Loads and validates every levels/**\/*.json file via Vite's import.meta.glob. */
export function loadAllLevels(): LevelDef[] {
  const modules = import.meta.glob('/levels/**/*.json', { eager: true, import: 'default' });
  return Object.values(modules).map((raw) => parseLevel(raw));
}

export function loadLevelById(id: string): LevelDef {
  const level = loadAllLevels().find((l) => l.id === id);
  if (!level) throw new Error(`Unknown level id "${id}"`);
  return level;
}
