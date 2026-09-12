import { type PartDef, zPartDef } from './PartDef.ts';

/**
 * Every part is a data file (GDD §5.3) — the registry's whole job is to load
 * and validate them once at startup so nothing downstream has to guard
 * against malformed part JSON.
 */
export class PartRegistry {
  private readonly byType = new Map<string, PartDef>();

  private constructor(defs: PartDef[]) {
    for (const def of defs) {
      if (this.byType.has(def.partType)) {
        throw new Error(`Duplicate partType "${def.partType}" in data/parts`);
      }
      this.byType.set(def.partType, def);
    }
  }

  static fromRawModules(rawModules: Record<string, unknown>): PartRegistry {
    const defs = Object.entries(rawModules).map(([path, raw]) => {
      const result = zPartDef.safeParse(raw);
      if (!result.success) {
        throw new Error(`Invalid part definition in ${path}: ${result.error.message}`);
      }
      return result.data;
    });
    return new PartRegistry(defs);
  }

  get(partType: string): PartDef {
    const def = this.byType.get(partType);
    if (!def) throw new Error(`Unknown partType "${partType}"`);
    return def;
  }

  has(partType: string): boolean {
    return this.byType.has(partType);
  }

  all(): PartDef[] {
    return [...this.byType.values()];
  }
}

/** Loads every data/parts/*.json file via Vite's import.meta.glob. */
export function loadPartRegistry(): PartRegistry {
  const modules = import.meta.glob('/data/parts/*.json', { eager: true, import: 'default' });
  return PartRegistry.fromRawModules(modules);
}
