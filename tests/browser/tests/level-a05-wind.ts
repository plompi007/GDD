import { GameSession } from '../../../src/core/level/GameSession.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { SIM } from '../../../src/core/sim/constants.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a05_wind: a powered fan pushes the balloon into the goal zone', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a05_wind');
  const session = new GameSession(level, registry);
  session.play();

  const maxTicks = Math.ceil(SIM.MAX_SIM_SECONDS / SIM.FIXED_DT);
  for (let i = 0; i < maxTicks; i++) {
    session.frame(SIM.FIXED_DT);
    if (session.state !== 'RUNNING') break;
  }

  if (session.state !== 'SOLVED') {
    const subject = session.runtime?.get('subject');
    const fan = session.runtime?.get('fan');
    const pos = subject?.rigidBody.translation();
    throw new Error(
      `expected SOLVED, got state=${session.state} reason=${session.failReason} ` +
        `subjectPos=${pos ? `${(pos.x * SIM.PIXELS_PER_METER).toFixed(1)},${(pos.y * SIM.PIXELS_PER_METER).toFixed(1)}` : 'n/a'} ` +
        `fan.powered=${fan?.state.powered}`,
    );
  }
  session.reset();
});
