import { THEME } from '../render/theme.ts';

const { css } = THEME;

let injected = false;

/** Injects the app's shared stylesheet once. Safe to call from every screen. */
export function ensureStylesInjected(): void {
  if (injected) return;
  injected = true;

  const style = document.createElement('style');
  style.textContent = `
    .cw-btn {
      font-family: ${THEME.font.display};
      font-weight: 700;
      font-size: 15px;
      padding: 10px 22px;
      border-radius: 10px;
      border: 1px solid ${css.brassDeep};
      background: linear-gradient(180deg, ${css.brassBright}, ${css.brass} 55%, ${css.brassDeep});
      color: ${css.walnutDeep};
      box-shadow: 0 2px 0 ${css.brassDeep}, 0 3px 8px rgba(0,0,0,0.35);
      cursor: pointer;
      transition: transform 0.08s ease, filter 0.15s ease;
      white-space: nowrap;
    }
    .cw-btn:hover:not(:disabled) { filter: brightness(1.08); }
    .cw-btn:active:not(:disabled) { transform: translateY(2px); box-shadow: 0 0 0 ${css.brassDeep}, 0 1px 4px rgba(0,0,0,0.35); }
    .cw-btn:disabled { opacity: 0.45; cursor: default; filter: grayscale(0.4); }

    .cw-btn-secondary {
      font-family: ${THEME.font.display};
      font-weight: 600;
      font-size: 14px;
      padding: 9px 18px;
      border-radius: 10px;
      border: 1px solid rgba(217,197,160,0.35);
      background: rgba(42,36,32,0.55);
      color: ${css.oakLight};
      cursor: pointer;
      transition: background 0.15s ease;
    }
    .cw-btn-secondary:hover { background: rgba(217,197,160,0.16); }

    .cw-bin-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      font-family: ${THEME.font.body};
      font-size: 12px;
      font-weight: 500;
      padding: 8px 14px 6px;
      border-radius: 12px;
      border: 1px solid ${css.brassDeep};
      background: linear-gradient(180deg, rgba(217,197,160,0.95), rgba(185,154,107,0.95));
      color: ${css.walnutDeep};
      cursor: grab;
      box-shadow: 0 2px 6px rgba(0,0,0,0.3);
      touch-action: none;
      user-select: none;
    }
    .cw-bin-item:active { cursor: grabbing; }
    .cw-bin-item:disabled, .cw-bin-item[data-empty="true"] { opacity: 0.35; cursor: default; }
    .cw-bin-count {
      font-family: ${THEME.font.mono};
      font-weight: 700;
      font-size: 11px;
      color: ${css.brassDeep};
    }

    .cw-goal-bar {
      font-family: ${THEME.font.body};
      font-size: 15px;
      font-weight: 500;
      color: ${css.oakLight};
      background: linear-gradient(180deg, rgba(28,24,19,0.92), rgba(28,24,19,0.55));
      border-bottom: 2px solid ${css.brass};
      padding: 10px 16px calc(10px + env(safe-area-inset-top, 0px));
      text-align: center;
    }

    .cw-state-badge {
      font-family: ${THEME.font.display};
      font-weight: 700;
      font-size: 13px;
      padding: 4px 12px;
      border-radius: 999px;
      display: inline-block;
    }
    .cw-state-EDIT { background: rgba(154,163,171,0.25); color: ${css.oakLight}; }
    .cw-state-RUNNING { background: rgba(79,157,167,0.3); color: #bfe7ec; }
    .cw-state-PAUSED { background: rgba(201,151,63,0.3); color: ${css.brassBright}; }
    .cw-state-SOLVED { background: rgba(79,174,127,0.3); color: ${css.successBright}; }
    .cw-state-FAILED { background: rgba(217,88,75,0.3); color: ${css.dangerBright}; }

    .cw-hud-top {
      position: absolute; top: 0; left: 0; right: 0;
      display: flex; align-items: center; justify-content: space-between; gap: 12px;
    }
    .cw-hud-title { font-family: ${THEME.font.display}; font-weight: 700; }

    .cw-bottom-bar {
      position: absolute; bottom: 0; left: 0; right: 0;
      padding-bottom: env(safe-area-inset-bottom, 0px);
      background: linear-gradient(0deg, rgba(28,24,19,0.92), rgba(28,24,19,0.0));
    }
    .cw-bin-row {
      display: flex; justify-content: center; gap: 10px; flex-wrap: wrap;
      padding: 14px 12px 10px;
    }
    .cw-controls-row {
      display: flex; justify-content: center; gap: 12px;
      padding: 0 12px 14px;
    }

    .cw-menu {
      position: absolute; inset: 0; overflow: auto;
      background:
        radial-gradient(rgba(201,151,63,0.10) 1.5px, transparent 1.5px) 0 0/28px 28px,
        linear-gradient(160deg, ${css.walnutDark}, ${css.walnutDeep});
      display: flex; flex-direction: column; align-items: center;
      padding: 48px 20px;
    }
    .cw-wordmark {
      font-family: ${THEME.font.display};
      font-weight: 800;
      font-size: clamp(32px, 8vw, 52px);
      color: ${css.brassBright};
      text-shadow: 0 2px 0 ${css.brassDeep}, 0 6px 18px rgba(0,0,0,0.5);
      letter-spacing: 0.5px;
      margin: 0;
    }
    .cw-tagline {
      font-family: ${THEME.font.body};
      color: ${css.oakLight};
      opacity: 0.75;
      margin: 6px 0 36px;
      font-size: 14px;
    }
    .cw-level-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
      gap: 14px;
      width: 100%;
      max-width: 900px;
    }
    .cw-level-card {
      font-family: ${THEME.font.body};
      text-align: right;
      border-radius: 14px;
      border: 1px solid rgba(201,151,63,0.4);
      background: linear-gradient(160deg, rgba(217,197,160,0.08), rgba(217,197,160,0.02));
      color: ${css.oakLight};
      padding: 16px 18px;
      cursor: pointer;
      transition: transform 0.12s ease, border-color 0.15s ease, background 0.15s ease;
    }
    .cw-level-card:hover {
      transform: translateY(-3px);
      border-color: ${css.brass};
      background: linear-gradient(160deg, rgba(217,197,160,0.16), rgba(217,197,160,0.04));
    }
    .cw-level-order {
      font-family: ${THEME.font.mono};
      font-size: 12px;
      color: ${css.brassBright};
      display: block;
      margin-bottom: 4px;
    }
    .cw-level-title {
      font-family: ${THEME.font.display};
      font-weight: 700;
      font-size: 18px;
      display: block;
    }
  `;
  document.head.appendChild(style);
}
