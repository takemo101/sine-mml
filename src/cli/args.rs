use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "sine-mml")]
#[command(version = "0.1.0")]
#[command(about = "MML Synthesizer CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Play(PlayArgs),
    History,
    Export(ExportArgs),
    /// Clear all playback history
    ClearHistory,
    /// MIDI device management
    #[cfg(feature = "midi-output")]
    Midi(MidiArgs),
}

/// MIDI subcommand arguments
#[cfg(feature = "midi-output")]
#[derive(Args, Debug)]
pub struct MidiArgs {
    #[command(subcommand)]
    pub command: MidiSubcommand,
}

/// MIDI subcommands
#[cfg(feature = "midi-output")]
#[derive(Subcommand, Debug)]
pub enum MidiSubcommand {
    /// List available MIDI output devices
    List,
}

#[derive(Args, Debug)]
#[command(group(
    clap::ArgGroup::new("input")
        .required(true)
        .args(["mml", "history_id", "file"]),
))]
pub struct PlayArgs {
    /// MML string to play
    pub mml: Option<String>,

    /// Replay from history by ID
    #[arg(long)]
    pub history_id: Option<i64>,

    /// Read MML from file (.mml extension required)
    #[arg(long, short = 'f', value_name = "FILE")]
    pub file: Option<String>,

    #[arg(short, long, default_value = "sine")]
    pub waveform: Waveform,

    #[arg(short, long, default_value_t = 1.0, value_parser = validate_volume)]
    pub volume: f32,

    #[arg(long, default_value_t = false)]
    pub loop_play: bool,

    #[arg(long, default_value_t = false)]
    pub metronome: bool,

    #[arg(long, value_parser = validate_metronome_beat, default_value_t = 4)]
    pub metronome_beat: u8,

    #[arg(long, value_parser = validate_volume, default_value_t = 0.3)]
    pub metronome_volume: f32,

    /// 履歴にメモを付与（最大500文字、UTF-8対応）
    #[arg(long)]
    pub note: Option<String>,

    /// MIDI output device ID or name (enables MIDI mode)
    #[cfg(feature = "midi-output")]
    #[arg(long, value_name = "DEVICE")]
    pub midi_out: Option<String>,

    /// MIDI channel (1-16, default: 1)
    #[cfg(feature = "midi-output")]
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=16))]
    pub midi_channel: u8,

    /// 履歴に保存しない
    #[arg(long, short = 'N', default_value_t = false)]
    pub no_history: bool,
}

#[cfg(test)]
impl PlayArgs {
    #[cfg(feature = "midi-output")]
    #[must_use]
    pub fn for_test(
        mml: Option<String>,
        history_id: Option<i64>,
        file: Option<String>,
        waveform: Waveform,
        volume: f32,
        note: Option<String>,
    ) -> Self {
        Self {
            mml,
            history_id,
            file,
            waveform,
            volume,
            loop_play: false,
            metronome: false,
            metronome_beat: 4,
            metronome_volume: 0.3,
            note,
            midi_out: None,
            midi_channel: 1,
            no_history: false,
        }
    }

    #[cfg(not(feature = "midi-output"))]
    #[must_use]
    pub fn for_test(
        mml: Option<String>,
        history_id: Option<i64>,
        file: Option<String>,
        waveform: Waveform,
        volume: f32,
        note: Option<String>,
    ) -> Self {
        Self {
            mml,
            history_id,
            file,
            waveform,
            volume,
            loop_play: false,
            metronome: false,
            metronome_beat: 4,
            metronome_volume: 0.3,
            note,
            no_history: false,
        }
    }

    /// テスト用のファクトリーメソッド（`no_history`指定可能版）
    #[cfg(feature = "midi-output")]
    #[must_use]
    pub fn for_test_with_no_history(
        mml: Option<String>,
        history_id: Option<i64>,
        file: Option<String>,
        waveform: Waveform,
        volume: f32,
        note: Option<String>,
        no_history: bool,
    ) -> Self {
        Self {
            mml,
            history_id,
            file,
            waveform,
            volume,
            loop_play: false,
            metronome: false,
            metronome_beat: 4,
            metronome_volume: 0.3,
            note,
            midi_out: None,
            midi_channel: 1,
            no_history,
        }
    }

    /// テスト用のファクトリーメソッド（`no_history`指定可能版）
    #[cfg(not(feature = "midi-output"))]
    #[must_use]
    pub fn for_test_with_no_history(
        mml: Option<String>,
        history_id: Option<i64>,
        file: Option<String>,
        waveform: Waveform,
        volume: f32,
        note: Option<String>,
        no_history: bool,
    ) -> Self {
        Self {
            mml,
            history_id,
            file,
            waveform,
            volume,
            loop_play: false,
            metronome: false,
            metronome_beat: 4,
            metronome_volume: 0.3,
            note,
            no_history,
        }
    }
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    #[arg(long)]
    pub history_id: i64,

    #[arg(short, long)]
    pub output: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
}

// Validation functions

/// Validates that the volume is between 0.0 and 1.0.
///
/// # Errors
/// Returns an error if the input string cannot be parsed as an f32 or if the value is out of range.
pub fn validate_volume(v: &str) -> Result<f32, String> {
    let val: f32 = v.parse().map_err(|_| "Invalid number".to_string())?;
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err("Volume must be between 0.0 and 1.0".to_string())
    }
}

/// Validates that the metronome beat is 4, 8, or 16.
///
/// # Errors
/// Returns an error if the input string cannot be parsed as a u8 or if the value is not 4, 8, or 16.
pub fn validate_metronome_beat(v: &str) -> Result<u8, String> {
    let val: u8 = v.parse().map_err(|_| "Invalid number".to_string())?;
    match val {
        4 | 8 | 16 => Ok(val),
        _ => Err(format!(
            "メトロノームビートは 4, 8, 16 のいずれかを指定してください（指定値: {val}）"
        )),
    }
}

/// Maximum allowed length for note field (in characters, not bytes).
pub const MAX_NOTE_LENGTH: usize = 500;

/// Validates that the note is within the maximum length.
///
/// # Errors
/// Returns an error if the note exceeds 500 characters.
pub fn validate_note(note: &str) -> Result<(), String> {
    let char_count = note.chars().count();
    if char_count > MAX_NOTE_LENGTH {
        Err(format!(
            "メモは{MAX_NOTE_LENGTH}文字以内で入力してください（現在: {char_count}文字）"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        assert!(result.is_ok());

        // History ID only -> OK
        let result = Cli::try_parse_from(&["sine-mml", "play", "--history-id", "1"]);
        assert!(result.is_ok());

        // File only -> OK
        let result = Cli::try_parse_from(&["sine-mml", "play", "--file", "test.mml"]);
        assert!(result.is_ok());

        // File with short flag -> OK
        let result = Cli::try_parse_from(&["sine-mml", "play", "-f", "test.mml"]);
        assert!(result.is_ok());

        // MML + history-id -> Error
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--history-id", "1"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

        // MML + file -> Error
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--file", "test.mml"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);

        // history-id + file -> Error
        let result = Cli::try_parse_from(&[
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
        let result = Cli::try_parse_from(&["sine-mml", "play"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_file_option_parsing() {
        // File option stores path
        let result = Cli::try_parse_from(&["sine-mml", "play", "--file", "my_song.mml"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.file, Some("my_song.mml".to_string()));
        assert_eq!(args.mml, None);
        assert_eq!(args.history_id, None);

        // Short flag -f
        let result = Cli::try_parse_from(&["sine-mml", "play", "-f", "another.mml"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.file, Some("another.mml".to_string()));
    }

    #[test]
    fn test_waveform_parsing() {
        // Default is sine
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.waveform, Waveform::Sine);

        // Explicit sawtooth
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--waveform", "sawtooth"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.waveform, Waveform::Sawtooth);

        // Short flag -w
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "-w", "square"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.waveform, Waveform::Square);
    }

    #[test]
    fn test_bpm_option_removed() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--bpm", "120"]);
        assert!(result.is_err(), "bpm option should be removed");
    }

    #[test]
    fn test_metronome_beat_valid() {
        for beat in ["4", "8", "16"] {
            let result =
                Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-beat", beat]);
            assert!(result.is_ok(), "Should accept metronome-beat {}", beat);
            let args = match result.unwrap().command {
                Command::Play(args) => args,
                _ => panic!("Unexpected command"),
            };
            assert_eq!(args.metronome_beat, beat.parse::<u8>().unwrap());
        }
    }

    #[test]
    fn test_metronome_beat_invalid() {
        for beat in ["5", "12", "32", "abc"] {
            let result =
                Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-beat", beat]);
            assert!(result.is_err(), "Should reject metronome-beat {}", beat);
        }
    }

    #[test]
    fn test_metronome_volume_valid() {
        for vol in ["0.0", "0.5", "1.0"] {
            let result =
                Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-volume", vol]);
            assert!(result.is_ok(), "Should accept metronome-volume {}", vol);
            let args = match result.unwrap().command {
                Command::Play(args) => args,
                _ => panic!("Unexpected command"),
            };
            assert_eq!(args.metronome_volume, vol.parse::<f32>().unwrap());
        }
    }

    #[test]
    fn test_metronome_volume_out_of_range() {
        for vol in ["-0.1", "1.1", "abc"] {
            let result =
                Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-volume", vol]);
            assert!(result.is_err(), "Should reject metronome-volume {}", vol);
        }
    }

    #[test]
    fn test_default_values() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.metronome_beat, 4);
        assert_eq!(args.metronome_volume, 0.3);
        assert_eq!(args.note, None);
    }

    #[test]
    fn test_note_option() {
        // With note
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--note", "My melody"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.note, Some("My melody".to_string()));

        // Without note
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.note, None);

        // Empty note
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--note", ""]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.note, Some(String::new()));
    }

    #[test]
    fn test_validate_note_valid() {
        assert!(validate_note("My melody").is_ok());
        assert!(validate_note("").is_ok());
        assert!(validate_note(&"a".repeat(500)).is_ok());
    }

    #[test]
    fn test_validate_note_too_long() {
        let result = validate_note(&"a".repeat(501));
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
        let note = "あ".repeat(500);
        assert!(validate_note(&note).is_ok());

        let note = "あ".repeat(501);
        assert!(validate_note(&note).is_err());
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_out_option() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--midi-out", "0"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.midi_out, Some("0".to_string()));
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_out_with_device_name() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--midi-out", "IAC Driver"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.midi_out, Some("IAC Driver".to_string()));
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_channel_default() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.midi_channel, 1);
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_channel_valid() {
        for ch in 1..=16 {
            let result = Cli::try_parse_from(&[
                "sine-mml",
                "play",
                "CDE",
                "--midi-channel",
                &ch.to_string(),
            ]);
            assert!(result.is_ok(), "Channel {ch} should be valid");
            let args = match result.unwrap().command {
                Command::Play(args) => args,
                _ => panic!("Unexpected command"),
            };
            assert_eq!(args.midi_channel, ch);
        }
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_channel_invalid() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--midi-channel", "0"]);
        assert!(result.is_err());

        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--midi-channel", "17"]);
        assert!(result.is_err());
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_midi_list_subcommand() {
        let result = Cli::try_parse_from(&["sine-mml", "midi", "list"]);
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
        let result = Cli::try_parse_from(&[
            "sine-mml",
            "play",
            "CDE",
            "--midi-out",
            "0",
            "--midi-channel",
            "10",
        ]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert_eq!(args.midi_out, Some("0".to_string()));
        assert_eq!(args.midi_channel, 10);
    }

    #[test]
    fn test_no_history_long_flag() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--no-history"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert!(args.no_history);
    }

    #[test]
    fn test_no_history_short_flag() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "-N"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert!(args.no_history);
    }

    #[test]
    fn test_no_history_default() {
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert!(!args.no_history);
    }

    #[cfg(feature = "midi-output")]
    #[test]
    fn test_no_history_with_midi_out() {
        let result =
            Cli::try_parse_from(&["sine-mml", "play", "CDE", "--midi-out", "0", "--no-history"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert!(args.no_history);
        assert_eq!(args.midi_out, Some("0".to_string()));
    }

    #[test]
    fn test_no_history_with_loop_play() {
        let result =
            Cli::try_parse_from(&["sine-mml", "play", "CDE", "--loop-play", "--no-history"]);
        assert!(result.is_ok());
        let args = match result.unwrap().command {
            Command::Play(args) => args,
            _ => panic!("Unexpected command"),
        };
        assert!(args.no_history);
        assert!(args.loop_play);
    }
}
