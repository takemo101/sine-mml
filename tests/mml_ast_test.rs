use sine_mml::mml::{Accidental, Command, Duration, Mml, Note, Pitch, Rest, Tempo, TiedDuration};

#[test]
fn pitch_from_char_valid() {
    assert_eq!(Pitch::from_char('C'), Some(Pitch::C));
    assert_eq!(Pitch::from_char('c'), Some(Pitch::C));
    assert_eq!(Pitch::from_char('G'), Some(Pitch::G));
}

#[test]
fn pitch_from_char_invalid() {
    assert_eq!(Pitch::from_char('X'), None);
    assert_eq!(Pitch::from_char('H'), None);
}

#[test]
fn note_to_midi_c4_equals_60() {
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(None, 0)),
    };
    assert_eq!(note.to_midi_note(4), 60);
}

#[test]
fn note_to_midi_a4_equals_69() {
    let note = Note {
        pitch: Pitch::A,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(None, 0)),
    };
    assert_eq!(note.to_midi_note(4), 69);
}

#[test]
fn note_duration_quarter_at_120bpm() {
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(Some(4), 0)),
    };
    let duration = note.duration_in_seconds(120, 4);
    assert!((duration - 0.5).abs() < 0.001);
}

#[test]
fn note_duration_dotted() {
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(Some(4), 1)),
    };
    let duration = note.duration_in_seconds(120, 4);
    assert!((duration - 0.75).abs() < 0.001);
}

#[test]
fn get_tempo_with_tempo_command() {
    let mml = Mml {
        commands: vec![
            Command::Tempo(Tempo { value: 180 }),
            Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(Some(4), 0)),
            }),
        ],
    };
    assert_eq!(mml.get_tempo(), 180);
}

#[test]
fn get_tempo_without_tempo_command_returns_default() {
    let mml = Mml {
        commands: vec![Command::Note(Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: TiedDuration::new(Duration::new(Some(4), 0)),
        })],
    };
    assert_eq!(mml.get_tempo(), 120);
}

// TiedDuration tests
#[test]
fn tied_duration_new() {
    let duration = TiedDuration::new(Duration::new(Some(4), 0));
    assert_eq!(duration.base.value, Some(4));
    assert_eq!(duration.base.dots, 0);
    assert!(duration.tied.is_empty());
}

#[test]
fn tied_duration_add_tie() {
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    duration.add_tie(Duration::new(Some(8), 0));
    assert_eq!(duration.tied.len(), 1);
    assert_eq!(duration.tied[0].value, Some(8));
}

#[test]
fn tied_duration_has_ties() {
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    assert!(!duration.has_ties());
    duration.add_tie(Duration::new(Some(8), 0));
    assert!(duration.has_ties());
}

#[test]
fn tied_duration_total_duration_simple() {
    // C4&8 at 120 BPM: 4分音符(0.5s) + 8分音符(0.25s) = 0.75s
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    duration.add_tie(Duration::new(Some(8), 0));
    let total = duration.total_duration_in_seconds(120, 4);
    assert!((total - 0.75).abs() < 0.001);
}

#[test]
fn tied_duration_total_duration_multiple_ties() {
    // C4&8&16 at 120 BPM: 4分音符(0.5s) + 8分音符(0.25s) + 16分音符(0.125s) = 0.875s
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    duration.add_tie(Duration::new(Some(8), 0));
    duration.add_tie(Duration::new(Some(16), 0));
    let total = duration.total_duration_in_seconds(120, 4);
    assert!((total - 0.875).abs() < 0.001);
}

#[test]
fn duration_in_seconds_with_default() {
    // デフォルト音長が4の場合、Noneは4分音符として扱われる
    let duration = Duration::new(None, 0);
    let seconds = duration.duration_in_seconds(120, 4);
    assert!((seconds - 0.5).abs() < 0.001);
}

#[test]
fn duration_in_seconds_with_dots() {
    // 4分付点音符 at 120 BPM: 0.5s * 1.5 = 0.75s
    let duration = Duration::new(Some(4), 1);
    let seconds = duration.duration_in_seconds(120, 4);
    assert!((seconds - 0.75).abs() < 0.001);
}

// Duration.to_beats() tests (Issue #127)
#[test]
fn duration_to_beats_quarter_note() {
    // 4分音符 = 1拍
    let duration = Duration::new(Some(4), 0);
    let beats = duration.to_beats(4);
    assert!((beats - 1.0).abs() < 0.001);
}

#[test]
fn duration_to_beats_half_note() {
    // 2分音符 = 2拍
    let duration = Duration::new(Some(2), 0);
    let beats = duration.to_beats(4);
    assert!((beats - 2.0).abs() < 0.001);
}

#[test]
fn duration_to_beats_eighth_note() {
    // 8分音符 = 0.5拍
    let duration = Duration::new(Some(8), 0);
    let beats = duration.to_beats(4);
    assert!((beats - 0.5).abs() < 0.001);
}

#[test]
fn duration_to_beats_sixteenth_note() {
    // 16分音符 = 0.25拍
    let duration = Duration::new(Some(16), 0);
    let beats = duration.to_beats(4);
    assert!((beats - 0.25).abs() < 0.001);
}

#[test]
fn duration_to_beats_dotted_quarter() {
    // 4分付点音符 = 1.5拍
    let duration = Duration::new(Some(4), 1);
    let beats = duration.to_beats(4);
    assert!((beats - 1.5).abs() < 0.001);
}

#[test]
fn duration_to_beats_double_dotted() {
    // 4分複付点音符 = 1.75拍
    let duration = Duration::new(Some(4), 2);
    let beats = duration.to_beats(4);
    assert!((beats - 1.75).abs() < 0.001);
}

#[test]
fn duration_to_beats_with_default_length() {
    // デフォルト音長が8の場合、Noneは8分音符 = 0.5拍
    let duration = Duration::new(None, 0);
    let beats = duration.to_beats(8);
    assert!((beats - 0.5).abs() < 0.001);
}

// TiedDuration.total_beats() tests (Issue #127)
#[test]
fn tied_duration_total_beats_simple() {
    // C4&8: 4分音符(1拍) + 8分音符(0.5拍) = 1.5拍
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    duration.add_tie(Duration::new(Some(8), 0));
    let total = duration.total_beats(4);
    assert!((total - 1.5).abs() < 0.001);
}

#[test]
fn tied_duration_total_beats_multiple_ties() {
    // C4&8&16: 4分音符(1拍) + 8分音符(0.5拍) + 16分音符(0.25拍) = 1.75拍
    let mut duration = TiedDuration::new(Duration::new(Some(4), 0));
    duration.add_tie(Duration::new(Some(8), 0));
    duration.add_tie(Duration::new(Some(16), 0));
    let total = duration.total_beats(4);
    assert!((total - 1.75).abs() < 0.001);
}

#[test]
fn tied_duration_total_beats_with_dotted() {
    // C4.&8: 4分付点音符(1.5拍) + 8分音符(0.5拍) = 2.0拍
    let mut duration = TiedDuration::new(Duration::new(Some(4), 1));
    duration.add_tie(Duration::new(Some(8), 0));
    let total = duration.total_beats(4);
    assert!((total - 2.0).abs() < 0.001);
}

#[test]
fn tied_duration_total_beats_no_ties() {
    // C4: 4分音符(1拍)
    let duration = TiedDuration::new(Duration::new(Some(4), 0));
    let total = duration.total_beats(4);
    assert!((total - 1.0).abs() < 0.001);
}

#[test]
fn tied_duration_total_beats_whole_note() {
    // C1&2: 全音符(4拍) + 2分音符(2拍) = 6拍
    let mut duration = TiedDuration::new(Duration::new(Some(1), 0));
    duration.add_tie(Duration::new(Some(2), 0));
    let total = duration.total_beats(4);
    assert!((total - 6.0).abs() < 0.001);
}

// Note.total_beats() tests (Issue #128)
#[test]
fn note_total_beats_quarter() {
    // C4 = 1拍
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(Some(4), 0)),
    };
    let beats = note.total_beats(4);
    assert!((beats - 1.0).abs() < 0.001);
}

#[test]
fn note_total_beats_with_tie() {
    // C4&8 = 1.5拍
    let mut tied = TiedDuration::new(Duration::new(Some(4), 0));
    tied.add_tie(Duration::new(Some(8), 0));
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: tied,
    };
    let beats = note.total_beats(4);
    assert!((beats - 1.5).abs() < 0.001);
}

#[test]
fn note_total_beats_dotted() {
    // C4. = 1.5拍
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(Some(4), 1)),
    };
    let beats = note.total_beats(4);
    assert!((beats - 1.5).abs() < 0.001);
}

#[test]
fn note_total_beats_default_duration() {
    // C (default=8) = 0.5拍
    let note = Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(None, 0)),
    };
    let beats = note.total_beats(8);
    assert!((beats - 0.5).abs() < 0.001);
}

// Rest.total_beats() tests (Issue #128)
#[test]
fn rest_total_beats_quarter() {
    // R4 = 1拍
    let rest = Rest {
        duration: TiedDuration::new(Duration::new(Some(4), 0)),
    };
    let beats = rest.total_beats(4);
    assert!((beats - 1.0).abs() < 0.001);
}

#[test]
fn rest_total_beats_with_tie() {
    // R4&8 = 1.5拍
    let mut tied = TiedDuration::new(Duration::new(Some(4), 0));
    tied.add_tie(Duration::new(Some(8), 0));
    let rest = Rest { duration: tied };
    let beats = rest.total_beats(4);
    assert!((beats - 1.5).abs() < 0.001);
}

#[test]
fn rest_total_beats_dotted() {
    // R4. = 1.5拍
    let rest = Rest {
        duration: TiedDuration::new(Duration::new(Some(4), 1)),
    };
    let beats = rest.total_beats(4);
    assert!((beats - 1.5).abs() < 0.001);
}

#[test]
fn rest_total_beats_default_duration() {
    // R (default=8) = 0.5拍
    let rest = Rest {
        duration: TiedDuration::new(Duration::new(None, 0)),
    };
    let beats = rest.total_beats(8);
    assert!((beats - 0.5).abs() < 0.001);
}

// ============================================================
// Command::Tuplet テスト（Issue #143）
// ============================================================

/// TC-TUP-AST-001: 基本的な連符コマンド生成
#[test]
fn tuplet_command_basic() {
    // {CDE}3 - 3連符
    let tuplet = Command::Tuplet {
        commands: vec![
            Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::D,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::E,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
        ],
        count: 3,
        base_duration: None,
    };

    if let Command::Tuplet {
        commands,
        count,
        base_duration,
    } = tuplet
    {
        assert_eq!(commands.len(), 3);
        assert_eq!(count, 3);
        assert!(base_duration.is_none());
    } else {
        panic!("Expected Command::Tuplet");
    }
}

/// TC-TUP-AST-002: ベース音長指定付き連符
#[test]
fn tuplet_command_with_base_duration() {
    // {CDE}3:2 - 2分音符ベースの3連符
    let tuplet = Command::Tuplet {
        commands: vec![Command::Note(Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: TiedDuration::new(Duration::new(None, 0)),
        })],
        count: 3,
        base_duration: Some(2),
    };

    if let Command::Tuplet {
        base_duration,
        count,
        ..
    } = tuplet
    {
        assert_eq!(count, 3);
        assert_eq!(base_duration, Some(2));
    } else {
        panic!("Expected Command::Tuplet");
    }
}

/// TC-TUP-AST-003: 連符内に休符を含む
#[test]
fn tuplet_command_with_rest() {
    // {CRE}3 - 休符を含む3連符
    let tuplet = Command::Tuplet {
        commands: vec![
            Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Rest(Rest {
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::E,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
        ],
        count: 3,
        base_duration: None,
    };

    if let Command::Tuplet { commands, .. } = tuplet {
        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[1], Command::Rest(_)));
    } else {
        panic!("Expected Command::Tuplet");
    }
}

/// TC-TUP-AST-004: ネストした連符
#[test]
fn tuplet_command_nested() {
    // {{CDE}3 FG}5 - ネストした連符
    let inner_tuplet = Command::Tuplet {
        commands: vec![
            Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::D,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::E,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
        ],
        count: 3,
        base_duration: None,
    };

    let outer_tuplet = Command::Tuplet {
        commands: vec![
            inner_tuplet,
            Command::Note(Note {
                pitch: Pitch::F,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
            Command::Note(Note {
                pitch: Pitch::G,
                accidental: Accidental::Natural,
                duration: TiedDuration::new(Duration::new(None, 0)),
            }),
        ],
        count: 5,
        base_duration: None,
    };

    if let Command::Tuplet {
        commands, count, ..
    } = outer_tuplet
    {
        assert_eq!(count, 5);
        assert_eq!(commands.len(), 3);
        // 最初の要素が内側の連符であることを確認
        assert!(matches!(commands[0], Command::Tuplet { .. }));
    } else {
        panic!("Expected Command::Tuplet");
    }
}

/// TC-TUP-AST-005: `Command::Tuplet` は `Clone` と `PartialEq` を実装
#[test]
fn tuplet_command_clone_and_eq() {
    let tuplet = Command::Tuplet {
        commands: vec![Command::Note(Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: TiedDuration::new(Duration::new(Some(4), 0)),
        })],
        count: 3,
        base_duration: Some(4),
    };

    let cloned = tuplet.clone();
    assert_eq!(tuplet, cloned);
}
