use crate::contract::validate_contract;
use crate::trace::{TraceEvent, read_trace};
use crate::validate::validate_events;
use std::path::Path;

pub fn render_trace_view(trace_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let events = read_trace(&trace_path)?;
    let protocol_issues = validate_events(&events);
    let contract_issues = validate_contract(&events);
    let mut out = String::new();
    out.push_str("MCP Doctor Trace Viewer\n");
    out.push_str("=======================\n\n");
    out.push_str(&format!("Trace: {}\n", trace_path.as_ref().display()));
    out.push_str(&format!("Events: {}\n", events.len()));
    out.push_str(&format!("Protocol issues: {}\n", protocol_issues.len()));
    out.push_str(&format!("Contract issues: {}\n\n", contract_issues.len()));

    out.push_str("Timeline\n--------\n");
    for event in &events {
        match event {
            TraceEvent::SessionStart {
                session_id,
                command,
                ..
            } => {
                out.push_str(&format!(
                    "session_start {session_id} cmd={}\n",
                    command.join(" ")
                ));
            }
            TraceEvent::RpcRequest { id, method, .. } => {
                out.push_str(&format!("→ request {id:?} {method}\n"));
            }
            TraceEvent::RpcResponse {
                id,
                duration_ms,
                payload,
                ..
            } => {
                let status = if payload.get("error").is_some() {
                    "ERR"
                } else {
                    "OK"
                };
                out.push_str(&format!("← response {id:?} {status} {duration_ms}ms\n"));
            }
            TraceEvent::Stderr { line, .. } => out.push_str(&format!("! stderr {line}\n")),
            TraceEvent::InvalidOutput { reason, .. } => {
                out.push_str(&format!("! invalid_output {reason}\n"));
            }
            TraceEvent::SessionEnd { status, .. } => {
                out.push_str(&format!("session_end {status}\n"))
            }
            TraceEvent::ProcessStart { pid, .. } => {
                out.push_str(&format!("process_start pid={pid:?}\n"))
            }
            TraceEvent::Validation {
                severity,
                target,
                message,
                ..
            } => {
                out.push_str(&format!("validation {severity:?} {target}: {message}\n"));
            }
        }
    }
    if !protocol_issues.is_empty() || !contract_issues.is_empty() {
        out.push_str("\nFindings\n--------\n");
        for issue in protocol_issues {
            out.push_str(&format!(
                "- {:?}: {} — {}\n",
                issue.severity, issue.target, issue.message
            ));
        }
        for issue in contract_issues {
            out.push_str(&format!(
                "- {:?}: {} — {}\n",
                issue.severity, issue.target, issue.message
            ));
        }
    }
    Ok(out)
}
