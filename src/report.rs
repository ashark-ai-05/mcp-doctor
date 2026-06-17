use crate::trace::{TraceEvent, read_trace, truncate};
use crate::validate::validate_events;
use anyhow::Context;
use std::fmt::Write as _;
use std::path::Path;

pub fn render_markdown(trace_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let events = read_trace(&trace_path)?;
    let issues = validate_events(&events);
    let mut out = String::new();
    writeln!(&mut out, "# MCP Doctor Report")?;
    writeln!(&mut out)?;
    writeln!(&mut out, "Trace: `{}`", trace_path.as_ref().display())?;
    writeln!(&mut out)?;
    let requests = events
        .iter()
        .filter(|event| matches!(event, TraceEvent::RpcRequest { .. }))
        .count();
    let responses = events
        .iter()
        .filter(|event| matches!(event, TraceEvent::RpcResponse { .. }))
        .count();
    let stderr = events
        .iter()
        .filter(|event| matches!(event, TraceEvent::Stderr { .. }))
        .count();
    writeln!(&mut out, "## Summary")?;
    writeln!(&mut out)?;
    writeln!(&mut out, "- Events: {}", events.len())?;
    writeln!(&mut out, "- RPC requests: {requests}")?;
    writeln!(&mut out, "- RPC responses: {responses}")?;
    writeln!(&mut out, "- Stderr lines: {stderr}")?;
    writeln!(&mut out, "- Validation issues: {}", issues.len())?;
    writeln!(&mut out)?;
    writeln!(&mut out, "## Findings")?;
    writeln!(&mut out)?;
    if issues.is_empty() {
        writeln!(&mut out, "No validation issues found.")?;
    } else {
        for issue in &issues {
            writeln!(
                &mut out,
                "- {:?}: `{}` — {}",
                issue.severity, issue.target, issue.message
            )?;
        }
    }
    writeln!(&mut out)?;
    writeln!(&mut out, "## Timeline")?;
    writeln!(&mut out)?;
    for event in &events {
        match event {
            TraceEvent::RpcRequest {
                method,
                id,
                payload,
                ..
            } => {
                writeln!(&mut out, "### Request {id:?}: `{method}`")?;
                writeln!(&mut out, "```json")?;
                writeln!(
                    &mut out,
                    "{}",
                    truncate(&serde_json::to_string_pretty(payload)?, 4000)
                )?;
                writeln!(&mut out, "```")?;
            }
            TraceEvent::RpcResponse {
                id,
                duration_ms,
                payload,
                ..
            } => {
                writeln!(&mut out, "### Response {id:?} ({duration_ms}ms)")?;
                writeln!(&mut out, "```json")?;
                writeln!(
                    &mut out,
                    "{}",
                    truncate(&serde_json::to_string_pretty(payload)?, 4000)
                )?;
                writeln!(&mut out, "```")?;
            }
            TraceEvent::InvalidOutput { line, reason, .. } => {
                writeln!(&mut out, "### Invalid output")?;
                writeln!(&mut out, "Reason: {reason}")?;
                writeln!(&mut out, "```text\n{}\n```", truncate(line, 1000))?;
            }
            TraceEvent::Stderr { line, .. } => {
                writeln!(&mut out, "### Stderr")?;
                writeln!(&mut out, "```text\n{}\n```", truncate(line, 1000))?;
            }
            _ => {}
        }
    }
    Ok(out)
}

pub fn write_markdown(
    trace_path: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let content = render_markdown(&trace_path)?;
    std::fs::write(output.as_ref(), content)
        .with_context(|| format!("write report {}", output.as_ref().display()))?;
    Ok(())
}
