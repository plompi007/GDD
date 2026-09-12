import './tests/determinism.ts';
import './tests/level-a01.ts';
import './tests/level-a02-bridge.ts';
import './tests/level-a03-pulley.ts';
import './tests/level-a05-wind.ts';
import './tests/level-a07-gear-train.ts';
import './tests/level-a08-fuse.ts';
import { runAll } from './registry.ts';

declare global {
  interface Window {
    __BROWSER_TEST_RESULTS__?: Awaited<ReturnType<typeof runAll>>;
  }
}

async function main(): Promise<void> {
  window.__BROWSER_TEST_RESULTS__ = await runAll();
}

void main();
