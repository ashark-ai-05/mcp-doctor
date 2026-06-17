# MCP Doctor next roadmap

This file tracks the post-v0.3.0 path after the launch-ready MVP.

## Immediate distribution work

1. **Activate CI**
   - Blocked until `gh` auth has `workflow` scope.
   - Prototype: `docs/prototypes/ci.yml`.
   - Target path after auth refresh: `.github/workflows/ci.yml`.

2. **Publish Homebrew tap**
   - Prototype formula: `Formula/mcp-doctor.rb`.
   - Docs: `docs/homebrew.md`.

3. **Launch posts**
   - Reddit: `docs/launch/reddit.md`.
   - Substack: `docs/launch/substack.md`.
   - Hacker News: `docs/launch/hacker-news.md`.
   - Do not post until K explicitly says to publish.

## Product roadmap

1. **GitHub issue bundle export**
   - Generate a self-contained Markdown issue body from a trace/report.
   - Include commands, environment summary, server command, validation issues, and replay instructions.

2. **Ratatui timeline browser**
   - Turn the current read-only text TUI into a real interactive trace browser.
   - Panes: event list, payload, validation issues, stderr, replay hints.

3. **Schema-aware golden tests**
   - Generate fixture tests from known-good tool calls.
   - Useful for MCP server authors who want contract tests in CI.

4. **Streamable HTTP/SSE support**
   - Move beyond basic HTTP probe support.
   - Capture MCP HTTP sessions with the same trace/report/replay mindset.

5. **Passive proxy mode**
   - Sit between an agent client and an MCP server to capture real client behavior.
   - High value, but more protocol-risky than the current stdio workflows.

## Feedback questions for launch

Ask MCP server authors:

- What does your current MCP debugging loop look like?
- What artifact would make a bug report actionable?
- Do you need replay scripts, JSONL traces, schema diffs, or Markdown reports most?
- Which MCP servers should be dogfooded next?
