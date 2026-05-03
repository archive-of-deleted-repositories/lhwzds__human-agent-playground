---
title: API and MCP
description: HTTP routes, session streams, and MCP tools for external clients.
covers:
  - apps/cli/src/*.rs
  - apps/backend/src/*.rs
  - apps/web/src/api.ts
  - e2e-tests/**/*.ts
---

# API and MCP

The backend exposes one runtime through HTTP and MCP. The `hap` CLI talks to the
same HTTP API, so browser UI, shell automation, and external agent hosts all
operate on the same session records.

## HTTP API

The Axum backend serves:

- `/health`;
- `/api/games`;
- `/api/sessions`;
- `/api/sessions/{session_id}`;
- `/api/sessions/{session_id}/stream`;
- `/api/sessions/{session_id}/legal-moves`;
- `/api/sessions/{session_id}/moves`;
- `/api/sessions/{session_id}/reset`;
- AI provider, auth profile, runtime settings, and AI seat routes.

Session streams use Server-Sent Events so the web UI can render the initial
session state and then receive live updates from the same runtime broadcaster.

## MCP Server

The MCP service is mounted at `/mcp` using Streamable HTTP. It exposes generic
session tools plus game-specific legal-move and play-move tools for supported
games.

The tool catalog includes:

- list games and sessions;
- search available tools;
- create, inspect, wait for, and reset sessions;
- get legal moves;
- play a move;
- play a move and wait for the next turn.

## CLI Control

`hap serve` starts the local service. Agent-facing commands such as
`hap games`, `hap session create`, `hap session show`, `hap legal-moves`,
`hap play`, `hap reset`, and `hap watch` connect to that service through
`HAP_SERVER_URL` or `--server`.

CLI commands default to compact JSON output for agent use. `--pretty` only
changes rendering; it does not change the API request shape.

## Client Rule

Clients should treat the backend as the source of truth. They should read the
latest state before choosing a move and submit moves through HTTP or MCP rather
than mutating local board state independently.
