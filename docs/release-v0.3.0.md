# MCP Doctor v0.3.0

This release is the launch-ready MVP cut. It does not add a new surface area feature. It finishes the packaging and launch collateral around the v0.2.x product hardening work.

## What changed

- Added prebuilt release binaries for:
  - macOS x86_64
  - macOS arm64 / Apple Silicon
  - Linux x86_64 musl
  - Linux arm64 musl
- Installed a local rustup toolchain and cross-build tooling so release artifacts are reproducible from this Mac.
- Added Reddit and Substack launch drafts under `docs/launch/`.
- Updated README status now that Linux and Apple Silicon release binaries exist.
- Updated launch checklist with the exact remaining CI blocker.

## Why this matters

MCP Doctor was already useful after v0.2.3: it could smoke test, record, validate, replay, diff, export, and preserve artifacts for failed tool calls. The missing piece was release readiness. People should not need to clone and build from source just to try a terminal debugger.

v0.3.0 makes the repo easier to try and easier to launch.

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

## Build targets verified

```text
target/x86_64-apple-darwin/release/mcp-doctor: Mach-O 64-bit executable x86_64
target/aarch64-apple-darwin/release/mcp-doctor: Mach-O 64-bit executable arm64
target/x86_64-unknown-linux-musl/release/mcp-doctor: ELF 64-bit LSB executable, x86-64, statically linked
target/aarch64-unknown-linux-musl/release/mcp-doctor: ELF 64-bit LSB executable, ARM aarch64, statically linked
```

## Known gap

GitHub Actions CI is still not active because the current GitHub token lacks `workflow` scope. The workflow prototype remains at `docs/prototypes/ci.yml`. Activate it after refreshing auth with workflow scope.
