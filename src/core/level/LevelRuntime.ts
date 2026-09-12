import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import type { PartDef } from '../parts/PartDef.ts';

export interface RuntimePart {
  id: string;
  def: PartDef;
  rigidBody: RAPIER.RigidBody;
  collider: RAPIER.Collider;
  tags: string[];
  /** Instance params merged from PartDef.params defaults + the placed part's overrides (GDD §5.2 `params`). */
  params: Record<string, number | string | boolean>;
  /** Free-form mutable state owned by the graph systems (angularVelocity, powered, burnProgress, ...). */
  state: Record<string, number | string | boolean>;
}

/**
 * Maps placed-part ids and tags to their live Rapier handles for a single
 * RUNNING session. Once a part is removed (burned away, exploded, popped)
 * its rigid body handle is invalid — every lookup here filters those out so
 * no system can accidentally touch a freed body.
 */
export class LevelRuntime {
  private readonly byId = new Map<string, RuntimePart>();
  private readonly byTag = new Map<string, RuntimePart[]>();
  private readonly byPartType = new Map<string, RuntimePart[]>();
  private readonly removed = new Set<string>();

  register(part: RuntimePart): void {
    this.byId.set(part.id, part);
    for (const tag of part.tags) {
      const list = this.byTag.get(tag) ?? [];
      list.push(part);
      this.byTag.set(tag, list);
    }
    const typeList = this.byPartType.get(part.def.partType) ?? [];
    typeList.push(part);
    this.byPartType.set(part.def.partType, typeList);
  }

  get(id: string): RuntimePart | undefined {
    if (this.removed.has(id)) return undefined;
    return this.byId.get(id);
  }

  getByTag(tag: string): RuntimePart[] {
    return (this.byTag.get(tag) ?? []).filter((p) => !this.removed.has(p.id));
  }

  getByPartType(partType: string): RuntimePart[] {
    return (this.byPartType.get(partType) ?? []).filter((p) => !this.removed.has(p.id));
  }

  /** Frees the part's rigid body from the physics world and marks it gone for good. */
  remove(world: RAPIER.World, id: string): void {
    if (this.removed.has(id)) return;
    const part = this.byId.get(id);
    if (!part) return;
    world.removeRigidBody(part.rigidBody);
    this.removed.add(id);
  }

  isRemoved(id: string): boolean {
    return this.removed.has(id);
  }

  isTagFullyRemoved(tag: string): boolean {
    const parts = this.byTag.get(tag) ?? [];
    return parts.length > 0 && parts.every((p) => this.removed.has(p.id));
  }

  all(): RuntimePart[] {
    return [...this.byId.values()].filter((p) => !this.removed.has(p.id));
  }
}
