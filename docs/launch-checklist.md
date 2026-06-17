# MCP Doctor launch checklist

Status: **launch assets are ready; public posting still needs K's explicit go-ahead**.

K asked for Reddit/Substack writeups to be prepared, not posted. Do not publish to Reddit, Substack, HN, X, or LinkedIn without a direct follow-up instruction.

## GitHub readiness

- [x] Strong README with clear positioning and demo GIF.
- [x] Local-first/privacy notes.
- [x] Install/build instructions.
- [x] 30-second demo.
- [x] Commands reference.
- [x] Safety/non-goals/roadmap.
- [x] GitHub repository topics set.
- [x] At least 3 real-world MCP servers dogfooded.
- [x] API-backed/auth failure path dogfooded without secrets.
- [x] Example traces/reports sanitized and checked in under `examples/`.
- [x] README viewed on GitHub after push to verify GIF renders and formatting is clean.
- [x] Prebuilt release binaries for macOS x86_64, macOS arm64, Linux x86_64 musl, and Linux arm64 musl.
- [x] Draft Reddit/Substack launch writeups prepared under `docs/launch/`.
- [ ] GitHub Actions CI active under `.github/workflows/`.

## CI blocker

The repository has a CI prototype at `docs/prototypes/ci.yml`, but the current GitHub token has scopes:

```text
admin:public_key, gist, read:org, repo
```

It does not have `workflow` scope. GitHub rejects pushes that add or update `.github/workflows/*` without that scope. Activate CI by refreshing `gh` auth with workflow scope, then move the prototype to `.github/workflows/ci.yml` and push.

## Test readiness

Required before public posting:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py
vhs validate docs/demo.tape
```

Release builds now also verify these targets:

```text
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
```

## Launch angles

Use the drafts in `docs/launch/`:

1. Reddit: "I built a terminal doctor for MCP servers".
2. Substack: "MCP needs reproducible bug reports".
3. Optional HN title: "Show HN: MCP Doctor, a terminal debugger for MCP servers".

## Claims to avoid

- Do not claim full MCP Streamable HTTP/SSE session parity yet.
- Do not claim production security auditing.
- Do not claim broad ecosystem compatibility beyond the dogfooded servers.
- Do not market it as an Inspector replacement. Position it as terminal-native trace/replay/debug support.
