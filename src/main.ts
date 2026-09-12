import { Application } from 'pixi.js';
import { loadAllLevels } from './core/level/LevelCatalog.ts';
import { loadPartRegistry } from './core/parts/PartRegistry.ts';
import { GameScreen } from './ui/GameScreen.ts';

declare global {
  interface Window {
    /** Test-only hook: the currently mounted game screen, if any. */
    __gameScreen?: GameScreen;
  }
}

// PixiJS's async Application.init() never resolves when awaited at module top
// level in a Vite production build (github.com/pixijs/pixijs/issues/10456).
// Wrapping in an async IIFE avoids top-level await entirely.
async function main(): Promise<void> {
  const app = new Application();
  await app.init({ background: '#2a2420', resizeTo: window, preference: 'webgl' });

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
    menuEl.style.cssText =
      'position:absolute; inset:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:10px; background:#2a2420; color:#fff; font-family:sans-serif; overflow:auto; padding:24px;';

    const title = document.createElement('h1');
    title.textContent = 'ChainWorks';
    title.style.marginBottom = '8px';
    menuEl.appendChild(title);

    for (const level of levels) {
      const btn = document.createElement('button');
      btn.textContent = `${level.title}${level.order ? ` (${level.order})` : ''}`;
      btn.dataset.levelId = level.id;
      btn.style.cssText = 'font-size:16px; padding:10px 28px; min-width:220px;';
      btn.addEventListener('click', () => showLevel(level.id));
      menuEl?.appendChild(btn);
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
    backBtn.textContent = '‹ תפריט';
    backBtn.dataset.testid = 'back-to-menu';
    backBtn.style.cssText = 'position:absolute; top:8px; right:8px; font-size:14px; padding:6px 14px; z-index:10;';
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
