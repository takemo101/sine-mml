use assert_cmd::Command;
use predicates::prelude::*;

/// Test: play command with basic MML
#[test]
fn test_play_basic_mml() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("C")
        .timeout(std::time::Duration::from_secs(5));

    // Should succeed (audio device may not exist in CI, but command should not fail)
    let assert = cmd.assert();
    // Either success or warning about audio device
    assert.code(predicate::in_iter([0i32]));
}

/// Test: history command
#[test]
fn test_history_command() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("history");
    cmd.assert().success();
}

/// Test: --help flag
#[test]
fn test_help_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("MML Synthesizer CLI"));
}

/// Test: --version flag  
#[test]
fn test_version_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sine-mml"));
}

/// Test: --bpm option removed (breaking change)
#[test]
fn test_bpm_option_removed() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play").arg("C").arg("--bpm").arg("120");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

/// Test: invalid metronome beat value
#[test]
fn test_invalid_metronome_beat() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play").arg("C").arg("--metronome-beat").arg("5");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("4, 8, 16"));
}

/// Test: valid metronome options
#[test]
fn test_valid_metronome_options() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("C")
        .arg("--metronome")
        .arg("--metronome-beat")
        .arg("8")
        .arg("--metronome-volume")
        .arg("0.5")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}

/// Test: play command missing input
#[test]
fn test_play_missing_input() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

/// Test: clear-history command runs without crash
#[test]
fn test_clear_history_runs() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("clear-history").write_stdin("n\n");
    cmd.assert().code(predicate::in_iter([0i32]));
}

/// Test: clear-history command confirm yes
#[test]
fn test_clear_history_confirm_yes() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("clear-history").write_stdin("y\n");
    cmd.assert().success();
}

/// Test: clear-history command confirm no
#[test]
fn test_clear_history_cancel() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("clear-history").write_stdin("n\n");
    cmd.assert().success().stdout(
        predicate::str::contains("キャンセル").or(predicate::str::contains("履歴がありません")),
    );
}

/// Test: clear-history command invalid input
#[test]
fn test_clear_history_invalid_input() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("clear-history").write_stdin("invalid\n");
    cmd.assert().code(predicate::in_iter([0i32, 1i32]));
}

// ============================================================================
// Loop Syntax E2E Tests (Issue #66, TC-023-E-xxx)
// ============================================================================

/// TC-023-E-001: ループ構文でのCLI再生
#[test]
fn test_cli_play_with_loop_syntax() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("[CDEF]3")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}

/// TC-023-E-002: ループ回数超過エラーのCLI表示
#[test]
fn test_cli_loop_count_error() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play").arg("[CDEF]100");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Loop count must be 1-99"));
}

/// TC-023-E-003: ネストループエラーのCLI表示
#[test]
fn test_cli_nested_loop_error() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play").arg("[[CDEF]2]3");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Nested loops are not supported"));
}

/// TC-024-E-002: 小文字とループの組み合わせ
#[test]
fn test_cli_lowercase_with_loop() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("[cdef]3")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}

/// TC-023-E-004: 脱出ポイント付きループでのCLI再生
#[test]
fn test_cli_loop_with_escape_point() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("[CD:EF]2")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}
