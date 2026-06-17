# MCP Doctor Launch Checklist

Status: **hold public writeups for now**.

K asked for Reddit/Substack writeups later only after the product is fully tested, polished, and has a strong GitHub page with proper tags. Do not publish or draft public claims as if launch is already approved.

## GitHub readiness

- [x] Strong README with clear positioning and demo GIF.
- [x] Local-first/privacy notes.
- [x] Install/build instructions.
- [x] 30-second demo.
- [x] Commands reference.
- [x] Safety/non-goals/roadmap.
- [x] GitHub repository topics set.
- [x] At least 3 real-world MCP servers dogfooded.
- [x] Example traces/reports sanitized and checked in under `examples/` or linked from docs.
- [x] README viewed on GitHub after push to verify GIF renders and formatting is clean.
- [ ] Prebuilt release binaries for macOS and Linux.
- [ ] GitHub Actions CI active under `.github/workflows/`.

## Test readiness

Required before any public writeup:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- smoke stdio -- python3 fixtures/fake_mcp_server.py
cargo run -- call stdio --tool echo --args '{"text":"hi"}' -- python3 fixtures/fake_mcp_server.py
```

Recommended dogfood targets:

- a simple official/example MCP server;
- one filesystem/server-style MCP server;
- one API-backed MCP server with auth failure paths redacted.

## Launch angles to save for later

Potential Reddit/HN/Substack framing once ready:

1. **"I built a terminal doctor for MCP servers"** — show trace → validate → replay workflow.
2. **"MCP needs reproducible bug reports"** — position around server authors and agent-app developers.
3. **"Debugging MCP JSON-RPC should feel like running a test"** — focus on replay and contract validation.

## Claims to avoid until proven

- Do not claim full HTTP/SSE transport parity yet.
- Do not claim production security auditing.
- Do not claim broad MCP ecosystem compatibility until dogfooded.
- Do not market as an Inspector replacement; position as terminal-native reproducibility and CI/debug support.
