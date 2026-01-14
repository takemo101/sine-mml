//! MIDI device management
//!
//! This module handles MIDI device enumeration and connection.

/// Placeholder for MIDI device info.
/// Actual implementation will be added in later issues.
#[derive(Debug, Clone)]
pub struct MidiDeviceInfo {
    /// Device ID
    pub id: usize,
    /// Device name
    pub name: String,
}

impl MidiDeviceInfo {
    /// Create a new `MidiDeviceInfo`.
    #[must_use]
    pub fn new(id: usize, name: String) -> Self {
        Self { id, name }
    }
}
