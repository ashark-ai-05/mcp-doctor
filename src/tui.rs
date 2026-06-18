use crate::contract::{ContractIssue, validate_contract};
use crate::trace::{Severity, TraceEvent, read_trace, truncate};
use crate::validate::{ValidationIssue, validate_events};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use serde_json::Value;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
struct TraceModel {
    trace_path: PathBuf,
    events: Vec<TraceEvent>,
    rows: Vec<EventRow>,
    findings: Vec<FindingRow>,
    errors: usize,
    warnings: usize,
}

#[derive(Debug, Clone)]
struct EventRow {
    marker: &'static str,
    label: String,
    detail: String,
    status: RowStatus,
    payload: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowStatus {
    Ok,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
struct FindingRow {
    severity: Severity,
    target: String,
    message: String,
}

#[derive(Debug)]
struct TuiApp {
    model: TraceModel,
    selected: usize,
    show_help: bool,
}

impl TuiApp {
    fn new(model: TraceModel) -> Self {
        Self {
            model,
            selected: 0,
            show_help: false,
        }
    }

    fn next(&mut self) {
        if !self.model.rows.is_empty() {
            self.selected = (self.selected + 1).min(self.model.rows.len() - 1);
        }
    }

    fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn selected_row(&self) -> Option<&EventRow> {
        self.model.rows.get(self.selected)
    }
}

pub fn run_trace_view(trace_path: impl AsRef<Path>) -> anyhow::Result<()> {
    if io::stdout().is_terminal() {
        run_interactive_trace_view(trace_path)
    } else {
        print!("{}", render_trace_view(trace_path)?);
        Ok(())
    }
}

pub fn render_trace_view(trace_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let model = load_model(trace_path)?;
    let mut out = String::new();
    out.push_str("╭─ MCP Doctor Trace Viewer / Browser ────────────────────────────────────╮\n");
    out.push_str(&format!(
        "│ Trace: {:<63} │\n",
        truncate(&model.trace_path.display().to_string(), 63)
    ));
    out.push_str(&format!(
        "│ Events: {:<5} Errors: {:<3} Warnings: {:<3} Findings: {:<18} │\n",
        model.events.len(),
        model.errors,
        model.warnings,
        model.findings.len()
    ));
    out.push_str("╰────────────────────────────────────────────────────────────────────────╯\n\n");
    out.push_str("Timeline\n");
    out.push_str("────────\n");
    for (idx, row) in model.rows.iter().enumerate() {
        out.push_str(&format!(
            "{:>2}. {} {:<11} {}\n",
            idx + 1,
            row.marker,
            status_label(row.status),
            row.label
        ));
        if !row.detail.is_empty() {
            out.push_str(&format!("    {}\n", row.detail));
        }
    }
    out.push_str("\nFindings\n");
    out.push_str("────────\n");
    if model.findings.is_empty() {
        out.push_str("✓ no protocol or contract findings\n");
    } else {
        for finding in &model.findings {
            out.push_str(&format!(
                "{} {:?} {} — {}\n",
                severity_marker(finding.severity),
                finding.severity,
                finding.target,
                finding.message
            ));
        }
    }
    out.push_str("\nPayload preview\n");
    out.push_str("───────────────\n");
    if let Some(row) = model.rows.iter().find(|row| row.payload.is_some()) {
        out.push_str(&format!("{}\n", row.label));
        let pretty = serde_json::to_string_pretty(row.payload.as_ref().expect("payload checked"))?;
        out.push_str(&truncate(&pretty, 1600));
        out.push('\n');
    } else {
        out.push_str("no JSON-RPC payloads in trace\n");
    }
    out.push_str("\nInteractive keys when run in a terminal: ↑/↓ move · h help · q quit\n");
    Ok(out)
}

fn run_interactive_trace_view(trace_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let model = load_model(trace_path)?;
    let mut app = TuiApp::new(model);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if event::poll(Duration::from_millis(200))? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Char('h') | KeyCode::Char('?') => app.show_help = !app.show_help,
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &mut TuiApp) {
    let area = frame.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(6),
        ])
        .split(area);

    draw_header(frame, app, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root[1]);
    draw_timeline(frame, app, body[0]);
    draw_payload(frame, app, body[1]);
    draw_findings(frame, app, root[2]);

    if app.show_help {
        draw_help(frame, centered_rect(64, 35, area));
    }
}

fn draw_header(frame: &mut ratatui::Frame<'_>, app: &TuiApp, area: Rect) {
    let model = &app.model;
    let title = Line::from(vec![
        Span::styled(
            " MCP Doctor ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" trace browser  "),
        Span::styled(
            format!("{} events", model.events.len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} errors", model.errors),
            Style::default().fg(if model.errors > 0 {
                Color::Red
            } else {
                Color::Green
            }),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} warnings", model.warnings),
            Style::default().fg(if model.warnings > 0 {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
        Span::raw("  q quit · ↑/↓ move · h help"),
    ]);
    let block = Paragraph::new(title).block(
        Block::default()
            .borders(Borders::ALL)
            .title(truncate(&model.trace_path.display().to_string(), 70)),
    );
    frame.render_widget(block, area);
}

fn draw_timeline(frame: &mut ratatui::Frame<'_>, app: &mut TuiApp, area: Rect) {
    let items: Vec<ListItem> = app
        .model
        .rows
        .iter()
        .enumerate()
        .map(|(idx, row)| {
            let style = style_for(row.status);
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>2} ", idx + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(row.marker, style.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(status_label(row.status), style),
                Span::raw(" "),
                Span::raw(row.label.clone()),
            ]);
            let detail = Line::from(Span::styled(
                format!("   {}", truncate(&row.detail, 82)),
                Style::default().fg(Color::DarkGray),
            ));
            ListItem::new(vec![line, detail])
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.selected));
    let list = List::new(items)
        .block(Block::default().title("Timeline").borders(Borders::ALL))
        .highlight_symbol("▶ ")
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_payload(frame: &mut ratatui::Frame<'_>, app: &TuiApp, area: Rect) {
    let Some(row) = app.selected_row() else {
        frame.render_widget(
            Paragraph::new("No events")
                .block(Block::default().title("Payload").borders(Borders::ALL)),
            area,
        );
        return;
    };
    let content = if let Some(payload) = &row.payload {
        serde_json::to_string_pretty(payload)
            .unwrap_or_else(|err| format!("payload render error: {err}"))
    } else {
        row.detail.clone()
    };
    let text = Text::from(content);
    let title = format!("Payload · {}", truncate(&row.label, 52));
    let paragraph = Paragraph::new(text)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_findings(frame: &mut ratatui::Frame<'_>, app: &TuiApp, area: Rect) {
    let lines: Vec<Line> = if app.model.findings.is_empty() {
        vec![Line::from(vec![Span::styled(
            "✓ no protocol or contract findings",
            Style::default().fg(Color::Green),
        )])]
    } else {
        app.model
            .findings
            .iter()
            .take(6)
            .map(|finding| {
                Line::from(vec![
                    Span::styled(
                        severity_marker(finding.severity),
                        style_for(severity_status(finding.severity)).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:?}", finding.severity),
                        style_for(severity_status(finding.severity)),
                    ),
                    Span::raw(format!(" {} — {}", finding.target, finding.message)),
                ])
            })
            .collect()
    };
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Findings").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut ratatui::Frame<'_>, area: Rect) {
    let help = Paragraph::new(vec![
        Line::from(Span::styled("MCP Doctor keys", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("↑ / k       previous event"),
        Line::from("↓ / j       next event"),
        Line::from("h / ?       toggle this help"),
        Line::from("q / Esc     quit"),
        Line::from(""),
        Line::from("Use the timeline to move through requests, responses, stderr, validation rows, and session status. The right pane shows the selected payload or event detail."),
    ])
    .block(Block::default().title("Help").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn load_model(trace_path: impl AsRef<Path>) -> anyhow::Result<TraceModel> {
    let trace_path = trace_path.as_ref().to_path_buf();
    let events = read_trace(&trace_path)?;
    let protocol_issues = validate_events(&events);
    let contract_issues = validate_contract(&events);
    let rows = events
        .iter()
        .map(event_row)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let findings = findings(protocol_issues, contract_issues);
    let errors = rows
        .iter()
        .filter(|row| row.status == RowStatus::Error)
        .count()
        + findings
            .iter()
            .filter(|row| row.severity == Severity::Error)
            .count();
    let warnings = rows
        .iter()
        .filter(|row| row.status == RowStatus::Warning)
        .count()
        + findings
            .iter()
            .filter(|row| row.severity == Severity::Warning)
            .count();
    Ok(TraceModel {
        trace_path,
        events,
        rows,
        findings,
        errors,
        warnings,
    })
}

fn event_row(event: &TraceEvent) -> anyhow::Result<EventRow> {
    Ok(match event {
        TraceEvent::SessionStart {
            session_id,
            command,
            ..
        } => EventRow {
            marker: "◉",
            label: format!("session start {session_id}"),
            detail: format!("cmd: {}", command.join(" ")),
            status: RowStatus::Info,
            payload: None,
        },
        TraceEvent::ProcessStart { pid, .. } => EventRow {
            marker: "●",
            label: "process start".to_string(),
            detail: format!("pid={pid:?}"),
            status: RowStatus::Info,
            payload: None,
        },
        TraceEvent::RpcRequest {
            id,
            method,
            payload,
            ..
        } => EventRow {
            marker: "→",
            label: format!("request id={id:?} {method}"),
            detail: payload_summary(payload),
            status: RowStatus::Info,
            payload: Some(payload.clone()),
        },
        TraceEvent::RpcResponse {
            id,
            duration_ms,
            payload,
            ..
        } => {
            let has_error = payload.get("error").is_some();
            EventRow {
                marker: if has_error { "✗" } else { "←" },
                label: format!("response id={id:?} {}ms", duration_ms),
                detail: payload_summary(payload),
                status: if has_error {
                    RowStatus::Error
                } else {
                    RowStatus::Ok
                },
                payload: Some(payload.clone()),
            }
        }
        TraceEvent::Stderr { line, .. } => EventRow {
            marker: "!",
            label: "stderr".to_string(),
            detail: line.clone(),
            status: RowStatus::Warning,
            payload: None,
        },
        TraceEvent::InvalidOutput { reason, line, .. } => EventRow {
            marker: "✗",
            label: format!("invalid output: {reason}"),
            detail: line.clone(),
            status: RowStatus::Error,
            payload: None,
        },
        TraceEvent::Validation {
            severity,
            target,
            message,
            ..
        } => EventRow {
            marker: severity_marker(*severity),
            label: format!("validation {target}"),
            detail: message.clone(),
            status: severity_status(*severity),
            payload: None,
        },
        TraceEvent::SessionEnd { status, .. } => EventRow {
            marker: if status == "ok" { "✓" } else { "✗" },
            label: format!("session end {status}"),
            detail: String::new(),
            status: if status == "ok" {
                RowStatus::Ok
            } else {
                RowStatus::Error
            },
            payload: None,
        },
    })
}

fn findings(
    protocol_issues: Vec<ValidationIssue>,
    contract_issues: Vec<ContractIssue>,
) -> Vec<FindingRow> {
    let mut findings = Vec::new();
    for issue in protocol_issues {
        findings.push(FindingRow {
            severity: issue.severity,
            target: issue.target,
            message: issue.message,
        });
    }
    for issue in contract_issues {
        findings.push(FindingRow {
            severity: issue.severity,
            target: issue.target,
            message: issue.message,
        });
    }
    findings
}

fn payload_summary(payload: &Value) -> String {
    if let Some(method) = payload.get("method").and_then(Value::as_str) {
        return format!("method={method}");
    }
    if let Some(error) = payload.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("<error without message>");
        return format!("error={}", truncate(message, 120));
    }
    if let Some(result) = payload.get("result") {
        if let Some(tools) = result.get("tools").and_then(Value::as_array) {
            return format!("result: {} tool(s)", tools.len());
        }
        if let Some(content) = result.get("content").and_then(Value::as_array) {
            return format!("result: {} content block(s)", content.len());
        }
        return "result object".to_string();
    }
    truncate(&payload.to_string(), 120)
}

fn style_for(status: RowStatus) -> Style {
    match status {
        RowStatus::Ok => Style::default().fg(Color::Green),
        RowStatus::Error => Style::default().fg(Color::Red),
        RowStatus::Warning => Style::default().fg(Color::Yellow),
        RowStatus::Info => Style::default().fg(Color::Cyan),
    }
}

fn status_label(status: RowStatus) -> &'static str {
    match status {
        RowStatus::Ok => "ok",
        RowStatus::Error => "error",
        RowStatus::Warning => "warning",
        RowStatus::Info => "info",
    }
}

fn severity_status(severity: Severity) -> RowStatus {
    match severity {
        Severity::Info => RowStatus::Info,
        Severity::Warning => RowStatus::Warning,
        Severity::Error => RowStatus::Error,
    }
}

fn severity_marker(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "i",
        Severity::Warning => "!",
        Severity::Error => "✗",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::TraceWriter;
    use std::fs;

    #[test]
    fn snapshot_view_has_polished_sections() {
        let dir = std::env::temp_dir().join(format!("mcp-doctor-tui-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let trace = dir.join("trace.jsonl");
        let mut writer = TraceWriter::new(&trace).unwrap();
        writer
            .write(&TraceEvent::SessionStart {
                session_id: "test".to_string(),
                ts: 1,
                command: vec!["fake-server".to_string()],
            })
            .unwrap();
        writer
            .write(&TraceEvent::RpcRequest {
                ts: 2,
                id: Some(1),
                method: "tools/list".to_string(),
                payload: serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
            })
            .unwrap();
        writer
            .write(&TraceEvent::RpcResponse {
                ts: 3,
                id: Some(1),
                duration_ms: 12,
                payload: serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"tools":[]}}),
            })
            .unwrap();
        writer
            .write(&TraceEvent::SessionEnd {
                ts: 4,
                status: "ok".to_string(),
            })
            .unwrap();
        let view = render_trace_view(&trace).unwrap();
        assert!(view.contains("MCP Doctor Trace Viewer"));
        assert!(view.contains("Timeline"));
        assert!(view.contains("Findings"));
        assert!(view.contains("Payload preview"));
        assert!(view.contains("tools/list"));
    }
}
