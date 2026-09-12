/**
 * Determinism helpers (GDD §2.6). Two rules live here:
 *  - stable iteration order for anything keyed by id (never trust Map/Object
 *    insertion order, since editorState may be rebuilt in a different order
 *    than it was authored)
 *  - a content hash over a world snapshot, used by tests to prove that N
 *    identical runs of the same level produce byte-identical physics state.
 */

/** Sorts entries by id ascending so body/part creation order is reproducible. */
export function sortedById<T extends { id: string }>(items: readonly T[]): T[] {
  return [...items].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}

/**
 * FNV-1a 64-bit hash, returned as a hex string. Not cryptographic — just a
 * fast, dependency-free way to detect any byte difference between two
 * world snapshots (GDD §2.6.7).
 */
export function hashBytes(bytes: Uint8Array): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (const byte of bytes) {
    hash ^= BigInt(byte);
    hash = (hash * prime) & mask;
  }
  return hash.toString(16).padStart(16, '0');
}
