use crate::contract::validate_contract;
use crate::protocol;
use crate::trace::{Severity, TraceEvent};

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub target: String,
    pub message: String,
}

pub fn validate_events(events: &[TraceEvent]) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut saw_start = false;
    let mut saw_end = false;
    for (idx, event) in events.iter().enumerate() {
        match event {
            TraceEvent::SessionStart { .. } => saw_start = true,
            TraceEvent::SessionEnd { .. } => saw_end = true,
            TraceEvent::RpcRequest {
                payload, method, ..
            } => {
                if payload.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        target: format!("line {} request {method}", idx + 1),
                        message: "request missing jsonrpc=2.0".to_string(),
                    });
                }
                if payload
                    .get("method")
                    .and_then(serde_json::Value::as_str)
                    .is_none()
                {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        target: format!("line {} request {method}", idx + 1),
                        message: "request missing method".to_string(),
                    });
                }
            }
            TraceEvent::RpcResponse { payload, id, .. } => {
                if !protocol::is_success_response(payload) && payload.get("error").is_none() {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        target: format!("line {} response id={id:?}", idx + 1),
                        message: "response is neither success nor error JSON-RPC shape".to_string(),
                    });
                }
            }
            TraceEvent::InvalidOutput { reason, .. } => issues.push(ValidationIssue {
                severity: Severity::Error,
                target: format!("line {} invalid_output", idx + 1),
                message: format!("server emitted invalid JSON: {reason}"),
            }),
            TraceEvent::Validation {
                severity,
                target,
                message,
                ..
            } if *severity == Severity::Error => issues.push(ValidationIssue {
                severity: *severity,
                target: target.clone(),
                message: message.clone(),
            }),
            _ => {}
        }
    }
    if !saw_start {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            target: "trace".to_string(),
            message: "trace has no session_start event".to_string(),
        });
    }
    if !saw_end {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            target: "trace".to_string(),
            message: "trace has no session_end event".to_string(),
        });
    }
    for issue in validate_contract(events) {
        issues.push(ValidationIssue {
            severity: issue.severity,
            target: issue.target,
            message: issue.message,
        });
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_invalid_response_shape() {
        let events = vec![TraceEvent::RpcResponse {
            ts: 1,
            id: Some(1),
            duration_ms: 1,
            payload: json!({"id": 1, "result": {}}),
        }];
        let issues = validate_events(&events);
        assert!(
            issues
                .iter()
                .any(|issue| issue.message.contains("neither success"))
        );
    }
}
