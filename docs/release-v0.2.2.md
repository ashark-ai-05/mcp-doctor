# MCP Doctor v0.2.2

This is a polish/dogfood release for MCP Doctor, a terminal-native tool for recording, validating, replaying, diffing, and reporting MCP JSON-RPC sessions.

## Highlights

- Marketable README with demo GIF and concrete examples.
- Dogfooded against three public MCP servers:
  - `@modelcontextprotocol/server-filesystem`
  - `@modelcontextprotocol/server-memory`
  - `@modelcontextprotocol/server-sequential-thinking`
- Added real example traces/reports under `examples/`.
- Added `docs/dogfood.md` with exact commands, versions, observed outcomes, and caveats.
- Fixed redaction behavior so non-secret string output preserves formatting/newlines. This fixed replay matching for JSON-looking text responses from sequential-thinking.
- Added regression coverage for preserving non-secret string formatting.
- Added inert CI workflow prototype at `docs/prototypes/ci.yml` until workflow-scoped GitHub auth is available.

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
sequential-thinking: smoke/record/validate/call/replay passed after redaction formatting fix
```

## Install from source

```bash
git clone https://github.com/ashark-ai-05/mcp-doctor.git
cd mcp-doctor
cargo build --release
./target/release/mcp-doctor --help
```

## Binary artifact

This release includes a macOS host build produced locally. Linux/macOS universal release automation is still future work.
