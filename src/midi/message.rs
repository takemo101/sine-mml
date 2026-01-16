//! MIDI message types and conversion functions
//!
//! This module defines MIDI message structures for note and control events,
//! and provides conversion functions between MML and MIDI formats.

use midir::MidiOutputConnection;

use super::error::MidiError;
use crate::mml::{Accidental, Pitch};

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
