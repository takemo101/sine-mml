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
    /// ループ展開後のコマンド数が多すぎる（最大10,000）
    LoopExpandedTooLarge {
        max_commands: usize,
        actual: usize,
    },
    /// MML-E012: タイの後に有効な音価がない
    InvalidTieSequence {
        position: usize,
    },
    /// MML-E013: タイで連結する音符の音高が異なる
    TiePitchMismatch {
        expected: String,
        found: String,
        position: usize,
    },
    /// MML-E014: タイチェーンが空（トップレベルの&）
    EmptyTieChain {
        position: usize,
    },
    /// MML-E020: 連符の閉じ括弧がない
    ///
    /// 連符の開始括弧 `{` に対応する閉じ括弧 `}` がない。
    /// 例: `{CDE` (閉じ括弧なし)
    UnclosedTuplet {
        position: usize,
    },
    /// MML-E021: 連符数が指定されていない
    ///
    /// 連符の閉じ括弧 `}` の後に連符数が指定されていない。
    /// 例: `{CDE}` (連符数なし)
    TupletCountMissing {
        position: usize,
    },
    /// MML-E022: 無効な連符数
    ///
    /// 連符数が2未満。
    /// 例: `{CDE}1`, `{CDE}0`
    InvalidTupletCount {
        count: u8,
        position: usize,
    },
    /// MML-E023: 連符のネスト深度超過
    ///
    /// 連符のネスト深度が最大値（5階層）を超えている。
    TupletNestTooDeep {
        max_depth: usize,
        position: usize,
    },
}

impl ParseError {
    fn fmt_token_error(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken {
                expected,
                found,
                position,
            } => write!(
                f,
                "位置 {position}: 期待されたトークン '{expected}' ですが、'{found:?}' が見つかりました"
            ),
            Self::UnexpectedCharacter {
                character,
                position,
            } => write!(f, "位置 {position}: 不明な文字 '{character}' が見つかりました"),
            Self::UnexpectedEof { expected, position } => write!(
                f,
                "位置 {position}: '{expected}' が期待されましたが、入力が終了しました"
            ),
            Self::EmptyInput => write!(f, "空のMML文字列が入力されました"),
            _ => unreachable!(),
        }
    }

    fn fmt_number_error(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber {
                value,
                range,
                position,
            } => write!(
                f,
                "位置 {position}: 数値 {value} は範囲 {}-{} を超えています",
                range.0, range.1
            ),
            _ => unreachable!(),
        }
    }

    fn fmt_loop_error(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnmatchedLoopStart { position } => write!(
                f,
                "位置 {position}: ループの開始括弧 '[' に対応する ']' がありません"
            ),
            Self::UnmatchedLoopEnd { position } => write!(
                f,
                "位置 {position}: ループの終了括弧 ']' に対応する '[' がありません"
            ),
            Self::InvalidLoopCount {
                value,
                range,
                position,
            } => write!(
                f,
                "位置 {position}: ループ回数 {value} は範囲 {}-{} を超えています",
                range.0, range.1
            ),
            Self::LoopEscapeOutsideLoop { position } => write!(
                f,
                "位置 {position}: 脱出ポイント ':' がループ外で使用されています"
            ),
            Self::MultipleEscapePoints { position } => write!(
                f,
                "位置 {position}: ループ内に複数の脱出ポイント ':' があります"
            ),
            Self::LoopNestTooDeep {
                max_depth,
                position,
            } => write!(
                f,
                "位置 {position}: ループのネストが深すぎます（最大{max_depth}階層）"
            ),
            Self::LoopExpandedTooLarge {
                max_commands,
                actual,
            } => write!(
                f,
                "ループ展開後のコマンド数が多すぎます（最大{max_commands}、実際: {actual}）"
            ),
            _ => unreachable!(),
        }
    }

    fn fmt_tie_error(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTieSequence { position } => write!(
                f,
                "位置 {position}: タイ記号 '&' の後に有効な音価がありません"
            ),
            Self::TiePitchMismatch {
                expected,
                found,
                position,
            } => write!(
                f,
                "位置 {position}: タイで連結する音符の音高が異なります（期待: {expected}, 実際: {found}）"
            ),
            Self::EmptyTieChain { position } => write!(
                f,
                "位置 {position}: タイ記号 '&' が音符/休符なしで使用されています"
            ),
            _ => unreachable!(),
        }
    }

    fn fmt_tuplet_error(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnclosedTuplet { position } => {
                write!(f, "位置 {position}: 連符の閉じ括弧 '}}' がありません")
            }
            Self::TupletCountMissing { position } => {
                write!(f, "位置 {position}: 連符数が指定されていません")
            }
            Self::InvalidTupletCount { count, position } => write!(
                f,
                "位置 {position}: 無効な連符数です（2以上を指定してください）: {count}"
            ),
            Self::TupletNestTooDeep {
                max_depth,
                position,
            } => write!(
                f,
                "位置 {position}: 連符のネストが深すぎます（最大{max_depth}階層）"
            ),
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { .. }
            | Self::UnexpectedCharacter { .. }
            | Self::UnexpectedEof { .. }
            | Self::EmptyInput => self.fmt_token_error(f),

            Self::InvalidNumber { .. } => self.fmt_number_error(f),

            Self::UnmatchedLoopStart { .. }
            | Self::UnmatchedLoopEnd { .. }
            | Self::InvalidLoopCount { .. }
            | Self::LoopEscapeOutsideLoop { .. }
            | Self::MultipleEscapePoints { .. }
            | Self::LoopNestTooDeep { .. }
            | Self::LoopExpandedTooLarge { .. } => self.fmt_loop_error(f),

            Self::InvalidTieSequence { .. }
            | Self::TiePitchMismatch { .. }
            | Self::EmptyTieChain { .. } => self.fmt_tie_error(f),

            Self::UnclosedTuplet { .. }
            | Self::TupletCountMissing { .. }
            | Self::InvalidTupletCount { .. }
            | Self::TupletNestTooDeep { .. } => self.fmt_tuplet_error(f),
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

    // TC-030-010: タイ記号エラーテスト - InvalidTieSequence
    #[test]
    fn display_invalid_tie_sequence() {
        let err = ParseError::InvalidTieSequence { position: 3 };
        assert_eq!(
            err.to_string(),
            "位置 3: タイ記号 '&' の後に有効な音価がありません"
        );
    }

    // TC-030-010: タイ記号エラーテスト - TiePitchMismatch
    #[test]
    fn display_tie_pitch_mismatch() {
        let err = ParseError::TiePitchMismatch {
            expected: "C".to_string(),
            found: "D".to_string(),
            position: 3,
        };
        assert_eq!(
            err.to_string(),
            "位置 3: タイで連結する音符の音高が異なります（期待: C, 実際: D）"
        );
    }

    // TC-030-013: タイ記号エラーテスト - EmptyTieChain
    #[test]
    fn display_empty_tie_chain() {
        let err = ParseError::EmptyTieChain { position: 0 };
        assert_eq!(
            err.to_string(),
            "位置 0: タイ記号 '&' が音符/休符なしで使用されています"
        );
    }

    // ======== 連符エラーテスト (Issue #144) ========

    #[test]
    fn display_unclosed_tuplet() {
        let err = ParseError::UnclosedTuplet { position: 0 };
        assert_eq!(err.to_string(), "位置 0: 連符の閉じ括弧 '}' がありません");
    }

    #[test]
    fn display_tuplet_count_missing() {
        let err = ParseError::TupletCountMissing { position: 4 };
        assert_eq!(err.to_string(), "位置 4: 連符数が指定されていません");
    }

    #[test]
    fn display_invalid_tuplet_count() {
        let err = ParseError::InvalidTupletCount {
            count: 1,
            position: 4,
        };
        assert_eq!(
            err.to_string(),
            "位置 4: 無効な連符数です（2以上を指定してください）: 1"
        );
    }

    #[test]
    fn display_tuplet_nest_too_deep() {
        let err = ParseError::TupletNestTooDeep {
            max_depth: 5,
            position: 5,
        };
        assert_eq!(
            err.to_string(),
            "位置 5: 連符のネストが深すぎます（最大5階層）"
        );
    }
}
