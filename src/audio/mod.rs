pub mod error;
pub mod exporter;
pub mod player;
pub mod synthesizer;
pub mod waveform;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("WAV encoding error: {0}")]
    WavWriteError(#[from] hound::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
