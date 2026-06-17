# MCP Doctor Product Spec

See the source strategy document in `ai-action-lab/2026-06-17-rust-tui-product-specs/mcp-doctor-v0.1-build-spec.md`.

## V0.1 implementation stance

V0.1 deliberately prioritizes a testable non-interactive workflow over TUI polish:

1. stdio process lifecycle;
2. JSON-RPC request/response timeline;
3. JSONL trace persistence;
4. deterministic replay of recorded tool calls;
5. Markdown report generation;
6. validation and redaction.

The future TUI should read the same trace model rather than owning protocol logic.
