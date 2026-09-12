import { Application } from 'pixi.js';
import { loadAllLevels } from './core/level/LevelCatalog.ts';
import { loadPartRegistry } from './core/parts/PartRegistry.ts';
import { GameScreen } from './ui/GameScreen.ts';
import { ensureStylesInjected } from './ui/styles.ts';
import { THEME } from './render/theme.ts';

declare global {
  interface Window {
    /** Test-only hook: the currently mounted game screen, if any. */
    __gameScreen?: GameScreen;
  }
}

const CHAPTER_LABEL: Record<string, string> = {
  A_FOUNDATIONS: 'יסודות',
  B_COMBOS: 'צירופים',
  C_TIMING: 'טיימינג',
  D_MASTER: 'מאסטר',
};

// PixiJS's async Application.init() never resolves when awaited at module top
// level in a Vite production build (github.com/pixijs/pixijs/issues/10456).
// Wrapping in an async IIFE avoids top-level await entirely.
async function main(): Promise<void> {
  ensureStylesInjected();
  const app = new Application();
  await app.init({ background: THEME.color.playfield, resizeTo: window, preference: 'webgl' });

  const root = document.getElementById('app')!;
  root.style.position = 'relative';
  root.appendChild(app.canvas);

  const registry = loadPartRegistry();
  const levels = loadAllLevels().sort((a, b) => (a.order ?? 0) - (b.order ?? 0));

  let currentScreen: GameScreen | null = null;
  let menuEl: HTMLDivElement | null = null;

  function showLevelSelect(): void {
    currentScreen?.destroy();
    currentScreen = null;

    menuEl = document.createElement('div');
    menuEl.className = 'cw-menu';

    const wordmark = document.createElement('h1');
    wordmark.className = 'cw-wordmark';
    wordmark.append('Chain', Object.assign(document.createElement('span'), { textContent: 'Works' }));
    menuEl.appendChild(wordmark);

    const tagline = document.createElement('p');
    tagline.className = 'cw-tagline';
    tagline.textContent = 'פאזלים של תגובת שרשרת פיזיקלית';
    menuEl.appendChild(tagline);

    const grid = document.createElement('div');
    grid.className = 'cw-level-grid';
    menuEl.appendChild(grid);

    for (const level of levels) {
      const card = document.createElement('button');
      card.className = 'cw-level-card';
      card.dataset.levelId = level.id;

      const orderEl = document.createElement('span');
      orderEl.className = 'cw-level-order';
      const chapterLabel = level.chapter ? CHAPTER_LABEL[level.chapter] : undefined;
      orderEl.textContent = [level.order ? `שלב ${level.order}` : null, chapterLabel].filter(Boolean).join(' · ');
      card.appendChild(orderEl);

      const titleEl = document.createElement('span');
      titleEl.className = 'cw-level-title';
      titleEl.textContent = level.title;
      card.appendChild(titleEl);

      card.addEventListener('click', () => showLevel(level.id));
      grid.appendChild(card);
    }
    root.appendChild(menuEl);
  }

  function showLevel(id: string): void {
    menuEl?.remove();
    menuEl = null;
    currentScreen?.destroy();
    const level = levels.find((l) => l.id === id);
    if (!level) return;
    const screen = new GameScreen(app, registry, level, root);
    currentScreen = screen;
    window.__gameScreen = screen;

    const backBtn = document.createElement('button');
    backBtn.textContent = 'תפריט ›';
    backBtn.className = 'cw-btn-secondary';
    backBtn.dataset.testid = 'back-to-menu';
    backBtn.style.cssText = 'position:absolute; bottom:14px; left:14px; padding:8px 14px; font-size:13px; z-index:10;';
    backBtn.addEventListener('click', () => showLevelSelect());
    root.appendChild(backBtn);

    const originalDestroy = screen.destroy.bind(screen);
    screen.destroy = () => {
      backBtn.remove();
      if (window.__gameScreen === screen) delete window.__gameScreen;
      originalDestroy();
    };
  }

  showLevelSelect();
}

void main();
