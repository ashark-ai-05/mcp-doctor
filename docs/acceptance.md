# MCP Doctor Acceptance Criteria

## Product contract

One-line promise:

```text
Record, replay, and debug MCP JSON-RPC from your terminal.
```

The current scope is local-first. The app must produce inspectable JSONL traces and Markdown reports without SaaS, accounts, or external telemetry. Stdio is the primary supported transport; HTTP probing is an early foundation command.

## Acceptance criteria

- [x] `mcp-doctor --help` works.
- [x] `smoke stdio -- <server>` initializes and lists tools.
- [x] `record stdio -- <server>` saves `.mcpdoctor/sessions/<id>/trace.jsonl` and `report.md`.
- [x] `call stdio --tool echo --args '{"text":"hi"}' -- <server>` records request/response and prints the response.
- [x] `replay <trace.jsonl> -- <server>` replays recorded `tools/call` requests and reports matches/mismatches.
- [x] `replay --allow-field content.0.text <trace.jsonl> -- <server>` can ignore volatile result fields.
- [x] `diff <old-trace> <new-trace>` detects removed/added tools and schema changes such as new required args.
- [x] `validate <trace.jsonl>` detects malformed trace/protocol events.
- [x] `validate <trace.jsonl>` detects tool argument contract errors against recorded `inputSchema`.
- [x] `validate <trace.jsonl>` detects malformed `tools/call` response content shapes.
- [x] `report <trace.jsonl> --output report.md` writes Markdown report with executive summary and findings.
- [x] `tui <trace.jsonl>` renders a read-only terminal trace overview.
- [x] `export-repro <trace.jsonl> --output replay.sh` writes a reproducible replay helper script.
- [x] `connect http <url>` provides a basic JSON POST HTTP MCP endpoint probe.
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
