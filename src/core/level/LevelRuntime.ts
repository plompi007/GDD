import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { PartDef } from '../parts/PartDef.ts';

export interface RuntimePart {
  id: string;
  def: PartDef;
  rigidBody: RAPIER.RigidBody;
  collider: RAPIER.Collider;
  tags: string[];
}

/** Maps placed-part ids and tags to their live Rapier handles for a single RUNNING session. */
export class LevelRuntime {
  private readonly byId = new Map<string, RuntimePart>();
  private readonly byTag = new Map<string, RuntimePart[]>();
  private readonly removed = new Set<string>();

  register(part: RuntimePart): void {
    this.byId.set(part.id, part);
    for (const tag of part.tags) {
      const list = this.byTag.get(tag) ?? [];
      list.push(part);
      this.byTag.set(tag, list);
    }
  }

  get(id: string): RuntimePart | undefined {
    return this.byId.get(id);
  }

  getByTag(tag: string): RuntimePart[] {
    return this.byTag.get(tag) ?? [];
  }

  markRemoved(id: string): void {
    this.removed.add(id);
  }

  isRemoved(id: string): boolean {
    return this.removed.has(id);
  }

  isTagFullyRemoved(tag: string): boolean {
    const parts = this.getByTag(tag);
    return parts.length > 0 && parts.every((p) => this.removed.has(p.id));
  }

  all(): RuntimePart[] {
    return [...this.byId.values()];
  }
}
