use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoctorError {
    #[error("server command is empty")]
    EmptyCommand,
    #[error("server process stdin/stdout unavailable")]
    MissingPipe,
    #[error("request timed out waiting for response to id {id}")]
    Timeout { id: u64 },
    #[error("server exited before response to id {id}; stderr: {stderr}")]
    ServerExited { id: u64, stderr: String },
    #[error("invalid JSON from server: {line}")]
    InvalidJson { line: String },
    #[error("JSON-RPC response id mismatch: expected {expected}, got {got}")]
    ResponseIdMismatch { expected: u64, got: String },
    #[error("server returned JSON-RPC error for {method}: {message}")]
    RpcError { method: String, message: String },
    #[error("trace validation failed with {0} error(s)")]
    TraceInvalid(usize),
}
