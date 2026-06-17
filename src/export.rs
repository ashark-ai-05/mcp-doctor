use crate::trace::{TraceEvent, read_trace};
use anyhow::Context;
use std::fmt::Write as _;
use std::path::Path;

pub fn export_repro(trace_path: impl AsRef<Path>, output: impl AsRef<Path>) -> anyhow::Result<()> {
    let events = read_trace(&trace_path)?;
    let command = events.iter().find_map(|event| match event {
        TraceEvent::SessionStart { command, .. } => Some(command.clone()),
        _ => None,
    });
    let mut out = String::new();
    writeln!(&mut out, "#!/usr/bin/env bash")?;
    writeln!(&mut out, "set -euo pipefail")?;
    writeln!(&mut out)?;
    writeln!(&mut out, "TRACE=${{1:-{}}}", trace_path.as_ref().display())?;
    if let Some(command) = command {
        writeln!(&mut out, "# Original server command captured in trace:")?;
        writeln!(&mut out, "# {}", shell_join(&command))?;
        writeln!(
            &mut out,
            "cargo run -- replay \"$TRACE\" -- {}",
            shell_join(&command)
        )?;
    } else {
        writeln!(
            &mut out,
            "echo 'No session_start command found in trace.' >&2"
        )?;
        writeln!(
            &mut out,
            "echo 'Run: mcp-doctor replay \"$TRACE\" -- <server command>' >&2"
        )?;
        writeln!(&mut out, "exit 2")?;
    }
    std::fs::write(output.as_ref(), out)
        .with_context(|| format!("write repro script {}", output.as_ref().display()))?;
    Ok(())
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ")
}
