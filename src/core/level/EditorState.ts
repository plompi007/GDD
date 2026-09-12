import type { ConnectionDef, PlacedPartData } from './LevelSchema.ts';

/** A placed part plus whether the player is allowed to move/delete it. */
export interface EditorPart extends PlacedPartData {
  locked: boolean;
}

/**
 * The single source of truth for what the player has placed (GDD §2.5).
 * Mutable only in the EDIT state; RUNNING reads it but never writes to it —
 * `reset()` always rebuilds simulation state from this, never the other
 * way around.
 */
export interface EditorState {
  parts: EditorPart[];
  connections: ConnectionDef[];
}

export function cloneEditorState(state: EditorState): EditorState {
  return {
    parts: state.parts.map((p) => ({ ...p, tags: [...p.tags], params: { ...p.params } })),
    connections: state.connections.map((c) => ({
      ...c,
      routedThrough: [...c.routedThrough],
    })),
  };
}
