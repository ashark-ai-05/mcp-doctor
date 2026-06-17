# MCP Doctor

Record, replay, and debug MCP JSON-RPC from your terminal.

MCP Doctor is an open-source Rust single-binary devtool for developers building Model Context Protocol servers. It is local-first: traces stay on your machine, reports are plain Markdown, and the core workflow does not require SaaS, accounts, or telemetry.

```text
broken MCP server/tool call
→ terminal command
→ JSON-RPC trace + stderr/stdout evidence
→ replayable JSONL artifact
→ contract/protocol validation
→ markdown repro report
```

## Status

Implemented foundation:

- stdio MCP server smoke check;
- JSON-RPC trace recording to JSONL;
- tool call recording;
- replay of recorded `tools/call` requests;
- replay tolerance with `--allow-field`;
- trace validation;
- tool input-schema contract validation;
- tool response-shape validation;
- trace diff for tool/schema breaking changes;
- Markdown report export;
- read-only terminal trace viewer;
- reproducible replay script export;
- basic HTTP JSON POST MCP endpoint probe;
- secret redaction for likely token/password/secret/api_key/auth keys;
- local fake MCP server fixture and tests.

Future:

- full interactive Ratatui timeline browser;
- full MCP Streamable HTTP/SSE session semantics;
- schema-aware golden test generation;
- passive proxy mode;
- GitHub Actions workflow template.

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

Validate, view, replay, and export the trace:

```bash
cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- tui .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- replay .mcpdoctor/sessions/<id>/trace.jsonl -- python3 fixtures/fake_mcp_server.py
cargo run -- report .mcpdoctor/sessions/<id>/trace.jsonl --output repro.md
cargo run -- export-repro .mcpdoctor/sessions/<id>/trace.jsonl --output replay.sh
```

## Commands

```bash
mcp-doctor smoke stdio -- <server command> [args...]
mcp-doctor record stdio -- <server command> [args...]
mcp-doctor call stdio --tool <name> --args '{"text":"hi"}' -- <server command> [args...]
mcp-doctor replay <trace.jsonl> [--allow-field content.0.text] -- <server command> [args...]
mcp-doctor diff <old-trace.jsonl> <new-trace.jsonl>
mcp-doctor validate <trace.jsonl>
mcp-doctor report <trace.jsonl> --output report.md
mcp-doctor tui <trace.jsonl>
mcp-doctor export-repro <trace.jsonl> --output replay.sh
mcp-doctor connect http <url>
```

## Useful failure demos

Bad argument contract:

```bash
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py --mode schema-required
cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
```

Replay mismatch with allowed volatile field:

```bash
cargo run -- replay .mcpdoctor/sessions/<id>/trace.jsonl -- python3 fixtures/fake_mcp_server.py --mode schema-change
cargo run -- replay .mcpdoctor/sessions/<id>/trace.jsonl --allow-field content.0.text -- python3 fixtures/fake_mcp_server.py --mode schema-change
```

Schema diff:

```bash
cargo run -- diff old-trace.jsonl new-trace.jsonl
```

## Non-goals

- Not an MCP gateway.
- Not a SaaS trace platform.
- Not a replacement for the official browser Inspector.
- Not a security/governance product in v0.2.
- Not a broad LLM observability dashboard.

## Safety and privacy

- No telemetry.
- No network calls by default for stdio workflows.
- HTTP probing only connects to the URL you provide.
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
