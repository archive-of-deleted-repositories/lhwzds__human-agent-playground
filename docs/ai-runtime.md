---
title: AI Runtime
description: Built-in provider capabilities, auth profiles, and AI seat controls.
covers:
  - crates/hap-runtime/src/ai.rs
  - apps/web/src/ai-runtime-ui.tsx
---

# AI Runtime

Human Agent Playground can use external MCP clients, but it also has built-in
AI seat configuration for local experiments.

## Provider Capabilities

The runtime reports provider capabilities for API-backed providers and local CLI
providers. Each capability includes:

- provider id and label;
- provider kind;
- availability status;
- supported models;
- supported auth provider ids.

CLI-backed providers are marked unavailable when the required executable is not
installed or, for Claude Code, when the CLI is not logged in.

## Auth Profiles and Seats

The backend exposes auth profile routes and session AI seat routes. The web UI
uses those routes to configure which provider/model can drive each side of a
session.

Built-in AI seats should still move through the same session runtime as humans
and MCP clients. They should read state, choose legal moves, submit through the
runtime, and let the normal event stream update the UI.
