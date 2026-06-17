use serde_json::Value;

const SECRET_MARKERS: &[&str] = &[
    "token",
    "secret",
    "password",
    "passwd",
    "api_key",
    "apikey",
    "authorization",
    "auth",
    "credential",
    "private_key",
];

pub fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, val) in map {
                if is_secret_key(key) {
                    out.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                } else {
                    out.insert(key.clone(), redact_value(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(values) => Value::Array(values.iter().map(redact_value).collect()),
        Value::String(text) => Value::String(redact_text(text)),
        other => other.clone(),
    }
}

pub fn redact_text(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let contains_secret_assignment = SECRET_MARKERS
        .iter()
        .any(|marker| lower.contains(marker) && text.contains('='));
    let contains_bearer = lower
        .split_whitespace()
        .any(|part| part.starts_with("bearer"));
    if !contains_secret_assignment && !contains_bearer {
        return text.to_string();
    }

    let mut out = Vec::new();
    for part in text.split_whitespace() {
        let lower = part.to_ascii_lowercase();
        if SECRET_MARKERS.iter().any(|marker| lower.contains(marker)) && part.contains('=') {
            let key = part.split('=').next().unwrap_or(part);
            out.push(format!("{key}=[REDACTED]"));
        } else if lower.starts_with("bearer") {
            out.push("Bearer [REDACTED]".to_string());
        } else {
            out.push(part.to_string());
        }
    }
    out.join(" ")
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SECRET_MARKERS.iter().any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_nested_secret_keys() {
        let input = json!({"headers":{"Authorization":"Bearer abc"},"safe":"ok","api_key":"x"});
        let output = redact_value(&input);
        assert_eq!(output["headers"]["Authorization"], "[REDACTED]");
        assert_eq!(output["api_key"], "[REDACTED]");
        assert_eq!(output["safe"], "ok");
    }

    #[test]
    fn preserves_non_secret_string_formatting() {
        let text = "{\n  \"thoughtNumber\": 1,\n  \"nextThoughtNeeded\": false\n}";
        assert_eq!(redact_text(text), text);
    }
}
