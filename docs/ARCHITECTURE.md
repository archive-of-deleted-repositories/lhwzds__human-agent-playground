---
title: Architecture
description: Runtime, backend, API, MCP, and UI boundaries for shared game sessions.
covers:
  - apps/cli/src/*.rs
  - apps/cli/build.rs
  - crates/hap-models/src/*.rs
  - crates/hap-runtime/src/*.rs
  - crates/hap-games/src/*.rs
  - apps/backend/src/*.rs
  - packages/core/src/*.ts
---

# Architecture

## Design Goal

Human Agent Playground supports multiple tabletop games without forcing the
browser UI, HTTP API, or MCP layer to be rewritten for every game.

The platform lets:

- a human play through the web UI;
- a shell user or agent control games through the `hap` CLI;
- one or more agents play through MCP;
- local tools use the HTTP API;
- every client operate on the same shared session and event stream.

## Platform Layers

`crates/hap-models`

- shared Rust DTOs for sessions, events, game catalog items, AI settings, auth
  profiles, and API inputs;
- the backend/runtime contract used by both HTTP handlers and MCP tools.

`crates/hap-runtime`

- session persistence;
- game adapter lookup;
- public/private state projection;
- legal move and move execution orchestration;
- AI provider capability and auth profile support.

`crates/hap-games`

- Rust game adapters used by the backend runtime;
- one adapter interface for catalog metadata, initial state, legal moves, move
  execution, and state projection.

`packages/core`

- shared TypeScript model package for browser and package consumers.

`apps/backend`

- Axum HTTP server;
- REST-style game/session/AI routes;
- SSE session streams;
- Streamable HTTP MCP service backed by the same runtime.

`apps/cli`

- `hap serve` creates the local runtime and serves API, MCP, SSE, and the built
  web UI from one Rust binary;
- non-serving subcommands call the local HTTP API, so CLI commands, the browser,
  and MCP clients share the same long-running service process.

`apps/web`

- React application;
- game selector and session workspace;
- per-game board renderers;
- shared activity feed and API client.

## Session Model

A session carries:

- `gameId`;
- current game state;
- timestamps;
- move and activity history;
- optional AI seat metadata;
- public projections for UI/API/MCP clients.

The agent does not need a per-game MCP server. It can use `hap` commands or the
shared MCP endpoint, select or create a session, read the state, ask for legal
moves, and submit validated moves through the same runtime path as the UI.

## Boundary Rule

Game-specific rules belong in adapters. Platform code owns sessions, routing,
state storage, stream delivery, provider/auth configuration, and the generic
tool catalog.
