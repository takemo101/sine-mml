use super::{
    Accidental, Command, DefaultLength, Mml, Note, Octave, ParseError, Rest, Tempo, Token,
    TokenWithPos, Volume,
};

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    current: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Self { tokens, current: 0 }
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
            commands.push(command);
        }

        Ok(Mml { commands })
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let token_with_pos = self.peek();
        match &token_with_pos.token {
            Token::Pitch(_) => Ok(Command::Note(self.parse_note()?)),
            Token::Rest => Ok(Command::Rest(self.parse_rest()?)),
            Token::Octave => Ok(Command::Octave(self.parse_octave()?)),
            Token::Tempo => Ok(Command::Tempo(self.parse_tempo()?)),
            Token::Length => Ok(Command::DefaultLength(self.parse_length()?)),
            Token::Volume => Ok(Command::Volume(self.parse_volume()?)),
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

    fn parse_volume(&mut self) -> Result<Volume, ParseError> {
        self.advance(); // Consume Volume
                        // Range 0-15 verified, safe to cast to u8
        #[allow(clippy::cast_possible_truncation)]
        let value = self.consume_number_in_range(0, 15)? as u8;
        Ok(Volume { value })
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
}
