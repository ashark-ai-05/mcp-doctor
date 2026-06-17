use crate::protocol;
use crate::session::StdioSession;
use crate::trace::{TraceEvent, read_trace};
use anyhow::Context;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ReplayResult {
    pub replayed: usize,
    pub matched: usize,
    pub mismatched: Vec<String>,
}

pub fn replay_trace(
    trace_path: impl AsRef<Path>,
    server: &[String],
    timeout: Duration,
) -> anyhow::Result<ReplayResult> {
    let events = read_trace(&trace_path)?;
    let calls = recorded_tool_calls(&events);
    let mut session = StdioSession::spawn(server, timeout, None)?;
    session.initialize_and_list_tools()?;
    let mut result = ReplayResult {
        replayed: 0,
        matched: 0,
        mismatched: Vec::new(),
    };
    for call in calls {
        result.replayed += 1;
        let name = call
            .request
            .get("params")
            .and_then(|params| params.get("name"))
            .and_then(Value::as_str)
            .context("recorded tools/call missing params.name")?;
        let args = call
            .request
            .get("params")
            .and_then(|params| params.get("arguments"))
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));
        let response = session.call_tool(name, args)?;
        if comparable_result(&response) == comparable_result(&call.response) {
            result.matched += 1;
        } else {
            result.mismatched.push(format!(
                "tool {name}: expected {}, got {}",
                comparable_result(&call.response),
                comparable_result(&response)
            ));
        }
    }
    session.finish("ok")?;
    Ok(result)
}

#[derive(Debug, Clone)]
struct RecordedCall {
    request: Value,
    response: Value,
}

fn recorded_tool_calls(events: &[TraceEvent]) -> Vec<RecordedCall> {
    let mut calls = Vec::new();
    for (idx, event) in events.iter().enumerate() {
        let TraceEvent::RpcRequest {
            method, payload, ..
        } = event
        else {
            continue;
        };
        if method != "tools/call" {
            continue;
        }
        let Some(id) = payload.get("id").and_then(Value::as_u64) else {
            continue;
        };
        if let Some(response) = events[idx + 1..]
            .iter()
            .find_map(|candidate| match candidate {
                TraceEvent::RpcResponse {
                    id: Some(response_id),
                    payload,
                    ..
                } if *response_id == id => Some(payload.clone()),
                _ => None,
            })
        {
            calls.push(RecordedCall {
                request: payload.clone(),
                response,
            });
        }
    }
    calls
}

fn comparable_result(response: &Value) -> Value {
    if let Some(result) = response.get("result") {
        result.clone()
    } else if let Some(error) = protocol::response_error_message(response) {
        Value::String(format!("error:{error}"))
    } else {
        response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_recorded_tool_calls() {
        let events = vec![
            TraceEvent::RpcRequest {
                ts: 1,
                id: Some(7),
                method: "tools/call".to_string(),
                payload: json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"echo","arguments":{}}}),
            },
            TraceEvent::RpcResponse {
                ts: 2,
                id: Some(7),
                duration_ms: 1,
                payload: json!({"jsonrpc":"2.0","id":7,"result":{"content":[]}}),
            },
        ];
        assert_eq!(recorded_tool_calls(&events).len(), 1);
    }
}
