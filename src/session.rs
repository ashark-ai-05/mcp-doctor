use crate::error::DoctorError;
use crate::protocol;
use crate::trace::{TraceEvent, TraceWriter, now_ms};
use anyhow::Context;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub trace_path: Option<std::path::PathBuf>,
    pub tools_count: usize,
    pub status: String,
}

pub struct StdioSession {
    child: Child,
    stdin: ChildStdin,
    stdout_rx: Receiver<OutputLine>,
    stderr_rx: Receiver<String>,
    timeout: Duration,
    trace: Option<TraceWriter>,
    stderr_tail: Vec<String>,
}

#[derive(Debug)]
struct OutputLine {
    line: String,
}

impl StdioSession {
    pub fn spawn(
        server: &[String],
        timeout: Duration,
        mut trace: Option<TraceWriter>,
    ) -> anyhow::Result<Self> {
        if server.is_empty() {
            return Err(DoctorError::EmptyCommand.into());
        }
        let mut cmd = Command::new(&server[0]);
        cmd.args(&server[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawn MCP server command: {}", server.join(" ")))?;
        let pid = child.id();
        let stdin = child.stdin.take().ok_or(DoctorError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(DoctorError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(DoctorError::MissingPipe)?;
        let (stdout_tx, stdout_rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = stdout_tx.send(OutputLine { line });
            }
        });
        let (stderr_tx, stderr_rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let _ = stderr_tx.send(line);
            }
        });
        if let Some(writer) = trace.as_mut() {
            writer.write(&TraceEvent::ProcessStart {
                ts: now_ms(),
                pid: Some(pid),
            })?;
        }
        Ok(Self {
            child,
            stdin,
            stdout_rx,
            stderr_rx,
            timeout,
            trace,
            stderr_tail: Vec::new(),
        })
    }

    pub fn trace_path(&self) -> Option<std::path::PathBuf> {
        self.trace.as_ref().map(|trace| trace.path().to_path_buf())
    }

    pub fn initialize_and_list_tools(&mut self) -> anyhow::Result<Value> {
        let (_, initialize) = protocol::initialize_request();
        self.request(&initialize, "initialize")?;
        self.notify(&protocol::initialized_notification())?;
        let (_, tools_list) = protocol::tools_list_request();
        self.request(&tools_list, "tools/list")
    }

    pub fn call_tool(&mut self, tool: &str, args: Value) -> anyhow::Result<Value> {
        let (_, request) = protocol::tools_call_request(tool, args);
        self.request(&request, "tools/call")
    }

    pub fn request(&mut self, payload: &Value, method_name: &str) -> anyhow::Result<Value> {
        self.drain_stderr()?;
        let id = payload
            .get("id")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        if let Some(trace) = self.trace.as_mut() {
            trace.write(&TraceEvent::RpcRequest {
                ts: now_ms(),
                id: Some(id),
                method: method_name.to_string(),
                payload: payload.clone(),
            })?;
        }
        let start = Instant::now();
        self.write_json(payload)?;
        self.drain_stderr()?;
        if let Some(_status) = self.child.try_wait()? {
            return Err(DoctorError::ServerExited {
                id,
                stderr: self.stderr_tail.join("\n"),
            }
            .into());
        }
        let remaining = self
            .timeout
            .checked_sub(start.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0));
        if remaining.is_zero() {
            return Err(DoctorError::Timeout { id }.into());
        }
        let line = self
            .stdout_rx
            .recv_timeout(remaining)
            .map_err(|_| DoctorError::Timeout { id })?;
        match serde_json::from_str::<Value>(&line.line) {
            Ok(response) => {
                let response_id = response.get("id").and_then(Value::as_u64);
                if response_id != Some(id) {
                    let got = protocol::id_string(&response);
                    if let Some(trace) = self.trace.as_mut() {
                        trace.write(&TraceEvent::Validation {
                            ts: now_ms(),
                            severity: crate::trace::Severity::Error,
                            target: format!("id={id}"),
                            message: format!("response id mismatch: got {got}"),
                        })?;
                    }
                    return Err(DoctorError::ResponseIdMismatch { expected: id, got }.into());
                }
                let duration_ms = start.elapsed().as_millis() as u64;
                if let Some(trace) = self.trace.as_mut() {
                    trace.write(&TraceEvent::RpcResponse {
                        ts: now_ms(),
                        id: Some(id),
                        duration_ms,
                        payload: response.clone(),
                    })?;
                }
                if let Some(message) = protocol::response_error_message(&response) {
                    return Err(DoctorError::RpcError {
                        method: method_name.to_string(),
                        message,
                    }
                    .into());
                }
                if !protocol::is_success_response(&response)
                    && let Some(trace) = self.trace.as_mut()
                {
                    trace.write(&TraceEvent::Validation {
                        ts: now_ms(),
                        severity: crate::trace::Severity::Error,
                        target: format!("id={id}"),
                        message: "response missing jsonrpc/id or contains malformed success shape"
                            .to_string(),
                    })?;
                }
                Ok(response)
            }
            Err(err) => {
                if let Some(trace) = self.trace.as_mut() {
                    trace.write(&TraceEvent::InvalidOutput {
                        ts: now_ms(),
                        line: line.line.clone(),
                        reason: err.to_string(),
                    })?;
                }
                Err(DoctorError::InvalidJson { line: line.line }.into())
            }
        }
    }

    pub fn notify(&mut self, payload: &Value) -> anyhow::Result<()> {
        if let Some(trace) = self.trace.as_mut() {
            trace.write(&TraceEvent::RpcRequest {
                ts: now_ms(),
                id: None,
                method: protocol::method(payload)
                    .unwrap_or("notification")
                    .to_string(),
                payload: payload.clone(),
            })?;
        }
        self.write_json(payload)
    }

    pub fn finish(mut self, status: &str) -> anyhow::Result<SessionSummary> {
        self.drain_stderr()?;
        let _ = self.child.kill();
        let _ = self.child.wait();
        let trace_path = self.trace_path();
        if let Some(trace) = self.trace.as_mut() {
            trace.write(&TraceEvent::SessionEnd {
                ts: now_ms(),
                status: status.to_string(),
            })?;
        }
        Ok(SessionSummary {
            trace_path,
            tools_count: 0,
            status: status.to_string(),
        })
    }

    fn write_json(&mut self, payload: &Value) -> anyhow::Result<()> {
        serde_json::to_writer(&mut self.stdin, payload)?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn drain_stderr(&mut self) -> anyhow::Result<()> {
        while let Ok(line) = self.stderr_rx.try_recv() {
            if self.stderr_tail.len() >= 20 {
                self.stderr_tail.remove(0);
            }
            self.stderr_tail.push(line.clone());
            if let Some(trace) = self.trace.as_mut() {
                trace.write(&TraceEvent::Stderr { ts: now_ms(), line })?;
            }
        }
        Ok(())
    }
}

pub fn count_tools(tools_list_response: &Value) -> usize {
    tools_list_response
        .get("result")
        .and_then(|result| result.get("tools"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}
