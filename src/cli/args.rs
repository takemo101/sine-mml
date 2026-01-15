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

// Tests moved to tests/cli_args_test.rs for module size reduction
