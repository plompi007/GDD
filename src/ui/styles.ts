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
      font-weight: 800;
      font-size: 16px;
      padding: 12px 26px;
      border-radius: 14px;
      border: none;
      background: ${css.success};
      color: #ffffff;
      box-shadow: 0 3px 0 ${css.successDark}, 0 6px 14px rgba(31,156,144,0.35);
      cursor: pointer;
      transition: transform 0.08s ease, filter 0.15s ease;
      white-space: nowrap;
    }
    .cw-btn:hover:not(:disabled) { filter: brightness(1.06); }
    .cw-btn:active:not(:disabled) { transform: translateY(3px); box-shadow: 0 0 0 ${css.successDark}, 0 2px 6px rgba(31,156,144,0.35); }
    .cw-btn:disabled { opacity: 0.4; cursor: default; filter: grayscale(0.3); box-shadow: none; }

    .cw-btn-secondary {
      font-family: ${THEME.font.display};
      font-weight: 700;
      font-size: 14px;
      padding: 10px 20px;
      border-radius: 14px;
      border: 2px solid ${css.ink}22;
      background: #ffffff;
      color: ${css.ink};
      cursor: pointer;
      box-shadow: 0 2px 6px rgba(36,33,29,0.12);
      transition: transform 0.08s ease, background 0.15s ease;
    }
    .cw-btn-secondary:hover { background: ${css.paperMid}; }
    .cw-btn-secondary:active { transform: translateY(2px); }

    .cw-bin-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      font-family: ${THEME.font.body};
      font-size: 12px;
      font-weight: 700;
      padding: 9px 16px 7px;
      border-radius: 16px;
      border: 2px solid ${css.ink};
      background: #ffffff;
      color: ${css.ink};
      cursor: grab;
      box-shadow: 0 3px 0 ${css.ink}, 0 5px 10px rgba(36,33,29,0.25);
      touch-action: none;
      user-select: none;
    }
    .cw-bin-item:active { cursor: grabbing; transform: translateY(2px); box-shadow: 0 1px 0 ${css.ink}; }
    .cw-bin-item[data-empty="true"] { opacity: 0.35; cursor: default; box-shadow: none; border-color: ${css.ink}55; }
    .cw-bin-count {
      font-family: ${THEME.font.mono};
      font-weight: 700;
      font-size: 11px;
      color: ${css.successDark};
    }

    .cw-goal-bar {
      font-family: ${THEME.font.body};
      font-size: 15px;
      font-weight: 600;
      color: ${css.ink};
      background: #ffffffee;
      border-bottom: 3px solid ${css.ink};
      padding: 10px 16px calc(10px + env(safe-area-inset-top, 0px));
      text-align: center;
    }

    .cw-state-badge {
      font-family: ${THEME.font.display};
      font-weight: 800;
      font-size: 13px;
      padding: 5px 14px;
      border-radius: 999px;
      display: inline-block;
      color: #ffffff;
    }
    .cw-state-EDIT { background: ${css.inkSoft}; }
    .cw-state-RUNNING { background: ${css.pneumatic}; }
    .cw-state-PAUSED { background: ${css.brassDark}; }
    .cw-state-SOLVED { background: ${css.success}; }
    .cw-state-FAILED { background: ${css.dangerDark}; }

    .cw-hud-top {
      position: absolute; top: 0; left: 0; right: 0;
      display: flex; align-items: center; justify-content: space-between; gap: 12px;
    }
    .cw-hud-title { font-family: ${THEME.font.display}; font-weight: 800; }

    .cw-bottom-bar {
      position: absolute; bottom: 0; left: 0; right: 0;
      padding-bottom: env(safe-area-inset-bottom, 0px);
      background: linear-gradient(0deg, ${css.paperLight} 55%, transparent);
    }
    .cw-bin-row {
      display: flex; justify-content: center; gap: 10px; flex-wrap: wrap;
      padding: 16px 12px 10px;
    }
    .cw-controls-row {
      display: flex; justify-content: center; gap: 12px;
      padding: 0 12px 16px;
    }

    .cw-menu {
      position: absolute; inset: 0; overflow: auto;
      background:
        radial-gradient(${css.brass}22 2px, transparent 2px) 0 0/30px 30px,
        linear-gradient(160deg, ${css.paperLight}, ${css.paperMid});
      display: flex; flex-direction: column; align-items: center;
      padding: 48px 20px;
    }
    .cw-wordmark {
      font-family: ${THEME.font.display};
      font-weight: 800;
      font-size: clamp(34px, 8vw, 54px);
      color: ${css.ink};
      margin: 0;
      letter-spacing: 0.5px;
    }
    .cw-wordmark span { color: ${css.success}; }
    .cw-tagline {
      font-family: ${THEME.font.body};
      color: ${css.inkSoft};
      margin: 6px 0 36px;
      font-size: 14px;
      font-weight: 600;
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
      border-radius: 18px;
      border: 3px solid ${css.ink};
      background: #ffffff;
      color: ${css.ink};
      padding: 16px 18px;
      cursor: pointer;
      box-shadow: 0 4px 0 ${css.ink};
      transition: transform 0.1s ease, box-shadow 0.1s ease;
    }
    .cw-level-card:hover { transform: translateY(-2px); }
    .cw-level-card:active { transform: translateY(3px); box-shadow: 0 1px 0 ${css.ink}; }
    .cw-level-order {
      font-family: ${THEME.font.mono};
      font-size: 12px;
      font-weight: 700;
      color: ${css.successDark};
      display: block;
      margin-bottom: 4px;
    }
    .cw-level-title {
      font-family: ${THEME.font.display};
      font-weight: 800;
      font-size: 18px;
      display: block;
    }
  `;
  document.head.appendChild(style);
}
