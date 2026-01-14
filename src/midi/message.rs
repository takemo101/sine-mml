//! MIDI message types
//!
//! This module defines MIDI message structures for note and control events.

/// MIDI channel number (1-16)
pub type MidiChannel = u8;

/// MIDI note number (0-127)
pub type MidiNote = u8;

/// MIDI velocity (0-127)
pub type MidiVelocity = u8;

/// MIDI message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessage {
    /// Note On event
    NoteOn {
        /// Channel (1-16)
        channel: MidiChannel,
        /// Note number (0-127)
        note: MidiNote,
        /// Velocity (0-127)
        velocity: MidiVelocity,
    },
    /// Note Off event
    NoteOff {
        /// Channel (1-16)
        channel: MidiChannel,
        /// Note number (0-127)
        note: MidiNote,
        /// Velocity (0-127, typically 0)
        velocity: MidiVelocity,
    },
}

impl MidiMessage {
    /// Create a Note On message.
    #[must_use]
    pub fn note_on(channel: MidiChannel, note: MidiNote, velocity: MidiVelocity) -> Self {
        Self::NoteOn {
            channel,
            note,
            velocity,
        }
    }

    /// Create a Note Off message.
    #[must_use]
    pub fn note_off(channel: MidiChannel, note: MidiNote, velocity: MidiVelocity) -> Self {
        Self::NoteOff {
            channel,
            note,
            velocity,
        }
    }

    /// Convert the message to raw MIDI bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 3] {
        match *self {
            Self::NoteOn {
                channel,
                note,
                velocity,
            } => {
                // Note On status byte: 0x90 | (channel - 1)
                let status = 0x90 | ((channel.saturating_sub(1)) & 0x0F);
                [status, note & 0x7F, velocity & 0x7F]
            }
            Self::NoteOff {
                channel,
                note,
                velocity,
            } => {
                // Note Off status byte: 0x80 | (channel - 1)
                let status = 0x80 | ((channel.saturating_sub(1)) & 0x0F);
                [status, note & 0x7F, velocity & 0x7F]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_creation() {
        let msg = MidiMessage::note_on(1, 60, 100);
        assert_eq!(
            msg,
            MidiMessage::NoteOn {
                channel: 1,
                note: 60,
                velocity: 100
            }
        );
    }

    #[test]
    fn test_note_off_creation() {
        let msg = MidiMessage::note_off(1, 60, 0);
        assert_eq!(
            msg,
            MidiMessage::NoteOff {
                channel: 1,
                note: 60,
                velocity: 0
            }
        );
    }

    #[test]
    fn test_note_on_to_bytes_channel_1() {
        let msg = MidiMessage::note_on(1, 60, 100);
        let bytes = msg.to_bytes();
        assert_eq!(bytes, [0x90, 60, 100]);
    }

    #[test]
    fn test_note_on_to_bytes_channel_10() {
        let msg = MidiMessage::note_on(10, 60, 127);
        let bytes = msg.to_bytes();
        assert_eq!(bytes, [0x99, 60, 127]);
    }

    #[test]
    fn test_note_off_to_bytes_channel_1() {
        let msg = MidiMessage::note_off(1, 60, 0);
        let bytes = msg.to_bytes();
        assert_eq!(bytes, [0x80, 60, 0]);
    }

    #[test]
    fn test_note_off_to_bytes_channel_16() {
        let msg = MidiMessage::note_off(16, 127, 64);
        let bytes = msg.to_bytes();
        assert_eq!(bytes, [0x8F, 127, 64]);
    }

    #[test]
    fn test_to_bytes_clamps_note() {
        // Note values > 127 should be clamped
        let msg = MidiMessage::note_on(1, 200, 100);
        let bytes = msg.to_bytes();
        // 200 & 0x7F = 72
        assert_eq!(bytes, [0x90, 72, 100]);
    }

    #[test]
    fn test_to_bytes_clamps_velocity() {
        // Velocity values > 127 should be clamped
        let msg = MidiMessage::note_on(1, 60, 200);
        let bytes = msg.to_bytes();
        // 200 & 0x7F = 72
        assert_eq!(bytes, [0x90, 60, 72]);
    }
}
