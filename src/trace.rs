use crate::redaction::{redact_text, redact_value};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEvent {
    SessionStart {
        session_id: String,
        ts: u64,
        command: Vec<String>,
    },
    ProcessStart {
        ts: u64,
        pid: Option<u32>,
    },
    RpcRequest {
        ts: u64,
        id: Option<u64>,
        method: String,
        payload: Value,
    },
    RpcResponse {
        ts: u64,
        id: Option<u64>,
        duration_ms: u64,
        payload: Value,
    },
    Stderr {
        ts: u64,
        line: String,
    },
    InvalidOutput {
        ts: u64,
        line: String,
        reason: String,
    },
    Validation {
        ts: u64,
        severity: Severity,
        target: String,
        message: String,
    },
    SessionEnd {
        ts: u64,
        status: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub struct TraceWriter {
    path: PathBuf,
    file: File,
}

impl TraceWriter {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("open trace {}", path.display()))?;
        Ok(Self { path, file })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&mut self, event: &TraceEvent) -> anyhow::Result<()> {
        let redacted = event.redacted();
        serde_json::to_writer(&mut self.file, &redacted)?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        Ok(())
    }
}

impl TraceEvent {
    pub fn redacted(&self) -> Self {
        match self {
            TraceEvent::RpcRequest {
                ts,
                id,
                method,
                payload,
            } => TraceEvent::RpcRequest {
                ts: *ts,
                id: *id,
                method: method.clone(),
                payload: redact_value(payload),
            },
            TraceEvent::RpcResponse {
                ts,
                id,
                duration_ms,
                payload,
            } => TraceEvent::RpcResponse {
                ts: *ts,
                id: *id,
                duration_ms: *duration_ms,
                payload: redact_value(payload),
            },
            TraceEvent::Stderr { ts, line } => TraceEvent::Stderr {
                ts: *ts,
                line: redact_text(line),
            },
            TraceEvent::InvalidOutput { ts, line, reason } => TraceEvent::InvalidOutput {
                ts: *ts,
                line: truncate(&redact_text(line), 4096),
                reason: reason.clone(),
            },
            other => other.clone(),
        }
    }
}

pub fn read_trace(path: impl AsRef<Path>) -> anyhow::Result<Vec<TraceEvent>> {
    let file =
        File::open(path.as_ref()).with_context(|| format!("open {}", path.as_ref().display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read trace line {}", idx + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let event: TraceEvent = serde_json::from_str(&line)
            .with_context(|| format!("parse trace line {}: {}", idx + 1, truncate(&line, 200)))?;
        events.push(event);
    }
    Ok(events)
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn new_session_id() -> String {
    format!("{}-{}", now_ms(), std::process::id())
}

pub fn default_trace_path(session_id: &str) -> PathBuf {
    PathBuf::from(".mcpdoctor")
        .join("sessions")
        .join(session_id)
        .join("trace.jsonl")
}

pub fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        input.to_string()
    } else {
        format!("{}…[truncated {} bytes]", &input[..max], input.len() - max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_trace_payloads() {
        let event = TraceEvent::RpcRequest {
            ts: 1,
            id: Some(1),
            method: "tools/call".to_string(),
            payload: json!({"params":{"api_token":"secret","safe":"ok"}}),
        };
        let redacted = event.redacted();
        if let TraceEvent::RpcRequest { payload, .. } = redacted {
            assert_eq!(payload["params"]["api_token"], "[REDACTED]");
            assert_eq!(payload["params"]["safe"], "ok");
        } else {
            panic!("wrong event");
        }
    }
}
