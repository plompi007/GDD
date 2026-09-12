// Runs tests/browser/*.ts scenarios inside a real Chromium instance against
// a live Vite dev server. These exercise the actual Rapier WASM build, which
// Vitest's Node environment cannot host reliably (see tests/browser/registry.ts).
import { createServer } from 'vite';
import { launchChromium } from './lib/launch-chromium.mjs';

async function main() {
  const server = await createServer({ server: { port: 0 } });
  await server.listen();
  const address = server.httpServer?.address();
  const port = typeof address === 'object' && address ? address.port : null;
  if (!port) throw new Error('Vite dev server did not report a port');

  const browser = await launchChromium();
  try {
    const page = await browser.newPage();
    const consoleErrors = [];
    page.on('pageerror', (err) => consoleErrors.push(String(err)));

    await page.goto(`http://localhost:${port}/tests/browser/index.html`, { waitUntil: 'load' });
    const results = await page.waitForFunction(() => window.__BROWSER_TEST_RESULTS__, undefined, {
      timeout: 30_000,
    });
    const values = await results.jsonValue();

    if (consoleErrors.length > 0) {
      console.error('Page errors during browser tests:', consoleErrors);
    }

    let failed = false;
    for (const r of values) {
      if (r.pass) {
        console.log(`  ✓ ${r.name}`);
      } else {
        failed = true;
        console.error(`  ✗ ${r.name}\n    ${r.error}`);
      }
    }
    if (failed || consoleErrors.length > 0) {
      process.exitCode = 1;
    } else {
      console.log(`\nbrowser tests: ${values.length} passed`);
    }
  } finally {
    await browser.close();
    await server.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
