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
/// Note: Uses short loop with fast tempo to avoid CI timeout
#[test]
fn test_cli_play_with_loop_syntax() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 [C]2") // Fast tempo, 16th notes, 2 repeats
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
        .stderr(predicate::str::contains("InvalidLoopCount"));
}

/// TC-023-E-003: ネストループエラーのCLI表示
#[test]
fn test_cli_nested_loop_error() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play").arg("[[CDEF]2]3");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("NestedLoop"));
}

/// TC-024-E-002: 小文字とループの組み合わせ
/// Note: Uses short loop with fast tempo to avoid CI timeout
#[test]
fn test_cli_lowercase_with_loop() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 [c]2") // Fast tempo, 16th notes, 2 repeats
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}

/// TC-023-E-004: 脱出ポイント付きループでのCLI再生
/// Note: Uses short loop with fast tempo to avoid CI timeout
#[test]
fn test_cli_loop_with_escape_point() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 [C:D]2") // Fast tempo, 16th notes, 2 repeats with escape
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));
}

// ============================================================================
// History Note E2E Tests (Issue #67, TC-025-E-xxx)
// ============================================================================

/// TC-025-E-001: --noteオプションでの再生
#[test]
fn test_cli_play_with_note() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 C")
        .arg("--note")
        .arg("My melody")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));

    let mut history_cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    history_cmd.arg("history");
    history_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("My melody"));
}

/// TC-025-E-002: UTF-8メモの表示
#[test]
fn test_cli_note_with_utf8() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 C")
        .arg("--note")
        .arg("あいうえお🎵")
        .timeout(std::time::Duration::from_secs(5));
    cmd.assert().code(predicate::in_iter([0i32]));

    let mut history_cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    history_cmd.arg("history");
    history_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("あいうえお🎵"));
}

/// TC-025-E-003: メモ長さ超過エラー
#[test]
fn test_cli_note_too_long() {
    let long_note = "a".repeat(501);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("T300 L16 C")
        .arg("--note")
        .arg(&long_note);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("500"));
}

/// TC-025-E-004: 履歴表示でのメモ列
#[test]
fn test_cli_history_displays_note_column() {
    let mut cmd1 = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd1.arg("play")
        .arg("T300 L16 C")
        .arg("--note")
        .arg("Test note for display")
        .timeout(std::time::Duration::from_secs(5));
    cmd1.assert().code(predicate::in_iter([0i32]));

    let mut history_cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    history_cmd.arg("history");
    history_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Note"))
        .stdout(predicate::str::contains("Test note for display"));
}
