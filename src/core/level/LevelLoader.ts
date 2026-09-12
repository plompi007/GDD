import * as RAPIER from '@dimforge/rapier2d-deterministic';
import { buildBody } from '../sim/BodyFactory.ts';
import { SIM } from '../sim/constants.ts';
import { sortedById } from '../sim/Determinism.ts';
import type { PartRegistry } from '../parts/PartRegistry.ts';
import type { EditorPart, EditorState } from './EditorState.ts';
import { LevelRuntime } from './LevelRuntime.ts';
import type { LevelDef } from './LevelSchema.ts';

/** Merges a level's fixed + preplaced parts into the single editable source of truth. */
export function editorStateFromLevel(level: LevelDef): EditorState {
  const parts: EditorPart[] = [
    ...level.fixedParts.map((p): EditorPart => ({ ...p, locked: true })),
    ...level.preplacedParts.map((p): EditorPart => ({ ...p, locked: false })),
  ];
  return { parts, connections: [...level.connections] };
}

/**
 * Builds live Rapier bodies for every part in editorState (GDD §2.5: this is
 * the *only* path into simulation state — never mutate a running world
 * in place). Creation order is sorted by part id so determinism never
 * depends on array/object iteration order (GDD §2.6.2).
 */
export function buildSimFromEditorState(
  world: RAPIER.World,
  editorState: EditorState,
  registry: PartRegistry,
): LevelRuntime {
  const runtime = new LevelRuntime();
  for (const part of sortedById(editorState.parts)) {
    const def = registry.get(part.partType);
    // Level JSON places parts in world-space units (GDD's WORLD_WIDTH/HEIGHT
    // grid); Rapier itself always works in meters (GDD §2.2).
    const { rigidBody, collider } = buildBody(world, def.body, {
      x: part.x / SIM.PIXELS_PER_METER,
      y: part.y / SIM.PIXELS_PER_METER,
      rotation: (part.rotation * Math.PI) / 180,
    });
    const params: Record<string, number | string | boolean> = {};
    for (const [key, spec] of Object.entries(def.params)) params[key] = spec.default;
    Object.assign(params, part.params);
    // Tags are the union of the part type's inherent material tags (FLAMMABLE,
    // DESTRUCTIBLE, ...) and this instance's own gameplay tags (SUBJECT, ...).
    const tags = [...new Set([...def.tags, ...part.tags])];
    runtime.register({ id: part.id, def, rigidBody, collider, tags, params, state: {} });
  }

  // PIVOT connections need both bodies to already exist, so they're wired up
  // in a second pass. Anchors are each part's own authored local-space anchor
  // point (GDD PartDef.anchors) — a lever_seesaw pinned this way tips freely
  // around its fulcrum without needing scripted rotation.
  for (const conn of editorState.connections) {
    if (conn.kind !== 'PIVOT') continue;
    const fromPart = runtime.get(conn.from.partId);
    const toPart = runtime.get(conn.to.partId);
    const fromAnchor = fromPart?.def.anchors[conn.from.anchorIdx];
    const toAnchor = toPart?.def.anchors[conn.to.anchorIdx];
    if (!fromPart || !toPart || !fromAnchor || !toAnchor) continue;
    const jointData = RAPIER.JointData.revolute(fromAnchor.offset, toAnchor.offset);
    world.createImpulseJoint(jointData, fromPart.rigidBody, toPart.rigidBody, true);
  }

  return runtime;
}
