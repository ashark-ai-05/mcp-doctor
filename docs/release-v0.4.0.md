# MCP Doctor v0.4.0

This release adds the first polished UI layer: an interactive terminal trace browser for MCP JSON-RPC sessions.

## What changed

- Added a Ratatui-powered interactive `mcp-doctor tui <trace.jsonl>` mode when run in a real terminal.
- Added keyboard navigation for trace events:
  - `↑` / `k` previous event
  - `↓` / `j` next event
  - `h` / `?` help overlay
  - `q` / `Esc` quit
- Added a multi-pane layout:
  - header with event/error/warning counts
  - timeline pane with colored statuses
  - payload pane with pretty JSON or event detail
  - findings pane for protocol/contract issues
- Kept a non-interactive snapshot fallback for tests, pipes, and CI.
- Added unit coverage for the polished snapshot output.
- Updated README docs for the new UI behavior.

## Why it matters

MCP Doctor already had the core debugging workflow: trace, validate, replay, diff, report. But a CLI-only workflow is hard to sell visually. The TUI makes the product easier to understand at a glance: users can see the request/response timeline, failed rows, stderr, validation findings, and selected JSON payload without opening raw JSONL.

This is still terminal-native. No Electron, no hosted dashboard, no accounts.

## Verification

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py
cargo run -- tui <generated-trace.jsonl>
```

## Known gaps

- The TUI is intentionally read-only. It does not edit traces or run replay actions from inside the UI yet.
- Mouse support and search/filter are not implemented yet.
- Full Streamable HTTP/SSE MCP support remains future work.
