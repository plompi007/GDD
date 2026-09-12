// End-to-end interactive tests: drive the real app UI the way a player
// would, using Playwright mouse events (identical code path to touch via
// pointer events). Verifies GDD's M5/M6 acceptance bar: "אפשר לבנות פתרון
// שלם באצבע אחת בטלפון" — a full solution buildable with one finger.
import { createServer } from 'vite';
import { launchChromium } from './lib/launch-chromium.mjs';

async function dragDropAndSolveBridge(page) {
  await page.click('button[data-level-id="lvl_a02_bridge"]');
  await page.waitForSelector('[data-testid="bin-plank_wood"]');

  const binBtn = page.locator('[data-testid="bin-plank_wood"]');
  const box = await binBtn.boundingBox();
  if (!box) throw new Error('bin button not visible');
  const canvasBox = await page.locator('canvas').boundingBox();
  if (!canvasBox) throw new Error('canvas not visible');
  // GameScreen frames the camera on the level's placed content rather than its
  // declared world box (see computeViewBounds), so the scale/pan varies by
  // level — ask the live screen for the real transform instead of assuming one.
  const localTarget = await page.evaluate(() => window.__gameScreen?.worldToScreen(432, 711));
  if (!localTarget) throw new Error('worldToScreen unavailable on window.__gameScreen');
  const dropTarget = { x: canvasBox.x + localTarget.x, y: canvasBox.y + localTarget.y };

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(dropTarget.x, dropTarget.y, { steps: 10 });
  await page.mouse.up();
  await page.waitForTimeout(50);

  const isEmpty = await binBtn.getAttribute('data-empty');
  if (isEmpty !== 'true') {
    throw new Error(`expected the bridge plank to be placed (bin marked empty), got data-empty="${isEmpty}"`);
  }

  await page.click('[data-testid="play-pause"]');
  const solved = await page.waitForFunction(
    () => window.__gameScreen?.session.state === 'SOLVED' || window.__gameScreen?.session.state === 'FAILED',
    undefined,
    { timeout: 15_000 },
  );
  const state = await solved.evaluate(() => window.__gameScreen?.session.state);
  if (state !== 'SOLVED') {
    throw new Error(`expected the level to reach SOLVED after placing the bridge, got ${state}`);
  }
  console.log('  ✓ drag a part from the bin, drop it on the gap, press play -> SOLVED');

  // Reset should keep the placed part (GDD §2.5: reset rebuilds from
  // editorState, it never discards what the player built) and replay
  // should reach the same outcome.
  await page.click('[data-testid="reset"]');
  await page.waitForTimeout(50);
  const stillEmpty = await binBtn.getAttribute('data-empty');
  if (stillEmpty !== 'true') {
    throw new Error(`expected the placed plank to survive reset, bin data-empty="${stillEmpty}"`);
  }
  await page.click('[data-testid="play-pause"]');
  await page.waitForFunction(() => window.__gameScreen?.session.state === 'SOLVED', undefined, { timeout: 15_000 });
  console.log('  ✓ reset preserves the placed part and replaying solves it again');

  await page.click('[data-testid="back-to-menu"]');
  await page.waitForSelector('.cw-level-grid');
}

/**
 * lvl_b01_bounce_combo (the project's first Chapter B "combo" level) reuses
 * lvl_a02_bridge's exact bridging geometry, then extends the causal chain
 * onto a gear-driven conveyor. Headless replay of the authored solution
 * isn't enough on its own here — a real drag-and-drop placement snaps to
 * SIM.GRID (16px) rather than landing on the authored floating-point
 * position, and this level's chain is long enough that a small placement
 * difference can matter. This exercises the real grid-snapped placement,
 * not just the exact authored coordinates.
 */
async function dragDropAndSolveBridgeCombo(page) {
  await page.click('button[data-level-id="lvl_b01_bounce_combo"]');
  await page.waitForSelector('[data-testid="bin-plank_wood"]');

  const binBtn = page.locator('[data-testid="bin-plank_wood"]');
  const box = await binBtn.boundingBox();
  if (!box) throw new Error('bin button not visible');
  const canvasBox = await page.locator('canvas').boundingBox();
  if (!canvasBox) throw new Error('canvas not visible');
  const localTarget = await page.evaluate(() => window.__gameScreen?.worldToScreen(432, 711));
  if (!localTarget) throw new Error('worldToScreen unavailable on window.__gameScreen');
  const dropTarget = { x: canvasBox.x + localTarget.x, y: canvasBox.y + localTarget.y };

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(dropTarget.x, dropTarget.y, { steps: 10 });
  await page.mouse.up();
  await page.waitForTimeout(50);

  await page.click('[data-testid="play-pause"]');
  const solved = await page.waitForFunction(
    () => window.__gameScreen?.session.state === 'SOLVED' || window.__gameScreen?.session.state === 'FAILED',
    undefined,
    { timeout: 30_000 },
  );
  const state = await solved.evaluate(() => window.__gameScreen?.session.state);
  if (state !== 'SOLVED') {
    throw new Error(`expected lvl_b01_bounce_combo to reach SOLVED after a real (grid-snapped) bridge placement, got ${state}`);
  }
  console.log('  ✓ lvl_b01_bounce_combo: a real (grid-snapped) bridge placement still reaches SOLVED');

  await page.click('[data-testid="back-to-menu"]');
  await page.waitForSelector('.cw-level-grid');
}

/**
 * Regression test for a real crash found via visual QA: GameScreen cached
 * {sprite, body} for every part at play() time, but never dropped the entry
 * when ThermalSystem later frees that part's Rapier body mid-RUNNING (here,
 * charge_barrel destroying the wall). The next render tick called
 * .translation() on the freed body and crashed the WASM instance outright.
 */
async function playThroughRemovalWithoutCrashing(page) {
  await page.click('button[data-level-id="lvl_a08_fuse"]');
  await page.waitForSelector('[data-testid="play-pause"]');
  await page.click('[data-testid="play-pause"]');
  await page.waitForFunction(() => window.__gameScreen?.session.state === 'SOLVED', undefined, { timeout: 15_000 });
  console.log('  ✓ a level with mid-game part removal (fuse -> barrel -> destroyed wall) plays through to SOLVED');

  await page.click('[data-testid="back-to-menu"]');
  await page.waitForSelector('.cw-level-grid');
  await page.click('button[data-level-id="lvl_a07_gear_train"]');
  await page.waitForSelector('[data-testid="play-pause"]');
  console.log('  ✓ navigating back to menu and into another level afterward is clean');
}

async function main() {
  const server = await createServer({ server: { port: 0 } });
  await server.listen();
  const address = server.httpServer?.address();
  const port = typeof address === 'object' && address ? address.port : null;
  if (!port) throw new Error('Vite dev server did not report a port');

  const browser = await launchChromium();
  let failed = false;
  try {
    const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
    const pageErrors = [];
    page.on('pageerror', (err) => pageErrors.push(String(err)));

    await page.goto(`http://localhost:${port}/`, { waitUntil: 'load' });
    await page.waitForSelector('button[data-level-id="lvl_a02_bridge"]');

    await dragDropAndSolveBridge(page);
    await dragDropAndSolveBridgeCombo(page);
    await playThroughRemovalWithoutCrashing(page);

    if (pageErrors.length > 0) {
      throw new Error(`page errors during interactive tests: ${pageErrors.join('; ')}`);
    }
    console.log('\ninteractive tests: all scenarios passed');
  } catch (err) {
    console.error('  ✗', err instanceof Error ? err.message : err);
    failed = true;
  } finally {
    await browser.close();
    await server.close();
  }
  if (failed) process.exitCode = 1;
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
