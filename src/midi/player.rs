//! MIDI player module for real-time MIDI streaming
//!
//! This module provides drift-free MIDI playback using the Next Event Time method.
//! It uses absolute time calculations to prevent cumulative timing drift during
//! long playback sessions.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use midir::MidiOutputConnection;

use super::error::MidiError;
use super::message::{
    mml_to_midi_note, mml_volume_to_velocity, send_all_notes_off, send_note_off, send_note_on,
};
use crate::mml::{Command, VolumeValue};

/// Default values for MIDI playback state
const DEFAULT_OCTAVE: u8 = 4;
const DEFAULT_BPM: u16 = 120;
const DEFAULT_LENGTH: u8 = 4;
const DEFAULT_VOLUME: u8 = 10;

/// MIDI playback state
struct PlaybackState {
    octave: u8,
    bpm: u16,
    default_length: u8,
    volume: u8,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            octave: DEFAULT_OCTAVE,
            bpm: DEFAULT_BPM,
            default_length: DEFAULT_LENGTH,
            volume: DEFAULT_VOLUME,
        }
    }
}

impl PlaybackState {
    /// Update state with a command and return whether to continue processing
    fn update_state(&mut self, command: &Command) {
        match command {
            Command::Octave(o) => {
                self.octave = o.value;
            }
            Command::OctaveUp => {
                if self.octave < 8 {
                    self.octave += 1;
                }
            }
            Command::OctaveDown => {
                if self.octave > 0 {
                    self.octave -= 1;
                }
            }
            Command::Tempo(t) => {
                self.bpm = t.value;
            }
            Command::DefaultLength(l) => {
                self.default_length = l.value;
            }
            Command::Volume(v) => {
                self.volume = match v.value {
                    VolumeValue::Absolute(val) => val.min(15),
                    #[allow(clippy::cast_sign_loss)]
                    VolumeValue::Relative(delta) => {
                        let new_vol = i16::from(self.volume) + i16::from(delta);
                        new_vol.clamp(0, 15) as u8
                    }
                };
            }
            _ => {}
        }
    }
}

/// Play MIDI stream from MML commands.
///
/// This function implements the Next Event Time method for drift-free timing.
/// It calculates absolute target times for each note event, preventing
/// cumulative timing drift during long playback sessions.
///
/// # Arguments
/// * `conn` - MIDI output connection
/// * `commands` - Slice of MML commands to play
/// * `channel` - MIDI channel (1-16)
///
/// # Errors
/// Returns `MidiError::InvalidChannel` if channel is not in 1-16 range.
/// Returns `MidiError::SendFailed` if sending MIDI messages fails.
///
/// # Example
/// ```ignore
/// let commands = parse("T120 CDEFGAB").unwrap();
/// play_midi_stream(&mut conn, &commands.commands, 1)?;
/// ```
pub fn play_midi_stream(
    conn: &mut MidiOutputConnection,
    commands: &[Command],
    channel: u8,
) -> Result<(), MidiError> {
    MidiError::validate_channel(channel)?;

    let mut state = PlaybackState::default();
    let start_time = Instant::now();
    let mut elapsed_duration = Duration::ZERO;

    play_commands_recursive(
        conn,
        commands,
        channel,
        &mut state,
        start_time,
        &mut elapsed_duration,
        None,
    )?;

    // Send All Notes Off for cleanup
    send_all_notes_off(conn, channel)?;

    Ok(())
}

/// Play MIDI stream with interrupt support.
///
/// This function is similar to `play_midi_stream` but supports early termination
/// via an `AtomicBool` flag. This is useful for handling Ctrl+C interrupts.
///
/// When interrupted, the function sends All Notes Off to prevent stuck notes.
///
/// # Arguments
/// * `conn` - MIDI output connection
/// * `commands` - Slice of MML commands to play
/// * `channel` - MIDI channel (1-16)
/// * `interrupt` - Atomic flag for interrupt signaling
///
/// # Errors
/// Returns `MidiError::InvalidChannel` if channel is not in 1-16 range.
/// Returns `MidiError::SendFailed` if sending MIDI messages fails.
///
/// # Example
/// ```ignore
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
///
/// let interrupt = Arc::new(AtomicBool::new(false));
/// let interrupt_clone = interrupt.clone();
///
/// // Set up Ctrl+C handler
/// ctrlc::set_handler(move || {
///     interrupt_clone.store(true, Ordering::SeqCst);
/// }).unwrap();
///
/// let commands = parse("T120 CDEFGAB").unwrap();
/// play_midi_stream_interruptible(&mut conn, &commands.commands, 1, interrupt)?;
/// ```
pub fn play_midi_stream_interruptible(
    conn: &mut MidiOutputConnection,
    commands: &[Command],
    channel: u8,
    interrupt: &Arc<AtomicBool>,
) -> Result<(), MidiError> {
    MidiError::validate_channel(channel)?;

    let mut state = PlaybackState::default();
    let start_time = Instant::now();
    let mut elapsed_duration = Duration::ZERO;

    play_commands_recursive(
        conn,
        commands,
        channel,
        &mut state,
        start_time,
        &mut elapsed_duration,
        Some(interrupt),
    )?;

    send_all_notes_off(conn, channel)?;

    Ok(())
}

fn is_interrupted(interrupt: Option<&Arc<AtomicBool>>) -> bool {
    interrupt.is_some_and(|flag| flag.load(Ordering::Relaxed))
}

fn wait_until_target(start_time: Instant, elapsed: Duration) {
    let target_time = start_time + elapsed;
    let now = Instant::now();
    if target_time > now {
        std::thread::sleep(target_time - now);
    }
}

fn play_note(
    conn: &mut MidiOutputConnection,
    note: &crate::mml::Note,
    channel: u8,
    state: &PlaybackState,
    start_time: Instant,
    elapsed_duration: &mut Duration,
) -> Result<(), MidiError> {
    let midi_note = mml_to_midi_note(note.pitch, note.accidental, state.octave);
    let velocity = mml_volume_to_velocity(state.volume);

    send_note_on(conn, channel, midi_note, velocity)?;

    let note_duration_secs = note.duration_in_seconds(state.bpm, state.default_length);
    *elapsed_duration += Duration::from_secs_f32(note_duration_secs);
    wait_until_target(start_time, *elapsed_duration);

    send_note_off(conn, channel, midi_note)
}

fn play_rest(
    rest: &crate::mml::Rest,
    state: &PlaybackState,
    start_time: Instant,
    elapsed_duration: &mut Duration,
) {
    let rest_duration_secs = rest.duration_in_seconds(state.bpm, state.default_length);
    *elapsed_duration += Duration::from_secs_f32(rest_duration_secs);
    wait_until_target(start_time, *elapsed_duration);
}

#[allow(clippy::too_many_arguments)]
fn play_tuplet(
    conn: &mut MidiOutputConnection,
    tuplet_commands: &[Command],
    count: u8,
    base_duration: Option<u8>,
    channel: u8,
    state: &mut PlaybackState,
    start_time: Instant,
    elapsed_duration: &mut Duration,
    interrupt: Option<&Arc<AtomicBool>>,
) -> Result<bool, MidiError> {
    let base_len = base_duration.unwrap_or(state.default_length);
    let beats_per_tuplet = 4.0 / f32::from(base_len);
    let seconds_per_tuplet = beats_per_tuplet * (60.0 / f32::from(state.bpm));
    let duration_per_note = seconds_per_tuplet / f32::from(count);

    for tuplet_cmd in tuplet_commands {
        if is_interrupted(interrupt) {
            return Ok(false);
        }

        match tuplet_cmd {
            Command::Note(note) => {
                let midi_note = mml_to_midi_note(note.pitch, note.accidental, state.octave);
                let velocity = mml_volume_to_velocity(state.volume);

                send_note_on(conn, channel, midi_note, velocity)?;
                *elapsed_duration += Duration::from_secs_f32(duration_per_note);
                wait_until_target(start_time, *elapsed_duration);
                send_note_off(conn, channel, midi_note)?;
            }
            Command::Rest(_) => {
                *elapsed_duration += Duration::from_secs_f32(duration_per_note);
                wait_until_target(start_time, *elapsed_duration);
            }
            _ => {
                state.update_state(tuplet_cmd);
            }
        }
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn play_commands_recursive(
    conn: &mut MidiOutputConnection,
    commands: &[Command],
    channel: u8,
    state: &mut PlaybackState,
    start_time: Instant,
    elapsed_duration: &mut Duration,
    interrupt: Option<&Arc<AtomicBool>>,
) -> Result<bool, MidiError> {
    for command in commands {
        if is_interrupted(interrupt) {
            return Ok(false);
        }

        match command {
            Command::Note(note) => {
                play_note(conn, note, channel, state, start_time, elapsed_duration)?;
            }
            Command::Rest(rest) => {
                play_rest(rest, state, start_time, elapsed_duration);
            }
            Command::Loop {
                commands: loop_commands,
                escape_index,
                repeat_count,
            } => {
                for i in 0..*repeat_count {
                    let is_last = i == repeat_count - 1;
                    let end_idx = if is_last {
                        escape_index.map_or(loop_commands.len(), |idx| idx)
                    } else {
                        loop_commands.len()
                    };

                    if !play_commands_recursive(
                        conn,
                        &loop_commands[..end_idx],
                        channel,
                        state,
                        start_time,
                        elapsed_duration,
                        interrupt,
                    )? {
                        return Ok(false);
                    }
                }
            }
            Command::Tuplet {
                commands: tuplet_commands,
                count,
                base_duration,
            } => {
                if !play_tuplet(
                    conn,
                    tuplet_commands,
                    *count,
                    *base_duration,
                    channel,
                    state,
                    start_time,
                    elapsed_duration,
                    interrupt,
                )? {
                    return Ok(false);
                }
            }
            _ => {
                state.update_state(command);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::mml::{Accidental, Duration as MmlDuration, Note, Pitch, Rest, TiedDuration};

    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert_eq!(state.octave, DEFAULT_OCTAVE);
        assert_eq!(state.bpm, DEFAULT_BPM);
        assert_eq!(state.default_length, DEFAULT_LENGTH);
        assert_eq!(state.volume, DEFAULT_VOLUME);
    }

    #[test]
    fn test_playback_state_update_octave() {
        let mut state = PlaybackState::default();
        state.update_state(&Command::Octave(crate::mml::Octave { value: 5 }));
        assert_eq!(state.octave, 5);
    }

    #[test]
    fn test_playback_state_update_octave_up() {
        let mut state = PlaybackState::default();
        state.octave = 4;
        state.update_state(&Command::OctaveUp);
        assert_eq!(state.octave, 5);
    }

    #[test]
    fn test_playback_state_update_octave_up_max() {
        let mut state = PlaybackState::default();
        state.octave = 8;
        state.update_state(&Command::OctaveUp);
        assert_eq!(state.octave, 8); // Should not exceed 8
    }

    #[test]
    fn test_playback_state_update_octave_down() {
        let mut state = PlaybackState::default();
        state.octave = 4;
        state.update_state(&Command::OctaveDown);
        assert_eq!(state.octave, 3);
    }

    #[test]
    fn test_playback_state_update_octave_down_min() {
        let mut state = PlaybackState::default();
        state.octave = 0;
        state.update_state(&Command::OctaveDown);
        assert_eq!(state.octave, 0); // Should not go below 0
    }

    #[test]
    fn test_playback_state_update_tempo() {
        let mut state = PlaybackState::default();
        state.update_state(&Command::Tempo(crate::mml::Tempo { value: 140 }));
        assert_eq!(state.bpm, 140);
    }

    #[test]
    fn test_playback_state_update_default_length() {
        let mut state = PlaybackState::default();
        state.update_state(&Command::DefaultLength(crate::mml::DefaultLength {
            value: 8,
        }));
        assert_eq!(state.default_length, 8);
    }

    #[test]
    fn test_playback_state_update_volume_absolute() {
        let mut state = PlaybackState::default();
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Absolute(12),
        }));
        assert_eq!(state.volume, 12);
    }

    #[test]
    fn test_playback_state_update_volume_relative_positive() {
        let mut state = PlaybackState::default();
        state.volume = 10;
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Relative(3),
        }));
        assert_eq!(state.volume, 13);
    }

    #[test]
    fn test_playback_state_update_volume_relative_negative() {
        let mut state = PlaybackState::default();
        state.volume = 10;
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Relative(-3),
        }));
        assert_eq!(state.volume, 7);
    }

    #[test]
    fn test_playback_state_update_volume_clamp_max() {
        let mut state = PlaybackState::default();
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Absolute(20),
        }));
        assert_eq!(state.volume, 15);
    }

    #[test]
    fn test_playback_state_update_volume_relative_clamp_max() {
        let mut state = PlaybackState::default();
        state.volume = 14;
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Relative(5),
        }));
        assert_eq!(state.volume, 15); // Clamped to max 15
    }

    #[test]
    fn test_playback_state_update_volume_relative_clamp_min() {
        let mut state = PlaybackState::default();
        state.volume = 3;
        state.update_state(&Command::Volume(crate::mml::Volume {
            value: VolumeValue::Relative(-10),
        }));
        assert_eq!(state.volume, 0); // Clamped to min 0
    }

    // ============================================================
    // Integration-like tests (without actual MIDI device)
    // ============================================================

    #[test]
    fn test_play_midi_stream_validates_channel() {
        // We can't test actual playback without a MIDI device,
        // but we can test that channel validation works
        // by checking that invalid channels cause the expected error type

        // Channel 0 should be invalid
        assert!(!MidiError::is_valid_channel(0));
        assert!(MidiError::validate_channel(0).is_err());

        // Channel 17 should be invalid
        assert!(!MidiError::is_valid_channel(17));
        assert!(MidiError::validate_channel(17).is_err());

        // Channels 1-16 should be valid
        for ch in 1..=16 {
            assert!(MidiError::is_valid_channel(ch));
            assert!(MidiError::validate_channel(ch).is_ok());
        }
    }

    // ============================================================
    // Duration calculation tests
    // ============================================================

    fn make_note(pitch: Pitch, duration: u8) -> Note {
        Note {
            pitch,
            accidental: Accidental::Natural,
            duration: TiedDuration {
                base: MmlDuration {
                    value: Some(duration),
                    dots: 0,
                },
                tied: vec![],
            },
        }
    }

    fn make_rest(duration: u8) -> Rest {
        Rest {
            duration: TiedDuration {
                base: MmlDuration {
                    value: Some(duration),
                    dots: 0,
                },
                tied: vec![],
            },
        }
    }

    #[test]
    fn test_note_duration_calculation() {
        let note = make_note(Pitch::C, 4);
        // At 120 BPM, quarter note = 0.5 seconds
        let duration = note.duration_in_seconds(120, 4);
        assert!((duration - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_rest_duration_calculation() {
        let rest = make_rest(4);
        // At 120 BPM, quarter rest = 0.5 seconds
        let duration = rest.duration_in_seconds(120, 4);
        assert!((duration - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_eighth_note_duration() {
        let note = make_note(Pitch::C, 8);
        // At 120 BPM, eighth note = 0.25 seconds
        let duration = note.duration_in_seconds(120, 4);
        assert!((duration - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_whole_note_duration() {
        let note = make_note(Pitch::C, 1);
        // At 120 BPM, whole note = 2.0 seconds
        let duration = note.duration_in_seconds(120, 4);
        assert!((duration - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_tempo_affects_duration() {
        let note = make_note(Pitch::C, 4);
        // At 60 BPM, quarter note = 1.0 second
        let duration_60 = note.duration_in_seconds(60, 4);
        assert!((duration_60 - 1.0).abs() < 0.001);

        // At 240 BPM, quarter note = 0.25 seconds
        let duration_240 = note.duration_in_seconds(240, 4);
        assert!((duration_240 - 0.25).abs() < 0.001);
    }
}
