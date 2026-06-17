# MCP Doctor v0.2.3

This is a hardening release focused on auth/API failure dogfood and better failure artifacts.

## What changed

- Dogfooded an API-backed/auth MCP server failure path with `@modelcontextprotocol/server-github` and token env vars explicitly unset.
- Fixed `mcp-doctor call` so JSON-RPC tool-call errors still write a trace and report before returning non-zero.
- Added integration coverage for failed tool calls producing trace/report artifacts.
- Updated dogfood docs and launch checklist with the GitHub auth-failure result.

## Why it matters

A core product promise is reproducible bug reports. Before this release, a failed `tools/call` could return an error before printing the trace/report path. That is exactly when users most need an artifact. Now MCP Doctor preserves the failure evidence.

## Verification

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py
vhs validate docs/demo.tape
```

## Dogfood result snapshot

```text
filesystem: smoke/record/validate/call/replay passed
memory: smoke/record/validate/call passed; replay correctly exposed stateful mismatch
sequential-thinking: smoke/record/validate/call/replay passed
github no-token create_issue: auth error returned; trace/report written; validate passed
```

## Binary artifact

This release includes a macOS x86_64 binary produced locally. Linux and Apple Silicon release automation remain future work because this host has Homebrew Rust without `rustup` cross targets.
