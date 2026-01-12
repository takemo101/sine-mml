# MML Synthesizer CLI タイ記号機能 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-005 |
| バージョン | 1.0.1 |
| ステータス | ドラフト |
| 作成日 | 2026-01-12 |
| 最終更新日 | 2026-01-12 |
| 作成者 | req-writer |
| 承認者 | - |
| 関連文書 | REQ-CLI-001_MML-Synthesizer.md (v1.1.0)<br>REQ-CLI-002_MML-Synthesizer-Enhancement.md (v1.0.0)<br>REQ-CLI-003_MML-Syntax-Extension.md (v1.0.0)<br>REQ-CLI-004_MML-Advanced-Features.md (v1.0.1)<br>docs/memos/タイ記号仕様追加.md |

---

## 1. プロジェクト概要

### 1.1 背景

sine-mml v2.1の実装完了後、音楽表現力のさらなる向上のため、タイ記号（`&`）機能の追加が必要となりました：

1. **付点では表現できない音長の欠如**: 現状、`C4.`（付点4分音符 = 4分 + 8分）は表現できるが、`4分 + 16分 = 5/16拍`のような付点では表現できない音長を記述する手段がない
2. **小節をまたぐ長い音の表現困難**: 全音符（`C1`）より長い音や、小節をまたぐ音を表現する手段がない
3. **音楽記譜法との乖離**: 一般的な楽譜ではタイ記号で音符を連結するが、MMLでは同等の機能がない

これらの改善により、MML記述の音楽表現力を大幅に向上させます。

### 1.2 目的

- タイ記号（`&`）により、同一音程の音符を連結して長い音を表現可能にする
- 付点では表現できない音長（4分 + 16分等）を記述可能にする
- 小節をまたぐ長い音（全音符2つ分等）を簡潔に記述可能にする
- 休符のタイも許可し、長い休符を柔軟に表現可能にする

### 1.3 ゴール

| 目標 | 成功指標 |
|------|---------|
| タイ記号の実装 | `C4&8` で4分音符と8分音符が連結され、3/8拍の音として発音される |
| 複数連結の対応 | `C4&8&16` で3つの音符が連結され、7/16拍の音として発音される |
| 休符タイの対応 | `R4&8` で付点4分休符相当（3/8拍）の休符が生成される |
| エラー検出 | `C4&D4`（異なる音程のタイ）が適切なエラーメッセージで検出される |

### 1.4 スコープ

#### 対象範囲
- MMLパーサーへのタイ記号 `&` の追加
- 同一音程の音符連結機能
- 休符のタイ対応
- タイ後の音符に付点指定可能
- タイの連鎖（無制限）
- 異なる音程のタイのエラー検出
- 音符と休符のタイのエラー検出

#### 対象外
- スラー記号（異なる音程を滑らかに繋ぐ）
- タイ記号の視覚的な表示（CLI出力）
- タイ記号のエクスポート（WAVファイルには影響しない）

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | 音楽表現力の向上、楽譜との互換性 | 高 |
| 開発チーム | - | パーサー拡張の複雑性、後方互換性 | 高 |
| エンドユーザー | 音楽制作者、趣味プログラマー | 複雑な音長の表現、楽譜の再現 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 楽譜を忠実に再現したい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、音楽経験豊富 |
| 課題 | 楽譜のタイ記号をMMLで表現できない、付点では表現できない音長がある |
| ニーズ | タイ記号で音符を連結し、楽譜通りの音長を表現したい |
| 利用シーン | クラシック音楽やジャズの楽譜をMMLで入力 |

#### ペルソナ2: 小節をまたぐ長い音を表現したい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 30代、音楽経験あり |
| 課題 | 全音符より長い音を表現する手段がない |
| ニーズ | タイ記号で全音符を連結し、2小節分の長い音を表現したい |
| 利用シーン | アンビエント音楽やドローン音楽の作成 |

#### ペルソナ3: 複雑なリズムを表現したい開発者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、プログラミング経験豊富 |
| 課題 | 付点では表現できない音長（4分 + 16分等）を記述できない |
| ニーズ | タイ記号で任意の音長を組み合わせたい |
| 利用シーン | ポリリズムやシンコペーションを含む楽曲の作成 |

---

## 3. 機能要件

### 3.1 機能一覧

**※ REQ-CLI-001（F-001〜F-014）、REQ-CLI-002（F-015〜F-022）、REQ-CLI-003（F-023〜F-026）、REQ-CLI-004（F-027〜F-029）との連番を維持**

| ID | 機能名 | 概要 | 優先度 | フェーズ | 備考 |
|----|--------|------|--------|---------|------|
| F-030 | MMLタイ記号 | `&` で同一音程の音符を連結 | 必須 | Phase 2.2 | **新規** |

### 3.2 ユーザーストーリー

#### US-018: タイ記号で音符を連結したい
- **ユーザー**: 作曲者
- **したいこと**: タイ記号（`&`）で同一音程の音符を連結し、長い音を表現したい
- **理由**: 付点では表現できない音長や、小節をまたぐ長い音を記述したい
- **受け入れ基準**:
  - [ ] `C4&8` で4分音符と8分音符が連結され、3/8拍の音として発音される
  - [ ] `C4&8&16` で複数の音符が連結され、7/16拍の音として発音される
  - [ ] `C1&1` で全音符2つ分（2小節分）の長い音が発音される
  - [ ] `R4&8` で付点4分休符相当（3/8拍）の休符が生成される
  - [ ] `C4&8.` でタイ後の音符に付点も指定可能
  - [ ] `C4 & 8` のようにタイ記号の前後に空白があっても正常に動作
  - [ ] `C4&D4` のように異なる音程のタイはエラーメッセージを表示
  - [ ] `C4&R4` のように音符と休符のタイはエラーメッセージを表示
  - [ ] `C4&` のようにタイの後に音符がない場合はエラーメッセージを表示
- **関連機能**: F-030

### 3.3 機能詳細

#### F-030: MMLタイ記号

**概要**: `&` で同一音程の音符を連結し、音長を合算して長い音を表現

**入力**:
- MML文字列（タイ記号を含む）
  - 例: `C4&8`, `C4&8&16`, `R4&8`, `C1&1`

**出力**:
- タイで連結された音符の音長を合算した音

**処理概要**:
1. トークナイザーで `&` を認識
2. パーサーで `&` の前後の音符を取得
3. 音程が同一であることを検証
4. 音長を合算
5. 合算された音長で音を発音

**ビジネスルール**:
- BR-084: タイで連結できるのは同一音程の音符のみ
- BR-085: 異なる音程のタイはエラー（例: `C4&D4` は不可）
- BR-086: 休符のタイは許可（`R4&8` = 付点4分休符相当）
- BR-087: タイ後の音符に付点も指定可能（`C4&8.`）
- BR-088: タイの連鎖は無制限（`C4&8&16&32` など）
- BR-089: タイ記号の前後に空白があっても許可（`C4 & 8`）

**制約事項**:
- 音符と休符のタイは不可（`C4&R4` はエラー）
- タイの前後に音符がない場合はエラー（`&C4`, `C4&` はエラー）

**技術実装のポイント**:

**1. トークン定義**:
```rust
// src/mml/mod.rs
pub enum Token {
    // 既存のトークン...
    Tie,  // &
}
```

**2. AST拡張**:
```rust
// src/mml/ast.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub pitch: Pitch,
    pub accidental: Option<Accidental>,
    pub duration: Option<u8>,
    pub dots: u8,
    pub tied_durations: Vec<TiedDuration>,  // 追加
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    pub duration: Option<u8>,
    pub dots: u8,
    pub tied_durations: Vec<TiedDuration>,  // 追加（休符タイ対応）
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TiedDuration {
    pub duration: Option<u8>,
    pub dots: u8,
}
```

**3. パーサー実装**:
```rust
// src/mml/parser.rs
fn parse_note(&mut self) -> Result<Command, ParseError> {
    let pitch = self.parse_pitch()?;
    let accidental = self.parse_accidental()?;
    let duration = self.parse_duration()?;
    let dots = self.parse_dots()?;
    
    let mut tied_durations = Vec::new();
    
    // タイ記号のチェック
    while self.check(Token::Tie) {
        self.advance(); // & を消費
        
        // タイ後の音符を解析
        let tied_pitch = self.parse_pitch()?;
        let tied_accidental = self.parse_accidental()?;
        
        // 音程の検証
        if tied_pitch != pitch || tied_accidental != accidental {
            return Err(ParseError::TieDifferentPitch {
                position: self.pos,
            });
        }
        
        let tied_duration = self.parse_duration()?;
        let tied_dots = self.parse_dots()?;
        
        tied_durations.push(TiedDuration {
            duration: tied_duration,
            dots: tied_dots,
        });
    }
    
    Ok(Command::Note(Note {
        pitch,
        accidental,
        duration,
        dots,
        tied_durations,
    }))
}
```

**4. 休符のタイ対応**:
```rust
// src/mml/parser.rs
fn parse_rest(&mut self) -> Result<Command, ParseError> {
    self.consume('R')?;
    let duration = self.parse_duration()?;
    let dots = self.parse_dots()?;
    
    let mut tied_durations = Vec::new();
    
    // タイ記号のチェック
    while self.check(Token::Tie) {
        self.advance(); // & を消費
        
        // タイ後が休符であることを確認
        if !self.check('R') {
            return Err(ParseError::TieNoteAndRest {
                position: self.pos,
            });
        }
        
        self.consume('R')?;
        let tied_duration = self.parse_duration()?;
        let tied_dots = self.parse_dots()?;
        
        tied_durations.push(TiedDuration {
            duration: tied_duration,
            dots: tied_dots,
        });
    }
    
    Ok(Command::Rest(Rest {
        duration,
        dots,
        tied_durations,
    }))
}
```

**5. シンセサイザー実装**:
```rust
// src/audio/synthesizer.rs
impl Note {
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let base_duration = self.calculate_base_duration(bpm, default_length);
        
        // タイで連結された音符の音長を合算
        let tied_duration: f32 = self.tied_durations
            .iter()
            .map(|td| td.calculate_duration(bpm, default_length))
            .sum();
        
        base_duration + tied_duration
    }
}

impl TiedDuration {
    fn calculate_duration(&self, bpm: u16, default_length: u8) -> f32 {
        let length = self.duration.unwrap_or(default_length);
        let base = 60.0 / bpm as f32 * 4.0 / length as f32;
        
        // 付点の処理
        let mut duration = base;
        for _ in 0..self.dots {
            duration += base / 2.0_f32.powi((self.dots + 1) as i32);
        }
        
        duration
    }
}

// ※ Rest構造体に対しても同様に duration_in_seconds メソッドを実装し、
// タイを含めた休符の長さを計算します。
impl Rest {
    pub fn duration_in_seconds(&self, bpm: u16, default_length: u8) -> f32 {
        let base_duration = self.calculate_base_duration(bpm, default_length);
        
        // タイで連結された休符の音長を合算
        let tied_duration: f32 = self.tied_durations
            .iter()
            .map(|td| td.calculate_duration(bpm, default_length))
            .sum();
        
        base_duration + tied_duration
    }
}
```

**6. エラー型追加**:
```rust
// src/mml/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    // 既存のエラー...
    
    /// MML-E012: 異なる音程のタイ
    #[error("[MML-E012] タイは同一音程の音符のみ連結できます: 位置 {position}")]
    TieDifferentPitch {
        position: usize,
    },
    
    /// MML-E013: タイの後に音符がない
    #[error("[MML-E013] タイの後に音符がありません: 位置 {position}")]
    TieNoFollowingNote {
        position: usize,
    },
    
    /// MML-E014: 音符と休符のタイ
    #[error("[MML-E014] 音符と休符をタイで連結できません: 位置 {position}")]
    TieNoteAndRest {
        position: usize,
    },
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_tie_basic() {
    let mml = parse("C4&8").unwrap();
    // C4 (1/4拍) + C8 (1/8拍) = 3/8拍
}

#[test]
fn test_tie_multiple() {
    let mml = parse("C4&8&16").unwrap();
    // C4 (1/4拍) + C8 (1/8拍) + C16 (1/16拍) = 7/16拍
}

#[test]
fn test_tie_rest() {
    let mml = parse("R4&8").unwrap();
    // R4 (1/4拍) + R8 (1/8拍) = 3/8拍の休符
}

#[test]
fn test_tie_with_dot() {
    let mml = parse("C4&8.").unwrap();
    // C4 (1/4拍) + C8. (3/16拍) = 7/16拍
}

#[test]
fn test_tie_different_pitch() {
    let result = parse("C4&D4");
    assert!(result.is_err());
    // Error: タイは同一音程の音符のみ連結できます
}

#[test]
fn test_tie_note_and_rest() {
    let result = parse("C4&R4");
    assert!(result.is_err());
    // Error: 音符と休符をタイで連結できません
}

#[test]
fn test_tie_no_following_note() {
    let result = parse("C4&");
    assert!(result.is_err());
    // Error: タイの後に音符がありません
}
```

**エラーメッセージ例**:
```
Error: タイは同一音程の音符のみ連結できます: 位置 3
  C4&D4
     ^
Expected: 同一音程の音符（C）

Error: タイの後に音符がありません: 位置 5
  C4&
     ^
Expected: 音符または休符

Error: 音符と休符をタイで連結できません: 位置 3
  C4&R4
     ^
Expected: 音符のみ
```

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-014 | タイ解析のオーバーヘッド | 5ms以内（100個のタイ） | ベンチマーク |
| NFR-P-015 | タイ音長計算のオーバーヘッド | 1%以内（合成時間の） | ベンチマーク |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-009 | タイ解析エラーのハンドリング | 詳細なエラーメッセージを表示 |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-012 | タイの連鎖数制限 | 実質的な制限なし（メモリ制約のみ） |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-014 | タイのエラーメッセージ | 位置情報と修正ヒントを明示 |
| NFR-U-015 | タイ記号の空白許容 | `C4 & 8` のように空白があっても動作 |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-014 | タイのテスト | 正常系、異常系を網羅 |
| NFR-M-015 | 後方互換性 | 既存のMMLが正常に動作 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| タイの対象 | 同一音程の音符のみ | 音楽理論上の制約 |
| 音符と休符のタイ | 不可 | 音楽理論上の制約 |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 2.2: 2週間 |
| リソース | 個人開発（1名） |

### 5.3 法規制・コンプライアンス

| 要件 | 詳細 |
|------|------|
| ライセンス | MIT License（変更なし） |
| 後方互換性 | 既存のMMLが正常に動作することを保証 |

---

## 6. 外部インターフェース

### 6.1 既存機能への影響

| 機能ID | 機能名 | 影響内容 | 対応 |
|--------|--------|----------|------|
| F-001 | MMLパーサー | **拡張** | タイ記号を追加 |

### 6.2 MML構文の拡張

#### 追加される構文
```
<音符>&<音符>         # タイ記号
<音符>&<音符>&<音符>  # 複数連結
R<音長>&<音長>        # 休符のタイ
```

#### 例
```
# 基本的なタイ
C4&8           # 4分音符 + 8分音符 = 3/8拍

# 複数連結
C4&8&16        # 4分音符 + 8分音符 + 16分音符 = 7/16拍

# 小節をまたぐ長い音
C1&1           # 全音符2つ分 = 2小節分

# 休符のタイ
R4&8           # 付点4分休符相当

# タイ後の付点
C4&8.          # 4分音符 + 付点8分音符 = 7/16拍

# 空白を含む
C4 & 8         # 空白があっても動作
```

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の全機能が実装済み（v1.0.0リリース済み）
- REQ-CLI-002の全機能が実装済み（v2.0.0リリース済み）
- REQ-CLI-003の全機能が実装済み（v2.1.0リリース済み）
- REQ-CLI-004の全機能が実装済み（v2.1.0リリース済み）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| MMLパーサー | タイ記号の解析 | パーサー拡張 |
| シンセサイザー | タイ音長の計算 | 音長計算ロジック拡張 |

### 7.3 新規依存クレート

なし（既存のクレートで対応可能）

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-017 | タイ解析のパーサーバグ | 高 | 中 | 豊富なテストケース、エッジケース対応 |
| R-018 | 音長計算の精度問題 | 中 | 低 | 浮動小数点演算の精度検証 |
| R-019 | 既存MMLとの互換性問題 | 低 | 低 | 既存のMMLが正常に動作することを確認 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-016 | タイの連鎖数の妥当性検証 | 開発チーム | 2026-01-26 |
| I-017 | スラー記号の実装検討 | 開発チーム | Phase 3.0で検討 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| タイ記号 | 同一音程の音符を連結して長い音を表現する記号（`&`） |
| スラー記号 | 異なる音程を滑らかに繋ぐ記号（本プロジェクトでは未対応） |
| 音長合算 | タイで連結された音符の音長を足し合わせること |
| 付点音符 | 音符の長さを1.5倍にする記号（`.`） |

---

## 10. 参考リンク

### 10.1 MML仕様

#### タイ記号の参考
- **PPMCK MML Reference**: [MML Command Reference](http://ppmck.wikidot.com/mml-command-reference) - タイ記号の実装例
- **MML (Music Macro Language) - Wikipedia**: [日本語版](https://ja.wikipedia.org/wiki/Music_Macro_Language) - MMLの歴史と構文
- **NuttX MML Parser**: [Documentation](https://nuttx.apache.org/docs/latest/applications/audioutils/mml_parser/index.html) - 基本的なMML仕様

### 10.2 音楽理論

#### タイとスラー
- **音楽理論 - タイ**: タイは同一音程の音符を連結する記号
- **音楽理論 - スラー**: スラーは異なる音程を滑らかに繋ぐ記号（タイとは異なる）

### 10.3 Rustパーサー実装

#### 再帰下降パーサー
- **Crafting Interpreters**: [Parsing Expressions](https://craftinginterpreters.com/parsing-expressions.html) - パーサー実装の基礎
- **Recursive Descent Parsing**: [Wikipedia](https://en.wikipedia.org/wiki/Recursive_descent_parser) - 再帰下降パーサーの理論

---

## 11. 実装優先順位と段階的ロールアウト

### Phase 2.2.1（Week 1: 2026-01-12〜01-19）
1. **トークナイザー拡張** - `&` トークンの追加
   - `Token::Tie` の定義
   - トークナイザーでの `&` 認識
2. **AST拡張** - `Note` 構造体の拡張
   - `tied_durations: Vec<TiedDuration>` の追加
   - `TiedDuration` 構造体の定義

### Phase 2.2.2（Week 2: 2026-01-20〜01-26）
3. **パーサー実装** - タイ解析ロジックの追加
   - `parse_note()` でのタイ検出
   - 音程の検証
   - エラーハンドリング
4. **シンセサイザー実装** - タイ音長計算
   - `duration_in_seconds()` の拡張
   - タイ音長の合算
5. **テストケース作成** - 豊富なテストケース
   - 正常系テスト
   - 異常系テスト
   - エッジケーステスト

---

## 12. テストケース概要

### 12.1 タイ記号のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-030-001 | `C4&8` | 3/8拍の音が発音される |
| TC-030-002 | `C8&8` | 1/4拍の音が発音される（C4相当） |
| TC-030-003 | `C4&4` | 1/2拍の音が発音される（C2相当） |
| TC-030-004 | `C4.&8` | 7/16拍の音が発音される |
| TC-030-005 | `C4&8.` | 7/16拍の音が発音される |
| TC-030-006 | `C4&8&16` | 7/16拍の音が発音される |
| TC-030-007 | `R4&8` | 3/8拍の休符が生成される |
| TC-030-008 | `C1&1` | 2拍の音が発音される（2小節分） |
| TC-030-009 | `C4 & 8` | 3/8拍の音が発音される（空白許容） |
| TC-030-010 | `C4&D4` | エラー（異なる音程） |
| TC-030-011 | `C4&` | エラー（タイ後に音符がない） |
| TC-030-012 | `C4&R4` | エラー（音符と休符のタイ） |
| TC-030-013 | `&C4` | エラー（タイの前に音符がない） |

---

## 13. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-12 | 初版作成 | req-writer |
| 1.0.1 | 2026-01-12 | レビュー指摘反映: Rest構造体定義追加、エラーコードID明記、Rest音長計算追加 | req-writer |
