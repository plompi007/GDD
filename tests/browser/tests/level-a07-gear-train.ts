import { GameSession } from '../../../src/core/level/GameSession.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { SIM } from '../../../src/core/sim/constants.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a07_gear_train: motor -> gears -> belt -> conveyor pushes the ball into the goal', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a07_gear_train');
  const session = new GameSession(level, registry);
  session.play();

  const maxTicks = Math.ceil(SIM.MAX_SIM_SECONDS / SIM.FIXED_DT);
  for (let i = 0; i < maxTicks; i++) {
    session.frame(SIM.FIXED_DT);
    if (session.state !== 'RUNNING') break;
  }

  if (session.state !== 'SOLVED') {
    const subject = session.runtime?.get('subject');
    const conveyor = session.runtime?.get('belt_conveyor');
    const gear2 = session.runtime?.get('gear2');
    const pos = subject?.rigidBody.translation();
    throw new Error(
      `expected SOLVED, got state=${session.state} reason=${session.failReason} ` +
        `subjectPos=${pos ? `${(pos.x * SIM.PIXELS_PER_METER).toFixed(1)},${(pos.y * SIM.PIXELS_PER_METER).toFixed(1)}` : 'n/a'} ` +
        `gear2.angularVelocity=${gear2?.state.angularVelocity} conveyor.angularVelocity=${conveyor?.state.angularVelocity}`,
    );
  }
  session.reset();
});
