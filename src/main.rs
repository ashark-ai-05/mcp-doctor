use anyhow::Context;
use clap::Parser;
use mcp_doctor::cli::{
    CallTransport, Cli, Command as CliCommand, ConnectTransport, StdioTransport,
};
use mcp_doctor::diff::diff_events;
use mcp_doctor::export;
use mcp_doctor::http;
use mcp_doctor::replay::replay_trace;
use mcp_doctor::report;
use mcp_doctor::session::{StdioSession, count_tools};
use mcp_doctor::trace::{TraceEvent, TraceWriter, default_trace_path, new_session_id, now_ms};
use mcp_doctor::tui;
use mcp_doctor::validate::validate_events;
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        CliCommand::Smoke(cmd) => match &cmd.transport {
            StdioTransport::Stdio(stdio) => smoke(&stdio.server, cli.timeout()),
        },
        CliCommand::Record(cmd) => match &cmd.transport {
            StdioTransport::Stdio(stdio) => record(&stdio.server, cli.timeout()),
        },
        CliCommand::Call(cmd) => match &cmd.transport {
            CallTransport::Stdio(stdio) => {
                call_tool(&stdio.server, &stdio.tool, &stdio.args, cli.timeout())
            }
        },
        CliCommand::Replay(cmd) => {
            let result = replay_trace(&cmd.trace, &cmd.server, cli.timeout(), &cmd.allow_fields)?;
            println!(
                "replay: {} calls, {} matched, {} mismatched",
                result.replayed,
                result.matched,
                result.mismatched.len()
            );
            for mismatch in result.mismatched {
                println!("mismatch: {mismatch}");
            }
            Ok(())
        }
        CliCommand::Diff(cmd) => {
            let old = mcp_doctor::trace::read_trace(&cmd.old)?;
            let new = mcp_doctor::trace::read_trace(&cmd.new)?;
            let diff = diff_events(&old, &new);
            if diff.is_empty() {
                println!("no tool/schema differences detected");
            } else {
                for change in diff.changes {
                    println!("change: {change}");
                }
            }
            Ok(())
        }
        CliCommand::Validate(cmd) => {
            let events = mcp_doctor::trace::read_trace(&cmd.trace)?;
            let issues = validate_events(&events);
            if issues.is_empty() {
                println!("trace valid: {} events", events.len());
                Ok(())
            } else {
                for issue in &issues {
                    println!("{:?}: {} — {}", issue.severity, issue.target, issue.message);
                }
                let errors = issues
                    .iter()
                    .filter(|issue| issue.severity == mcp_doctor::trace::Severity::Error)
                    .count();
                if errors > 0 {
                    anyhow::bail!(mcp_doctor::error::DoctorError::TraceInvalid(errors));
                }
                Ok(())
            }
        }
        CliCommand::Report(cmd) => {
            report::write_markdown(&cmd.trace, &cmd.output)?;
            println!("wrote report: {}", cmd.output.display());
            Ok(())
        }
        CliCommand::Tui(cmd) => {
            print!("{}", tui::render_trace_view(&cmd.trace)?);
            Ok(())
        }
        CliCommand::ExportRepro(cmd) => {
            export::export_repro(&cmd.trace, &cmd.output)?;
            println!("wrote repro script: {}", cmd.output.display());
            Ok(())
        }
        CliCommand::Connect(cmd) => match &cmd.transport {
            ConnectTransport::Http(http_cmd) => {
                let result = http::connect_http(&http_cmd.url, cli.timeout())?;
                println!(
                    "initialize: {}",
                    serde_json::to_string_pretty(&result.initialize)?
                );
                println!(
                    "tools/list: {}",
                    serde_json::to_string_pretty(&result.tools)?
                );
                Ok(())
            }
        },
    }
}

fn smoke(server: &[String], timeout: std::time::Duration) -> anyhow::Result<()> {
    let mut session = StdioSession::spawn(server, timeout, None)?;
    let tools = session.initialize_and_list_tools()?;
    let tools_count = count_tools(&tools);
    session.finish("ok")?;
    println!("ok: initialized server and discovered {tools_count} tool(s)");
    Ok(())
}

fn record(server: &[String], timeout: std::time::Duration) -> anyhow::Result<()> {
    let session_id = new_session_id();
    let trace_path = default_trace_path(&session_id);
    let mut writer = TraceWriter::new(&trace_path)?;
    writer.write(&TraceEvent::SessionStart {
        session_id: session_id.clone(),
        ts: now_ms(),
        command: server.to_vec(),
    })?;
    let mut session = StdioSession::spawn(server, timeout, Some(writer))?;
    let tools = session.initialize_and_list_tools()?;
    let tools_count = count_tools(&tools);
    let summary = session.finish("ok")?;
    let trace_path = summary.trace_path.context("trace path unavailable")?;
    let report_path = trace_path.with_file_name("report.md");
    report::write_markdown(&trace_path, &report_path)?;
    println!("ok: recorded {tools_count} tool(s)");
    println!("trace: {}", trace_path.display());
    println!("report: {}", report_path.display());
    Ok(())
}

fn call_tool(
    server: &[String],
    tool: &str,
    args: &str,
    timeout: std::time::Duration,
) -> anyhow::Result<()> {
    let parsed_args: Value =
        serde_json::from_str(args).with_context(|| format!("parse --args JSON: {args}"))?;
    if !parsed_args.is_object() {
        anyhow::bail!("--args must be a JSON object");
    }
    let session_id = new_session_id();
    let trace_path = default_trace_path(&session_id);
    let mut writer = TraceWriter::new(&trace_path)?;
    writer.write(&TraceEvent::SessionStart {
        session_id,
        ts: now_ms(),
        command: server.to_vec(),
    })?;
    let mut session = StdioSession::spawn(server, timeout, Some(writer))?;
    session.initialize_and_list_tools()?;
    let response = session.call_tool(tool, parsed_args)?;
    let summary = session.finish("ok")?;
    let trace_path = summary.trace_path.context("trace path unavailable")?;
    let report_path = trace_path.with_file_name("report.md");
    report::write_markdown(&trace_path, &report_path)?;
    println!("ok: called tool `{tool}`");
    println!("response: {}", serde_json::to_string_pretty(&response)?);
    println!("trace: {}", trace_path.display());
    println!("report: {}", report_path.display());
    Ok(())
}
