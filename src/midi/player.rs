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
pub struct PlaybackState {
    pub octave: u8,
    pub bpm: u16,
    pub default_length: u8,
    pub volume: u8,
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
    /// Update state with a command
    pub fn update_state(&mut self, command: &Command) {
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

/// MMLコマンド列の全体再生時間を計算する（ミリ秒）
///
/// # Arguments
/// * `commands` - MMLコマンドのスライス
///
/// # Returns
/// 全体の再生時間（ミリ秒）
#[must_use]
pub fn calculate_total_duration_ms(commands: &[Command]) -> u64 {
    let mut state = PlaybackState::default();
    let total_secs = calculate_duration_recursive(commands, &mut state);

    // f32 -> u64変換: 音声時間は現実的に2^64ms（約584,942,417年）を超えない
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_ms = (total_secs * 1000.0) as u64;
    total_ms
}

fn calculate_duration_recursive(commands: &[Command], state: &mut PlaybackState) -> f32 {
    let mut total = 0.0;

    for cmd in commands {
        state.update_state(cmd);

        match cmd {
            Command::Note(note) => {
                total += note.duration_in_seconds(state.bpm, state.default_length);
            }
            Command::Rest(rest) => {
                total += rest.duration_in_seconds(state.bpm, state.default_length);
            }
            Command::Tuplet {
                commands: _,
                count: _,
                base_duration,
            } => {
                let base_len = base_duration.unwrap_or(state.default_length);
                let beats_per_tuplet = 4.0 / f32::from(base_len);
                let seconds_per_tuplet = beats_per_tuplet * (60.0 / f32::from(state.bpm));
                total += seconds_per_tuplet;
            }
            // Note: Chordコマンドは現在のMMLパーサーではサポートされていないため処理を省略
            // MMLパーサーが[CEG]構文をChordとしてパースする実装が追加されたら以下を追加:
            // Command::Chord(chord) => {
            //     let max_duration = chord.notes.iter()
            //         .map(|n| n.duration_in_seconds(state.bpm, state.default_length))
            //         .fold(0.0_f32, f32::max);
            //     total += max_duration;
            // }
            Command::Loop {
                commands: loop_commands,
                escape_index,
                repeat_count,
            } => {
                // ループの全回数分の時間を計算
                // Note: stateはミュータブル参照で渡されるため、ループ内でのTempo/Octave等の
                // 状態変更は累積的に反映される（例: L[T60 C T120 C]2 で2回目はT120から開始）
                for i in 0..*repeat_count {
                    let is_last = i == repeat_count - 1;
                    let end_idx = if is_last {
                        escape_index.map_or(loop_commands.len(), |idx| idx)
                    } else {
                        loop_commands.len()
                    };
                    total += calculate_duration_recursive(&loop_commands[..end_idx], state);
                }
            }
            _ => {}
        }
    }

    total
}
