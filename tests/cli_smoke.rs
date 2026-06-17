use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mcp-doctor"))
}

fn fixture() -> &'static str {
    "fixtures/fake_mcp_server.py"
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

#[test]
fn help_works() {
    let output = Command::new(bin()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Record, replay, and debug"));
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
fn call_records_trace_and_report_then_validate_and_replay() {
    let cwd = unique_temp_dir();
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(fixture());
    let fixture_path = fixture_path.to_string_lossy().to_string();
    let output = run(
        &cwd,
        &[
            "call",
            "stdio",
            "--tool",
            "echo",
            "--args",
            r#"{"text":"hi","api_key":"secret"}"#,
            "--",
            "python3",
            &fixture_path,
        ],
    );
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace_line = stdout
        .lines()
        .find(|line| line.starts_with("trace: "))
        .unwrap();
    let trace_path_raw = trace_line.trim_start_matches("trace: ").trim();
    let trace_path_buf = cwd.join(trace_path_raw);
    let trace_path = trace_path_buf.to_string_lossy().to_string();
    assert!(trace_path_buf.exists());
    let trace_content = std::fs::read_to_string(&trace_path_buf).unwrap();
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
    assert!(
        std::fs::read_to_string(report_path)
            .unwrap()
            .contains("MCP Doctor Report")
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
