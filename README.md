# MCP Doctor

Record, replay, and debug MCP JSON-RPC from your terminal.

MCP Doctor is an open-source Rust single-binary devtool for developers building Model Context Protocol servers. It is intentionally local-first: traces stay on your machine, reports are plain Markdown, and v0.1 focuses on stdio servers.

```text
broken MCP server/tool call
→ terminal command
→ JSON-RPC trace + stderr/stdout evidence
→ replayable JSONL artifact
→ contract/protocol validation
→ markdown repro report
```

## Status

Early v0.1 foundation. Implemented:

- stdio MCP server smoke check;
- JSON-RPC trace recording to JSONL;
- tool call recording;
- replay of recorded `tools/call` requests;
- trace validation;
- Markdown report export;
- secret redaction for likely token/password/auth keys;
- local fake MCP server fixture and tests.

Future:

- interactive Ratatui timeline browser;
- HTTP/SSE/Streamable HTTP transports;
- schema-aware contract tests;
- passive proxy mode;
- CI templates.

## Install / build

```bash
cargo build --release
./target/release/mcp-doctor --help
```

## 30-second demo

Run against the included fake MCP server:

```bash
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hello"}' -- python3 fixtures/fake_mcp_server.py
```

The `call` command prints the tool response and writes artifacts:

```text
.mcpdoctor/sessions/<id>/trace.jsonl
.mcpdoctor/sessions/<id>/report.md
```

Validate and replay the trace:

```bash
cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- replay .mcpdoctor/sessions/<id>/trace.jsonl -- python3 fixtures/fake_mcp_server.py
```

Generate a report explicitly:

```bash
cargo run -- report .mcpdoctor/sessions/<id>/trace.jsonl --output repro.md
```

## Commands

```bash
mcp-doctor smoke stdio -- <server command> [args...]
mcp-doctor record stdio -- <server command> [args...]
mcp-doctor call stdio --tool <name> --args '{"text":"hi"}' -- <server command> [args...]
mcp-doctor replay <trace.jsonl> -- <server command> [args...]
mcp-doctor validate <trace.jsonl>
mcp-doctor report <trace.jsonl> --output report.md
```

## Non-goals

- Not an MCP gateway.
- Not a SaaS trace platform.
- Not a replacement for the official browser Inspector.
- Not a security/governance product in v0.1.
- Not a broad LLM observability dashboard.

## Safety and privacy

- No telemetry.
- No network calls by default in v0.1 beyond launching the server command you provide.
- Trace/report values are redacted for likely secret keys such as token, password, secret, api_key and Authorization.
- MCP tool responses can still contain sensitive application data; inspect traces before sharing.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT
