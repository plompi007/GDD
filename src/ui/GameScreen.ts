import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Container, Graphics } from 'pixi.js';
import { GameSession } from '../core/level/GameSession.ts';
import type { EditorPart } from '../core/level/EditorState.ts';
import type { LevelDef } from '../core/level/LevelSchema.ts';
import type { PartRegistry } from '../core/parts/PartRegistry.ts';
import { SIM } from '../core/sim/constants.ts';
import { spriteForPart } from '../render/PartSprite.ts';
import { THEME } from '../render/theme.ts';
import { ensureStylesInjected } from './styles.ts';

const STATE_LABEL: Record<string, string> = {
  EDIT: 'ערוך',
  RUNNING: 'רץ',
  PAUSED: 'מושהה',
  SOLVED: 'נפתר!',
  FAILED: 'נכשל',
};

/**
 * A self-contained game screen: canvas world + an HTML overlay for the
 * parts bin and play/pause/reset controls (GDD §3.1/§3.2, implemented with
 * plain pointer events so mouse and touch both work identically — the
 * "drag-drop controller" and "UI shell" milestones folded into one file
 * for now rather than GDD's full file-per-concern layout).
 */
export class GameScreen {
  readonly session: GameSession;
  private readonly app: Application;
  private readonly worldLayer: Container;
  private readonly connectionsLayer: Graphics;
  private readonly partsLayer: Container;
  private readonly viewBounds: { minX: number; minY: number; maxX: number; maxY: number };
  private readonly overlay: HTMLDivElement;
  private readonly binEl: HTMLDivElement;
  private readonly playBtn: HTMLButtonElement;
  private readonly resetBtn: HTMLButtonElement;
  private readonly stateBadge: HTMLSpanElement;
  private readonly remainingCounts = new Map<string, number>();
  private readonly editSprites = new Map<string, Graphics>();
  private runtimeSprites: { id: string; sprite: Graphics; body: RAPIER.RigidBody }[] = [];
  private nextPlacedId = 0;
  private dragGhostEl: HTMLDivElement | null = null;
  private dragPartType: string | null = null;
  private onLeave: (() => void) | null = null;

  constructor(
    app: Application,
    private readonly registry: PartRegistry,
    level: LevelDef,
    root: HTMLElement,
  ) {
    ensureStylesInjected();
    this.app = app;
    this.session = new GameSession(level, registry);
    for (const entry of level.partsBin) this.remainingCounts.set(entry.partType, entry.count);

    this.viewBounds = this.computeViewBounds();
    this.worldLayer = new Container();
    app.stage.addChild(this.worldLayer);
    this.worldLayer.addChild(this.drawBackground());
    this.connectionsLayer = new Graphics();
    this.worldLayer.addChild(this.connectionsLayer);
    this.partsLayer = new Container();
    this.worldLayer.addChild(this.partsLayer);
    this.fitWorld();

    this.overlay = document.createElement('div');
    this.overlay.style.cssText = 'position:absolute; inset:0; pointer-events:none;';
    root.appendChild(this.overlay);

    const topBar = document.createElement('div');
    topBar.className = 'cw-goal-bar cw-hud-top';
    this.overlay.appendChild(topBar);

    const titleEl = document.createElement('span');
    titleEl.className = 'cw-hud-title';
    titleEl.textContent = level.title;
    topBar.appendChild(titleEl);

    const goalEl = document.createElement('span');
    goalEl.textContent = level.goalText;
    goalEl.style.cssText = 'flex: 1; text-align: center;';
    topBar.appendChild(goalEl);

    this.stateBadge = document.createElement('span');
    this.stateBadge.className = 'cw-state-badge';
    topBar.appendChild(this.stateBadge);

    const bottomBar = document.createElement('div');
    bottomBar.className = 'cw-bottom-bar';
    bottomBar.style.pointerEvents = 'none';
    this.overlay.appendChild(bottomBar);

    this.binEl = document.createElement('div');
    this.binEl.className = 'cw-bin-row';
    this.binEl.style.pointerEvents = 'auto';
    bottomBar.appendChild(this.binEl);

    const controlsEl = document.createElement('div');
    controlsEl.className = 'cw-controls-row';
    controlsEl.style.pointerEvents = 'auto';
    bottomBar.appendChild(controlsEl);

    this.playBtn = document.createElement('button');
    this.playBtn.className = 'cw-btn';
    this.playBtn.dataset.testid = 'play-pause';
    this.playBtn.addEventListener('click', () => this.onPlayPauseClick());
    controlsEl.appendChild(this.playBtn);

    this.resetBtn = document.createElement('button');
    this.resetBtn.textContent = 'איפוס';
    this.resetBtn.className = 'cw-btn-secondary';
    this.resetBtn.dataset.testid = 'reset';
    this.resetBtn.addEventListener('click', () => this.onResetClick());
    controlsEl.appendChild(this.resetBtn);

    this.renderBin();
    this.renderEditState();
    this.updateHud();

    const tickerFn = (ticker: { deltaMS: number }): void => this.frame(ticker.deltaMS / 1000);
    app.ticker.add(tickerFn);
    const resizeFn = (): void => this.fitWorld();
    window.addEventListener('resize', resizeFn);
    this.onLeave = () => {
      app.ticker.remove(tickerFn);
      window.removeEventListener('resize', resizeFn);
      this.overlay.remove();
      this.worldLayer.destroy({ children: true });
    };
  }

  /** Detaches this screen's DOM/Pixi resources (used when switching back to level select). */
  destroy(): void {
    this.onLeave?.();
  }

  /**
   * Levels declare a generous `world` box mainly as an out-of-bounds fail
   * plane (see LEFT_BOUNDS in WinConditions.ts) — it's typically much bigger
   * than where the actual mechanism lives, which used to force the camera
   * to fit that whole box and render every part tiny. Frame the camera on
   * the placed parts instead (fixed + preplaced + every known solution, so
   * the goal and the intended solve are always in view), padded generously
   * for headroom during play. A "backstop" shape's own half-extent is
   * clamped so an intentionally huge floor/wall doesn't blow the frame back
   * out — only its center still anchors the box.
   */
  private computeViewBounds(): { minX: number; minY: number; maxX: number; maxY: number } {
    const level = this.session.level;
    const MAX_HALF_EXTENT = 220;
    const PADDING = 180;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    const allPlaced = [...level.fixedParts, ...level.preplacedParts, ...level.solutions.flatMap((s) => s.parts)];
    for (const p of allPlaced) {
      const shape = this.registry.get(p.partType).body.shape;
      const rawHalfW = (shape.kind === 'ball' ? shape.radius : shape.w / 2) * SIM.PIXELS_PER_METER;
      const rawHalfH = (shape.kind === 'ball' ? shape.radius : shape.h / 2) * SIM.PIXELS_PER_METER;
      const halfW = Math.min(rawHalfW, MAX_HALF_EXTENT);
      const halfH = Math.min(rawHalfH, MAX_HALF_EXTENT);
      minX = Math.min(minX, p.x - halfW);
      maxX = Math.max(maxX, p.x + halfW);
      minY = Math.min(minY, p.y - halfH);
      maxY = Math.max(maxY, p.y + halfH);
    }
    if (!Number.isFinite(minX)) return { minX: 0, minY: 0, maxX: level.world.width, maxY: level.world.height };
    return {
      minX: Math.max(0, minX - PADDING),
      minY: Math.max(0, minY - PADDING),
      maxX: Math.min(level.world.width, maxX + PADDING),
      maxY: Math.min(level.world.height, maxY + PADDING),
    };
  }

  private fitWorld(): void {
    const b = this.viewBounds;
    const w = Math.max(1, b.maxX - b.minX);
    const h = Math.max(1, b.maxY - b.minY);
    const scale = Math.min(this.app.screen.width / w, this.app.screen.height / h);
    this.worldLayer.scale.set(scale);
    const cx = (b.minX + b.maxX) / 2;
    const cy = (b.minY + b.maxY) / 2;
    this.worldLayer.position.set(this.app.screen.width / 2 - cx * scale, this.app.screen.height / 2 - cy * scale);
  }

  /** Converts a world-space (pixel) point to the current on-canvas screen position — used for input and by tests. */
  worldToScreen(worldX: number, worldY: number): { x: number; y: number } {
    return { x: this.worldLayer.position.x + worldX * this.worldLayer.scale.x, y: this.worldLayer.position.y + worldY * this.worldLayer.scale.y };
  }

  /** A subtle blueprint-style grid so empty playfield space reads as a designed board, not a blank void. */
  private drawBackground(): Graphics {
    const { width, height } = this.session.level.world;
    const g = new Graphics();
    g.rect(0, 0, width, height).fill(THEME.color.playfield);
    const step = 64;
    for (let x = step; x < width; x += step) {
      const major = x % (step * 4) === 0;
      g.moveTo(x, 0).lineTo(x, height).stroke({ width: major ? 1.5 : 1, color: 0xffffff, alpha: major ? 0.55 : 0.3 });
    }
    for (let y = step; y < height; y += step) {
      const major = y % (step * 4) === 0;
      g.moveTo(0, y).lineTo(width, y).stroke({ width: major ? 1.5 : 1, color: 0xffffff, alpha: major ? 0.55 : 0.3 });
    }
    return g;
  }

  /**
   * ROPE/BELT/WIRE connections are physics constraints (GDD §1.4.3) with no
   * rigid body of their own, so nothing drew them — a pulley level rendered
   * only the wheel and the weight with no visible rope between them.
   * RopeNetwork's own tension solver already resolves a ROPE to a chain of
   * part-center positions (from -> routedThrough -> to); this mirrors that
   * exact chain purely for drawing.
   */
  private drawConnections(): void {
    const { color } = THEME;
    this.connectionsLayer.clear();
    const isEdit = this.session.state === 'EDIT';
    const runtime = this.session.runtime;

    const posOf = (id: string): { x: number; y: number } | undefined => {
      if (isEdit || !runtime) {
        const part = this.session.editorState.parts.find((p) => p.id === id);
        return part ? { x: part.x, y: part.y } : undefined;
      }
      const part = runtime.get(id);
      if (!part || runtime.isRemoved(id)) return undefined;
      const t = part.rigidBody.translation();
      return { x: t.x * SIM.PIXELS_PER_METER, y: t.y * SIM.PIXELS_PER_METER };
    };

    for (const conn of this.session.editorState.connections) {
      const ids = [conn.from.partId, ...conn.routedThrough.map((r) => r.partId), conn.to.partId];
      const pts = ids.map(posOf).filter((p): p is { x: number; y: number } => p !== undefined);
      if (pts.length < 2) continue;

      const g = this.connectionsLayer;
      const first = pts[0]!;
      if (conn.kind === 'BELT') {
        g.moveTo(first.x, first.y);
        for (const p of pts.slice(1)) g.lineTo(p.x, p.y);
        g.stroke({ width: 7, color: color.ink, alpha: 0.9 });
        g.moveTo(first.x, first.y);
        for (const p of pts.slice(1)) g.lineTo(p.x, p.y);
        g.stroke({ width: 2.5, color: color.inkSoft, alpha: 0.6 });
      } else if (conn.kind === 'WIRE') {
        g.moveTo(first.x, first.y);
        for (const p of pts.slice(1)) g.lineTo(p.x, p.y);
        g.stroke({ width: 3, color: color.electricDark, alpha: 0.85 });
      } else if (conn.kind === 'ROPE') {
        // A thick base strand plus a thinner offset highlight fakes a twisted/braided look.
        g.moveTo(first.x, first.y);
        for (const p of pts.slice(1)) g.lineTo(p.x, p.y);
        g.stroke({ width: 4, color: color.woodDark });
        g.moveTo(first.x, first.y);
        for (const p of pts.slice(1)) g.lineTo(p.x, p.y);
        g.stroke({ width: 1.5, color: color.brass, alpha: 0.8 });
      }
    }
  }

  private renderBin(): void {
    this.binEl.innerHTML = '';
    for (const [partType, count] of this.remainingCounts) {
      const btn = document.createElement('button');
      btn.className = 'cw-bin-item';
      const label = document.createElement('span');
      label.textContent = this.registry.get(partType).displayKey.replace('part.', '');
      const countEl = document.createElement('span');
      countEl.className = 'cw-bin-count';
      countEl.textContent = `× ${count}`;
      btn.append(label, countEl);
      const disabled = count <= 0 || this.session.state !== 'EDIT';
      if (disabled) btn.dataset.empty = 'true';
      btn.dataset.testid = `bin-${partType}`;
      btn.addEventListener('pointerdown', (e) => this.startDrag(partType, e));
      this.binEl.appendChild(btn);
    }
  }

  private startDrag(partType: string, downEvent: PointerEvent): void {
    if (this.session.state !== 'EDIT') return;
    if ((this.remainingCounts.get(partType) ?? 0) <= 0) return;
    this.dragPartType = partType;

    const ghost = document.createElement('div');
    const def = this.registry.get(partType);
    const sizePx =
      def.body.shape.kind === 'ball' ? def.body.shape.radius * 2 * SIM.PIXELS_PER_METER * this.worldLayer.scale.x : Math.max(def.body.shape.w, def.body.shape.h) * SIM.PIXELS_PER_METER * this.worldLayer.scale.x;
    ghost.style.cssText = `position:fixed; width:${sizePx}px; height:${sizePx}px; margin-left:${-sizePx / 2}px; margin-top:${-sizePx / 2}px; border-radius:${def.body.shape.kind === 'ball' ? '50%' : '4px'}; background:rgba(236,200,121,0.55); border:2px solid rgba(201,151,63,0.9); pointer-events:none; z-index:1000;`;
    document.body.appendChild(ghost);
    this.dragGhostEl = ghost;
    this.moveGhost(downEvent.clientX, downEvent.clientY);

    const onMove = (e: PointerEvent): void => this.moveGhost(e.clientX, e.clientY);
    const onUp = (e: PointerEvent): void => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      this.endDrag(e.clientX, e.clientY);
    };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  private moveGhost(clientX: number, clientY: number): void {
    if (!this.dragGhostEl) return;
    this.dragGhostEl.style.left = `${clientX}px`;
    this.dragGhostEl.style.top = `${clientY}px`;
  }

  private endDrag(clientX: number, clientY: number): void {
    this.dragGhostEl?.remove();
    this.dragGhostEl = null;
    const partType = this.dragPartType;
    this.dragPartType = null;
    if (!partType) return;

    const canvasRect = this.app.canvas.getBoundingClientRect();
    const withinCanvas =
      clientX >= canvasRect.left && clientX <= canvasRect.right && clientY >= canvasRect.top && clientY <= canvasRect.bottom;
    if (!withinCanvas) return;

    const scale = this.worldLayer.scale.x;
    const worldX = (clientX - canvasRect.left - this.worldLayer.position.x) / scale;
    const worldY = (clientY - canvasRect.top - this.worldLayer.position.y) / scale;
    const snapped = { x: Math.round(worldX / SIM.GRID) * SIM.GRID, y: Math.round(worldY / SIM.GRID) * SIM.GRID };

    const id = `player_${this.nextPlacedId++}`;
    const part: EditorPart = {
      id,
      partType,
      x: snapped.x,
      y: snapped.y,
      rotation: 0,
      flipX: false,
      flipY: false,
      scale: 1,
      tags: [],
      params: {},
      locked: false,
    };
    this.session.editorState.parts.push(part);
    this.remainingCounts.set(partType, (this.remainingCounts.get(partType) ?? 0) - 1);
    this.renderBin();
    this.renderEditState();
  }

  private removePlacedPart(id: string): void {
    if (this.session.state !== 'EDIT') return;
    const idx = this.session.editorState.parts.findIndex((p) => p.id === id);
    if (idx === -1) return;
    const [part] = this.session.editorState.parts.splice(idx, 1);
    if (!part) return;
    this.remainingCounts.set(part.partType, (this.remainingCounts.get(part.partType) ?? 0) + 1);
    this.renderBin();
    this.renderEditState();
  }

  private renderEditState(): void {
    for (const sprite of this.editSprites.values()) sprite.destroy();
    this.editSprites.clear();
    this.partsLayer.removeChildren();

    for (const part of this.session.editorState.parts) {
      const def = this.registry.get(part.partType);
      const sprite = spriteForPart(def);
      sprite.position.set(part.x, part.y);
      sprite.rotation = (part.rotation * Math.PI) / 180;
      if (!part.locked) {
        sprite.eventMode = 'static';
        sprite.cursor = 'pointer';
        sprite.on('pointertap', () => this.removePlacedPart(part.id));
      }
      this.partsLayer.addChild(sprite);
      this.editSprites.set(part.id, sprite);
    }
    this.drawConnections();
  }

  private renderRuntimeState(): void {
    this.partsLayer.removeChildren();
    this.editSprites.clear();
    this.runtimeSprites = [];
    if (!this.session.runtime) return;
    for (const part of this.session.runtime.all()) {
      const sprite = spriteForPart(part.def);
      this.partsLayer.addChild(sprite);
      this.runtimeSprites.push({ id: part.id, sprite, body: part.rigidBody });
    }
    this.syncRuntimeSprites();
  }

  /**
   * Parts can be freed mid-RUNNING (a fuse's charge_barrel destroying a wall,
   * a popped balloon, ...) — LevelRuntime.remove() frees the underlying
   * Rapier body, so touching it again after that (even just reading its
   * translation) is a use-after-free that crashes the WASM. Drop those
   * sprites instead of syncing them.
   */
  private syncRuntimeSprites(): void {
    if (!this.session.runtime) return;
    const runtime = this.session.runtime;
    this.runtimeSprites = this.runtimeSprites.filter(({ id, sprite }) => {
      if (runtime.isRemoved(id)) {
        sprite.destroy();
        return false;
      }
      return true;
    });
    for (const { sprite, body } of this.runtimeSprites) {
      const t = body.translation();
      sprite.position.set(t.x * SIM.PIXELS_PER_METER, t.y * SIM.PIXELS_PER_METER);
      sprite.rotation = body.rotation();
    }
    this.drawConnections();
  }

  private onPlayPauseClick(): void {
    if (this.session.state === 'EDIT') {
      this.session.play();
      this.renderRuntimeState();
    } else if (this.session.state === 'RUNNING') {
      this.session.pause();
    } else if (this.session.state === 'PAUSED') {
      this.session.resume();
    }
    this.renderBin();
    this.updateHud();
  }

  private onResetClick(): void {
    this.session.reset();
    this.renderEditState();
    this.renderBin();
    this.updateHud();
  }

  private updateHud(): void {
    const state = this.session.state;
    const label = STATE_LABEL[state] ?? state;
    this.stateBadge.textContent = `${label}${this.session.failReason ? ` · ${this.session.failReason}` : ''}`;
    this.stateBadge.className = `cw-state-badge cw-state-${state}`;

    if (state === 'EDIT') this.playBtn.textContent = 'הפעל ▶';
    else if (state === 'RUNNING') this.playBtn.textContent = 'השהה ⏸';
    else if (state === 'PAUSED') this.playBtn.textContent = 'המשך ▶';
    else this.playBtn.textContent = 'הפעל ▶';
    this.playBtn.disabled = state === 'SOLVED' || state === 'FAILED';
  }

  private frame(realDtSeconds: number): void {
    this.session.frame(realDtSeconds);
    if (this.session.state === 'RUNNING' || this.session.state === 'SOLVED') {
      this.syncRuntimeSprites();
    }
    this.updateHud();
  }
}
