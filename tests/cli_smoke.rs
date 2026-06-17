use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mcp-doctor"))
}

fn fixture() -> &'static str {
    "fixtures/fake_mcp_server.py"
}

fn fixture_abs() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(fixture())
        .to_string_lossy()
        .to_string()
}

fn unique_temp_dir() -> PathBuf {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let path = std::env::temp_dir().join(format!("mcp-doctor-test-{id}-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run mcp-doctor")
}

fn make_call_trace(cwd: &Path, mode: Option<&str>) -> (String, String) {
    let fixture_path = fixture_abs();
    let mut args = vec![
        "call",
        "stdio",
        "--tool",
        "echo",
        "--args",
        r#"{"text":"hi","api_key":"secret"}"#,
        "--",
        "python3",
        &fixture_path,
    ];
    if let Some(mode) = mode {
        args.push("--mode");
        args.push(mode);
    }
    let output = run(cwd, &args);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let trace_line = stdout
        .lines()
        .find(|line| line.starts_with("trace: "))
        .unwrap();
    let trace_path_raw = trace_line.trim_start_matches("trace: ").trim();
    let trace_path_buf = cwd.join(trace_path_raw);
    assert!(trace_path_buf.exists());
    (trace_path_buf.to_string_lossy().to_string(), stdout)
}

#[test]
fn help_works() {
    let output = Command::new(bin()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Record, replay, and debug"));
    assert!(stdout.contains("diff"));
    assert!(stdout.contains("tui"));
}

#[test]
fn smoke_works_against_fake_server() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = run(&cwd, &["smoke", "stdio", "--", "python3", fixture()]);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("discovered 1 tool"));
}

#[test]
fn call_records_trace_and_report_then_validate_replay_tui_and_export() {
    let cwd = unique_temp_dir();
    let fixture_path = fixture_abs();
    let (trace_path, _stdout) = make_call_trace(&cwd, None);
    let trace_content = std::fs::read_to_string(&trace_path).unwrap();
    assert!(trace_content.contains("[REDACTED]"));
    assert!(!trace_content.contains(r#""api_key":"secret""#));

    let validate = run(&cwd, &["validate", &trace_path]);
    assert!(
        validate.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&validate.stderr)
    );

    let replay = run(
        &cwd,
        &["replay", &trace_path, "--", "python3", &fixture_path],
    );
    assert!(
        replay.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(String::from_utf8_lossy(&replay.stdout).contains("1 matched"));

    let report_path = cwd.join("repro.md");
    let report_path_s = report_path.to_string_lossy().to_string();
    let report = run(&cwd, &["report", &trace_path, "--output", &report_path_s]);
    assert!(
        report.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&report.stderr)
    );
    let report_text = std::fs::read_to_string(report_path).unwrap();
    assert!(report_text.contains("MCP Doctor Report"));
    assert!(report_text.contains("Executive summary"));

    let tui = run(&cwd, &["tui", &trace_path]);
    assert!(tui.status.success());
    assert!(String::from_utf8_lossy(&tui.stdout).contains("MCP Doctor Trace Viewer"));

    let export_path = cwd.join("replay.sh");
    let export_path_s = export_path.to_string_lossy().to_string();
    let export = run(
        &cwd,
        &["export-repro", &trace_path, "--output", &export_path_s],
    );
    assert!(export.status.success());
    assert!(
        std::fs::read_to_string(export_path)
            .unwrap()
            .contains("cargo run -- replay")
    );
}

#[test]
fn invalid_json_server_fails_without_panic() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = run(
        &cwd,
        &[
            "smoke",
            "stdio",
            "--",
            "python3",
            fixture(),
            "--mode",
            "invalid-json",
        ],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid JSON"), "stderr={stderr}");
}

#[test]
fn validates_bad_arguments_and_malformed_content() {
    let cwd = unique_temp_dir();
    let (trace_path, _) = make_call_trace(&cwd, Some("schema-required"));
    let validate = run(&cwd, &["validate", &trace_path]);
    assert!(!validate.status.success());
    let stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(
        stdout.contains("missing required property"),
        "stdout={stdout}"
    );

    let (bad_response_trace, _) = make_call_trace(&cwd, Some("malformed-content"));
    let validate = run(&cwd, &["validate", &bad_response_trace]);
    assert!(!validate.status.success());
    let stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(stdout.contains("missing string text"), "stdout={stdout}");
}

#[test]
fn diff_and_replay_tolerance_work() {
    let cwd = unique_temp_dir();
    let fixture_path = fixture_abs();
    let (old_trace, _) = make_call_trace(&cwd, None);
    let (new_trace, _) = make_call_trace(&cwd, Some("schema-required"));

    let diff = run(&cwd, &["diff", &old_trace, &new_trace]);
    assert!(diff.status.success());
    let stdout = String::from_utf8_lossy(&diff.stdout);
    assert!(
        stdout.contains("added required argument `limit`"),
        "stdout={stdout}"
    );

    let replay_strict = run(
        &cwd,
        &[
            "replay",
            &old_trace,
            "--",
            "python3",
            &fixture_path,
            "--mode",
            "schema-change",
        ],
    );
    assert!(replay_strict.status.success());
    assert!(String::from_utf8_lossy(&replay_strict.stdout).contains("1 mismatched"));

    let replay_allowed = run(
        &cwd,
        &[
            "replay",
            &old_trace,
            "--allow-field",
            "content.0.text",
            "--",
            "python3",
            &fixture_path,
            "--mode",
            "schema-change",
        ],
    );
    assert!(replay_allowed.status.success());
    assert!(String::from_utf8_lossy(&replay_allowed.stdout).contains("1 matched"));
}
