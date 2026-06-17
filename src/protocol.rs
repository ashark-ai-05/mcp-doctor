use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn initialize_request() -> (u64, Value) {
    let id = next_id();
    (
        id,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "mcp-doctor", "version": env!("CARGO_PKG_VERSION")}
            }
        }),
    )
}

pub fn initialized_notification() -> Value {
    json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
}

pub fn tools_list_request() -> (u64, Value) {
    let id = next_id();
    (
        id,
        json!({"jsonrpc": "2.0", "id": id, "method": "tools/list", "params": {}}),
    )
}

pub fn tools_call_request(name: &str, args: Value) -> (u64, Value) {
    let id = next_id();
    (
        id,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {"name": name, "arguments": args}
        }),
    )
}

pub fn method(payload: &Value) -> Option<&str> {
    payload.get("method")?.as_str()
}

pub fn id_string(payload: &Value) -> String {
    payload
        .get("id")
        .map(|id| match id {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .unwrap_or_else(|| "<none>".to_string())
}

pub fn is_success_response(payload: &Value) -> bool {
    payload.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && payload.get("id").is_some()
        && payload.get("error").is_none()
}

pub fn response_error_message(payload: &Value) -> Option<String> {
    let error = payload.get("error")?;
    if let Some(message) = error.get("message").and_then(Value::as_str) {
        Some(message.to_string())
    } else {
        Some(error.to_string())
    }
}
