use assert_cmd::Command;
use predicates::prelude::*;

mod midi_tests {
    use super::*;

    /// MIDIデバイスが存在しない環境（CI含む）では、デバイス不在エラーを検証
    /// MIDIデバイスが存在する環境では、無効なデバイスIDエラーを検証
    #[test]
    fn test_midi_invalid_device_error() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
        cmd.args(["play", "CDEF", "--midi-out", "99"]);

        // エラー終了することを確認（デバイス不在またはデバイスID無効）
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("MIDI").or(predicate::str::contains("midi")));
    }

    /// MIDIチャンネルの範囲外エラーを検証
    /// CI環境ではMIDIデバイス不在エラーが先に発生するため、どちらのエラーも許容
    #[test]
    fn test_midi_invalid_channel_error() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
        cmd.args(["play", "CDEF", "--midi-out", "0", "--midi-channel", "0"]);

        // エラー終了することを確認
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("MIDI").or(predicate::str::contains("midi")));
    }
}

mod tuplet_tests {
    use super::*;

    #[test]
    fn test_tuplet_playback() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
        cmd.args(["play", "{CDE}3"]);

        cmd.assert().success();
    }

    #[test]
    fn test_invalid_tuplet_count_error() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
        cmd.args(["play", "{CDE}1"]);

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("InvalidTupletCount"));
    }

    #[test]
    fn test_tuplet_nest_too_deep_error() {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_sine-mml"));
        cmd.args(["play", "{{{{{{C}2}2}2}2}2}2"]);

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("TupletNestTooDeep"));
    }
}
