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
    allow_fields: &[String],
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
        let expected = comparable_result(&call.response, allow_fields);
        let actual = comparable_result(&response, allow_fields);
        if expected == actual {
            result.matched += 1;
        } else {
            result
                .mismatched
                .push(format!("tool {name}: expected {expected}, got {actual}"));
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

fn comparable_result(response: &Value, allow_fields: &[String]) -> Value {
    let mut value = if let Some(result) = response.get("result") {
        result.clone()
    } else if let Some(error) = protocol::response_error_message(response) {
        Value::String(format!("error:{error}"))
    } else {
        response.clone()
    };
    for path in allow_fields {
        strip_path(&mut value, path);
    }
    value
}

fn strip_path(value: &mut Value, path: &str) {
    let parts: Vec<_> = path.split('.').filter(|part| !part.is_empty()).collect();
    strip_path_parts(value, &parts);
}

fn strip_path_parts(value: &mut Value, parts: &[&str]) {
    if parts.is_empty() {
        *value = Value::String("[IGNORED]".to_string());
        return;
    }
    match value {
        Value::Object(map) => {
            if let Some(child) = map.get_mut(parts[0]) {
                strip_path_parts(child, &parts[1..]);
            }
        }
        Value::Array(items) => {
            if let Some(child) = parts[0]
                .parse::<usize>()
                .ok()
                .and_then(|idx| items.get_mut(idx))
            {
                strip_path_parts(child, &parts[1..]);
            }
        }
        _ => {}
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

    #[test]
    fn ignores_allowed_field_path() {
        let mut left = json!({"content":[{"type":"text","text":"a"}]});
        let mut right = json!({"content":[{"type":"text","text":"b"}]});
        strip_path(&mut left, "content.0.text");
        strip_path(&mut right, "content.0.text");
        assert_eq!(left, right);
    }
}
