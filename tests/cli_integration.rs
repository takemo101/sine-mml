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
    cmd.arg("play")
        .arg("C")
        .arg("--bpm")
        .arg("120");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

/// Test: invalid metronome beat value
#[test]
fn test_invalid_metronome_beat() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
    cmd.arg("play")
        .arg("C")
        .arg("--metronome-beat")
        .arg("5");
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
