import './tests/determinism.ts';
import './tests/level-a01.ts';
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
