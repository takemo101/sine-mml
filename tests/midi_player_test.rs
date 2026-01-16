use sine_mml::midi::player::{calculate_total_duration_ms, PlaybackState};
use sine_mml::mml::{
    Accidental, Command, Duration as MmlDuration, Note, Pitch, Rest, TiedDuration, VolumeValue,
};

#[test]
fn test_playback_state_default() {
    let state = PlaybackState::default();
    assert_eq!(state.octave, 4);
    assert_eq!(state.bpm, 120);
    assert_eq!(state.default_length, 4);
    assert_eq!(state.volume, 10);
}

#[test]
fn test_playback_state_update_octave() {
    let mut state = PlaybackState::default();
    state.update_state(&Command::Octave(sine_mml::mml::Octave { value: 5 }));
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
    assert_eq!(state.octave, 8);
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
    assert_eq!(state.octave, 0);
}

#[test]
fn test_playback_state_update_tempo() {
    let mut state = PlaybackState::default();
    state.update_state(&Command::Tempo(sine_mml::mml::Tempo { value: 140 }));
    assert_eq!(state.bpm, 140);
}

#[test]
fn test_playback_state_update_default_length() {
    let mut state = PlaybackState::default();
    state.update_state(&Command::DefaultLength(sine_mml::mml::DefaultLength {
        value: 8,
    }));
    assert_eq!(state.default_length, 8);
}

#[test]
fn test_playback_state_update_volume_absolute() {
    let mut state = PlaybackState::default();
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Absolute(12),
    }));
    assert_eq!(state.volume, 12);
}

#[test]
fn test_playback_state_update_volume_relative_positive() {
    let mut state = PlaybackState::default();
    state.volume = 10;
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Relative(3),
    }));
    assert_eq!(state.volume, 13);
}

#[test]
fn test_playback_state_update_volume_relative_negative() {
    let mut state = PlaybackState::default();
    state.volume = 10;
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Relative(-3),
    }));
    assert_eq!(state.volume, 7);
}

#[test]
fn test_playback_state_update_volume_clamp_max() {
    let mut state = PlaybackState::default();
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Absolute(20),
    }));
    assert_eq!(state.volume, 15);
}

#[test]
fn test_playback_state_update_volume_relative_clamp_max() {
    let mut state = PlaybackState::default();
    state.volume = 14;
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Relative(5),
    }));
    assert_eq!(state.volume, 15);
}

#[test]
fn test_playback_state_update_volume_relative_clamp_min() {
    let mut state = PlaybackState::default();
    state.volume = 3;
    state.update_state(&Command::Volume(sine_mml::mml::Volume {
        value: VolumeValue::Relative(-10),
    }));
    assert_eq!(state.volume, 0);
}

#[test]
fn test_play_midi_stream_validates_channel() {
    use sine_mml::midi::error::MidiError;

    assert!(!MidiError::is_valid_channel(0));
    assert!(MidiError::validate_channel(0).is_err());

    assert!(!MidiError::is_valid_channel(17));
    assert!(MidiError::validate_channel(17).is_err());

    for ch in 1..=16 {
        assert!(MidiError::is_valid_channel(ch));
        assert!(MidiError::validate_channel(ch).is_ok());
    }
}

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
    let duration = note.duration_in_seconds(120, 4);
    assert!((duration - 0.5).abs() < 0.001);
}

#[test]
fn test_rest_duration_calculation() {
    let rest = make_rest(4);
    let duration = rest.duration_in_seconds(120, 4);
    assert!((duration - 0.5).abs() < 0.001);
}

#[test]
fn test_eighth_note_duration() {
    let note = make_note(Pitch::C, 8);
    let duration = note.duration_in_seconds(120, 4);
    assert!((duration - 0.25).abs() < 0.001);
}

#[test]
fn test_whole_note_duration() {
    let note = make_note(Pitch::C, 1);
    let duration = note.duration_in_seconds(120, 4);
    assert!((duration - 2.0).abs() < 0.001);
}

#[test]
fn test_tempo_affects_duration() {
    let note = make_note(Pitch::C, 4);
    let duration_60 = note.duration_in_seconds(60, 4);
    assert!((duration_60 - 1.0).abs() < 0.001);

    let duration_240 = note.duration_in_seconds(240, 4);
    assert!((duration_240 - 0.25).abs() < 0.001);
}

#[test]
fn test_calculate_total_duration_ms_simple() {
    use sine_mml::mml::parse;
    let ast = parse("T120 L4 CDEF").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 2000).abs() < 100,
        "Expected ~2000ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_with_rest() {
    use sine_mml::mml::parse;
    let ast = parse("T120 L4 CRC").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 1500).abs() < 100,
        "Expected ~1500ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_empty() {
    let duration = calculate_total_duration_ms(&[]);
    assert_eq!(duration, 0);
}

#[test]
fn test_calculate_total_duration_ms_tempo_change() {
    use sine_mml::mml::parse;
    let ast = parse("T60 L4 C T120 C").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 1500).abs() < 100,
        "Expected ~1500ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_tuplet() {
    use sine_mml::mml::parse;
    let ast = parse("T120 {CDE}4").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 500).abs() < 100,
        "Expected ~500ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_complex() {
    use sine_mml::mml::parse;
    let ast = parse("T120 L4 C D R {DEF}4").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 2000).abs() < 100,
        "Expected ~2000ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_with_loop() {
    use sine_mml::mml::parse;
    let ast = parse("T120 L4 [C]3").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 1500).abs() < 100,
        "Expected ~1500ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_loop_with_tempo_change() {
    use sine_mml::mml::parse;
    let ast = parse("L4 [T60 C T120 C]2").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 3000).abs() < 150,
        "Expected ~3000ms, got {duration}ms"
    );
}

#[test]
fn test_calculate_total_duration_ms_loop_with_escape() {
    use sine_mml::mml::parse;
    let ast = parse("T120 L4 [C D :C]2").unwrap();
    let duration = calculate_total_duration_ms(&ast.commands);
    assert!(
        (duration as i64 - 2500).abs() < 150,
        "Expected ~2500ms, got {duration}ms"
    );
}
