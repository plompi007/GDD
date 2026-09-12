# ChainWorks

Physics-based chain-reaction puzzle game. Full spec: `docs/GDD.md` — read it before writing a line of code.

We work by Milestones (GDD §5.6). Implement only the current milestone and stop; do not touch anything beyond it.

## Iron rules

1. TypeScript strict. Zero `any`.
2. Zero `Math.random()` and zero `Date.now()` inside `core/sim` and `core/graph`.
3. Every part is defined in data JSON (`data/parts/*.json`), never hard-coded.
4. `editorState` is read-only while in the `RUNNING` state.
5. After each milestone: run `pnpm test` and confirm everything passes before moving on.

## Legal / branding

Never use names from the "source" column in GDD §4.4 (e.g. "Professor Tim", "Mort the Mouse", "Rube Goldberg") anywhere — not in code, assets, filenames, or commit messages.
