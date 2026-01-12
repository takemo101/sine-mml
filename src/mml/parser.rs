use super::{
    Accidental, Command, DefaultLength, Mml, Note, Octave, ParseError, Rest, Tempo, Token,
    TokenWithPos, Volume, VolumeValue,
};

/// ループコマンドを展開してフラットなコマンド列に変換（再帰対応）
///
/// # 引数
/// - `commands`: ループ内のコマンド列
/// - `escape_index`: 脱出ポイントのインデックス（Noneの場合は脱出なし）
/// - `repeat_count`: 繰り返し回数
///
/// # 戻り値
/// 展開されたコマンド列
#[must_use]
pub fn expand_loop(
    commands: &[Command],
    escape_index: Option<usize>,
    repeat_count: usize,
) -> Vec<Command> {
    let mut expanded = Vec::with_capacity(commands.len() * repeat_count);

    for i in 0..repeat_count {
        let is_last_iteration = i == repeat_count - 1;

        let end_index = if let (true, Some(idx)) = (is_last_iteration, escape_index) {
            idx
        } else {
            commands.len()
        };

        for cmd in &commands[..end_index] {
            // ネストしたループも再帰的に展開 (Issue #93, #94)
            if let Command::Loop {
                commands: inner_cmds,
                escape_index: inner_escape,
                repeat_count: inner_count,
            } = cmd
            {
                let inner_expanded = expand_loop(inner_cmds, *inner_escape, *inner_count);
                expanded.extend(inner_expanded);
            } else {
                expanded.push(cmd.clone());
            }
        }
    }

    expanded
}

/// 最大ループネスト深度
const MAX_LOOP_DEPTH: usize = 5;

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    current: usize,
    loop_depth: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
        }
    }

    /// Parses the tokens into an MML AST.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - An unexpected token is encountered.
    /// - A number is out of valid range.
    /// - EOF is reached unexpectedly.
    pub fn parse(&mut self) -> Result<Mml, ParseError> {
        let mut commands = Vec::new();

        while !self.is_at_end() {
            let command = self.parse_command()?;

            if let Command::Loop {
                commands: loop_commands,
                escape_index,
                repeat_count,
            } = command
            {
                let expanded = expand_loop(&loop_commands, escape_index, repeat_count);
                commands.extend(expanded);
            } else {
                commands.push(command);
            }
        }

        Ok(Mml { commands })
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let token_with_pos = self.peek();
        match &token_with_pos.token {
            Token::Pitch(_) => Ok(Command::Note(self.parse_note()?)),
            Token::Rest => Ok(Command::Rest(self.parse_rest()?)),
            Token::Octave => Ok(Command::Octave(self.parse_octave()?)),
            Token::OctaveUp => {
                self.advance();
                Ok(Command::OctaveUp)
            }
            Token::OctaveDown => {
                self.advance();
                Ok(Command::OctaveDown)
            }
            Token::Tempo => Ok(Command::Tempo(self.parse_tempo()?)),
            Token::Length => Ok(Command::DefaultLength(self.parse_length()?)),
            Token::Volume => Ok(Command::Volume(self.parse_volume()?)),
            Token::LoopStart => self.parse_loop(),
            Token::LoopEnd => Err(ParseError::UnmatchedLoopEnd {
                position: token_with_pos.position,
            }),
            Token::LoopEscape => Err(ParseError::LoopEscapeOutsideLoop {
                position: token_with_pos.position,
            }),
            Token::Eof => Err(ParseError::UnexpectedEof {
                expected: "command".to_string(),
                position: token_with_pos.position,
            }),
            _ => Err(ParseError::UnexpectedToken {
                expected: "command".to_string(),
                found: token_with_pos.token.clone(),
                position: token_with_pos.position,
            }),
        }
    }

    fn parse_loop(&mut self) -> Result<Command, ParseError> {
        // ネスト深度チェック (Issue #93)
        if self.loop_depth >= MAX_LOOP_DEPTH {
            return Err(ParseError::LoopNestTooDeep {
                max_depth: MAX_LOOP_DEPTH,
                position: self.peek().position,
            });
        }

        let start_pos = self.peek().position;
        self.advance();
        self.loop_depth += 1; // ネスト深度を増やす

        let mut commands = Vec::new();
        let mut escape_index = None;
        let mut escape_count = 0;

        while !self.check_loop_end() {
            if self.is_at_end() {
                self.loop_depth -= 1; // エラー時も深度を戻す
                return Err(ParseError::UnmatchedLoopStart {
                    position: start_pos,
                });
            }

            if self.check_loop_escape() {
                escape_count += 1;
                if escape_count > 1 {
                    self.loop_depth -= 1; // エラー時も深度を戻す
                    return Err(ParseError::MultipleEscapePoints {
                        position: self.peek().position,
                    });
                }
                self.advance();
                escape_index = Some(commands.len());
                continue;
            }

            // ネストしたループを許可（parse_command経由で再帰的にparse_loopが呼ばれる）
            let command = self.parse_command()?;
            commands.push(command);
        }

        self.advance();
        self.loop_depth -= 1; // ネスト深度を戻す

        let repeat_count = if self.check_number() {
            let token_with_pos = self.advance();
            if let Token::Number(n) = token_with_pos.token {
                if n == 0 || n > 99 {
                    return Err(ParseError::InvalidLoopCount {
                        value: n,
                        range: (1, 99),
                        position: token_with_pos.position,
                    });
                }
                n as usize
            } else {
                unreachable!()
            }
        } else {
            1
        };

        Ok(Command::Loop {
            commands,
            escape_index,
            repeat_count,
        })
    }

    fn check_loop_end(&self) -> bool {
        matches!(self.peek().token, Token::LoopEnd)
    }

    fn check_loop_escape(&self) -> bool {
        matches!(self.peek().token, Token::LoopEscape)
    }

    fn check_number(&self) -> bool {
        matches!(self.peek().token, Token::Number(_))
    }

    fn parse_note(&mut self) -> Result<Note, ParseError> {
        let token_with_pos = self.advance(); // Consume pitch
        let Token::Pitch(pitch) = token_with_pos.token else {
            unreachable!("parse_note called without Pitch token")
        };

        let accidental = if matches!(self.peek().token, Token::Sharp | Token::Flat) {
            match self.advance().token {
                Token::Sharp => Accidental::Sharp,
                Token::Flat => Accidental::Flat,
                _ => unreachable!(),
            }
        } else {
            Accidental::Natural
        };

        let duration = if let Token::Number(_) = self.peek().token {
            // Range 1-64 verified, safe to cast to u8
            #[allow(clippy::cast_possible_truncation)]
            Some(self.consume_number_in_range(1, 64)? as u8)
        } else {
            None
        };

        let mut dots = 0;
        while matches!(self.peek().token, Token::Dot) {
            self.advance();
            dots += 1;
        }

        Ok(Note {
            pitch,
            accidental,
            duration,
            dots,
        })
    }

    fn parse_rest(&mut self) -> Result<Rest, ParseError> {
        self.advance(); // Consume Rest

        let duration = if let Token::Number(_) = self.peek().token {
            // Range 1-64 verified, safe to cast to u8
            #[allow(clippy::cast_possible_truncation)]
            Some(self.consume_number_in_range(1, 64)? as u8)
        } else {
            None
        };

        let mut dots = 0;
        while matches!(self.peek().token, Token::Dot) {
            self.advance();
            dots += 1;
        }

        Ok(Rest { duration, dots })
    }

    fn parse_octave(&mut self) -> Result<Octave, ParseError> {
        self.advance(); // Consume Octave
                        // Range 1-8 verified, safe to cast to u8
        #[allow(clippy::cast_possible_truncation)]
        let value = self.consume_number_in_range(1, 8)? as u8;
        Ok(Octave { value })
    }

    fn parse_tempo(&mut self) -> Result<Tempo, ParseError> {
        self.advance(); // Consume Tempo
        let value = self.consume_number_in_range(30, 300)?;
        Ok(Tempo { value })
    }

    fn parse_length(&mut self) -> Result<DefaultLength, ParseError> {
        self.advance(); // Consume Length
                        // Range 1-64 verified, safe to cast to u8
        #[allow(clippy::cast_possible_truncation)]
        let value = self.consume_number_in_range(1, 64)? as u8;
        Ok(DefaultLength { value })
    }

    /// ボリュームコマンドを解析（絶対値/相対値対応）
    ///
    /// # 構文
    /// - `V<0-15>` - 絶対値指定
    /// - `V+<n>` - 相対値指定（増加）
    /// - `V-<n>` - 相対値指定（減少）
    /// - `V+` - デフォルト増加（+1）
    /// - `V-` - デフォルト減少（-1）
    ///
    /// # エラー
    /// - `InvalidNumber` - 絶対値が範囲外（0-15以外）
    ///
    /// # 注意
    /// - 相対値の範囲チェックはシンセサイザー側で実施（クランプ処理）
    /// - `+`と`-`は既存の`Token::Sharp`と`Token::Flat`を流用
    fn parse_volume(&mut self) -> Result<Volume, ParseError> {
        self.advance(); // Consume 'V'

        // 相対指定のチェック
        let value = if self.check_sharp() {
            // V+ の場合（Sharpトークンを流用）
            self.advance(); // Consume '+'
            let delta = if self.check_number() {
                // オーバーフロー防止: 15を上限としてクランプしてからキャスト
                let raw = self.consume_number()?;
                #[allow(clippy::cast_possible_truncation)]
                let clamped = raw.min(15) as i8;
                clamped
            } else {
                1 // デフォルト増減値
            };
            VolumeValue::Relative(delta)
        } else if self.check_flat() {
            // V- の場合（Flatトークンを流用）
            self.advance(); // Consume '-'
            let delta = if self.check_number() {
                // オーバーフロー防止: 15を上限としてクランプしてからキャスト
                let raw = self.consume_number()?;
                #[allow(clippy::cast_possible_truncation)]
                let clamped = -(raw.min(15) as i8);
                clamped
            } else {
                -1 // デフォルト増減値
            };
            VolumeValue::Relative(delta)
        } else {
            // 絶対値
            let val = self.consume_number_in_range(0, 15)?;
            #[allow(clippy::cast_possible_truncation)]
            VolumeValue::Absolute(val as u8)
        };

        Ok(Volume { value })
    }

    /// 次のトークンがSharpかチェック
    fn check_sharp(&self) -> bool {
        matches!(self.peek().token, Token::Sharp)
    }

    /// 次のトークンがFlatかチェック
    fn check_flat(&self) -> bool {
        matches!(self.peek().token, Token::Flat)
    }

    /// 数値を消費（範囲チェックなし）
    fn consume_number(&mut self) -> Result<u16, ParseError> {
        let token_with_pos = self.peek();
        if let Token::Number(val) = token_with_pos.token {
            self.advance();
            Ok(val)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "number".to_string(),
                found: token_with_pos.token.clone(),
                position: token_with_pos.position,
            })
        }
    }

    fn consume_number_in_range(&mut self, min: u16, max: u16) -> Result<u16, ParseError> {
        let token_with_pos = self.peek();
        if let Token::Number(val) = token_with_pos.token {
            if val >= min && val <= max {
                self.advance();
                Ok(val)
            } else {
                Err(ParseError::InvalidNumber {
                    value: val,
                    range: (min, max),
                    position: token_with_pos.position,
                })
            }
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "number".to_string(),
                found: token_with_pos.token.clone(),
                position: token_with_pos.position,
            })
        }
    }

    fn peek(&self) -> &TokenWithPos {
        if self.current >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1] // Return EOF which should be last
        } else {
            &self.tokens[self.current]
        }
    }

    fn advance(&mut self) -> &TokenWithPos {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &TokenWithPos {
        if self.current == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.current - 1]
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token, Token::Eof)
    }
}

/// Parses an MML string into an MML AST.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - The input is empty.
/// - The input contains invalid MML syntax.
pub fn parse(input: &str) -> Result<Mml, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    let tokens = super::tokenize(input)?;
    if tokens.is_empty() || (tokens.len() == 1 && matches!(tokens[0].token, Token::Eof)) {
        // Tokenize returns EOF token for empty string, but check input empty first
        // If input is whitespace only, tokens will contain only EOF
        return Ok(Mml { commands: vec![] });
    }

    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::super::Pitch;
    use super::*;

    #[test]
    fn parse_single_note() {
        let input = "C";
        let mml = parse(input).unwrap();
        assert_eq!(mml.commands.len(), 1);
        match &mml.commands[0] {
            Command::Note(n) => {
                assert_eq!(n.pitch, Pitch::C);
                assert_eq!(n.accidental, Accidental::Natural);
                assert_eq!(n.duration, None);
                assert_eq!(n.dots, 0);
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
                assert_eq!(n.duration, Some(4));
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
                assert_eq!(n.duration, Some(4));
                assert_eq!(n.dots, 1);
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
                assert_eq!(r.duration, Some(4));
                assert_eq!(r.dots, 0);
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
                duration: None,
                dots: 0,
            }),
            Command::Note(Note {
                pitch: Pitch::D,
                accidental: Accidental::Natural,
                duration: None,
                dots: 0,
            }),
        ];
        let expanded = expand_loop(&commands, None, 3);
        assert_eq!(expanded.len(), 6);
    }

    #[test]
    fn test_expand_loop_with_escape() {
        let commands = vec![
            Command::Note(Note {
                pitch: Pitch::C,
                accidental: Accidental::Natural,
                duration: None,
                dots: 0,
            }),
            Command::Note(Note {
                pitch: Pitch::D,
                accidental: Accidental::Natural,
                duration: None,
                dots: 0,
            }),
            Command::Note(Note {
                pitch: Pitch::E,
                accidental: Accidental::Natural,
                duration: None,
                dots: 0,
            }),
        ];
        let expanded = expand_loop(&commands, Some(1), 2);
        assert_eq!(expanded.len(), 4);
    }

    #[test]
    fn test_expand_loop_empty() {
        let commands: Vec<Command> = vec![];
        let expanded = expand_loop(&commands, None, 5);
        assert_eq!(expanded.len(), 0);
    }

    #[test]
    fn test_expand_loop_escape_at_start() {
        let commands = vec![Command::Note(Note {
            pitch: Pitch::C,
            accidental: Accidental::Natural,
            duration: None,
            dots: 0,
        })];
        let expanded = expand_loop(&commands, Some(0), 3);
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
            ParseError::LoopNestTooDeep {
                max_depth: 5,
                ..
            }
        ));
    }

    #[test]
    fn parse_loop_nest_7_levels_error() {
        // 7階層ネスト - エラー
        let err = parse("[[[[[[[C]2]2]2]2]2]2]2").unwrap_err();
        assert!(matches!(
            err,
            ParseError::LoopNestTooDeep {
                max_depth: 5,
                ..
            }
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
}
