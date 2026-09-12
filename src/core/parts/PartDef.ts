import { z } from 'zod';
import { ENERGY_TYPES } from './EnergyType.ts';

const zVec2 = z.object({ x: z.number(), y: z.number() });

const zShapeSpec = z.union([
  z.object({ kind: z.literal('ball'), radius: z.number().positive() }),
  z.object({ kind: z.literal('box'), w: z.number().positive(), h: z.number().positive() }),
]);

const zBodySpec = z.object({
  type: z.enum(['fixed', 'dynamic']),
  shape: zShapeSpec,
  mass: z.number().positive().optional(),
  restitution: z.number().min(0).max(2).optional(),
  friction: z.number().min(0).optional(),
  gravityScale: z.number().optional(),
  linearDamping: z.number().min(0).optional(),
  angularDamping: z.number().min(0).optional(),
  ccd: z.boolean().optional(),
  isSensor: z.boolean().optional(),
});

const zPort = z.object({
  id: z.string(),
  dir: z.enum(['IN', 'OUT']),
  energy: z.enum(ENERGY_TYPES),
  offset: zVec2,
});

const zAnchor = z.object({
  idx: z.number().int().min(0),
  kind: z.enum([...ENERGY_TYPES, 'GENERIC']),
  offset: zVec2,
});

const zEditorSpec = z.object({
  rotatable: z.boolean(),
  rotationSnap: z.number().min(0).max(360),
  flippable: z.boolean(),
  sprite: z.string(),
  binIcon: z.string(),
  hitboxPadding: z.number().min(0).optional(),
});

const zParamSpec = z.union([
  z.object({ type: z.literal('number'), min: z.number(), max: z.number(), step: z.number().optional(), default: z.number() }),
  z.object({ type: z.literal('enum'), values: z.array(z.string()).min(1), default: z.string() }),
  z.object({ type: z.literal('boolean'), default: z.boolean() }),
]);

export const zPartDef = z.object({
  partType: z.string().regex(/^[a-z][a-z0-9_]*$/),
  displayKey: z.string(),
  category: z.enum([
    'STATIC',
    'DYNAMIC',
    'MECHANISM',
    'POWER',
    'PNEUMATIC',
    'THERMAL',
    'LIGHT',
    'ACTUATOR',
    'GOAL',
  ]),
  tier: z.enum(['P0', 'P1', 'P2']),
  body: zBodySpec,
  tags: z.array(z.string()),
  windFactor: z.number().optional(),
  ports: z.array(zPort),
  anchors: z.array(zAnchor),
  editor: zEditorSpec,
  params: z.record(z.string(), zParamSpec),
});

export type PartDef = z.infer<typeof zPartDef>;
export type Port = z.infer<typeof zPort>;
export type Anchor = z.infer<typeof zAnchor>;
export type ParamSpec = z.infer<typeof zParamSpec>;
