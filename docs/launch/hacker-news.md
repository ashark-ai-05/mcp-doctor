# Hacker News launch draft

## Title

```text
Show HN: MCP Doctor, a terminal debugger for MCP servers
```

## Post/comment

I built a small Rust CLI for debugging Model Context Protocol servers from the terminal:

https://github.com/ashark-ai-05/mcp-doctor

The goal is to make MCP failures reproducible. It records JSON-RPC sessions to JSONL, validates tool call arguments and response shapes, replays recorded calls, diffs traces, and exports Markdown repro reports/replay scripts that can be attached to issues.

Example workflow:

```bash
mcp-doctor smoke stdio -- <server command>
mcp-doctor call stdio --tool <name> --args '{"text":"hi"}' -- <server command>
mcp-doctor validate .mcpdoctor/sessions/<id>/trace.jsonl
mcp-doctor replay .mcpdoctor/sessions/<id>/trace.jsonl -- <server command>
mcp-doctor report .mcpdoctor/sessions/<id>/trace.jsonl --output report.md
```

It is local-first: no telemetry, no hosted collector, and obvious token/password/api_key/auth fields are redacted before writing traces.

I dogfooded it against filesystem, memory, sequential-thinking, and the GitHub MCP server auth-failure path. The auth-failure test found a real issue in my own tool: failed tool calls were returning before preserving useful artifacts. That is fixed now; failed calls still write a trace/report before returning non-zero.

Known gaps: stdio is the main path right now; HTTP support is basic probe-level support, not full Streamable HTTP/SSE parity. CI is also not active yet because my current GitHub token lacks workflow scope. Binaries are attached for macOS and Linux, x86_64 and ARM64.

I would like feedback from anyone building MCP servers: what would make a trace/report useful enough to attach to a bug report?
