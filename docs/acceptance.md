# MCP Doctor v0.1 Acceptance Criteria

## Product contract

One-line promise:

```text
Record, replay, and debug MCP JSON-RPC from your terminal.
```

V0.1 scope is stdio-only and local-first. The app must produce inspectable JSONL traces and Markdown reports without SaaS, accounts, or external telemetry.

## Acceptance criteria

- [x] `mcp-doctor --help` works.
- [x] `smoke stdio -- <server>` initializes and lists tools.
- [x] `record stdio -- <server>` saves `.mcpdoctor/sessions/<id>/trace.jsonl` and `report.md`.
- [x] `call stdio --tool echo --args '{"text":"hi"}' -- <server>` records request/response and prints the response.
- [x] `replay <trace.jsonl> -- <server>` replays recorded `tools/call` requests and reports matches/mismatches.
- [x] `validate <trace.jsonl>` detects malformed trace/protocol events.
- [x] `report <trace.jsonl> --output report.md` writes Markdown report.
- [x] Tests include fake MCP server fixture.
- [x] Secret-like keys are redacted in trace events.
- [x] Timeout/invalid JSON paths return errors instead of panics.

## Quality gates

Required before claiming a stable milestone:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py
```
