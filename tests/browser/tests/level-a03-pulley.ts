import { GameSession } from '../../../src/core/level/GameSession.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { SIM } from '../../../src/core/sim/constants.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a03_pulley_lift: a falling weight lifts the subject through the pulley', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a03_pulley_lift');
  const session = new GameSession(level, registry);
  session.play();

  const maxTicks = Math.ceil(SIM.MAX_SIM_SECONDS / SIM.FIXED_DT);
  for (let i = 0; i < maxTicks; i++) {
    session.frame(SIM.FIXED_DT);
    if (session.state !== 'RUNNING') break;
  }

  if (session.state !== 'SOLVED') {
    const subject = session.runtime?.get('subject');
    const weight = session.runtime?.get('weight');
    const subjectPos = subject?.rigidBody.translation();
    const weightPos = weight?.rigidBody.translation();
    throw new Error(
      `expected SOLVED, got state=${session.state} reason=${session.failReason} ` +
        `subjectPos=${subjectPos ? `${(subjectPos.x * SIM.PIXELS_PER_METER).toFixed(1)},${(subjectPos.y * SIM.PIXELS_PER_METER).toFixed(1)}` : 'n/a'} ` +
        `weightPos=${weightPos ? `${(weightPos.x * SIM.PIXELS_PER_METER).toFixed(1)},${(weightPos.y * SIM.PIXELS_PER_METER).toFixed(1)}` : 'n/a'}`,
    );
  }
  session.reset();
});
