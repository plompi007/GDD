// End-to-end interactive test: drives the real app UI (level select -> drag
// a part from the bin onto the canvas -> play) exactly the way a player
// would, using Playwright mouse events (identical code path to touch via
// pointer events). Verifies GDD's M5/M6 acceptance bar: "אפשר לבנות פתרון
// שלם באצבע אחת בטלפון" — a full solution buildable with one finger.
import { createServer } from 'vite';
import { launchChromium } from './lib/launch-chromium.mjs';

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
    await page.click('button[data-level-id="lvl_a02_bridge"]');
    await page.waitForSelector('[data-testid="bin-plank_wood"]');

    // Drag the bridge plank from the bin onto the gap between the two ledges.
    const binBtn = page.locator('[data-testid="bin-plank_wood"]');
    const box = await binBtn.boundingBox();
    if (!box) throw new Error('bin button not visible');
    const canvasBox = await page.locator('canvas').boundingBox();
    if (!canvasBox) throw new Error('canvas not visible');
    // The level's world is 1600x1200 and the viewport is 800x600, so CONTAIN-fit scale is 0.5.
    const worldToScreen = (wx, wy) => ({
      x: canvasBox.x + wx * 0.5,
      y: canvasBox.y + wy * 0.5,
    });
    const dropTarget = worldToScreen(432, 711);

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(dropTarget.x, dropTarget.y, { steps: 10 });
    await page.mouse.up();
    await page.waitForTimeout(50);

    const remainingText = await binBtn.textContent();
    if (!remainingText?.includes('(0)')) {
      throw new Error(`expected the bridge plank to be placed (bin shows 0 remaining), got "${remainingText}"`);
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
    const binAfterReset = await binBtn.textContent();
    if (!binAfterReset?.includes('(0)')) {
      throw new Error(`expected the placed plank to survive reset, bin shows "${binAfterReset}"`);
    }
    await page.click('[data-testid="play-pause"]');
    await page.waitForFunction(() => window.__gameScreen?.session.state === 'SOLVED', undefined, { timeout: 15_000 });
    console.log('  ✓ reset preserves the placed part and replaying solves it again');

    if (pageErrors.length > 0) {
      throw new Error(`page errors during interactive test: ${pageErrors.join('; ')}`);
    }

    console.log('\ninteractive test: 1 scenario passed');
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
