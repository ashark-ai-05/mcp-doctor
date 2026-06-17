use crate::contract::ToolCatalog;
use crate::trace::TraceEvent;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct DiffReport {
    pub changes: Vec<String>,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

pub fn diff_events(old: &[TraceEvent], new: &[TraceEvent]) -> DiffReport {
    let old_catalog = ToolCatalog::from_events(old);
    let new_catalog = ToolCatalog::from_events(new);
    let mut changes = Vec::new();
    let old_names: BTreeSet<_> = old_catalog.tools.keys().cloned().collect();
    let new_names: BTreeSet<_> = new_catalog.tools.keys().cloned().collect();

    for removed in old_names.difference(&new_names) {
        changes.push(format!("removed tool `{removed}`"));
    }
    for added in new_names.difference(&old_names) {
        changes.push(format!("added tool `{added}`"));
    }
    for shared in old_names.intersection(&new_names) {
        let old_schema = old_catalog.input_schema(shared);
        let new_schema = new_catalog.input_schema(shared);
        if old_schema != new_schema {
            changes.push(format!("changed input schema for tool `{shared}`"));
            for required in added_required(old_schema, new_schema) {
                changes.push(format!(
                    "tool `{shared}` added required argument `{required}`"
                ));
            }
        }
    }

    DiffReport { changes }
}

fn added_required(old_schema: Option<&Value>, new_schema: Option<&Value>) -> Vec<String> {
    let old_required = required_set(old_schema);
    let new_required = required_set(new_schema);
    new_required.difference(&old_required).cloned().collect()
}

fn required_set(schema: Option<&Value>) -> BTreeSet<String> {
    schema
        .and_then(|schema| schema.get("required"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tools_event(tools: Value) -> TraceEvent {
        TraceEvent::RpcResponse {
            ts: 1,
            id: Some(1),
            duration_ms: 1,
            payload: json!({"jsonrpc":"2.0","id":1,"result":{"tools":tools}}),
        }
    }

    #[test]
    fn detects_added_required_argument() {
        let old = vec![tools_event(
            json!([{"name":"echo","inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":[]}}]),
        )];
        let new = vec![tools_event(
            json!([{"name":"echo","inputSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}}]),
        )];
        let diff = diff_events(&old, &new);
        assert!(
            diff.changes
                .iter()
                .any(|change| change.contains("added required argument `text`"))
        );
    }
}
