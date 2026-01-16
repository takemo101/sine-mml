use super::{
    Accidental, Command, DefaultLength, Duration, Mml, Note, Octave, ParseError, Rest, Tempo,
    TiedDuration, Token, TokenWithPos, Volume, VolumeValue,
};

const MAX_EXPANDED_COMMANDS: usize = 10_000;

/// # Errors
/// Returns `ParseError::LoopExpandedTooLarge` if expanded commands exceed 10,000
pub fn expand_loop(
    commands: &[Command],
    escape_index: Option<usize>,
    repeat_count: usize,
) -> Result<Vec<Command>, ParseError> {
    let mut expanded = Vec::with_capacity(commands.len() * repeat_count);

    for i in 0..repeat_count {
        let is_last_iteration = i == repeat_count - 1;

        let end_index = if let (true, Some(idx)) = (is_last_iteration, escape_index) {
            idx
        } else {
            commands.len()
        };

        for cmd in &commands[..end_index] {
            if let Command::Loop {
                commands: inner_cmds,
                escape_index: inner_escape,
                repeat_count: inner_count,
            } = cmd
            {
                let inner_expanded = expand_loop(inner_cmds, *inner_escape, *inner_count)?;
                expanded.extend(inner_expanded);
            } else {
                expanded.push(cmd.clone());
            }

            if expanded.len() > MAX_EXPANDED_COMMANDS {
                return Err(ParseError::LoopExpandedTooLarge {
                    max_commands: MAX_EXPANDED_COMMANDS,
                    actual: expanded.len(),
                });
            }
        }
    }

    Ok(expanded)
}

/// 最大ループネスト深度
const MAX_LOOP_DEPTH: usize = 5;

/// 最大連符ネスト深度
const MAX_TUPLET_DEPTH: usize = 5;

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    current: usize,
    loop_depth: usize,
    tuplet_depth: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
            tuplet_depth: 0,
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
                let expanded = expand_loop(&loop_commands, escape_index, repeat_count)?;
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
            // 連符処理
            Token::TupletStart => self.parse_tuplet(),
            Token::TupletEnd => Err(ParseError::UnexpectedToken {
                expected: "command".to_string(),
                found: Token::TupletEnd,
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

    /// 連符構文を解析
    ///
    /// # 構文
    /// `{<コマンド>...}n[:base_duration]`
    ///
    /// # Returns
    /// * `Ok(Command::Tuplet)` - 連符コマンド
    /// * `Err(ParseError)` - エラー
    ///
    /// # エラー
    /// - `TupletNestTooDeep` - ネスト深度が5を超える
    /// - `UnclosedTuplet` - 閉じ括弧がない
    /// - `TupletCountMissing` - 連符数が指定されていない
    /// - `InvalidTupletCount` - 連符数が2未満
    fn parse_tuplet(&mut self) -> Result<Command, ParseError> {
        // ネスト深度チェック（最大5階層）
        if self.tuplet_depth >= MAX_TUPLET_DEPTH {
            return Err(ParseError::TupletNestTooDeep {
                max_depth: MAX_TUPLET_DEPTH,
                position: self.peek().position,
            });
        }

        let start_pos = self.peek().position;
        self.advance(); // Consume '{'
        self.tuplet_depth += 1; // ネスト深度を増やす

        let mut commands = Vec::new();

        // 括弧内のコマンドを解析
        while !self.check_tuplet_end() {
            if self.is_at_end() {
                self.tuplet_depth -= 1; // エラー時も深度を戻す
                return Err(ParseError::UnclosedTuplet {
                    position: start_pos,
                });
            }

            // 再帰的にコマンドをパース（ネスト対応）
            let cmd = self.parse_command()?;
            commands.push(cmd);
        }

        self.advance(); // Consume '}'
        self.tuplet_depth -= 1; // ネスト深度を戻す

        // 連符数を取得
        if !self.check_number() {
            return Err(ParseError::TupletCountMissing {
                position: self.peek().position,
            });
        }

        let token_with_pos = self.advance();
        let count = if let Token::Number(n) = token_with_pos.token {
            if n < 2 {
                #[allow(clippy::cast_possible_truncation)]
                return Err(ParseError::InvalidTupletCount {
                    count: n as u8,
                    position: token_with_pos.position,
                });
            }
            if n > 99 {
                #[allow(clippy::cast_possible_truncation)]
                return Err(ParseError::InvalidTupletCount {
                    count: 99,
                    position: token_with_pos.position,
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            {
                n as u8
            }
        } else {
            unreachable!("check_number() returned true but token is not Number")
        };

        // ベース音長の指定（オプション）
        let base_duration = if self.check_colon() {
            self.advance(); // Consume ':'
            if !self.check_number() {
                return Err(ParseError::UnexpectedToken {
                    expected: "number".to_string(),
                    found: self.peek().token.clone(),
                    position: self.peek().position,
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            Some(self.consume_number_in_range(1, 64)? as u8)
        } else {
            None
        };

        Ok(Command::Tuplet {
            commands,
            count,
            base_duration,
        })
    }

    /// 次のトークンが連符終了かチェック
    fn check_tuplet_end(&self) -> bool {
        matches!(self.peek().token, Token::TupletEnd)
    }

    /// 次のトークンがコロン（LoopEscape）かチェック
    fn check_colon(&self) -> bool {
        matches!(self.peek().token, Token::LoopEscape)
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

        let duration_val = if let Token::Number(_) = self.peek().token {
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

        let mut tied_duration = TiedDuration::new(Duration::new(duration_val, dots));

        while self.consume_tie() {
            let tie_position = self.previous().position;

            let tied_duration_val = if let Token::Number(_) = self.peek().token {
                #[allow(clippy::cast_possible_truncation)]
                Some(self.consume_number_in_range(1, 64).map_err(|_| {
                    ParseError::InvalidTieSequence {
                        position: tie_position,
                    }
                })? as u8)
            } else {
                None
            };

            if tied_duration_val.is_none() && !matches!(self.peek().token, Token::Dot) {
                return Err(ParseError::EmptyTieChain {
                    position: tie_position,
                });
            }

            let mut tied_dots = 0;
            while matches!(self.peek().token, Token::Dot) {
                self.advance();
                tied_dots += 1;
            }

            tied_duration.add_tie(Duration::new(tied_duration_val, tied_dots));
        }

        Ok(Note {
            pitch,
            accidental,
            duration: tied_duration,
        })
    }

    fn parse_rest(&mut self) -> Result<Rest, ParseError> {
        self.advance(); // Consume Rest

        let duration_val = if let Token::Number(_) = self.peek().token {
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

        let mut tied_duration = TiedDuration::new(Duration::new(duration_val, dots));

        while self.consume_tie() {
            let tie_position = self.previous().position;

            let tied_duration_val = if let Token::Number(_) = self.peek().token {
                #[allow(clippy::cast_possible_truncation)]
                Some(self.consume_number_in_range(1, 64).map_err(|_| {
                    ParseError::InvalidTieSequence {
                        position: tie_position,
                    }
                })? as u8)
            } else {
                None
            };

            if tied_duration_val.is_none() && !matches!(self.peek().token, Token::Dot) {
                return Err(ParseError::EmptyTieChain {
                    position: tie_position,
                });
            }

            let mut tied_dots = 0;
            while matches!(self.peek().token, Token::Dot) {
                self.advance();
                tied_dots += 1;
            }

            tied_duration.add_tie(Duration::new(tied_duration_val, tied_dots));
        }

        Ok(Rest {
            duration: tied_duration,
        })
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

    /// Check if the next token is a Tie
    #[must_use]
    pub fn is_next_tie(&self) -> bool {
        matches!(self.peek().token, Token::Tie)
    }

    /// Consume a Tie token if present
    pub fn consume_tie(&mut self) -> bool {
        if self.is_next_tie() {
            self.advance();
            true
        } else {
            false
        }
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

    /// Peek at the current token without consuming it
    #[must_use]
    pub fn peek(&self) -> &TokenWithPos {
        if self.current >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1] // Return EOF which should be last
        } else {
            &self.tokens[self.current]
        }
    }

    /// Advance to the next token and return the previous one
    pub fn advance(&mut self) -> &TokenWithPos {
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
