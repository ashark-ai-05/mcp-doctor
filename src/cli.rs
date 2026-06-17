use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(name = "mcp-doctor")]
#[command(
    version,
    about = "Record, replay, and debug MCP JSON-RPC from your terminal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Request timeout in milliseconds.
    #[arg(long, global = true, default_value_t = 5000)]
    pub timeout_ms: u64,
}

impl Cli {
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Verify a stdio MCP server can initialize and list tools.
    Smoke(StdioCommand),
    /// Record initialize + tools/list into a JSONL trace.
    Record(StdioCommand),
    /// Call one MCP tool and record the request/response.
    Call(CallCommand),
    /// Replay recorded tools/call requests against a server.
    Replay(ReplayCommand),
    /// Validate a saved trace without running a server.
    Validate(TracePathCommand),
    /// Export a markdown diagnostic report from a trace.
    Report(ReportCommand),
}

#[derive(Debug, Args)]
pub struct StdioCommand {
    #[command(subcommand)]
    pub transport: StdioTransport,
}

#[derive(Debug, Subcommand)]
pub enum StdioTransport {
    /// Launch a server over stdio. Put server command after `--`.
    Stdio(ServerCommand),
}

#[derive(Debug, Args)]
pub struct ServerCommand {
    /// Server command and args, after `--`.
    #[arg(last = true, required = true)]
    pub server: Vec<String>,
}

#[derive(Debug, Args)]
pub struct CallCommand {
    #[command(subcommand)]
    pub transport: CallTransport,
}

#[derive(Debug, Subcommand)]
pub enum CallTransport {
    /// Launch a server over stdio and call a tool.
    Stdio(CallStdioCommand),
}

#[derive(Debug, Args)]
pub struct CallStdioCommand {
    /// Tool name to call.
    #[arg(long)]
    pub tool: String,

    /// JSON object arguments for the tool.
    #[arg(long, default_value = "{}")]
    pub args: String,

    /// Server command and args, after `--`.
    #[arg(last = true, required = true)]
    pub server: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ReplayCommand {
    /// Trace JSONL file to replay.
    pub trace: PathBuf,

    /// Server command and args, after `--`.
    #[arg(last = true, required = true)]
    pub server: Vec<String>,
}

#[derive(Debug, Args)]
pub struct TracePathCommand {
    /// Trace JSONL file.
    pub trace: PathBuf,
}

#[derive(Debug, Args)]
pub struct ReportCommand {
    /// Trace JSONL file.
    pub trace: PathBuf,

    /// Markdown output path.
    #[arg(long)]
    pub output: PathBuf,
}
