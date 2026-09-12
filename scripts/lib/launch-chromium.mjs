// Resolves which Chromium binary to launch: an explicit override, this
// sandbox's pre-installed browser if present, or Playwright's own bundled
// download otherwise (e.g. after `playwright install chromium` in CI).
import { existsSync } from 'node:fs';
import { chromium } from 'playwright';

const KNOWN_SANDBOX_PATH = '/opt/pw-browsers/chromium';

export async function launchChromium(options = {}) {
  const executablePath = process.env.PLAYWRIGHT_EXECUTABLE_PATH ?? (existsSync(KNOWN_SANDBOX_PATH) ? KNOWN_SANDBOX_PATH : undefined);
  return chromium.launch({ ...options, ...(executablePath ? { executablePath } : {}) });
}
