<p align="center">
  <h1 align="center">MCP Doctor</h1>
  <p align="center">
    <strong>Record, replay, and debug MCP servers from your terminal.</strong><br>
    <em>A local-first Model Context Protocol doctor that turns flaky JSON-RPC sessions into trace files, contract checks, replay scripts, and shareable Markdown repros.</em>
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/rust-2024-orange" alt="Rust 2024">
    <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue" alt="Platform">
    <img src="https://img.shields.io/badge/MCP-JSON--RPC-purple" alt="MCP JSON-RPC">
    <img src="https://img.shields.io/badge/local--first-no%20telemetry-brightgreen" alt="Local-first, no telemetry">
    <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
  </p>
</p>

<p align="center">
  <img src="assets/demo.gif" alt="MCP Doctor terminal demo recording an MCP tool call, validating the trace, exporting a repro script, and replaying the session" width="820">
</p>

<p align="center">
  <em>One command records the MCP conversation. The trace can be validated, viewed, replayed, diffed, and attached to a bug report.</em>
</p>

---

MCP servers are becoming the tool layer for agents, but debugging them is still awkward: raw stdin/stdout, JSON-RPC envelopes, schema drift, tool-call mismatches, hidden stderr, and “works in Inspector but not in my agent” reports.

**MCP Doctor gives MCP server authors a repeatable terminal workflow:**

```text
broken MCP server/tool call
→ local terminal command
→ JSONL trace + stderr/stdout evidence
→ contract/protocol validation
→ replayable reproduction
→ Markdown report for issues/PRs
```

Think of it as **one zero-config binary for MCP smoke tests, trace recording, replay, contract validation, and repro reports** — built for developers who need evidence fast without sending traces to a SaaS.

<samp>Rust single binary · JSONL traces · no telemetry · secret-key redaction · fixture-backed tests</samp>

## Why MCP Doctor

- 🩺 **Smoke test any stdio MCP server** — initialize the server, run `tools/list`, and confirm it speaks MCP before wiring it into an agent.
- 🎥 **Record the real JSON-RPC conversation** — capture initialize, tool discovery, tool calls, responses, errors, timings, and stderr into a local `.mcpdoctor` session.
- 🔁 **Replay tool calls** — prove whether a bug still reproduces against the current server implementation.
- 📜 **Validate contracts** — check recorded tool arguments against the server’s advertised `inputSchema`, and flag malformed `tools/call` response content.
- 🧬 **Diff traces for breaking changes** — catch removed tools, new required args, and schema drift between two server versions.
- 📦 **Export a repro bundle** — generate a Markdown report and a replay shell script you can attach to GitHub issues.
- 🔐 **Private by default** — no telemetry, no accounts, no hosted collector; likely secret keys are redacted before writing traces/reports.
- 🧪 **Built to be tested locally** — ships with a fake MCP server fixture that exercises success, invalid JSON, missing tools, schema changes, malformed content, and stderr auth failures.

## Install / build

```bash
git clone https://github.com/ashark-ai-05/mcp-doctor.git
cd mcp-doctor
cargo build --release
./target/release/mcp-doctor --help
```

> A macOS x86_64 binary is attached to the latest GitHub release. Source builds remain the most portable path until Linux/macOS release automation is added.

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

Then inspect, validate, replay, and export the trace:

```bash
TRACE=.mcpdoctor/sessions/<id>/trace.jsonl

cargo run -- validate "$TRACE"
cargo run -- tui "$TRACE"
cargo run -- replay "$TRACE" -- python3 fixtures/fake_mcp_server.py
cargo run -- report "$TRACE" --output repro.md
cargo run -- export-repro "$TRACE" --output replay.sh
```

Example outputs checked into this repo:

- [`examples/fake-echo-trace.jsonl`](examples/fake-echo-trace.jsonl)
- [`examples/fake-echo-report.md`](examples/fake-echo-report.md)
- [`examples/filesystem-list-directory-trace.jsonl`](examples/filesystem-list-directory-trace.jsonl)
- [`examples/filesystem-list-directory-report.md`](examples/filesystem-list-directory-report.md)
- [`examples/sequential-thinking-trace.jsonl`](examples/sequential-thinking-trace.jsonl)
- [`examples/sequential-thinking-report.md`](examples/sequential-thinking-report.md)
- [`examples/github-auth-failure-trace.jsonl`](examples/github-auth-failure-trace.jsonl)
- [`examples/github-auth-failure-report.md`](examples/github-auth-failure-report.md)

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

## What it catches

### Bad tool arguments vs advertised schema

```bash
cargo run -- call stdio \
  --tool echo \
  --args '{"text":"hi"}' \
  -- python3 fixtures/fake_mcp_server.py --mode schema-required

cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
```

MCP Doctor reports the new required field that the client did not send.

### Malformed `tools/call` response content

```bash
cargo run -- call stdio \
  --tool echo \
  --args '{"text":"hi"}' \
  -- python3 fixtures/fake_mcp_server.py --mode malformed-content

cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
```

MCP Doctor flags text content blocks missing the required string `text` field.

### Breaking changes between server versions

```bash
cargo run -- diff old-trace.jsonl new-trace.jsonl
```

Useful for CI-style checks before shipping a new MCP server version.

### Volatile replay fields

```bash
cargo run -- replay old-trace.jsonl \
  --allow-field content.0.text \
  -- python3 fixtures/fake_mcp_server.py --mode schema-change
```

Ignore known volatile fields while still catching structural mismatches.

## Current status

MCP Doctor has been fixture-tested and dogfooded against three public MCP servers:

- `@modelcontextprotocol/server-filesystem`
- `@modelcontextprotocol/server-memory`
- `@modelcontextprotocol/server-sequential-thinking`
- `@modelcontextprotocol/server-github` no-token/auth-failure path

See [`docs/dogfood.md`](docs/dogfood.md) for commands, versions, observed results, and caveats.

Implemented foundation:

- stdio MCP server smoke checks;
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

## Safety and privacy

- No telemetry.
- No network calls by default for stdio workflows.
- HTTP probing only connects to the URL you provide.
- Trace/report values are redacted for likely secret keys such as `token`, `password`, `secret`, `api_key`, and `Authorization`.
- MCP tool responses can still contain sensitive application data; inspect traces before sharing.

## Non-goals

- Not an MCP gateway.
- Not a SaaS trace platform.
- Not a replacement for the official browser Inspector.
- Not a security/governance product yet.
- Not a broad LLM observability dashboard.

## Roadmap

Near-term polish:

- publish Linux and Apple Silicon release binaries;
- add Homebrew/crates.io packaging;
- add GitHub Actions once workflow-scoped auth is available;
- dogfood against API-backed MCP servers with auth failure paths.

Product expansion:

- full interactive Ratatui timeline browser;
- full MCP Streamable HTTP/SSE session semantics;
- schema-aware golden test generation;
- passive proxy mode;
- GitHub issue bundle export.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
```

## Launch note

Reddit/Substack/HN-style writeups should wait until the repo has been dogfooded against real MCP servers, release binaries are available, GitHub topics are set, and the README/demo have been rechecked on GitHub. See [`docs/launch-checklist.md`](docs/launch-checklist.md).

## License

MIT
