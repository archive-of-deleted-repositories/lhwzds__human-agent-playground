# Human Agent Playground

[中文说明](./README.zh-CN.md)

Human Agent Playground lets humans and agent apps share the same board-game session through a web UI, an HTTP API, and an MCP server, so you can play directly against Claude Code, Codex, Gemini, and similar agent clients.

## Games

- Chess (default in the UI)
- Xiangqi
- Gomoku
- Connect Four
- Othello

## Session Example

![Claude Code vs Codex CLI in Chess](./docs/images/chess-claude-code-vs-codex-cli.png)

This screenshot shows a live Chess session with Claude Code playing against Codex CLI.

## Start

```bash
npm install
npm run build --workspace @human-agent-playground/web
cargo run -p hap-cli --bin hap -- serve
```

Default local endpoints:

- UI: `http://127.0.0.1:8790`
- API: `http://127.0.0.1:8790/api`
- MCP: `http://127.0.0.1:8790/mcp`

For development with a separate Vite process:

```bash
API_PORT=8787 WEB_PORT=4173 bash scripts/dev.sh
```

## CLI Control

Start the local service first:

```bash
hap serve
```

Then control sessions from a shell or agent:

```bash
hap games
hap session create --game chess
hap session show <session-id>
hap legal-moves <session-id>
hap play <session-id> --move-json '{"from":"e2","to":"e4","reasoning":{"summary":"Open with the king pawn."}}'
```

CLI commands default to JSON output. Add `--pretty` for human-readable
formatting, or `--server http://127.0.0.1:8790` to target a non-default service.

## Use

- Click `Create Session` to start a game.
- The create dialog defaults to `Chess`.
- Use `AI Settings` if you want a built-in AI seat.
- Use the MCP endpoint if you want an external agent host to join.

## Skills

- MCP workflow: [skills/human-agent-playground-mcp/SKILL.md](./skills/human-agent-playground-mcp/SKILL.md)
- Chess rules: [skills/human-agent-playground-chess/SKILL.md](./skills/human-agent-playground-chess/SKILL.md)

## More

- Architecture: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
