//! MML Parser Tests
//!
//! This module contains integration tests for the MML parser.
//! Tests were extracted from src/mml/parser.rs for better organization.

use sine_mml::mml::parser::{expand_loop, parse, Parser};
use sine_mml::mml::{
    Accidental, Command, Duration, Note, ParseError, Pitch, TiedDuration, Token, Volume,
    VolumeValue,
};

#[test]
fn parse_single_note() {
    let input = "C";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.accidental, Accidental::Natural);
            assert_eq!(n.duration.base.value, None);
            assert_eq!(n.duration.base.dots, 0);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_with_sharp() {
    let input = "C#";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.accidental, Accidental::Sharp);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_with_duration() {
    let input = "C4";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_dotted_note() {
    let input = "C4.";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.base.dots, 1);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_rest() {
    let input = "R4";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Rest(r) => {
            assert_eq!(r.duration.base.value, Some(4));
            assert_eq!(r.duration.base.dots, 0);
        }
        _ => panic!("Expected Rest"),
    }
}

#[test]
fn parse_octave() {
    let input = "O5";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Octave(o) => {
            assert_eq!(o.value, 5);
        }
        _ => panic!("Expected Octave"),
    }
}

#[test]
fn parse_tempo() {
    let input = "T120";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tempo(t) => {
            assert_eq!(t.value, 120);
        }
        _ => panic!("Expected Tempo"),
    }
}

#[test]
fn parse_complex_mml() {
    let input = "O4L4T120 C D E F G";
    let mml = parse(input).unwrap();
    // O4, L4, T120, C, D, E, F, G -> 8 commands
    assert_eq!(mml.commands.len(), 8);
}

#[test]
fn parse_empty_input() {
    let err = parse("").unwrap_err();
    assert!(matches!(err, ParseError::EmptyInput));
}

#[test]
fn parse_whitespace_only() {
    let mml = parse("   ").unwrap();
    assert_eq!(mml.commands.len(), 0);
}

#[test]
fn parse_invalid_number_range() {
    let err = parse("O9").unwrap_err();
    match err {
        ParseError::InvalidNumber { value, range, .. } => {
            assert_eq!(value, 9);
            assert_eq!(range, (1, 8));
        }
        _ => panic!("Expected InvalidNumber"),
    }
}

#[test]
fn parse_octave_up() {
    let mml = parse(">").unwrap();
    assert_eq!(mml.commands.len(), 1);
    assert!(matches!(mml.commands[0], Command::OctaveUp));
}

#[test]
fn parse_octave_down() {
    let mml = parse("<").unwrap();
    assert_eq!(mml.commands.len(), 1);
    assert!(matches!(mml.commands[0], Command::OctaveDown));
}

#[test]
fn parse_octave_change_with_notes() {
    let mml = parse("C >C <C").unwrap();
    assert_eq!(mml.commands.len(), 5);
    assert!(matches!(mml.commands[0], Command::Note(_)));
    assert!(matches!(mml.commands[1], Command::OctaveUp));
    assert!(matches!(mml.commands[2], Command::Note(_)));
    assert!(matches!(mml.commands[3], Command::OctaveDown));
    assert!(matches!(mml.commands[4], Command::Note(_)));
}

#[test]
fn parse_basic_loop_3_times() {
    let mml = parse("[CDEF]3").unwrap();
    assert_eq!(mml.commands.len(), 12);
    for i in 0..3 {
        let base = i * 4;
        assert!(matches!(&mml.commands[base], Command::Note(n) if n.pitch == Pitch::C));
        assert!(matches!(&mml.commands[base + 1], Command::Note(n) if n.pitch == Pitch::D));
        assert!(matches!(&mml.commands[base + 2], Command::Note(n) if n.pitch == Pitch::E));
        assert!(matches!(&mml.commands[base + 3], Command::Note(n) if n.pitch == Pitch::F));
    }
}

#[test]
fn parse_loop_with_escape_point() {
    let mml = parse("[CD:EF]2").unwrap();
    assert_eq!(mml.commands.len(), 6);
    assert!(matches!(&mml.commands[0], Command::Note(n) if n.pitch == Pitch::C));
    assert!(matches!(&mml.commands[1], Command::Note(n) if n.pitch == Pitch::D));
    assert!(matches!(&mml.commands[2], Command::Note(n) if n.pitch == Pitch::E));
    assert!(matches!(&mml.commands[3], Command::Note(n) if n.pitch == Pitch::F));
    assert!(matches!(&mml.commands[4], Command::Note(n) if n.pitch == Pitch::C));
    assert!(matches!(&mml.commands[5], Command::Note(n) if n.pitch == Pitch::D));
}

#[test]
fn parse_loop_default_count() {
    let mml = parse("[CDEF]").unwrap();
    assert_eq!(mml.commands.len(), 4);
}

#[test]
fn parse_loop_count_1() {
    let mml = parse("[CDEF]1").unwrap();
    assert_eq!(mml.commands.len(), 4);
}

#[test]
fn parse_loop_count_99() {
    let mml = parse("[C]99").unwrap();
    assert_eq!(mml.commands.len(), 99);
}

#[test]
fn parse_loop_count_0_error() {
    let err = parse("[CDEF]0").unwrap_err();
    assert!(matches!(
        err,
        ParseError::InvalidLoopCount {
            value: 0,
            range: (1, 99),
            ..
        }
    ));
}

#[test]
fn parse_loop_count_100_error() {
    let err = parse("[CDEF]100").unwrap_err();
    assert!(matches!(
        err,
        ParseError::InvalidLoopCount {
            value: 100,
            range: (1, 99),
            ..
        }
    ));
}

#[test]
fn parse_nested_loop_allowed() {
    // ネストしたループは許可されるようになった (Issue #93)
    let mml = parse("[[CDEF]2]3").unwrap();
    // 内側: CDEF × 2 = 8, 外側: 8 × 3 = 24
    assert_eq!(mml.commands.len(), 24);
}

#[test]
fn parse_unmatched_loop_start_error() {
    let err = parse("[CDEF").unwrap_err();
    assert!(matches!(err, ParseError::UnmatchedLoopStart { .. }));
}

#[test]
fn parse_unmatched_loop_end_error() {
    let err = parse("CDEF]").unwrap_err();
    assert!(matches!(err, ParseError::UnmatchedLoopEnd { .. }));
}

#[test]
fn parse_loop_escape_outside_loop_error() {
    let err = parse("CDEF:GAB").unwrap_err();
    assert!(matches!(err, ParseError::LoopEscapeOutsideLoop { .. }));
}

#[test]
fn parse_multiple_escape_points_error() {
    let err = parse("[C:D:E]2").unwrap_err();
    assert!(matches!(err, ParseError::MultipleEscapePoints { .. }));
}

#[test]
fn parse_empty_loop() {
    let mml = parse("[]").unwrap();
    assert_eq!(mml.commands.len(), 0);
}

#[test]
fn parse_loop_with_rest() {
    let mml = parse("[R4 C4]2").unwrap();
    assert_eq!(mml.commands.len(), 4);
    assert!(matches!(mml.commands[0], Command::Rest(_)));
    assert!(matches!(mml.commands[1], Command::Note(_)));
    assert!(matches!(mml.commands[2], Command::Rest(_)));
    assert!(matches!(mml.commands[3], Command::Note(_)));
}

#[test]
fn parse_loop_with_octave_change() {
    let mml = parse("[>C <C]2").unwrap();
    assert_eq!(mml.commands.len(), 8);
    assert!(matches!(mml.commands[0], Command::OctaveUp));
    assert!(matches!(mml.commands[1], Command::Note(_)));
    assert!(matches!(mml.commands[2], Command::OctaveDown));
    assert!(matches!(mml.commands[3], Command::Note(_)));
    assert!(matches!(mml.commands[4], Command::OctaveUp));
    assert!(matches!(mml.commands[5], Command::Note(_)));
    assert!(matches!(mml.commands[6], Command::OctaveDown));
    assert!(matches!(mml.commands[7], Command::Note(_)));
}

#[test]
fn parse_multiple_loops() {
    let mml = parse("[CD]2 [EF]2").unwrap();
    assert_eq!(mml.commands.len(), 8);
}

#[test]
fn parse_loop_with_tempo_and_volume() {
    let mml = parse("T120 [CD]2 V10").unwrap();
    assert_eq!(mml.commands.len(), 6);
    assert!(matches!(mml.commands[0], Command::Tempo(_)));
    assert!(matches!(mml.commands[1], Command::Note(_)));
    assert!(matches!(mml.commands[4], Command::Note(_)));
    assert!(matches!(mml.commands[5], Command::Volume(_)));
}

#[test]
fn test_expand_loop_basic() {
    let commands = vec![
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
    ];
    let expanded = expand_loop(&commands, None, 3).unwrap();
    assert_eq!(expanded.len(), 6);
}

#[test]
fn test_expand_loop_with_escape() {
    let commands = vec![
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
    ];
    let expanded = expand_loop(&commands, Some(1), 2).unwrap();
    assert_eq!(expanded.len(), 4);
}

#[test]
fn test_expand_loop_empty() {
    let commands: Vec<Command> = vec![];
    let expanded = expand_loop(&commands, None, 5).unwrap();
    assert_eq!(expanded.len(), 0);
}

#[test]
fn test_expand_loop_escape_at_start() {
    let commands = vec![Command::Note(Note {
        pitch: Pitch::C,
        accidental: Accidental::Natural,
        duration: TiedDuration::new(Duration::new(None, 0)),
    })];
    let expanded = expand_loop(&commands, Some(0), 3).unwrap();
    assert_eq!(expanded.len(), 2);
}

// 相対ボリュームテスト (Issue #90, #91)
#[test]
fn parse_volume_absolute() {
    let mml = parse("V10 C").unwrap();
    assert_eq!(mml.commands.len(), 2);
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Absolute(10)
        })
    ));
}

#[test]
fn parse_volume_relative_increase() {
    let mml = parse("V10 C V+2 D").unwrap();
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Absolute(10)
        })
    ));
    assert!(matches!(
        mml.commands[2],
        Command::Volume(Volume {
            value: VolumeValue::Relative(2)
        })
    ));
}

#[test]
fn parse_volume_relative_decrease() {
    let mml = parse("V10 C V-3 D").unwrap();
    assert!(matches!(
        mml.commands[2],
        Command::Volume(Volume {
            value: VolumeValue::Relative(-3)
        })
    ));
}

#[test]
fn parse_volume_default_increase() {
    let mml = parse("V10 C V+ D").unwrap();
    assert!(matches!(
        mml.commands[2],
        Command::Volume(Volume {
            value: VolumeValue::Relative(1)
        })
    ));
}

#[test]
fn parse_volume_default_decrease() {
    let mml = parse("V10 C V- D").unwrap();
    assert!(matches!(
        mml.commands[2],
        Command::Volume(Volume {
            value: VolumeValue::Relative(-1)
        })
    ));
}

#[test]
fn parse_volume_clamp_large_relative() {
    // V+128 should be clamped to +15 at parse time
    let mml = parse("V+128").unwrap();
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Relative(15)
        })
    ));
}

#[test]
fn parse_volume_clamp_large_negative_relative() {
    // V-128 should be clamped to -15 at parse time
    let mml = parse("V-128").unwrap();
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Relative(-15)
        })
    ));
}

#[test]
fn parse_volume_invalid_absolute_out_of_range() {
    let err = parse("V20 C").unwrap_err();
    match err {
        ParseError::InvalidNumber { value, range, .. } => {
            assert_eq!(value, 20);
            assert_eq!(range, (0, 15));
        }
        _ => panic!("Expected InvalidNumber"),
    }
}

#[test]
fn parse_volume_zero() {
    let mml = parse("V0 C").unwrap();
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Absolute(0)
        })
    ));
}

#[test]
fn parse_volume_fifteen() {
    let mml = parse("V15 C").unwrap();
    assert!(matches!(
        mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Absolute(15)
        })
    ));
}

#[test]
fn parse_volume_consecutive_relative() {
    // V+ V+ V+ should parse as three relative +1
    let mml = parse("V+ V+ V+").unwrap();
    assert_eq!(mml.commands.len(), 3);
    for cmd in &mml.commands {
        assert!(matches!(
            cmd,
            Command::Volume(Volume {
                value: VolumeValue::Relative(1)
            })
        ));
    }
}

// ======== Loop Nest Depth Tests (Issue #93) ========

#[test]
fn parse_loop_nest_2_levels() {
    // 2階層ネスト - 許可
    let mml = parse("[[C]2]2").unwrap();
    // 展開: C C × 2 = C C C C (4コマンド)
    assert_eq!(mml.commands.len(), 4);
}

#[test]
fn parse_loop_nest_3_levels() {
    // 3階層ネスト - 許可
    let mml = parse("[[[C]2]2]2").unwrap();
    // 展開: 2^3 = 8コマンド
    assert_eq!(mml.commands.len(), 8);
}

#[test]
fn parse_loop_nest_4_levels() {
    // 4階層ネスト - 許可
    let mml = parse("[[[[C]2]2]2]2").unwrap();
    // 展開: 2^4 = 16コマンド
    assert_eq!(mml.commands.len(), 16);
}

#[test]
fn parse_loop_nest_5_levels() {
    // 5階層ネスト - 許可（上限）
    let mml = parse("[[[[[C]2]2]2]2]2").unwrap();
    // 展開: 2^5 = 32コマンド
    assert_eq!(mml.commands.len(), 32);
}

#[test]
fn parse_loop_nest_6_levels_error() {
    // 6階層ネスト - エラー
    let err = parse("[[[[[[C]2]2]2]2]2]2").unwrap_err();
    assert!(matches!(
        err,
        ParseError::LoopNestTooDeep { max_depth: 5, .. }
    ));
}

#[test]
fn parse_loop_nest_7_levels_error() {
    // 7階層ネスト - エラー
    let err = parse("[[[[[[[C]2]2]2]2]2]2]2").unwrap_err();
    assert!(matches!(
        err,
        ParseError::LoopNestTooDeep { max_depth: 5, .. }
    ));
}

#[test]
fn parse_loop_nest_with_commands() {
    // 2階層ネストに複数コマンド
    let mml = parse("[CDE[FG]2AB]2").unwrap();
    // 内側: FG × 2 = FGFG (4)
    // 外側: CDE(3) + FGFG(4) + AB(2) = 9コマンド × 2 = 18
    assert_eq!(mml.commands.len(), 18);
}

#[test]
fn parse_loop_nest_with_escape_point() {
    // ネスト内での脱出ポイント
    let mml = parse("[[CD:EF]2]2").unwrap();
    // 内側: CDEF CD (6) × 2 = 12
    assert_eq!(mml.commands.len(), 12);
}

// ======== Loop Expansion Limit Tests (Issue #94) ========

#[test]
fn parse_loop_expansion_within_limit() {
    // 10,000コマンド以下は許可
    // [[[C]10]10]99 = 10 × 10 × 99 = 9,900コマンド
    let mml = parse("[[[C]10]10]99").unwrap();
    assert_eq!(mml.commands.len(), 9_900);
}

#[test]
fn parse_loop_expansion_too_large() {
    // 10,000コマンド超過でエラー
    // [[C]99]99 × 2 = 99 × 99 × 2 = 19,602コマンド
    let err = parse("[[[C]99]99]2").unwrap_err();
    assert!(matches!(
        err,
        ParseError::LoopExpandedTooLarge {
            max_commands: 10_000,
            ..
        }
    ));
}

#[test]
fn parse_loop_expansion_barely_exceeds() {
    // ギリギリ超過
    // [[C]50]50]5 = 50 × 50 × 5 = 12,500コマンド
    let err = parse("[[[C]50]50]5").unwrap_err();
    assert!(matches!(
        err,
        ParseError::LoopExpandedTooLarge {
            max_commands: 10_000,
            ..
        }
    ));
}

// TC-028-U-009: 複数の相対値指定
#[test]
fn test_volume_multiple_relative() {
    let input = "V5 C V+10 D V-8 E";
    let mml = parse(input).unwrap();

    // パース成功を確認（6コマンド: V5, C, V+10, D, V-8, E）
    assert_eq!(mml.commands.len(), 6);

    // V5 - 絶対値
    assert!(matches!(
        &mml.commands[0],
        Command::Volume(Volume {
            value: VolumeValue::Absolute(5)
        })
    ));

    // V+10 - 相対値（クランプされて15になる、ただしパーサー時点では+10）
    match &mml.commands[2] {
        Command::Volume(Volume {
            value: VolumeValue::Relative(delta),
        }) => {
            assert_eq!(*delta, 10);
        }
        _ => panic!("Expected Relative volume"),
    }

    // V-8 - 相対値
    match &mml.commands[4] {
        Command::Volume(Volume {
            value: VolumeValue::Relative(delta),
        }) => {
            assert_eq!(*delta, -8);
        }
        _ => panic!("Expected Relative volume"),
    }
}

#[test]
fn parse_nested_loop_zero_count_error() {
    // TC-029-U-009: ループ回数0のネスト
    let err = parse("[ [ C ]0 ]2").unwrap_err();
    match err {
        ParseError::InvalidLoopCount { value, range, .. } => {
            assert_eq!(value, 0);
            assert_eq!(range, (1, 99));
        }
        _ => panic!("Expected InvalidLoopCount, got {:?}", err),
    }
}

#[test]
fn test_loop_depth_tracking() {
    // TC-029-U-010: ネスト深度カウントの正確性
    // ネストしたループの後、深度が正しくリセットされることを確認
    // [ [ C ]2 ]2 -> C x 2 x 2 = 4 commands
    // [ D ]2      -> D x 2     = 2 commands
    // Total: 6 commands
    let input = "[ [ C ]2 ]2 [ D ]2";
    let mml = parse(input).unwrap();

    assert_eq!(mml.commands.len(), 6);
}

#[test]
fn test_loop_depth_reset_after_sequence() {
    // Multiple sequential nested loops should each work correctly
    // [[[C]2]2]2 -> 2^3 = 8 commands
    // [[[D]2]2]2 -> 2^3 = 8 commands
    // Total: 16 commands
    let input = "[[[C]2]2]2 [[[D]2]2]2";
    let mml = parse(input).unwrap();

    assert_eq!(mml.commands.len(), 16);
}

#[test]
fn test_is_next_tie_true() {
    let tokens = sine_mml::mml::tokenize("C4&8").unwrap();
    let parser = Parser::new(tokens);
    // Skip C, 4 to get to &
    let mut parser = parser;
    parser.advance(); // C
    parser.advance(); // 4
    assert!(parser.is_next_tie());
}

#[test]
fn test_is_next_tie_false() {
    let tokens = sine_mml::mml::tokenize("C4").unwrap();
    let parser = Parser::new(tokens);
    assert!(!parser.is_next_tie()); // C is not Tie
}

#[test]
fn test_consume_tie_success() {
    let tokens = sine_mml::mml::tokenize("&C").unwrap();
    let mut parser = Parser::new(tokens);
    assert!(parser.consume_tie());
    // After consuming, next token should be C (Pitch)
    assert!(matches!(parser.peek().token, Token::Pitch(_)));
}

#[test]
fn test_consume_tie_failure() {
    let tokens = sine_mml::mml::tokenize("C4").unwrap();
    let mut parser = Parser::new(tokens);
    // C is not Tie, should return false and parser position should not change
    assert!(!parser.consume_tie());
    assert!(matches!(parser.peek().token, Token::Pitch(_)));
}

// === Issue #125 & #126: Tie notation parsing tests ===

#[test]
fn parse_note_with_single_tie() {
    // C4&8 -> Note with TiedDuration[base=4, tied=[8]]
    let input = "C4&8";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.base.dots, 0);
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, Some(8));
            assert_eq!(n.duration.tied[0].dots, 0);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_with_multiple_ties() {
    // C4&8&16 -> Note with TiedDuration[base=4, tied=[8, 16]]
    let input = "C4&8&16";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.tied.len(), 2);
            assert_eq!(n.duration.tied[0].value, Some(8));
            assert_eq!(n.duration.tied[1].value, Some(16));
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_tie_with_dots() {
    // C4.&8. -> Note with TiedDuration[base=(4,1), tied=[(8,1)]]
    let input = "C4.&8.";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.base.dots, 1);
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, Some(8));
            assert_eq!(n.duration.tied[0].dots, 1);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_tie_dot_only() {
    // C4&. -> Note with TiedDuration[base=4, tied=[(None, 1)]]
    // Dot-only tie inherits default length
    let input = "C4&.";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, None);
            assert_eq!(n.duration.tied[0].dots, 1);
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_note_empty_tie_error() {
    // C4& (tie with nothing after) -> EmptyTieChain error
    let input = "C4&";
    let err = parse(input).unwrap_err();
    match err {
        ParseError::EmptyTieChain { .. } => {}
        _ => panic!("Expected EmptyTieChain error, got {:?}", err),
    }
}

#[test]
fn parse_note_tie_followed_by_note() {
    // C4&D -> EmptyTieChain error (D is not a duration)
    let input = "C4&D";
    let err = parse(input).unwrap_err();
    match err {
        ParseError::EmptyTieChain { .. } => {}
        _ => panic!("Expected EmptyTieChain error, got {:?}", err),
    }
}

#[test]
fn parse_rest_with_single_tie() {
    // R4&8 -> Rest with TiedDuration[base=4, tied=[8]]
    let input = "R4&8";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Rest(r) => {
            assert_eq!(r.duration.base.value, Some(4));
            assert_eq!(r.duration.base.dots, 0);
            assert_eq!(r.duration.tied.len(), 1);
            assert_eq!(r.duration.tied[0].value, Some(8));
        }
        _ => panic!("Expected Rest"),
    }
}

#[test]
fn parse_rest_with_multiple_ties() {
    // R4&8&16 -> Rest with TiedDuration[base=4, tied=[8, 16]]
    let input = "R4&8&16";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Rest(r) => {
            assert_eq!(r.duration.base.value, Some(4));
            assert_eq!(r.duration.tied.len(), 2);
            assert_eq!(r.duration.tied[0].value, Some(8));
            assert_eq!(r.duration.tied[1].value, Some(16));
        }
        _ => panic!("Expected Rest"),
    }
}

#[test]
fn parse_rest_empty_tie_error() {
    // R4& (tie with nothing after) -> EmptyTieChain error
    let input = "R4&";
    let err = parse(input).unwrap_err();
    match err {
        ParseError::EmptyTieChain { .. } => {}
        _ => panic!("Expected EmptyTieChain error, got {:?}", err),
    }
}

#[test]
fn parse_note_with_accidental_and_tie() {
    // C#4&8 -> Note with Sharp and TiedDuration
    let input = "C#4&8";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.accidental, Accidental::Sharp);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, Some(8));
        }
        _ => panic!("Expected Note"),
    }
}

#[test]
fn parse_multiple_notes_with_ties() {
    // C4&8 D2&4 -> Two notes each with ties
    let input = "C4&8 D2&4";
    let mml = parse(input).unwrap();
    assert_eq!(mml.commands.len(), 2);

    match &mml.commands[0] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::C);
            assert_eq!(n.duration.base.value, Some(4));
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, Some(8));
        }
        _ => panic!("Expected Note"),
    }

    match &mml.commands[1] {
        Command::Note(n) => {
            assert_eq!(n.pitch, Pitch::D);
            assert_eq!(n.duration.base.value, Some(2));
            assert_eq!(n.duration.tied.len(), 1);
            assert_eq!(n.duration.tied[0].value, Some(4));
        }
        _ => panic!("Expected Note"),
    }
}

// ======== Tuplet Tests (Issue #144) ========

/// TC-TUP-001: 基本的な3連符
#[test]
fn parse_tuplet_basic_3() {
    let mml = parse("{CDE}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet {
            commands,
            count,
            base_duration,
        } => {
            assert_eq!(commands.len(), 3);
            assert_eq!(*count, 3);
            assert_eq!(*base_duration, None);

            // 各音符の確認
            assert!(matches!(&commands[0], Command::Note(n) if n.pitch == Pitch::C));
            assert!(matches!(&commands[1], Command::Note(n) if n.pitch == Pitch::D));
            assert!(matches!(&commands[2], Command::Note(n) if n.pitch == Pitch::E));
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-002: ベース音長指定付き連符
#[test]
fn parse_tuplet_with_base_duration() {
    let mml = parse("{CDE}3:2").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet {
            commands,
            count,
            base_duration,
        } => {
            assert_eq!(commands.len(), 3);
            assert_eq!(*count, 3);
            assert_eq!(*base_duration, Some(2)); // 2分音符ベース
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-003: 連符数なしエラー
#[test]
fn parse_tuplet_count_missing_error() {
    let err = parse("{CDE}").unwrap_err();
    match err {
        ParseError::TupletCountMissing { .. } => {}
        _ => panic!("Expected TupletCountMissing, got {:?}", err),
    }
}

/// TC-TUP-004: 無効な連符数（1未満）
#[test]
fn parse_tuplet_invalid_count_1() {
    let err = parse("{CDE}1").unwrap_err();
    match err {
        ParseError::InvalidTupletCount { count, .. } => {
            assert_eq!(count, 1);
        }
        _ => panic!("Expected InvalidTupletCount, got {:?}", err),
    }
}

/// TC-TUP-005: 無効な連符数（0）
#[test]
fn parse_tuplet_invalid_count_0() {
    let err = parse("{CDE}0").unwrap_err();
    match err {
        ParseError::InvalidTupletCount { count, .. } => {
            assert_eq!(count, 0);
        }
        _ => panic!("Expected InvalidTupletCount, got {:?}", err),
    }
}

/// TC-TUP-006: 閉じ括弧なしエラー
#[test]
fn parse_tuplet_unclosed_error() {
    let err = parse("{CDE").unwrap_err();
    match err {
        ParseError::UnclosedTuplet { .. } => {}
        _ => panic!("Expected UnclosedTuplet, got {:?}", err),
    }
}

/// TC-TUP-007: 連符内に休符
#[test]
fn parse_tuplet_with_rest() {
    let mml = parse("{CRE}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            assert_eq!(commands.len(), 3);
            assert!(matches!(commands[0], Command::Note(_)));
            assert!(matches!(commands[1], Command::Rest(_)));
            assert!(matches!(commands[2], Command::Note(_)));
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-008: 連符内にオクターブ変更
#[test]
fn parse_tuplet_with_octave() {
    let mml = parse("{C>DE}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            assert_eq!(commands.len(), 4);
            assert!(matches!(commands[0], Command::Note(_)));
            assert!(matches!(commands[1], Command::OctaveUp));
            assert!(matches!(commands[2], Command::Note(_)));
            assert!(matches!(commands[3], Command::Note(_)));
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-009: 5連符
#[test]
fn parse_tuplet_5() {
    let mml = parse("{CDEFG}5").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet {
            commands, count, ..
        } => {
            assert_eq!(commands.len(), 5);
            assert_eq!(*count, 5);
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-010: 連符後に音符
#[test]
fn parse_tuplet_followed_by_notes() {
    let mml = parse("{CDE}3 GAB").unwrap();
    assert_eq!(mml.commands.len(), 4); // Tuplet + G + A + B
    assert!(matches!(mml.commands[0], Command::Tuplet { .. }));
    assert!(matches!(mml.commands[1], Command::Note(_)));
    assert!(matches!(mml.commands[2], Command::Note(_)));
    assert!(matches!(mml.commands[3], Command::Note(_)));
}

/// TC-TUP-011: ネストした連符（2階層）
#[test]
fn parse_tuplet_nested_2_levels() {
    let mml = parse("{{CD}2 E}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet {
            commands, count, ..
        } => {
            assert_eq!(*count, 3);
            assert_eq!(commands.len(), 2); // inner tuplet + E
            assert!(matches!(commands[0], Command::Tuplet { .. }));
            assert!(matches!(commands[1], Command::Note(_)));
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-012: 5階層ネスト（上限）
#[test]
fn parse_tuplet_nest_5_levels() {
    // 5階層ネスト - 許可
    let mml = parse("{{{{{C}2}2}2}2}2").unwrap();
    assert_eq!(mml.commands.len(), 1);
    // 最外層が5連符であることを確認
    assert!(matches!(mml.commands[0], Command::Tuplet { .. }));
}

/// TC-TUP-013: 6階層ネストエラー
#[test]
fn parse_tuplet_nest_6_levels_error() {
    let err = parse("{{{{{{C}2}2}2}2}2}2").unwrap_err();
    match err {
        ParseError::TupletNestTooDeep { max_depth, .. } => {
            assert_eq!(max_depth, 5);
        }
        _ => panic!("Expected TupletNestTooDeep, got {:?}", err),
    }
}

/// TC-TUP-014: 連符内にタイ記号
#[test]
fn parse_tuplet_with_tie() {
    let mml = parse("{C4&8 D}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            assert_eq!(commands.len(), 2);
            match &commands[0] {
                Command::Note(n) => {
                    assert!(n.duration.has_ties());
                }
                _ => panic!("Expected Note with tie"),
            }
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-015: 連符内にループは許可（混在）
#[test]
fn parse_tuplet_with_loop() {
    // 連符内にループ - 許可（Loopコマンドとして保持される）
    let mml = parse("{[C]2 D}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            // ループはCommand::Loopとして保持される（展開はSynthesizerが行う）
            assert_eq!(commands.len(), 2); // Loop + Note
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-016: 空の連符
#[test]
fn parse_tuplet_empty() {
    let mml = parse("{}3").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            assert_eq!(commands.len(), 0);
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-017: 連符数99（上限）
#[test]
fn parse_tuplet_count_99() {
    let mml = parse("{C}99").unwrap();
    assert_eq!(mml.commands.len(), 1);
    match &mml.commands[0] {
        Command::Tuplet { count, .. } => {
            assert_eq!(*count, 99);
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-018: ベース音長指定4（4分音符）
#[test]
fn parse_tuplet_base_duration_4() {
    let mml = parse("{CDE}3:4").unwrap();
    match &mml.commands[0] {
        Command::Tuplet { base_duration, .. } => {
            assert_eq!(*base_duration, Some(4));
        }
        _ => panic!("Expected Tuplet"),
    }
}

/// TC-TUP-019: 不正な閉じ括弧のみ
#[test]
fn parse_tuplet_end_without_start() {
    let err = parse("}3").unwrap_err();
    match err {
        ParseError::UnexpectedToken { .. } => {}
        _ => panic!("Expected UnexpectedToken, got {:?}", err),
    }
}

/// TC-TUP-020: 連符内で複数の個別音長指定
#[test]
fn parse_tuplet_individual_durations() {
    let mml = parse("{C4 D8 E}3").unwrap();
    match &mml.commands[0] {
        Command::Tuplet { commands, .. } => {
            assert_eq!(commands.len(), 3);
            match &commands[0] {
                Command::Note(n) => assert_eq!(n.duration.base.value, Some(4)),
                _ => panic!("Expected Note"),
            }
            match &commands[1] {
                Command::Note(n) => assert_eq!(n.duration.base.value, Some(8)),
                _ => panic!("Expected Note"),
            }
            match &commands[2] {
                Command::Note(n) => assert_eq!(n.duration.base.value, None),
                _ => panic!("Expected Note"),
            }
        }
        _ => panic!("Expected Tuplet"),
    }
}
