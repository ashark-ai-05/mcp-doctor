# Substack launch draft

# Debugging MCP servers should produce evidence, not vibes

MCP is quickly becoming the tool layer for AI agents. That is good. It is also going to make a lot of ordinary debugging feel weird for a while.

A tool call fails inside an agent. The server logs say one thing. The client logs say another. Somewhere in the middle there is a JSON-RPC message, a schema, maybe stderr, maybe an auth problem, maybe a tool response that technically returned but did not match what the client expected.

The bug report usually sounds like this:

```text
It works in Inspector but fails in my agent.
```

That is not enough to debug from.

So I built MCP Doctor:

https://github.com/ashark-ai-05/mcp-doctor

It is a small Rust terminal tool for recording, validating, replaying, and reporting MCP JSON-RPC sessions.

## The workflow I wanted

I did not want another hosted observability product. I wanted something closer to a local repro harness:

```text
run the MCP server
call the tool
save the JSON-RPC trace
validate the arguments and response shape
replay the failing call
export a report I can attach to an issue
```

That is the core of MCP Doctor.

```bash
mcp-doctor smoke stdio -- <server command>
mcp-doctor call stdio --tool <name> --args '{"text":"hi"}' -- <server command>
mcp-doctor validate .mcpdoctor/sessions/<id>/trace.jsonl
mcp-doctor replay .mcpdoctor/sessions/<id>/trace.jsonl -- <server command>
mcp-doctor report .mcpdoctor/sessions/<id>/trace.jsonl --output report.md
```

It writes local artifacts:

```text
.mcpdoctor/sessions/<id>/trace.jsonl
.mcpdoctor/sessions/<id>/report.md
```

No telemetry. No account. No collector. You own the trace.

## Why this matters

MCP has a nice protocol shape, but debugging protocol-shaped systems always comes down to evidence.

What did the client send?

What did the server advertise?

Did the arguments match the advertised input schema?

Did the tool response have the shape the client expected?

Did stderr contain the real failure?

Can another person replay the same call?

If you cannot answer those questions, you do not have a reproducible bug. You have a story about a bug.

MCP Doctor is meant to turn that story into an artifact.

## What it catches today

The current version can:

- smoke test a stdio MCP server
- record initialize, tools/list, tools/call, responses, errors, timings, and stderr
- validate tool call arguments against advertised input schemas
- flag malformed `tools/call` response content
- replay recorded tool calls against a server
- diff traces for removed tools, new required args, and schema drift
- export a Markdown report and replay script
- redact obvious secret fields before writing traces

It is not trying to replace the official Inspector. The Inspector is a good visual tool for exploring servers. MCP Doctor is aimed at the terminal workflow: trace, validate, replay, attach evidence.

## The dogfood bug that changed the tool

The most useful test was not a happy path.

I tried an API-backed server with auth missing: `@modelcontextprotocol/server-github`, with the token variables explicitly unset.

The server correctly returned an auth error. But MCP Doctor had a bad behavior: the command returned non-zero before preserving the artifact paths clearly enough.

That was exactly backwards. The failure case is when you most need a trace and report.

So I changed the behavior. Failed `tools/call` responses now still write the trace and Markdown report, mark the session as an error, print the artifact paths, and then return non-zero.

That is the product thesis in miniature: failures should leave evidence.

## Current state

The repo has been dogfooded against:

```text
@modelcontextprotocol/server-filesystem
@modelcontextprotocol/server-memory
@modelcontextprotocol/server-sequential-thinking
@modelcontextprotocol/server-github auth-failure path
```

The release includes binaries for:

```text
macOS x86_64
macOS arm64 / Apple Silicon
Linux x86_64 musl
Linux arm64 musl
```

The test suite is small but real: fixture-backed integration tests cover smoke, call, validation, replay, diffing, export, and the failure-artifact behavior.

## Known gaps

This is still an early tool.

The stdio path is the main workflow. HTTP support is basic probe-level support right now, not full Streamable HTTP/SSE parity.

It is not a security scanner.

It is not a hosted trace platform.

CI is not active yet because my current GitHub token lacks `workflow` scope. The workflow file is prepared as a prototype, but GitHub will not accept it under `.github/workflows/` until I refresh auth with the right scope.

Homebrew and crates.io packaging are still on the list.

## What I want feedback on

If you build or maintain MCP servers, I want to know what your current debug loop looks like.

What do you wish a user attached to a GitHub issue?

A JSONL trace?

A Markdown report?

A replay script?

A schema diff?

A redacted environment summary?

MCP is early enough that the debugging workflow is not settled yet. My bet is that reproducible local artifacts will matter more than another dashboard.

Repo:

https://github.com/ashark-ai-05/mcp-doctor
