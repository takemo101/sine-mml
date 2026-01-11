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
}

#[derive(Args, Debug)]
#[command(group(
    clap::ArgGroup::new("input")
        .required(true)
        .args(["mml", "history_id"]),
))]
pub struct PlayArgs {
    pub mml: Option<String>,

    #[arg(long)]
    pub history_id: Option<i64>,

    #[arg(short, long, default_value = "sine")]
    pub waveform: Waveform,

    #[arg(short, long, default_value_t = 1.0, value_parser = validate_volume)]
    pub volume: f32,

    #[arg(short, long, default_value_t = 120, value_parser = validate_bpm)]
    pub bpm: u32,

    #[arg(long, default_value_t = false)]
    pub loop_play: bool,

    #[arg(long, default_value_t = false)]
    pub metronome: bool,
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

/// Validates that the BPM is between 30 and 300.
///
/// # Errors
/// Returns an error if the input string cannot be parsed as a u32 or if the value is out of range.
pub fn validate_bpm(v: &str) -> Result<u32, String> {
    let val: u32 = v.parse().map_err(|_| "Invalid number".to_string())?;
    if (30..=300).contains(&val) {
        Ok(val)
    } else {
        Err("BPM must be between 30 and 300".to_string())
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
    fn test_validate_bpm_valid() {
        assert!(validate_bpm("30").is_ok());
        assert!(validate_bpm("300").is_ok());
        assert!(validate_bpm("120").is_ok());
    }

    #[test]
    fn test_validate_bpm_invalid() {
        assert!(validate_bpm("29").is_err());
        assert!(validate_bpm("301").is_err());
        assert!(validate_bpm("abc").is_err());
    }

    #[test]
    fn test_play_args_conflict() {
        // MML only -> OK
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE"]);
        assert!(result.is_ok());

        // History ID only -> OK
        let result = Cli::try_parse_from(&["sine-mml", "play", "--history-id", "1"]);
        assert!(result.is_ok());

        // Both -> Error
        let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--history-id", "1"]);
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
        // 現在は bpm が存在するため Ok になるが、削除後は Err になるべき
        assert!(result.is_err(), "bpm option should be removed");
    }

    #[test]
    fn test_metronome_beat_valid() {
        for beat in ["4", "8", "16"] {
            let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-beat", beat]);
            assert!(result.is_ok(), "Should accept metronome-beat {}", beat);
        }
    }

    #[test]
    fn test_metronome_beat_invalid() {
        for beat in ["5", "12", "32", "abc"] {
            let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-beat", beat]);
            assert!(result.is_err(), "Should reject metronome-beat {}", beat);
        }
    }

    #[test]
    fn test_metronome_volume_valid() {
        for vol in ["0.0", "0.5", "1.0"] {
            let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-volume", vol]);
            assert!(result.is_ok(), "Should accept metronome-volume {}", vol);
        }
    }

    #[test]
    fn test_metronome_volume_out_of_range() {
        for vol in ["-0.1", "1.1", "abc"] {
            let result = Cli::try_parse_from(&["sine-mml", "play", "CDE", "--metronome-volume", vol]);
            assert!(result.is_err(), "Should reject metronome-volume {}", vol);
        }
    }
}
