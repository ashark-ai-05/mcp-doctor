# Real MCP Server Dogfood — 2026-06-17

Dogfood target: prove MCP Doctor works beyond the local fake fixture against public MCP servers installed via `npx`.

Environment:

```text
node: v25.5.0
npm/npx: 11.8.0
```

Packages tested:

```text
@modelcontextprotocol/server-filesystem        2026.1.14
@modelcontextprotocol/server-memory            2026.1.26
@modelcontextprotocol/server-sequential-thinking 2025.12.18
```

## Summary

| Server | Smoke | Record | Validate | Call | Replay | Notes |
|---|---:|---:|---:|---:|---:|---|
| filesystem | pass | pass | pass | pass | pass | `list_directory` matched on replay when using canonical `/private/tmp/...` path on macOS. |
| memory | pass | pass | pass | pass | expected mismatch | `create_entities` is stateful/non-idempotent, so replay correctly exposed a changed result. |
| sequential-thinking | pass | pass | pass | pass | pass | Dogfood found and fixed string-format redaction bug that previously collapsed JSON-looking text. |

## Commands run

### Filesystem

```bash
ROOT=/private/tmp/mcp-doctor-real-calls/fs-root
cargo run -- --timeout-ms 30000 smoke stdio -- \
  npx -y @modelcontextprotocol/server-filesystem "$ROOT"

cargo run -- --timeout-ms 30000 call stdio \
  --tool list_directory \
  --args "{\"path\":\"$ROOT\"}" \
  -- npx -y @modelcontextprotocol/server-filesystem "$ROOT"

cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- --timeout-ms 30000 replay .mcpdoctor/sessions/<id>/trace.jsonl -- \
  npx -y @modelcontextprotocol/server-filesystem "$ROOT"
```

Observed:

```text
ok: initialized server and discovered 14 tool(s)
trace valid: 12 events
replay: 1 calls, 1 matched, 0 mismatched
```

### Memory

```bash
cargo run -- --timeout-ms 30000 smoke stdio -- \
  npx -y @modelcontextprotocol/server-memory

cargo run -- --timeout-ms 30000 call stdio \
  --tool create_entities \
  --args '{"entities":[{"name":"mcp-doctor-dogfood","entityType":"test","observations":["MCP Doctor can call and validate the memory server"]}]}' \
  -- npx -y @modelcontextprotocol/server-memory

cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- --timeout-ms 30000 replay .mcpdoctor/sessions/<id>/trace.jsonl -- \
  npx -y @modelcontextprotocol/server-memory
```

Observed:

```text
ok: initialized server and discovered 9 tool(s)
trace valid: 11 events
replay: 1 calls, 0 matched, 1 mismatched
```

Interpretation: mismatch is useful evidence, not a tool failure. The call mutates server state, so a second replay returns a different entity creation result.

### Sequential Thinking

```bash
cargo run -- --timeout-ms 30000 smoke stdio -- \
  npx -y @modelcontextprotocol/server-sequential-thinking

cargo run -- --timeout-ms 30000 call stdio \
  --tool sequentialthinking \
  --args '{"thought":"MCP Doctor dogfood replay formatting check","nextThoughtNeeded":false,"thoughtNumber":1,"totalThoughts":1}' \
  -- npx -y @modelcontextprotocol/server-sequential-thinking

cargo run -- validate .mcpdoctor/sessions/<id>/trace.jsonl
cargo run -- --timeout-ms 30000 replay .mcpdoctor/sessions/<id>/trace.jsonl -- \
  npx -y @modelcontextprotocol/server-sequential-thinking
```

Observed after the redaction formatting fix:

```text
ok: initialized server and discovered 1 tool(s)
trace valid: 17 events
replay: 1 calls, 1 matched, 0 mismatched
```

## Bug found and fixed

Dogfooding `@modelcontextprotocol/server-sequential-thinking` exposed a bug in MCP Doctor's redaction layer: non-secret strings were passed through `split_whitespace()`, which collapsed newlines/indentation in JSON-looking tool output. That made replay compare a whitespace-flattened recorded response against the live pretty-printed response.

Fix: `redact_text` now preserves non-secret strings exactly and only rewrites strings that contain likely secret assignments or bearer tokens.

Regression test added:

```text
redaction::tests::preserves_non_secret_string_formatting
```

## Remaining launch caveats

- Need more dogfood against API-backed MCP servers with auth failure paths.
- Need release binaries for more platforms than the current build host.
- GitHub Actions is still a prototype under `docs/prototypes/ci.yml` until workflow-scoped GitHub auth is available.
