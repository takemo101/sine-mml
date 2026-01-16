mod ast;
pub mod error;
pub mod file;

pub use ast::*;
pub use error::ParseError;
pub use file::read_mml_file;

pub mod parser;
pub use parser::*;

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
    OctaveUp,
    OctaveDown,
    Tempo,
    Length,
    Volume,
    Rest,
    /// Loop start bracket `[`
    LoopStart,
    /// Loop end bracket `]`
    LoopEnd,
    /// Loop escape point `:` (also used for tuplet base duration)
    LoopEscape,
    /// Tie symbol `&` for connecting notes of the same pitch
    Tie,
    /// Tuplet start brace `{`
    TupletStart,
    /// Tuplet end brace `}`
    TupletEnd,
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

#[allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
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
            '>' => {
                chars.next();
                let tok = TokenWithPos::new(Token::OctaveUp, position);
                position += 1;
                tok
            }
            '<' => {
                chars.next();
                let tok = TokenWithPos::new(Token::OctaveDown, position);
                position += 1;
                tok
            }
            '[' => {
                chars.next();
                let tok = TokenWithPos::new(Token::LoopStart, position);
                position += 1;
                tok
            }
            ']' => {
                chars.next();
                let tok = TokenWithPos::new(Token::LoopEnd, position);
                position += 1;
                tok
            }
            ':' => {
                chars.next();
                let tok = TokenWithPos::new(Token::LoopEscape, position);
                position += 1;
                tok
            }
            '&' => {
                chars.next();
                let tok = TokenWithPos::new(Token::Tie, position);
                position += 1;
                tok
            }
            '{' => {
                chars.next();
                let tok = TokenWithPos::new(Token::TupletStart, position);
                position += 1;
                tok
            }
            '}' => {
                chars.next();
                let tok = TokenWithPos::new(Token::TupletEnd, position);
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
