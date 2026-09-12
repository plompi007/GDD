import type { LevelRuntime, RuntimePart } from '../level/LevelRuntime.ts';
import type { ConnectionDef } from '../level/LevelSchema.ts';

/**
 * Propagates ELECTRIC power from sources (outlet_power) through WIRE
 * connections to whatever consumes it (motor_electric, etc). Iterates a
 * few passes so short chains (outlet -> switch -> motor) settle within one
 * tick without needing a full topological sort (GDD §1.4.1: "ELECTRIC —
 * חיווט מקבילי דרך outlet").
 */
const MAX_PASSES = 4;

function isSource(part: RuntimePart): boolean {
  return part.def.partType === 'outlet_power';
}

export function updateElectricBus(runtime: LevelRuntime, connections: ConnectionDef[]): void {
  const wires = connections.filter((c) => c.kind === 'WIRE');

  for (const part of runtime.all()) {
    if (isSource(part)) {
      part.state.powered = part.params.startsOn !== false;
    } else if (part.def.ports.some((p) => p.dir === 'IN' && p.energy === 'ELECTRIC')) {
      part.state.powered = false;
    }
  }

  for (let pass = 0; pass < MAX_PASSES; pass++) {
    for (const wire of wires) {
      const from = runtime.get(wire.from.partId);
      const to = runtime.get(wire.to.partId);
      if (!from || !to) continue;
      if (from.state.powered) to.state.powered = true;
    }
  }
}
