import type * as RAPIER from '@dimforge/rapier2d-deterministic';
import { Application, Container, Graphics, Text } from 'pixi.js';
import { GameSession } from '../core/level/GameSession.ts';
import type { EditorPart } from '../core/level/EditorState.ts';
import type { LevelDef } from '../core/level/LevelSchema.ts';
import type { PartRegistry } from '../core/parts/PartRegistry.ts';
import { SIM } from '../core/sim/constants.ts';
import { spriteForPart } from '../render/PartSprite.ts';

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
  private readonly hudLabel: Text;
  private readonly overlay: HTMLDivElement;
  private readonly binEl: HTMLDivElement;
  private readonly playBtn: HTMLButtonElement;
  private readonly resetBtn: HTMLButtonElement;
  private readonly remainingCounts = new Map<string, number>();
  private readonly editSprites = new Map<string, Graphics>();
  private runtimeSprites: { sprite: Graphics; body: RAPIER.RigidBody }[] = [];
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
    this.app = app;
    this.session = new GameSession(level, registry);
    for (const entry of level.partsBin) this.remainingCounts.set(entry.partType, entry.count);

    this.worldLayer = new Container();
    app.stage.addChild(this.worldLayer);
    this.fitWorld();

    this.hudLabel = new Text({ text: '', style: { fill: 0xffffff, fontSize: 16 } });
    this.hudLabel.position.set(12, 44);
    app.stage.addChild(this.hudLabel);

    this.overlay = document.createElement('div');
    this.overlay.style.cssText = 'position:absolute; inset:0; pointer-events:none; font-family:sans-serif; color:#fff;';
    root.appendChild(this.overlay);

    const goalEl = document.createElement('div');
    goalEl.textContent = level.goalText;
    goalEl.style.cssText = 'position:absolute; top:8px; left:8px; right:8px; text-align:center; font-size:16px;';
    this.overlay.appendChild(goalEl);

    this.binEl = document.createElement('div');
    this.binEl.style.cssText =
      'position:absolute; bottom:64px; left:0; right:0; display:flex; justify-content:center; gap:8px; flex-wrap:wrap; pointer-events:auto;';
    this.overlay.appendChild(this.binEl);

    const controlsEl = document.createElement('div');
    controlsEl.style.cssText = 'position:absolute; bottom:8px; left:0; right:0; display:flex; justify-content:center; gap:8px; pointer-events:auto;';
    this.overlay.appendChild(controlsEl);

    this.playBtn = document.createElement('button');
    this.playBtn.dataset.testid = 'play-pause';
    this.playBtn.style.cssText = 'font-size:16px; padding:8px 20px;';
    this.playBtn.addEventListener('click', () => this.onPlayPauseClick());
    controlsEl.appendChild(this.playBtn);

    this.resetBtn = document.createElement('button');
    this.resetBtn.textContent = 'איפוס';
    this.resetBtn.dataset.testid = 'reset';
    this.resetBtn.style.cssText = 'font-size:16px; padding:8px 20px;';
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
      this.hudLabel.destroy();
    };
  }

  /** Detaches this screen's DOM/Pixi resources (used when switching back to level select). */
  destroy(): void {
    this.onLeave?.();
  }

  private fitWorld(): void {
    const level = this.session.level;
    const scale = Math.min(this.app.screen.width / level.world.width, this.app.screen.height / level.world.height);
    this.worldLayer.scale.set(scale);
  }

  private renderBin(): void {
    this.binEl.innerHTML = '';
    for (const [partType, count] of this.remainingCounts) {
      const btn = document.createElement('button');
      btn.textContent = `${this.registry.get(partType).displayKey.replace('part.', '')} (${count})`;
      btn.disabled = count <= 0 || this.session.state !== 'EDIT';
      btn.dataset.testid = `bin-${partType}`;
      btn.style.cssText = 'font-size:14px; padding:6px 12px;';
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
    ghost.style.cssText = `position:fixed; width:${sizePx}px; height:${sizePx}px; margin-left:${-sizePx / 2}px; margin-top:${-sizePx / 2}px; border-radius:${def.body.shape.kind === 'ball' ? '50%' : '4px'}; background:rgba(255,255,255,0.5); pointer-events:none; z-index:1000;`;
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
    const worldX = (clientX - canvasRect.left) / scale;
    const worldY = (clientY - canvasRect.top) / scale;
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
    this.worldLayer.removeChildren();

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
      this.worldLayer.addChild(sprite);
      this.editSprites.set(part.id, sprite);
    }
  }

  private renderRuntimeState(): void {
    this.worldLayer.removeChildren();
    this.editSprites.clear();
    this.runtimeSprites = [];
    if (!this.session.runtime) return;
    for (const part of this.session.runtime.all()) {
      const sprite = spriteForPart(part.def);
      this.worldLayer.addChild(sprite);
      this.runtimeSprites.push({ sprite, body: part.rigidBody });
    }
    this.syncRuntimeSprites();
  }

  private syncRuntimeSprites(): void {
    for (const { sprite, body } of this.runtimeSprites) {
      const t = body.translation();
      sprite.position.set(t.x * SIM.PIXELS_PER_METER, t.y * SIM.PIXELS_PER_METER);
      sprite.rotation = body.rotation();
    }
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
    const label = STATE_LABEL[this.session.state] ?? this.session.state;
    this.hudLabel.text = `${this.session.level.title} — ${label}${this.session.failReason ? ` (${this.session.failReason})` : ''}`;

    if (this.session.state === 'EDIT') this.playBtn.textContent = 'הפעל ▶';
    else if (this.session.state === 'RUNNING') this.playBtn.textContent = 'השהה ⏸';
    else if (this.session.state === 'PAUSED') this.playBtn.textContent = 'המשך ▶';
    else this.playBtn.textContent = 'הפעל ▶';
    this.playBtn.disabled = this.session.state === 'SOLVED' || this.session.state === 'FAILED';
  }

  private frame(realDtSeconds: number): void {
    this.session.frame(realDtSeconds);
    if (this.session.state === 'RUNNING' || this.session.state === 'SOLVED') {
      this.syncRuntimeSprites();
    }
    this.updateHud();
  }
}
