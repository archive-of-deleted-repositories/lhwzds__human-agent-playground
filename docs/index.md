---
title: Human Agent Playground Documentation
description: Shared board-game sessions for humans and MCP-capable AI agent apps.
covers:
  - apps/cli/src/*.rs
  - apps/cli/build.rs
  - packages/core/src/*.ts
  - crates/hap-models/src/*.rs
  - scripts/*.sh
---

# Human Agent Playground Documentation

Human Agent Playground is a local-first board-game environment where humans and
agent apps operate on the same session through a browser UI, an HTTP API, and a
Streamable HTTP MCP server.

## Current Surface

- Browser UI for creating sessions, choosing games, playing moves, and watching
  the activity timeline.
- `hap` CLI for serving the local UI/API and controlling sessions from shell or
  agent tools.
- HTTP API for games, sessions, legal moves, move submission, session reset,
  AI provider configuration, auth profiles, and session streams.
- MCP server for external hosts such as Codex, Claude Code, and Gemini CLI.
- Shared model package and Rust runtime crates that keep game state, session
  events, and API payloads aligned.

## Documentation Map

- [Architecture](./ARCHITECTURE.md) covers the platform boundaries and runtime
  ownership.
- [Games](./games.md) covers registered game adapters and UI workspaces.
- [API and MCP](./api-and-mcp.md) covers HTTP routes and MCP tools.
- [Web UI](./web-ui.md) covers the browser application and static website.
- [AI Runtime](./ai-runtime.md) covers built-in provider and AI seat support.
