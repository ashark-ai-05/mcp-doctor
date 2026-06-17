use crate::trace::{Severity, TraceEvent};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractIssue {
    pub severity: Severity,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ToolCatalog {
    pub tools: BTreeMap<String, Value>,
}

impl ToolCatalog {
    pub fn from_events(events: &[TraceEvent]) -> Self {
        let mut catalog = ToolCatalog::default();
        for event in events {
            let TraceEvent::RpcResponse { payload, .. } = event else {
                continue;
            };
            let Some(tools) = payload
                .get("result")
                .and_then(|result| result.get("tools"))
                .and_then(Value::as_array)
            else {
                continue;
            };
            for tool in tools {
                if let Some(name) = tool.get("name").and_then(Value::as_str) {
                    catalog.tools.insert(name.to_string(), tool.clone());
                }
            }
        }
        catalog
    }

    pub fn input_schema(&self, tool_name: &str) -> Option<&Value> {
        self.tools.get(tool_name)?.get("inputSchema")
    }
}

pub fn validate_contract(events: &[TraceEvent]) -> Vec<ContractIssue> {
    let catalog = ToolCatalog::from_events(events);
    let mut issues = Vec::new();
    for event in events {
        match event {
            TraceEvent::RpcRequest {
                method, payload, ..
            } if method == "tools/call" => {
                validate_tool_call_request(&catalog, payload, &mut issues);
            }
            TraceEvent::RpcResponse { payload, id, .. } => {
                if is_tool_call_response(events, *id) {
                    validate_tool_call_response(payload, &mut issues, *id);
                }
            }
            _ => {}
        }
    }
    issues
}

fn is_tool_call_response(events: &[TraceEvent], response_id: Option<u64>) -> bool {
    let Some(response_id) = response_id else {
        return false;
    };
    events.iter().any(|event| matches!(event, TraceEvent::RpcRequest { id: Some(id), method, .. } if *id == response_id && method == "tools/call"))
}

fn validate_tool_call_request(
    catalog: &ToolCatalog,
    payload: &Value,
    issues: &mut Vec<ContractIssue>,
) {
    let params = payload.get("params").unwrap_or(&Value::Null);
    let tool_name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let args = params.get("arguments").unwrap_or(&Value::Null);
    if tool_name == "<missing>" {
        issues.push(ContractIssue {
            severity: Severity::Error,
            target: "tools/call".to_string(),
            message: "tools/call is missing params.name".to_string(),
        });
        return;
    }
    let Some(schema) = catalog.input_schema(tool_name) else {
        issues.push(ContractIssue {
            severity: Severity::Error,
            target: format!("tool.{tool_name}"),
            message: "recorded trace has no inputSchema for this tool".to_string(),
        });
        return;
    };
    validate_value_against_schema(args, schema, &format!("tool.{tool_name}.arguments"), issues);
}

fn validate_tool_call_response(payload: &Value, issues: &mut Vec<ContractIssue>, id: Option<u64>) {
    if payload.get("error").is_some() {
        return;
    }
    let Some(result) = payload.get("result") else {
        issues.push(ContractIssue {
            severity: Severity::Error,
            target: format!("response.{id:?}"),
            message: "tools/call response is missing result".to_string(),
        });
        return;
    };
    let Some(content) = result.get("content").and_then(Value::as_array) else {
        issues.push(ContractIssue {
            severity: Severity::Error,
            target: format!("response.{id:?}.result"),
            message: "tools/call result is missing content array".to_string(),
        });
        return;
    };
    for (idx, item) in content.iter().enumerate() {
        let Some(kind) = item.get("type").and_then(Value::as_str) else {
            issues.push(ContractIssue {
                severity: Severity::Error,
                target: format!("response.{id:?}.content[{idx}]"),
                message: "content item is missing type".to_string(),
            });
            continue;
        };
        match kind {
            "text" => {
                if item.get("text").and_then(Value::as_str).is_none() {
                    issues.push(ContractIssue {
                        severity: Severity::Error,
                        target: format!("response.{id:?}.content[{idx}]"),
                        message: "text content item is missing string text".to_string(),
                    });
                }
            }
            "image" | "audio" | "resource" => {}
            other => issues.push(ContractIssue {
                severity: Severity::Warning,
                target: format!("response.{id:?}.content[{idx}]"),
                message: format!("unknown MCP content type `{other}`"),
            }),
        }
    }
}

pub fn validate_value_against_schema(
    value: &Value,
    schema: &Value,
    target: &str,
    issues: &mut Vec<ContractIssue>,
) {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        let ok = match expected_type {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "number" => value.as_f64().is_some(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => true,
        };
        if !ok {
            issues.push(ContractIssue {
                severity: Severity::Error,
                target: target.to_string(),
                message: format!("expected `{expected_type}`, got `{}`", json_type(value)),
            });
            return;
        }
    }

    if schema.get("type").and_then(Value::as_str) == Some("object") {
        let Some(obj) = value.as_object() else {
            return;
        };
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !obj.contains_key(key) {
                    issues.push(ContractIssue {
                        severity: Severity::Error,
                        target: format!("{target}.{key}"),
                        message: "missing required property".to_string(),
                    });
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (key, prop_schema) in properties {
                if let Some(child) = obj.get(key) {
                    validate_value_against_schema(
                        child,
                        prop_schema,
                        &format!("{target}.{key}"),
                        issues,
                    );
                }
            }
        }
    }
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.as_i64().is_some() || number.as_u64().is_some() => {
            "integer"
        }
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_bad_tool_argument_type() {
        let mut issues = Vec::new();
        let schema =
            json!({"type":"object","properties":{"limit":{"type":"integer"}},"required":["limit"]});
        validate_value_against_schema(
            &json!({"limit":"10"}),
            &schema,
            "tool.search.arguments",
            &mut issues,
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.message.contains("expected `integer`"))
        );
    }
}
