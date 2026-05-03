---
title: Games
description: Registered game adapters, board renderers, and shared game contracts.
covers:
  - crates/hap-games/src/*.rs
  - games/*/src/*.ts
  - games/*/src/__tests__/*.ts
  - apps/web/src/games/**/*.ts
  - apps/web/src/games/**/*.tsx
---

# Games

Human Agent Playground keeps game rules behind adapters so the platform can
host many games through one session, API, and MCP surface.

## Registered Games

The current app registers:

- Xiangqi;
- Chess;
- Liar's Bar;
- Gomoku;
- Connect Four;
- Othello.

The backend catalog is provided by `crates/hap-games`, and the browser registry
in `apps/web/src/game-registry.ts` maps each game id to a React workspace.

## Adapter Contract

Each game adapter owns:

- catalog metadata;
- initial state;
- state normalization;
- legal move generation;
- move validation and transition output;
- public or seat-specific state projection.

The platform expects every move to produce the next state plus enough event
metadata for the activity feed and agent reasoning loop.

## TypeScript Game Packages

The `games/<game-id>` packages provide reusable TypeScript rules engines and
tests for browser-facing game logic. The React app uses per-game API helpers,
workspace components, board components, and shared viewport/activity utilities
to keep the application shell consistent while the inner renderer changes by
game.
