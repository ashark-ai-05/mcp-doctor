# Reddit launch draft

Suggested subreddits, only after checking each community's self-promo rules:

- r/rust
- r/commandline
- r/selfhosted, only if framed as local-first tooling
- r/LocalLLaMA, only if MCP/tooling discussion is welcome
- r/programming, if the post is technical and not salesy

## Title options

```text
I built MCP Doctor: a terminal debugger for MCP servers
```

```text
MCP Doctor: record, validate, replay, and report MCP JSON-RPC sessions from the terminal
```

## Post

I got tired of debugging MCP servers through scattered logs and half-copied JSON-RPC payloads, so I built a small Rust CLI for it.

Repo:

https://github.com/ashark-ai-05/mcp-doctor

MCP Doctor is a local terminal tool for MCP server authors and people integrating MCP tools into agents. The basic workflow is:

```text
run a server/tool call
capture the JSON-RPC trace
validate the trace against the advertised tool schemas
replay the failing call
export a Markdown repro report
```

The part I care about most is reproducibility. If an MCP tool fails, I want a trace and report I can attach to an issue instead of saying "it failed in my agent".

It currently supports:

```text
mcp-doctor smoke stdio -- <server command>
mcp-doctor call stdio --tool <name> --args '{...}' -- <server command>
mcp-doctor validate <trace.jsonl>
mcp-doctor replay <trace.jsonl> -- <server command>
mcp-doctor diff <old-trace.jsonl> <new-trace.jsonl>
mcp-doctor report <trace.jsonl> --output report.md
mcp-doctor export-repro <trace.jsonl> --output replay.sh
```

It writes local artifacts under `.mcpdoctor/`:

```text
trace.jsonl
report.md
```

No telemetry. No hosted collector. Secrets with obvious token/password/api_key/auth fields are redacted before writing traces, but you should still inspect traces before sharing them because tool responses can contain application data.

I dogfooded it against:

```text
@modelcontextprotocol/server-filesystem
@modelcontextprotocol/server-memory
@modelcontextprotocol/server-sequential-thinking
@modelcontextprotocol/server-github auth-failure path
```

The GitHub auth-failure case exposed a real bug: failed tool calls were returning before preserving the trace/report. That is fixed now. Failed `tools/call` responses still write artifacts before returning non-zero.

Binaries are attached for macOS and Linux, x86_64 and ARM64.

Known gaps:

```text
- stdio is the main path; HTTP support is basic probe-level right now
- this is not a replacement for the official Inspector
- GitHub Actions CI is not active yet because my current GitHub token lacks workflow scope
- no Homebrew/crates.io packaging yet
```

I would especially like feedback from people building MCP servers: what does your current debug loop look like, and what artifact would you want attached to a useful bug report?
