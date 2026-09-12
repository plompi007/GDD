import { GameSession } from '../../../src/core/level/GameSession.ts';
import { loadLevelById } from '../../../src/core/level/LevelCatalog.ts';
import { SIM } from '../../../src/core/sim/constants.ts';
import { loadPartRegistry } from '../../../src/core/parts/PartRegistry.ts';
import { registerTest } from '../registry.ts';

registerTest('lvl_a08_fuse: candle ignites the fuse, which detonates the barrel and destroys the wall', () => {
  const registry = loadPartRegistry();
  const level = loadLevelById('lvl_a08_fuse');
  const session = new GameSession(level, registry);
  session.play();

  const maxTicks = Math.ceil(SIM.MAX_SIM_SECONDS / SIM.FIXED_DT);
  for (let i = 0; i < maxTicks; i++) {
    session.frame(SIM.FIXED_DT);
    if (session.state !== 'RUNNING') break;
  }

  if (session.state !== 'SOLVED') {
    const fuse = session.runtime?.get('fuse');
    const barrel = session.runtime?.get('barrel');
    throw new Error(
      `expected SOLVED, got state=${session.state} reason=${session.failReason} ` +
        `fuse.burning=${fuse?.state.burning} fuse.burnProgress=${fuse?.state.burnProgress} ` +
        `barrel.triggered=${barrel?.state.triggered} wallRemoved=${session.runtime?.isRemoved('wall')}`,
    );
  }
  session.reset();
});
