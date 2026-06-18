# Homebrew formula prototype

Status: prototype formula lives in this repo at `Formula/mcp-doctor.rb`.

This is not a published tap yet. It is ready to copy into a future Homebrew tap such as:

```text
ashark-ai-05/homebrew-tap
```

## Local test

Homebrew 6 rejects installing formula files directly from arbitrary repo paths. Test through a local tap:

```bash
brew tap-new ashark-ai-05/tap
TAP_DIR="$(brew --repository ashark-ai-05/tap)"
mkdir -p "$TAP_DIR/Formula"
cp Formula/mcp-doctor.rb "$TAP_DIR/Formula/mcp-doctor.rb"
brew style ashark-ai-05/tap/mcp-doctor
brew audit --formula ashark-ai-05/tap/mcp-doctor
brew install ashark-ai-05/tap/mcp-doctor
mcp-doctor --version
brew test ashark-ai-05/tap/mcp-doctor
brew uninstall mcp-doctor
```

Verified locally on 2026-06-17:

```text
brew style ashark-ai-05/tap/mcp-doctor: no offenses
brew install ashark-ai-05/tap/mcp-doctor: installed 0.4.0
mcp-doctor --version: mcp-doctor 0.4.0
brew test ashark-ai-05/tap/mcp-doctor: passed
```

## Publish path

1. Create or reuse a tap repo named `homebrew-tap`.
2. Copy `Formula/mcp-doctor.rb` into that repo under `Formula/`.
3. Commit and push.
4. Install with:

```bash
brew tap ashark-ai-05/tap
brew install mcp-doctor
```

## Updating the formula

For each MCP Doctor release:

1. Build and upload the four release tarballs.
2. Compute sha256 for each tarball.
3. Update `version`, URLs, and SHA values in `Formula/mcp-doctor.rb`.
4. Run `brew audit --strict --online mcp-doctor` inside the tap repo when ready.

Current formula targets:

```text
macOS x86_64
macOS arm64 / Apple Silicon
Linux x86_64 musl
Linux arm64 musl
```
