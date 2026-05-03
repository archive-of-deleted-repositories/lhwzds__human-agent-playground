---
title: Web UI
description: React workspace, game modules, shared UI utilities, and public website copy.
covers:
  - apps/cli/src/*.rs
  - apps/cli/build.rs
  - apps/web/src/*.ts
  - apps/web/src/*.tsx
  - apps/web/src/__tests__/*.ts
  - apps/web/src/__tests__/*.tsx
  - apps/web/src/test/*.ts
  - apps/web/vite.config.ts
  - apps/web/src/games/**/*.ts
  - apps/web/src/games/**/*.tsx
  - website/src/**/*.ts
  - website/tests/*.ts
  - website/*.ts
  - website/*.mjs
---

# Web UI

The browser application is the human-facing control surface for the shared game
runtime.

## Application Shell

`apps/web` owns:

- app bootstrap and routing;
- API client functions;
- game registry lookup;
- internationalized UI strings;
- shared session and game types;
- AI runtime configuration UI;
- tests for registry, API, and application behavior.

## Game Workspaces

Each game workspace owns its board renderer, API wrapper, move controls, and
workspace tests. Shared components handle activity feed rendering and board
viewport behavior so game modules do not duplicate platform UI.

## Static Website

The `website` directory holds the product website content, demo session data,
and browser tests for the public-facing site. Public docs are published through
the unified `huaiwu_ai_web` host, but this repository remains the source for
product behavior and local app documentation.

## Embedded App

The `hap` CLI embeds `apps/web/dist` when it is available at Rust build time.
`hap serve` serves that static app and the API from one origin, so the browser
client uses same-origin requests unless `VITE_API_URL` or `VITE_API_PORT` is
provided for development.
