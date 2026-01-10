mod ast;
pub mod error;

pub use ast::*;
pub use error::ParseError;

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Pitch(Pitch),
    Sharp,
    Flat,
    Dot,
    Number(u16),
    Octave,
    Tempo,
    Length,
    Volume,
    Rest,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos {
    pub token: Token,
    pub position: usize,
}

impl TokenWithPos {
    #[must_use]
    pub const fn new(token: Token, position: usize) -> Self {
        Self { token, position }
    }
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn tokenize(input: &str) -> Result<Vec<TokenWithPos>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut position = 0;

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            position += 1;
            continue;
        }

        let token = match c.to_ascii_uppercase() {
            'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => {
                let pitch = Pitch::from_char(c).expect("validated by match arm");
                chars.next();
                let tok = TokenWithPos::new(Token::Pitch(pitch), position);
                position += 1;
                tok
            }
            '+' | '#' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Sharp, position);
                position += 1;
                tok
            }
            '-' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Flat, position);
                position += 1;
                tok
            }
            '.' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Dot, position);
                position += 1;
                tok
            }
            'O' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Octave, position);
                position += 1;
                tok
            }
            'T' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Tempo, position);
                position += 1;
                tok
            }
            'L' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Length, position);
                position += 1;
                tok
            }
            'V' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Volume, position);
                position += 1;
                tok
            }
            'R' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Rest, position);
                position += 1;
                tok
            }
            _ if c.is_ascii_digit() => {
                let start_pos = position;
                let (number, consumed) = parse_number(&mut chars)?;
                position += consumed;
                TokenWithPos::new(Token::Number(number), start_pos)
            }
            _ => {
                return Err(ParseError::UnexpectedCharacter {
                    character: c,
                    position,
                });
            }
        };
        tokens.push(token);
    }

    tokens.push(TokenWithPos::new(Token::Eof, position));
    Ok(tokens)
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<(u16, usize), ParseError> {
    let mut num_str = String::new();
    let mut consumed = 0;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
            consumed += 1;
        } else {
            break;
        }
    }

    num_str
        .parse::<u16>()
        .map(|n| (n, consumed))
        .map_err(|_| ParseError::InvalidNumber {
            value: 0,
            range: (0, u16::MAX),
            position: 0,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_note() {
        let tokens = tokenize("C").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::C));
        assert_eq!(tokens[1].token, Token::Eof);
    }

    #[test]
    fn tokenize_note_with_sharp() {
        let tokens = tokenize("C#").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::C));
        assert_eq!(tokens[1].token, Token::Sharp);
    }

    #[test]
    fn tokenize_note_with_flat() {
        let tokens = tokenize("D-").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::D));
        assert_eq!(tokens[1].token, Token::Flat);
    }

    #[test]
    fn tokenize_note_with_duration() {
        let tokens = tokenize("C4").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::C));
        assert_eq!(tokens[1].token, Token::Number(4));
    }

    #[test]
    fn tokenize_dotted_note() {
        let tokens = tokenize("C4.").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[2].token, Token::Dot);
    }

    #[test]
    fn tokenize_octave_command() {
        let tokens = tokenize("O4").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Octave);
        assert_eq!(tokens[1].token, Token::Number(4));
    }

    #[test]
    fn tokenize_tempo_command() {
        let tokens = tokenize("T120").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Tempo);
        assert_eq!(tokens[1].token, Token::Number(120));
    }

    #[test]
    fn tokenize_length_command() {
        let tokens = tokenize("L8").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Length);
        assert_eq!(tokens[1].token, Token::Number(8));
    }

    #[test]
    fn tokenize_volume_command() {
        let tokens = tokenize("V10").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Volume);
        assert_eq!(tokens[1].token, Token::Number(10));
    }

    #[test]
    fn tokenize_rest() {
        let tokens = tokenize("R4").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Rest);
        assert_eq!(tokens[1].token, Token::Number(4));
    }

    #[test]
    fn tokenize_ignores_whitespace() {
        let tokens = tokenize("C D E").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::C));
        assert_eq!(tokens[1].token, Token::Pitch(Pitch::D));
        assert_eq!(tokens[2].token, Token::Pitch(Pitch::E));
    }

    #[test]
    fn tokenize_case_insensitive() {
        let tokens = tokenize("c d e").unwrap();
        assert_eq!(tokens[0].token, Token::Pitch(Pitch::C));
        assert_eq!(tokens[1].token, Token::Pitch(Pitch::D));
        assert_eq!(tokens[2].token, Token::Pitch(Pitch::E));
    }

    #[test]
    fn tokenize_complex_mml() {
        let tokens = tokenize("O4 L4 T120 C D E F G").unwrap();
        assert_eq!(tokens.len(), 12);
    }

    #[test]
    fn tokenize_invalid_character() {
        let result = tokenize("CX");
        assert!(result.is_err());
        match result {
            Err(ParseError::UnexpectedCharacter { character, .. }) => {
                assert_eq!(character, 'X');
            }
            _ => panic!("Expected UnexpectedCharacter error"),
        }
    }

    #[test]
    fn tokenize_empty_input() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::Eof);
    }

    #[test]
    fn tokenize_positions_are_correct() {
        let tokens = tokenize("C D").unwrap();
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[1].position, 2);
    }
}
