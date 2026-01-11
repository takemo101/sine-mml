use crate::db::DbError;
use chrono::{DateTime, Utc};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
}

impl FromStr for Waveform {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sine" => Ok(Waveform::Sine),
            "sawtooth" => Ok(Waveform::Sawtooth),
            "square" => Ok(Waveform::Square),
            _ => Err(DbError::InvalidWaveform(s.to_string())),
        }
    }
}

impl Waveform {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Waveform::Sine => "sine",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Square => "square",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub id: Option<i64>,
    pub mml: String,
    pub waveform: Waveform,
    pub volume: f32,
    pub bpm: u16,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl HistoryEntry {
    /// Creates a new `HistoryEntry` with an optional note.
    ///
    /// # Arguments
    ///
    /// * `mml` - The MML string
    /// * `waveform` - The waveform type
    /// * `volume` - The volume (0.0 to 1.0)
    /// * `bpm` - The tempo in BPM
    /// * `note` - Optional note/memo for this entry (max 500 characters)
    #[must_use]
    pub fn new(
        mml: String,
        waveform: Waveform,
        volume: f32,
        bpm: u16,
        note: Option<String>,
    ) -> Self {
        Self {
            id: None,
            mml,
            waveform,
            volume,
            bpm,
            note,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_from_str() {
        assert_eq!("sine".parse::<Waveform>().unwrap(), Waveform::Sine);
        assert_eq!("sawtooth".parse::<Waveform>().unwrap(), Waveform::Sawtooth);
        assert_eq!("square".parse::<Waveform>().unwrap(), Waveform::Square);
        assert!("invalid".parse::<Waveform>().is_err());
    }

    #[test]
    fn test_waveform_as_str() {
        assert_eq!(Waveform::Sine.as_str(), "sine");
        assert_eq!(Waveform::Sawtooth.as_str(), "sawtooth");
        assert_eq!(Waveform::Square.as_str(), "square");
    }

    #[test]
    fn test_history_entry_new_with_note() {
        let entry = HistoryEntry::new(
            "CDE".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some("My melody".to_string()),
        );
        assert_eq!(entry.mml, "CDE");
        assert_eq!(entry.waveform, Waveform::Sine);
        assert_eq!(entry.volume, 0.5);
        assert_eq!(entry.bpm, 120);
        assert_eq!(entry.note, Some("My melody".to_string()));
        assert!(entry.id.is_none());
    }

    #[test]
    fn test_history_entry_new_without_note() {
        let entry = HistoryEntry::new("CDE".to_string(), Waveform::Sine, 0.5, 120, None);
        assert_eq!(entry.mml, "CDE");
        assert_eq!(entry.waveform, Waveform::Sine);
        assert_eq!(entry.volume, 0.5);
        assert_eq!(entry.bpm, 120);
        assert_eq!(entry.note, None);
        assert!(entry.id.is_none());
    }

    #[test]
    fn test_history_entry_new_with_empty_note() {
        let entry = HistoryEntry::new(
            "CDE".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some(String::new()),
        );
        assert_eq!(entry.note, Some(String::new()));
    }

    #[test]
    fn test_history_entry_new_with_utf8_note() {
        let entry = HistoryEntry::new(
            "CDE".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some("🎵 私のメロディ 🎶".to_string()),
        );
        assert_eq!(entry.note, Some("🎵 私のメロディ 🎶".to_string()));
    }
}
