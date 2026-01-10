use clap::{Parser, Subcommand, Args, ValueEnum};

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
    pub history_id: Option<i64>,
    
    #[arg(short, long)]
    pub output: String,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
}

// Validation functions
pub fn validate_volume(v: &str) -> Result<f32, String> {
    let val: f32 = v.parse().map_err(|_| "Invalid number".to_string())?;
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err("Volume must be between 0.0 and 1.0".to_string())
    }
}

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
}
