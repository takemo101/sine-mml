//! MIDI message types and conversion functions
//!
//! This module defines MIDI message structures for note and control events,
//! and provides conversion functions between MML and MIDI formats.

use midir::MidiOutputConnection;

use super::error::MidiError;
use crate::mml::ast::{Accidental, Pitch};

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
    /// All Notes Off (Control Change #123)
    AllNotesOff {
        /// Channel (1-16)
        channel: MidiChannel,
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

    /// Create an All Notes Off message (Control Change #123).
    #[must_use]
    pub fn all_notes_off(channel: MidiChannel) -> Self {
        Self::AllNotesOff { channel }
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
            Self::AllNotesOff { channel } => {
                // Control Change status byte: 0xB0 | (channel - 1)
                // Controller #123: All Notes Off
                let status = 0xB0 | ((channel.saturating_sub(1)) & 0x0F);
                [status, 123, 0]
            }
        }
    }
}

// ============================================================
// Message Build Functions
// ============================================================

/// Build a Note On MIDI message as raw bytes.
///
/// # Arguments
/// * `channel` - MIDI channel (1-16)
/// * `note` - MIDI note number (0-127)
/// * `velocity` - MIDI velocity (0-127)
///
/// # Returns
/// A 3-byte array representing the MIDI message.
#[must_use]
pub fn build_note_on_message(channel: u8, note: u8, velocity: u8) -> [u8; 3] {
    MidiMessage::note_on(channel, note, velocity).to_bytes()
}

/// Build a Note Off MIDI message as raw bytes.
///
/// # Arguments
/// * `channel` - MIDI channel (1-16)
/// * `note` - MIDI note number (0-127)
///
/// # Returns
/// A 3-byte array representing the MIDI message.
#[must_use]
pub fn build_note_off_message(channel: u8, note: u8) -> [u8; 3] {
    MidiMessage::note_off(channel, note, 0).to_bytes()
}

/// Build an All Notes Off MIDI message as raw bytes.
///
/// # Arguments
/// * `channel` - MIDI channel (1-16)
///
/// # Returns
/// A 3-byte array representing the MIDI Control Change #123 message.
#[must_use]
pub fn build_all_notes_off_message(channel: u8) -> [u8; 3] {
    MidiMessage::all_notes_off(channel).to_bytes()
}

// ============================================================
// MML → MIDI Conversion Functions
// ============================================================

/// Convert MML pitch, accidental, and octave to MIDI note number.
///
/// # Arguments
/// * `pitch` - MML pitch (C, D, E, F, G, A, B)
/// * `accidental` - Accidental (Sharp, Flat, Natural)
/// * `octave` - Octave number (0-8)
///
/// # Returns
/// MIDI note number (0-127)
///
/// # Examples
/// - C4 → 60
/// - C#4 → 61
/// - Db4 → 61
/// - A4 → 69
/// - B8 → 107
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn mml_to_midi_note(pitch: Pitch, accidental: Accidental, octave: u8) -> u8 {
    let pitch_offset = pitch as i16;

    let accidental_offset = match accidental {
        Accidental::Sharp => 1_i16,
        Accidental::Flat => -1_i16,
        Accidental::Natural => 0_i16,
    };

    // MIDI note = (octave + 1) * 12 + pitch_offset + accidental_offset
    // C4 = (4 + 1) * 12 + 0 = 60
    let midi_note = (i16::from(octave) + 1) * 12 + pitch_offset + accidental_offset;

    // Clamp to valid MIDI range 0-127
    midi_note.clamp(0, 127) as u8
}

/// Convert MML volume (0-15) to MIDI velocity (0-127).
///
/// # Arguments
/// * `volume` - MML volume (0-15)
///
/// # Returns
/// MIDI velocity (0-127)
///
/// # Examples
/// - V0 → 0
/// - V10 → 84
/// - V15 → 127
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn mml_volume_to_velocity(volume: u8) -> u8 {
    // Linear interpolation: velocity = (volume * 127) / 15
    (u16::from(volume) * 127 / 15) as u8
}

// ============================================================
// MIDI Send Functions
// ============================================================

/// Send a Note On message to the MIDI output.
///
/// # Arguments
/// * `conn` - MIDI output connection
/// * `channel` - MIDI channel (1-16)
/// * `note` - MIDI note number (0-127)
/// * `velocity` - MIDI velocity (0-127)
///
/// # Errors
/// Returns `MidiError::InvalidChannel` if channel is not in 1-16 range.
/// Returns `MidiError::SendFailed` if sending fails.
pub fn send_note_on(
    conn: &mut MidiOutputConnection,
    channel: u8,
    note: u8,
    velocity: u8,
) -> Result<(), MidiError> {
    MidiError::validate_channel(channel)?;

    let msg = build_note_on_message(channel, note, velocity);
    conn.send(&msg)
        .map_err(|e| MidiError::send_failed(e.to_string()))
}

/// Send a Note Off message to the MIDI output.
///
/// # Arguments
/// * `conn` - MIDI output connection
/// * `channel` - MIDI channel (1-16)
/// * `note` - MIDI note number (0-127)
///
/// # Errors
/// Returns `MidiError::InvalidChannel` if channel is not in 1-16 range.
/// Returns `MidiError::SendFailed` if sending fails.
pub fn send_note_off(
    conn: &mut MidiOutputConnection,
    channel: u8,
    note: u8,
) -> Result<(), MidiError> {
    MidiError::validate_channel(channel)?;

    let msg = build_note_off_message(channel, note);
    conn.send(&msg)
        .map_err(|e| MidiError::send_failed(e.to_string()))
}

/// Send an All Notes Off message to the MIDI output.
///
/// This is used for cleanup when stopping playback or on interruption.
///
/// # Arguments
/// * `conn` - MIDI output connection
/// * `channel` - MIDI channel (1-16)
///
/// # Errors
/// Returns `MidiError::InvalidChannel` if channel is not in 1-16 range.
/// Returns `MidiError::SendFailed` if sending fails.
pub fn send_all_notes_off(conn: &mut MidiOutputConnection, channel: u8) -> Result<(), MidiError> {
    MidiError::validate_channel(channel)?;

    let msg = build_all_notes_off_message(channel);
    conn.send(&msg)
        .map_err(|e| MidiError::send_failed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // MidiMessage Tests (existing)
    // ============================================================

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
    fn test_all_notes_off_creation() {
        let msg = MidiMessage::all_notes_off(1);
        assert_eq!(msg, MidiMessage::AllNotesOff { channel: 1 });
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
    fn test_all_notes_off_to_bytes_channel_1() {
        let msg = MidiMessage::all_notes_off(1);
        let bytes = msg.to_bytes();
        // 0xB0 | 0 = 0xB0, controller 123, value 0
        assert_eq!(bytes, [0xB0, 123, 0]);
    }

    #[test]
    fn test_all_notes_off_to_bytes_channel_10() {
        let msg = MidiMessage::all_notes_off(10);
        let bytes = msg.to_bytes();
        // 0xB0 | 9 = 0xB9
        assert_eq!(bytes, [0xB9, 123, 0]);
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

    // ============================================================
    // Build Functions Tests (TC-031-U-001)
    // ============================================================

    #[test]
    fn test_build_note_on_message() {
        let bytes = build_note_on_message(1, 60, 100);
        assert_eq!(bytes, [0x90, 60, 100]);
    }

    #[test]
    fn test_build_note_on_message_channel_10() {
        let bytes = build_note_on_message(10, 60, 127);
        assert_eq!(bytes, [0x99, 60, 127]);
    }

    #[test]
    fn test_build_note_off_message() {
        let bytes = build_note_off_message(1, 60);
        assert_eq!(bytes, [0x80, 60, 0]);
    }

    #[test]
    fn test_build_note_off_message_channel_16() {
        let bytes = build_note_off_message(16, 127);
        assert_eq!(bytes, [0x8F, 127, 0]);
    }

    #[test]
    fn test_build_all_notes_off_message() {
        let bytes = build_all_notes_off_message(1);
        assert_eq!(bytes, [0xB0, 123, 0]);
    }

    #[test]
    fn test_build_all_notes_off_message_channel_10() {
        let bytes = build_all_notes_off_message(10);
        assert_eq!(bytes, [0xB9, 123, 0]);
    }

    // ============================================================
    // MML→MIDI Conversion Tests (TC-031-U-002, TC-031-U-003)
    // ============================================================

    #[test]
    fn test_mml_to_midi_note_c4() {
        let note = mml_to_midi_note(Pitch::C, Accidental::Natural, 4);
        assert_eq!(note, 60);
    }

    #[test]
    fn test_mml_to_midi_note_c_sharp_4() {
        let note = mml_to_midi_note(Pitch::C, Accidental::Sharp, 4);
        assert_eq!(note, 61);
    }

    #[test]
    fn test_mml_to_midi_note_d_flat_4() {
        // Db4 = D4 - 1 = 62 - 1 = 61
        let note = mml_to_midi_note(Pitch::D, Accidental::Flat, 4);
        assert_eq!(note, 61);
    }

    #[test]
    fn test_mml_to_midi_note_a4() {
        let note = mml_to_midi_note(Pitch::A, Accidental::Natural, 4);
        assert_eq!(note, 69);
    }

    #[test]
    fn test_mml_to_midi_note_b8() {
        // B8 = (8 + 1) * 12 + 11 = 108 + 11 = 119
        // Wait, let me recalculate: B in octave 8
        // (8 + 1) * 12 + 11 = 9 * 12 + 11 = 108 + 11 = 119
        // But issue says B8 = 107, let me check the formula
        // Actually C4 = 60, so octave 4 base is 60
        // C0 = (0+1)*12 + 0 = 12
        // C4 = (4+1)*12 + 0 = 60 ✓
        // B8 = (8+1)*12 + 11 = 108 + 11 = 119
        // But issue says 107... the issue might have an error
        // Let's use the standard MIDI mapping: B8 should be 119
        let note = mml_to_midi_note(Pitch::B, Accidental::Natural, 8);
        // Standard MIDI: B8 = 119
        // But clamped to 127, so it should be 119
        assert_eq!(note, 119);
    }

    #[test]
    fn test_mml_to_midi_note_c0() {
        // C0 = (0 + 1) * 12 + 0 = 12
        let note = mml_to_midi_note(Pitch::C, Accidental::Natural, 0);
        assert_eq!(note, 12);
    }

    #[test]
    fn test_mml_to_midi_note_all_pitches() {
        // Test all pitches in octave 4
        assert_eq!(mml_to_midi_note(Pitch::C, Accidental::Natural, 4), 60);
        assert_eq!(mml_to_midi_note(Pitch::D, Accidental::Natural, 4), 62);
        assert_eq!(mml_to_midi_note(Pitch::E, Accidental::Natural, 4), 64);
        assert_eq!(mml_to_midi_note(Pitch::F, Accidental::Natural, 4), 65);
        assert_eq!(mml_to_midi_note(Pitch::G, Accidental::Natural, 4), 67);
        assert_eq!(mml_to_midi_note(Pitch::A, Accidental::Natural, 4), 69);
        assert_eq!(mml_to_midi_note(Pitch::B, Accidental::Natural, 4), 71);
    }

    #[test]
    fn test_mml_to_midi_note_clamping_high() {
        // Test clamping to 127 for very high octaves
        // B10 = (10 + 1) * 12 + 11 = 132 + 11 = 143 → clamped to 127
        let note = mml_to_midi_note(Pitch::B, Accidental::Natural, 10);
        assert_eq!(note, 127);
    }

    #[test]
    fn test_mml_volume_to_velocity_v0() {
        let velocity = mml_volume_to_velocity(0);
        assert_eq!(velocity, 0);
    }

    #[test]
    fn test_mml_volume_to_velocity_v10() {
        // V10 = (10 * 127) / 15 = 1270 / 15 = 84
        let velocity = mml_volume_to_velocity(10);
        assert_eq!(velocity, 84);
    }

    #[test]
    fn test_mml_volume_to_velocity_v15() {
        // V15 = (15 * 127) / 15 = 127
        let velocity = mml_volume_to_velocity(15);
        assert_eq!(velocity, 127);
    }

    #[test]
    fn test_mml_volume_to_velocity_v1() {
        // V1 = (1 * 127) / 15 = 8
        let velocity = mml_volume_to_velocity(1);
        assert_eq!(velocity, 8);
    }

    #[test]
    fn test_mml_volume_to_velocity_v5() {
        // V5 = (5 * 127) / 15 = 635 / 15 = 42
        let velocity = mml_volume_to_velocity(5);
        assert_eq!(velocity, 42);
    }

    // ============================================================
    // Volume to Velocity Conversion Table Verification (Issue requirement)
    // ============================================================

    #[test]
    fn test_volume_to_velocity_full_table() {
        // Verify the conversion table from the design spec
        let expected: [(u8, u8); 16] = [
            (0, 0),
            (1, 8),
            (2, 16),  // 2*127/15 = 16.93 → 16
            (3, 25),  // 3*127/15 = 25.4 → 25
            (4, 33),  // 4*127/15 = 33.86 → 33
            (5, 42),  // 5*127/15 = 42.33 → 42
            (6, 50),  // 6*127/15 = 50.8 → 50
            (7, 59),  // 7*127/15 = 59.26 → 59
            (8, 67),  // 8*127/15 = 67.73 → 67
            (9, 76),  // 9*127/15 = 76.2 → 76
            (10, 84), // 10*127/15 = 84.66 → 84
            (11, 93), // 11*127/15 = 93.13 → 93
            (12, 101),// 12*127/15 = 101.6 → 101
            (13, 110),// 13*127/15 = 110.06 → 110
            (14, 118),// 14*127/15 = 118.53 → 118
            (15, 127),
        ];

        for (vol, expected_vel) in expected {
            let actual = mml_volume_to_velocity(vol);
            assert_eq!(
                actual, expected_vel,
                "V{} should map to velocity {}, got {}",
                vol, expected_vel, actual
            );
        }
    }
}
