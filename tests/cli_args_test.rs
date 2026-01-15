//! CLI argument parsing tests
//!
//! Moved from src/cli/args.rs to reduce module size

use sine_mml::cli::args::{
    validate_note, validate_volume, Cli, Command, Waveform, MAX_NOTE_LENGTH,
};

#[cfg(feature = "midi-output")]
use sine_mml::cli::args::MidiSubcommand;

use clap::Parser;

#[test]
fn test_validate_volume_valid() {
    assert!(validate_volume("0.5").is_ok());
    assert!(validate_volume("0.0").is_ok());
    assert!(validate_volume("1.0").is_ok());
}

#[test]
fn test_validate_volume_invalid() {
    assert!(validate_volume("-0.1").is_err());
    assert!(validate_volume("1.1").is_err());
    assert!(validate_volume("abc").is_err());
}

#[test]
fn test_play_args_conflict() {
    // MML only -> OK
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    assert!(result.is_ok());

    // History ID only -> OK
    let result = Cli::try_parse_from(["sine-mml", "play", "--history-id", "1"]);
    assert!(result.is_ok());

    // File only -> OK
    let result = Cli::try_parse_from(["sine-mml", "play", "--file", "test.mml"]);
    assert!(result.is_ok());

    // File with short flag -> OK
    let result = Cli::try_parse_from(["sine-mml", "play", "-f", "test.mml"]);
    assert!(result.is_ok());

    // MML + history-id -> Error
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--history-id", "1"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

    // MML + file -> Error
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--file", "test.mml"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

    // history-id + file -> Error
    let result = Cli::try_parse_from([
        "sine-mml",
        "play",
        "--history-id",
        "1",
        "--file",
        "test.mml",
    ]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

    // Neither -> Error
    let result = Cli::try_parse_from(["sine-mml", "play"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
}

#[test]
fn test_file_option_parsing() {
    // File option stores path
    let result = Cli::try_parse_from(["sine-mml", "play", "--file", "my_song.mml"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.file, Some("my_song.mml".to_string()));
    assert_eq!(args.mml, None);
    assert_eq!(args.history_id, None);

    // Short flag -f
    let result = Cli::try_parse_from(["sine-mml", "play", "-f", "another.mml"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.file, Some("another.mml".to_string()));
}

#[test]
fn test_waveform_parsing() {
    // Default is sine
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.waveform, Waveform::Sine);

    // Explicit sawtooth
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--waveform", "sawtooth"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.waveform, Waveform::Sawtooth);

    // Short flag -w
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "-w", "square"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.waveform, Waveform::Square);
}

#[test]
fn test_bpm_option_removed() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--bpm", "120"]);
    assert!(result.is_err(), "bpm option should be removed");
}

#[test]
fn test_metronome_beat_valid() {
    for beat in ["4", "8", "16"] {
        let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--metronome-beat", beat]);
        assert!(result.is_ok(), "Should accept metronome-beat {beat}");
        let Command::Play(args) = result.unwrap().command else {
            panic!("Unexpected command")
        };
        assert_eq!(args.metronome_beat, beat.parse::<u8>().unwrap());
    }
}

#[test]
fn test_metronome_beat_invalid() {
    for beat in ["5", "12", "32", "abc"] {
        let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--metronome-beat", beat]);
        assert!(result.is_err(), "Should reject metronome-beat {beat}");
    }
}

#[test]
fn test_metronome_volume_valid() {
    for vol in ["0.0", "0.5", "1.0"] {
        let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--metronome-volume", vol]);
        assert!(result.is_ok(), "Should accept metronome-volume {vol}");
        let Command::Play(args) = result.unwrap().command else {
            panic!("Unexpected command")
        };
        assert_eq!(args.metronome_volume, vol.parse::<f32>().unwrap());
    }
}

#[test]
fn test_metronome_volume_out_of_range() {
    for vol in ["-0.1", "1.1", "abc"] {
        let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--metronome-volume", vol]);
        assert!(result.is_err(), "Should reject metronome-volume {vol}");
    }
}

#[test]
fn test_default_values() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.metronome_beat, 4);
    assert_eq!(args.metronome_volume, 0.3);
    assert_eq!(args.note, None);
}

#[test]
fn test_note_option() {
    // With note
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--note", "My melody"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.note, Some("My melody".to_string()));

    // Without note
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.note, None);

    // Empty note
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--note", ""]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.note, Some(String::new()));
}

#[test]
fn test_validate_note_valid() {
    assert!(validate_note("My melody").is_ok());
    assert!(validate_note("").is_ok());
    assert!(validate_note(&"a".repeat(MAX_NOTE_LENGTH)).is_ok());
}

#[test]
fn test_validate_note_too_long() {
    let result = validate_note(&"a".repeat(MAX_NOTE_LENGTH + 1));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("500文字以内"));
}

#[test]
fn test_validate_note_utf8() {
    // UTF-8 characters (emoji)
    assert!(validate_note("🎵🎶🎵").is_ok());
    assert!(validate_note("あいうえお").is_ok());
}

#[test]
fn test_validate_note_char_count() {
    let note = "あ".repeat(MAX_NOTE_LENGTH);
    assert!(validate_note(&note).is_ok());

    let note = "あ".repeat(MAX_NOTE_LENGTH + 1);
    assert!(validate_note(&note).is_err());
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_option() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-out", "0"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_out_with_device_name() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-out", "IAC Driver"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.midi_out, Some("IAC Driver".to_string()));
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_channel_default() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.midi_channel, 1);
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_channel_valid() {
    for ch in 1..=16 {
        let result =
            Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-channel", &ch.to_string()]);
        assert!(result.is_ok(), "Channel {ch} should be valid");
        let Command::Play(args) = result.unwrap().command else {
            panic!("Unexpected command")
        };
        assert_eq!(args.midi_channel, ch);
    }
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_channel_invalid() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-channel", "0"]);
    assert!(result.is_err());

    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-channel", "17"]);
    assert!(result.is_err());
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_list_subcommand() {
    let result = Cli::try_parse_from(["sine-mml", "midi", "list"]);
    assert!(result.is_ok());
    match result.unwrap().command {
        Command::Midi(args) => match args.command {
            MidiSubcommand::List => {}
        },
        _ => panic!("Expected Midi command"),
    }
}

#[cfg(feature = "midi-output")]
#[test]
fn test_midi_combined_options() {
    let result = Cli::try_parse_from([
        "sine-mml",
        "play",
        "CDE",
        "--midi-out",
        "0",
        "--midi-channel",
        "10",
    ]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert_eq!(args.midi_out, Some("0".to_string()));
    assert_eq!(args.midi_channel, 10);
}

#[test]
fn test_no_history_long_flag() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--no-history"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert!(args.no_history);
}

#[test]
fn test_no_history_short_flag() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "-N"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert!(args.no_history);
}

#[test]
fn test_no_history_default() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert!(!args.no_history);
}

#[cfg(feature = "midi-output")]
#[test]
fn test_no_history_with_midi_out() {
    let result =
        Cli::try_parse_from(["sine-mml", "play", "CDE", "--midi-out", "0", "--no-history"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert!(args.no_history);
    assert_eq!(args.midi_out, Some("0".to_string()));
}

#[test]
fn test_no_history_with_loop_play() {
    let result = Cli::try_parse_from(["sine-mml", "play", "CDE", "--loop-play", "--no-history"]);
    assert!(result.is_ok());
    let Command::Play(args) = result.unwrap().command else {
        panic!("Unexpected command")
    };
    assert!(args.no_history);
    assert!(args.loop_play);
}
