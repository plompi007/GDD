import { z } from 'zod';
import { ENERGY_TYPES } from '../parts/EnergyType.ts';

/** Zod mirror of schemas/level.schema.json (GDD §5.1) — the single source of truth for level JSON shape. */

const zPlacedPart = z.object({
  id: z.string().regex(/^[a-z0-9_]+$/),
  partType: z.string(),
  x: z.number(),
  y: z.number(),
  rotation: z.number().default(0),
  flipX: z.boolean().default(false),
  flipY: z.boolean().default(false),
  scale: z.number().min(0.5).max(2).default(1),
  tags: z.array(z.string()).default([]),
  params: z.record(z.string(), z.union([z.number(), z.string(), z.boolean()])).default({}),
});

const zAnchorRef = z.object({
  partId: z.string(),
  anchorIdx: z.number().int().min(0).default(0),
});

const zConnection = z.object({
  id: z.string(),
  kind: z.enum(['ROPE', 'BELT', 'WIRE', 'PIVOT']),
  from: zAnchorRef,
  to: zAnchorRef,
  maxLength: z.number().optional(),
  routedThrough: z.array(zAnchorRef).default([]),
});

const zCondition: z.ZodType<Condition> = z.lazy(() =>
  z.object({
    type: z.enum([
      'CONTAINED',
      'REACHED_ZONE',
      'ENERGY_STATE',
      'DESTROYED',
      'TIMEOUT',
      'SUBJECT_DESTROYED',
      'LEFT_BOUNDS',
      'ALL_OF',
      'ANY_OF',
    ]),
    subjectTag: z.string().optional(),
    subjectId: z.string().optional(),
    containerId: z.string().optional(),
    zoneId: z.string().optional(),
    targetId: z.string().optional(),
    nodeId: z.string().optional(),
    energy: z.enum(ENERGY_TYPES).optional(),
    active: z.boolean().optional(),
    holdMs: z.number().default(500),
    conditions: z.array(zCondition).optional(),
  }),
);

export interface Condition {
  type:
    | 'CONTAINED'
    | 'REACHED_ZONE'
    | 'ENERGY_STATE'
    | 'DESTROYED'
    | 'TIMEOUT'
    | 'SUBJECT_DESTROYED'
    | 'LEFT_BOUNDS'
    | 'ALL_OF'
    | 'ANY_OF';
  subjectTag?: string;
  subjectId?: string;
  containerId?: string;
  zoneId?: string;
  targetId?: string;
  nodeId?: string;
  energy?: (typeof ENERGY_TYPES)[number];
  active?: boolean;
  holdMs: number;
  conditions?: Condition[];
}

const zPartsBinEntry = z.object({
  partType: z.string(),
  count: z.number().int().min(1).max(99),
  lockedParams: z.array(z.string()).default([]),
});

const zSolution = z.object({
  label: z.string(),
  parts: z.array(zPlacedPart),
  connections: z.array(zConnection).default([]),
  expectedSolveTick: z.number().int().optional(),
});

export const zLevel = z.object({
  schemaVersion: z.literal(1),
  id: z.string().regex(/^lvl_[a-z0-9_]+$/),
  title: z.string().max(48),
  chapter: z.enum(['A_FOUNDATIONS', 'B_COMBOS', 'C_TIMING', 'D_MASTER']).optional(),
  order: z.number().int().min(1).optional(),
  difficulty: z.number().int().min(1).max(5).optional(),
  goalText: z.string().max(120),
  hints: z.array(z.string()).max(3).default([]),
  world: z.object({
    width: z.number().default(1600),
    height: z.number().default(1200),
    gravityY: z.number().default(9.81),
    timeLimitSec: z.number().default(90),
    theme: z.string().default('brass_timber'),
  }),
  fixedParts: z.array(zPlacedPart).default([]),
  preplacedParts: z.array(zPlacedPart).default([]),
  partsBin: z.array(zPartsBinEntry).default([]),
  connections: z.array(zConnection).default([]),
  winConditions: z.array(zCondition).min(1),
  failConditions: z.array(zCondition).default([]),
  solutions: z.array(zSolution).min(1),
});

export type LevelDef = z.infer<typeof zLevel>;
export type PlacedPartData = z.infer<typeof zPlacedPart>;
export type ConnectionDef = z.infer<typeof zConnection>;
export type AnchorRef = z.infer<typeof zAnchorRef>;
export type SolutionDef = z.infer<typeof zSolution>;

export function parseLevel(raw: unknown): LevelDef {
  return zLevel.parse(raw);
}
