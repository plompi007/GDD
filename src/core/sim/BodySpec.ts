export type ShapeSpec = { kind: 'ball'; radius: number } | { kind: 'box'; w: number; h: number };

export interface BodySpec {
  type: 'fixed' | 'dynamic';
  shape: ShapeSpec;
  mass?: number;
  restitution?: number;
  friction?: number;
  gravityScale?: number;
  linearDamping?: number;
  angularDamping?: number;
  ccd?: boolean;
  isSensor?: boolean;
}

export interface Transform2D {
  x: number;
  y: number;
  rotation?: number;
}
