use super::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Token,
        position: usize,
    },
    InvalidNumber {
        value: u16,
        range: (u16, u16),
        position: usize,
    },
    UnexpectedCharacter {
        character: char,
        position: usize,
    },
    UnexpectedEof {
        expected: String,
        position: usize,
    },
    EmptyInput,
    UnmatchedLoopStart {
        position: usize,
    },
    UnmatchedLoopEnd {
        position: usize,
    },
    InvalidLoopCount {
        value: u16,
        range: (u16, u16),
        position: usize,
    },
    LoopEscapeOutsideLoop {
        position: usize,
    },
    MultipleEscapePoints {
        position: usize,
    },
    /// ループのネストが深すぎる（最大5階層）
    LoopNestTooDeep {
        max_depth: usize,
        position: usize,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "位置 {position}: 期待されたトークン '{expected}' ですが、'{found:?}' が見つかりました"
                )
            }
            Self::InvalidNumber {
                value,
                range,
                position,
            } => {
                write!(
                    f,
                    "位置 {position}: 数値 {value} は範囲 {}-{} を超えています",
                    range.0, range.1
                )
            }
            Self::UnexpectedCharacter {
                character,
                position,
            } => {
                write!(
                    f,
                    "位置 {position}: 不明な文字 '{character}' が見つかりました"
                )
            }
            Self::UnexpectedEof { expected, position } => {
                write!(
                    f,
                    "位置 {position}: '{expected}' が期待されましたが、入力が終了しました"
                )
            }
            Self::EmptyInput => {
                write!(f, "空のMML文字列が入力されました")
            }
            Self::UnmatchedLoopStart { position } => {
                write!(
                    f,
                    "位置 {position}: ループの開始括弧 '[' に対応する ']' がありません"
                )
            }
            Self::UnmatchedLoopEnd { position } => {
                write!(
                    f,
                    "位置 {position}: ループの終了括弧 ']' に対応する '[' がありません"
                )
            }
            Self::InvalidLoopCount {
                value,
                range,
                position,
            } => {
                write!(
                    f,
                    "位置 {position}: ループ回数 {value} は範囲 {}-{} を超えています",
                    range.0, range.1
                )
            }
            Self::NestedLoop { position } => {
                write!(f, "位置 {position}: ネストしたループは非対応です")
            }
            Self::LoopEscapeOutsideLoop { position } => {
                write!(
                    f,
                    "位置 {position}: 脱出ポイント ':' がループ外で使用されています"
                )
            }
            Self::MultipleEscapePoints { position } => {
                write!(
                    f,
                    "位置 {position}: ループ内に複数の脱出ポイント ':' があります"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mml::Pitch;

    #[test]
    fn display_unexpected_token() {
        let err = ParseError::UnexpectedToken {
            expected: "Number".to_string(),
            found: Token::Pitch(Pitch::C),
            position: 5,
        };
        assert_eq!(
            err.to_string(),
            "位置 5: 期待されたトークン 'Number' ですが、'Pitch(C)' が見つかりました"
        );
    }

    #[test]
    fn display_invalid_number() {
        let err = ParseError::InvalidNumber {
            value: 200,
            range: (0, 100),
            position: 10,
        };
        assert_eq!(
            err.to_string(),
            "位置 10: 数値 200 は範囲 0-100 を超えています"
        );
    }

    #[test]
    fn display_unexpected_character() {
        let err = ParseError::UnexpectedCharacter {
            character: 'X',
            position: 3,
        };
        assert_eq!(err.to_string(), "位置 3: 不明な文字 'X' が見つかりました");
    }

    #[test]
    fn display_unexpected_eof() {
        let err = ParseError::UnexpectedEof {
            expected: "Number".to_string(),
            position: 8,
        };
        assert_eq!(
            err.to_string(),
            "位置 8: 'Number' が期待されましたが、入力が終了しました"
        );
    }

    #[test]
    fn display_empty_input() {
        let err = ParseError::EmptyInput;
        assert_eq!(err.to_string(), "空のMML文字列が入力されました");
    }
}
