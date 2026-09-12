/**
 * A minimal test registry for scenarios that must run against the real
 * Rapier WASM build in a real browser engine — determinism checks and
 * (from M8 on) golden solution replays. Vitest's Node environment can't
 * host this WASM build reliably, so these run via Playwright instead
 * (see scripts/browser-test-runner.mjs).
 */

export interface BrowserTestResult {
  name: string;
  pass: boolean;
  error?: string;
}

const tests: { name: string; fn: () => void | Promise<void> }[] = [];

export function registerTest(name: string, fn: () => void | Promise<void>): void {
  tests.push({ name, fn });
}

export async function runAll(): Promise<BrowserTestResult[]> {
  const results: BrowserTestResult[] = [];
  for (const t of tests) {
    try {
      await t.fn();
      results.push({ name: t.name, pass: true });
    } catch (err) {
      results.push({ name: t.name, pass: false, error: err instanceof Error ? err.message : String(err) });
    }
  }
  return results;
}
