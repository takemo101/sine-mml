# MML Synthesizer CLI MML構文拡張 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-003 |
| バージョン | 1.0.0 |
| ステータス | ドラフト |
| 作成日 | 2026-01-11 |
| 最終更新日 | 2026-01-11 |
| 作成者 | req-writer |
| 承認者 | - |
| 関連文書 | REQ-CLI-001_MML-Synthesizer.md (v1.1.0)<br>REQ-CLI-002_MML-Synthesizer-Enhancement.md (v1.0.0) |

---

## 1. プロジェクト概要

### 1.1 背景

sine-mml v2.0の実装完了後、ユーザーからの要望とユーザビリティ向上のため、以下の機能追加が必要となりました：

1. **MMLループ構文の欠如**: 繰り返しパターンを毎回書く必要があり、MML文字列が冗長になる
2. **大文字のみの記述制約**: MMLコマンドが大文字のみで、小文字での記述ができない（タイピング効率の低下）
3. **履歴メモ機能の不在**: 履歴にメモを付けられず、後で見返したときに何の曲か分からない
4. **履歴削除機能の不在**: 不要な履歴を削除する手段がなく、履歴が肥大化する

これらの改善により、MML記述の効率化とユーザー体験の向上を実現します。

### 1.2 目的

- MMLループ構文 `[]` により、繰り返しパターンを簡潔に記述可能にする
- 小文字MML記述を許可し、タイピング効率を向上させる
- `--note`オプションで履歴にメモを付与し、後で見返しやすくする
- `clear-history`コマンドで不要な履歴を一括削除可能にする

### 1.3 ゴール

| 目標 | 成功指標 |
|------|---------|
| MMLループ構文の実装 | `[CDEF]3` で3回ループ、`[CD:EF]2` で脱出ポイント機能が動作 |
| 小文字MML記述の対応 | `cdefgab`, `r`, `o5`, `l8`, `t140`, `v10` が正常に解析される |
| 履歴メモ機能の実装 | `--note "メモ"` で履歴にメモが保存され、`history`コマンドで表示される |
| 履歴削除機能の実装 | `clear-history` で全履歴削除、確認プロンプトが表示される |

### 1.4 スコープ

#### 対象範囲
- MMLパーサーへのループ構文 `[]` の追加
- MMLパーサーへの小文字記述対応（正規化処理）
- CLIへの `--note` オプション追加
- DBスキーマへの `note` カラム追加
- `clear-history` サブコマンドの追加
- 確認プロンプトの実装

#### 対象外
- ネストしたループ構文（`[[CDEF]2 GAB]3` 等）
- ループ回数の動的変更（MML内での変数指定等）
- 履歴の選択的削除（個別削除、条件付き削除等）
- 履歴のエクスポート/インポート機能

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | ユーザー要望への対応、ユーザビリティ向上 | 高 |
| 開発チーム | - | パーサー拡張の複雑性、DB互換性 | 高 |
| エンドユーザー | 音楽制作者、趣味プログラマー | MML記述の効率化、履歴管理の利便性 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 繰り返しパターンを多用する作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、音楽経験あり |
| 課題 | 同じフレーズを何度も書くのが面倒、MML文字列が長くなる |
| ニーズ | ループ構文で繰り返しパターンを簡潔に記述したい |
| 利用シーン | リフやコード進行の繰り返しを含む曲の作成 |

#### ペルソナ2: 小文字でタイピングしたい開発者
| 項目 | 内容 |
|------|------|
| 属性 | 30代、プログラミング経験豊富 |
| 課題 | 大文字入力のためにShiftキーを押すのが面倒 |
| ニーズ | 小文字でMMLを書きたい（`cdefgab` 等） |
| 利用シーン | 作業用BGMを素早く入力 |

#### ペルソナ3: 履歴を整理したいユーザー
| 項目 | 内容 |
|------|------|
| 属性 | 20-50代、音楽制作者 |
| 課題 | 履歴にメモがなく、後で見返したときに何の曲か分からない |
| ニーズ | 履歴にメモを付けたい、不要な履歴を削除したい |
| 利用シーン | 過去の作品を整理、試作品を削除 |

---

## 3. 機能要件

### 3.1 機能一覧

**※ REQ-CLI-001（F-001〜F-014）、REQ-CLI-002（F-015〜F-022）との連番を維持**

| ID | 機能名 | 概要 | 優先度 | フェーズ | 備考 |
|----|--------|------|--------|---------|------|
| F-023 | MMLループ構文 | `[]` で囲んだ範囲をループ | 必須 | Phase 2.0 | **新規** |
| F-024 | 小文字MML記述 | 小文字でMMLコマンドを記述可能 | 必須 | Phase 2.0 | **新規** |
| F-025 | 履歴メモ機能 | `--note` オプションで履歴にメモを付与 | 重要 | Phase 2.0 | **新規** |
| F-026 | 履歴削除機能 | `clear-history` コマンドで全履歴削除 | 重要 | Phase 2.0 | **新規** |

### 3.2 ユーザーストーリー

#### US-011: MMLでループを使いたい
- **ユーザー**: 作曲者
- **したいこと**: 繰り返しパターンを `[]` で囲んで簡潔に記述したい
- **理由**: 同じフレーズを何度も書くのが面倒、MML文字列が長くなる
- **受け入れ基準**:
  - [ ] `[CDEF]3` で「CDEF」を3回繰り返し再生される
  - [ ] `[CDEF]` （回数なし）は1回のみ再生される（無限ループ防止）
  - [ ] `[CD:EF]2` で、1回目は「CDEF」、2回目は「CD」のみ再生される（脱出ポイント）
  - [ ] ループ構文のネストは非対応（エラー表示）
- **関連機能**: F-023

#### US-012: 小文字でMMLを書きたい
- **ユーザー**: 開発者
- **したいこと**: `cdefgab`, `r`, `o5`, `l8`, `t140`, `v10` のように小文字で書きたい
- **理由**: Shiftキーを押すのが面倒、タイピング効率を上げたい
- **受け入れ基準**:
  - [ ] `cdefgab` が `CDEFGAB` と同じように解析される
  - [ ] `r` が `R` と同じように解析される
  - [ ] `o5`, `l8`, `t140`, `v10` が `O5`, `L8`, `T140`, `V10` と同じように解析される
  - [ ] 大文字と小文字を混在させても正常に動作する（例: `CdEfGaB`）
- **関連機能**: F-024

#### US-013: 履歴にメモを付けたい
- **ユーザー**: 作曲者
- **したいこと**: 履歴にメモを付けて、後で見返したときに何の曲か分かるようにしたい
- **理由**: 履歴が増えると、どのMMLが何の曲か分からなくなる
- **受け入れ基準**:
  - [ ] `sine-mml play "CDEFGAB" --note "My first melody"` でメモが保存される
  - [ ] `sine-mml history` でメモが表示される
  - [ ] `--loop-play` と併用可能
  - [ ] メモなしでも動作する（既存の動作を維持）
- **関連機能**: F-025

#### US-014: 履歴を全削除したい
- **ユーザー**: 作曲者
- **したいこと**: 不要な履歴を一括削除したい
- **理由**: 試作品が溜まって履歴が見づらくなる
- **受け入れ基準**:
  - [ ] `sine-mml clear-history` で全履歴が削除される
  - [ ] 確認プロンプトが表示される（誤操作防止）
  - [ ] 確認で「y」を入力すると削除、「n」を入力するとキャンセル
  - [ ] 削除後、`sine-mml history` で履歴が空になる
- **関連機能**: F-026

### 3.3 機能詳細

#### F-023: MMLループ構文

**概要**: `[]` で囲んだ範囲を指定回数ループ、`:` で脱出ポイントを設定

**入力**:
- MML文字列（ループ構文を含む）
  - 例: `[CDEF]3`, `[CD:EF]2`, `[CDEFGAB]`

**出力**:
- ループ展開されたMMLコマンド列

**処理概要**:
1. トークナイザーで `[`, `]`, `:`, 数値を認識
2. パーサーで `[]` の範囲を特定
3. `]` の後の数値を取得（なければ1回）
4. ループ内の `:` を脱出ポイントとして記録
5. ループ回数分、コマンド列を展開
   - 最終ループ時は `:` 以降を除外

**ビジネスルール**:
- BR-050: `[]` の後に数値がない場合は1回のみ実行（無限ループ防止）
- BR-051: ループ回数は 1-99 の範囲内（100回以上はエラー）
- BR-052: `:` はループ内でのみ有効（ループ外では無視またはエラー）
- BR-053: ネストしたループ（`[[CDEF]2]3` 等）は非対応（エラー）
- BR-054: `[` と `]` の対応が取れない場合はエラー

**制約事項**:
- ループのネストは非対応（Phase 2.0では実装しない）
- ループ回数の上限は99回

**技術実装のポイント**:
```rust
// トークン定義（追加）
pub enum Token {
    // 既存のトークン...
    LoopStart,      // [
    LoopEnd,        // ]
    LoopEscape,     // :
    Number(u32),    // 既存（ループ回数にも使用）
}

// パーサー実装例
fn parse_loop(&mut self) -> Result<Command, ParseError> {
    self.consume(Token::LoopStart)?;
    
    let mut commands = Vec::new();
    let mut escape_index = None;
    
    while !self.check(Token::LoopEnd) {
        if self.check(Token::LoopEscape) {
            self.advance();
            escape_index = Some(commands.len());
        } else {
            commands.push(self.parse_command()?);
        }
    }
    
    self.consume(Token::LoopEnd)?;
    
    let repeat_count = if self.check(Token::Number(_)) {
        let Token::Number(n) = self.advance().token else { unreachable!() };
        if n > 99 {
            return Err(ParseError::InvalidLoopCount(n));
        }
        n as usize
    } else {
        1  // デフォルトは1回（無限ループ防止）
    };
    
    Ok(Command::Loop {
        commands,
        escape_index,
        repeat_count,
    })
}

// ループ展開例
fn expand_loop(commands: &[Command], escape_index: Option<usize>, repeat_count: usize) -> Vec<Command> {
    let mut expanded = Vec::new();
    for i in 0..repeat_count {
        let is_last = i == repeat_count - 1;
        let end_index = if is_last && escape_index.is_some() {
            escape_index.unwrap()
        } else {
            commands.len()
        };
        expanded.extend_from_slice(&commands[..end_index]);
    }
    expanded
}
```

**エラーメッセージ例**:
```
Error: Unmatched '[' at position 12
  O4 L4 [CDEFGAB
            ^
Expected: ']' to close loop

Error: Loop count must be 1-99 (found: 150)
  [CDEF]150
        ^^^
```

---

#### F-024: 小文字MML記述

**概要**: MMLコマンドを小文字で記述可能にする（正規化処理）

**入力**:
- MML文字列（小文字を含む）
  - 例: `cdefgab`, `r`, `o5`, `l8`, `t140`, `v10`

**出力**:
- 大文字に正規化されたMMLコマンド列

**処理概要**:
1. トークナイザーの前処理で、MML文字列を大文字に変換
2. 既存のパーサーロジックはそのまま使用（大文字前提）

**ビジネスルール**:
- BR-055: 小文字のMMLコマンドは大文字に正規化される
- BR-056: 大文字と小文字の混在も許可される（例: `CdEfGaB`）
- BR-057: 正規化は `A-Z`, `a-z` のみ対象（記号、数値は変換しない）

**制約事項**:
- 正規化は ASCII 範囲内のみ（Unicode は非対応）

**技術実装のポイント**:
```rust
// トークナイザーの前処理
pub fn tokenize(input: &str) -> Result<Vec<TokenWithPos>, ParseError> {
    // 小文字を大文字に正規化
    let normalized = input.to_uppercase();
    
    // 既存のトークナイザーロジック
    tokenize_impl(&normalized)
}

// または、トークナイザー内で文字ごとに正規化
fn next_char(&mut self) -> Option<char> {
    self.input.next().map(|c| c.to_ascii_uppercase())
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_lowercase_mml() {
    let mml = "cdefgab r o5 l8 t140 v10";
    let ast = parse(mml).unwrap();
    // 大文字と同じ結果になることを確認
    assert_eq!(ast, parse("CDEFGAB R O5 L8 T140 V10").unwrap());
}
```

---

#### F-025: 履歴メモ機能

**概要**: `--note` オプションで履歴にメモを付与

**入力**:
- `--note "メモ内容"` オプション（任意）

**出力**:
- 履歴DBへのメモ保存

**処理概要**:
1. CLIに `--note` オプションを追加（`PlayArgs` に `note: Option<String>` を追加）
2. DBスキーマに `note` カラムを追加（`TEXT NULL`）
3. 履歴保存時にメモも保存
4. 履歴一覧表示時にメモも表示

**ビジネスルール**:
- BR-058: メモは任意（指定しなくても動作する）
- BR-059: メモの最大長は 500 文字
- BR-060: メモは UTF-8 文字列（絵文字も可）
- BR-061: `--loop-play` と併用可能

**制約事項**:
- メモの最大長は 500 文字（DB制約）

**技術実装のポイント**:

**1. CLIオプション追加**:
```rust
// src/cli/args.rs
#[derive(Args, Debug)]
pub struct PlayArgs {
    // 既存のフィールド...
    
    #[arg(long)]
    pub note: Option<String>,
}
```

**2. DBスキーマ変更**:
```sql
-- マイグレーション: v1 → v2
ALTER TABLE history ADD COLUMN note TEXT NULL;

-- 新規テーブル定義
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mml TEXT NOT NULL,
    waveform TEXT NOT NULL CHECK(waveform IN ('sine', 'sawtooth', 'square')),
    volume REAL NOT NULL CHECK(volume >= 0.0 AND volume <= 1.0),
    bpm INTEGER NOT NULL CHECK(bpm >= 30 AND bpm <= 300),
    note TEXT NULL CHECK(length(note) <= 500),  -- 追加
    created_at TEXT NOT NULL
);
```

**3. 履歴保存処理**:
```rust
// src/db/history.rs
pub fn save(
    &self,
    mml: &str,
    waveform: &str,
    volume: f32,
    bpm: u16,
    note: Option<&str>,  // 追加
) -> Result<i64, DbError> {
    let conn = self.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO history (mml, waveform, volume, bpm, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        (mml, waveform, volume, bpm, note),
    )?;
    Ok(conn.last_insert_rowid())
}
```

**4. 履歴一覧表示**:
```rust
// src/cli/handlers.rs
pub fn history_handler() -> Result<()> {
    let db = Database::init()?;
    let entries = db.get_all()?;
    
    let mut table = Table::new();
    table.set_header(vec!["ID", "Created At", "MML", "Waveform", "BPM", "Volume", "Note"]);
    
    for entry in entries {
        table.add_row(vec![
            entry.id.to_string(),
            entry.created_at,
            entry.mml,
            entry.waveform,
            entry.bpm.to_string(),
            entry.volume.to_string(),
            entry.note.unwrap_or_else(|| "-".to_string()),  // メモなしは "-"
        ]);
    }
    
    println!("{table}");
    Ok(())
}
```

**出力形式例**:
```
ID | Created At          | MML          | Waveform | BPM | Volume | Note
---|---------------------|--------------|----------|-----|--------|------------------
5  | 2026-01-11 10:30:00 | CDEFGAB      | sine     | 120 | 0.5    | My first melody
4  | 2026-01-09 14:20:00 | [CDEF]4      | square   | 140 | 0.7    | Loop test
3  | 2026-01-08 09:15:00 | O5 T180 CRCR | sawtooth | 180 | 0.8    | -
```

---

#### F-026: 履歴削除機能

**概要**: `clear-history` コマンドで全履歴を削除

**入力**:
- `clear-history` サブコマンド

**出力**:
- 全履歴の削除（確認プロンプト付き）

**処理概要**:
1. CLIに `ClearHistory` サブコマンドを追加
2. 確認プロンプトを表示（標準入力から読み取り）
3. 「y」入力で全履歴削除、「n」入力でキャンセル
4. 削除後、成功メッセージを表示

**ビジネスルール**:
- BR-062: 確認プロンプトは必須（誤操作防止）
- BR-063: 「y」「Y」「yes」「Yes」で削除実行
- BR-064: 「n」「N」「no」「No」でキャンセル
- BR-065: それ以外の入力はエラー（再入力を促す）
- BR-066: 削除は取り消し不可（警告メッセージを表示）

**制約事項**:
- 個別削除、条件付き削除は非対応（Phase 2.0では実装しない）

**技術実装のポイント**:

**1. CLIサブコマンド追加**:
```rust
// src/cli/args.rs
#[derive(Subcommand, Debug)]
pub enum Command {
    Play(PlayArgs),
    History,
    Export(ExportArgs),
    ClearHistory,  // 追加
}
```

**2. 確認プロンプト実装**:
```rust
// src/cli/handlers.rs
use std::io::{self, Write};

pub fn clear_history_handler() -> Result<()> {
    println!("⚠️  警告: 全ての履歴を削除します。この操作は取り消せません。");
    print!("本当に削除しますか？ (y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            let db = Database::init()?;
            db.clear_all()?;
            println!("✓ 全ての履歴を削除しました。");
            Ok(())
        }
        "n" | "no" => {
            println!("キャンセルしました。");
            Ok(())
        }
        _ => {
            Err(anyhow::anyhow!("無効な入力です。'y' または 'n' を入力してください。"))
        }
    }
}
```

**3. DB削除処理**:
```rust
// src/db/history.rs
pub fn clear_all(&self) -> Result<(), DbError> {
    let conn = self.conn.lock().unwrap();
    conn.execute("DELETE FROM history", [])?;
    Ok(())
}
```

**実行例**:
```bash
$ sine-mml clear-history
⚠️  警告: 全ての履歴を削除します。この操作は取り消せません。
本当に削除しますか？ (y/n): y
✓ 全ての履歴を削除しました。

$ sine-mml history
履歴がありません。
```

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-008 | ループ展開のオーバーヘッド | 10ms以内（ループ回数99回） | ベンチマーク |
| NFR-P-009 | 小文字正規化のオーバーヘッド | 1ms以内（1000文字のMML） | ベンチマーク |
| NFR-P-010 | 履歴削除速度 | 100ms以内（1000件） | SQLite DELETE測定 |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-005 | ループ構文のエラーハンドリング | 不正なループ構文でクラッシュしない |
| NFR-A-006 | DB互換性 | 既存の履歴データを保持（マイグレーション） |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-006 | ループ回数の制限 | 1-99回に制限（DoS攻撃防止） |
| NFR-S-007 | メモの長さ制限 | 500文字に制限（DB肥大化防止） |
| NFR-S-008 | 履歴削除の確認 | 確認プロンプト必須（誤操作防止） |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-008 | ループ構文のエラーメッセージ | 位置情報と修正ヒントを表示 |
| NFR-U-009 | 小文字記述の透過性 | ユーザーは大文字/小文字を意識しない |
| NFR-U-010 | 履歴メモの表示 | メモなしは "-" と表示（空欄にしない） |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-008 | DBマイグレーション | スキーマバージョン管理（v1→v2） |
| NFR-M-009 | ループ展開のテスト | エッジケース（ネスト、不正な構文等）を網羅 |
| NFR-M-010 | 後方互換性 | 既存のMML文字列が正常に動作 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| ループのネスト | 非対応 | パーサーの複雑性を抑える |
| ループ回数の上限 | 99回 | DoS攻撃防止、パフォーマンス維持 |
| メモの最大長 | 500文字 | DB肥大化防止 |
| DBスキーマバージョン | v2に更新 | `note` カラム追加のため |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 2.0: 2週間 |
| リソース | 個人開発（1名） |

### 5.3 法規制・コンプライアンス

| 要件 | 詳細 |
|------|------|
| ライセンス | MIT License（変更なし） |
| データ保護 | 履歴削除は取り消し不可（警告表示） |

---

## 6. 外部インターフェース

### 6.1 既存機能への影響

| 機能ID | 機能名 | 影響内容 | 対応 |
|--------|--------|----------|------|
| F-001 | MMLパーサー | **拡張** | ループ構文、小文字対応を追加 |
| F-007 | 履歴保存 | **修正** | `note` カラムを追加 |
| F-008 | 履歴一覧表示 | **修正** | メモ列を追加 |

### 6.2 CLIインターフェース変更

#### 追加されるオプション
```bash
# playコマンドに追加
--note <TEXT>  # 履歴にメモを付与
```

#### 追加されるサブコマンド
```bash
sine-mml clear-history  # 全履歴削除
```

#### 変更なし
```bash
sine-mml play "MML"
sine-mml history
sine-mml export --history-id <ID> --output <FILE>
```

### 6.3 MML構文の拡張

#### 追加される構文
```
[<commands>]<count>  # ループ構文
[<commands>:<commands>]<count>  # 脱出ポイント付きループ
```

#### 例
```
[CDEF]3           # CDEFを3回繰り返し
[CD:EF]2          # 1回目: CDEF、2回目: CD
[CDEFGAB]         # 1回のみ（無限ループ防止）
cdefgab           # 小文字でも記述可能
```

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の全機能が実装済み（v1.0.0リリース済み）
- REQ-CLI-002の全機能が実装済み（v2.0.0リリース済み）
- 既存の履歴データが存在する可能性がある（マイグレーション必要）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| rusqlite | DBマイグレーション | スキーマバージョン管理 |
| clap | CLIオプション追加 | `--note` オプション、`clear-history` サブコマンド |

### 7.3 DBマイグレーション

```rust
// src/db/schema.rs
pub const CURRENT_VERSION: i64 = 2;  // v1 → v2

pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let version: i64 = conn.query_row(
        "SELECT version FROM schema_version",
        [],
        |row| row.get(0),
    )?;
    
    if version < 2 {
        // v1 → v2: note カラム追加
        conn.execute("ALTER TABLE history ADD COLUMN note TEXT NULL", [])?;
        conn.execute("UPDATE schema_version SET version = 2", [])?;
    }
    
    Ok(())
}
```

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-010 | ループ構文のパーサーバグ | 高 | 中 | 豊富なテストケース、エッジケース対応 |
| R-011 | DBマイグレーション失敗 | 高 | 低 | バックアップ推奨、ロールバック手順準備 |
| R-012 | 既存MMLとの互換性問題 | 中 | 低 | 既存のMMLが正常に動作することを確認 |
| R-013 | ループ回数の上限不足 | 低 | 低 | ユーザーフィードバックで調整 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-009 | ループのネスト対応の検討 | 開発チーム | Phase 3.0で検討 |
| I-010 | 履歴の選択的削除機能 | 開発チーム | Phase 3.0で検討 |
| I-011 | メモの検索機能 | 開発チーム | Phase 3.0で検討 |
| I-012 | ループ回数の上限値の妥当性検証 | 開発チーム | 2026-01-25 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| ループ構文 | MMLで繰り返しパターンを記述する構文（`[]`） |
| 脱出ポイント | ループの最終回で実行を中断する位置（`:`） |
| 正規化 | 小文字を大文字に変換する処理 |
| マイグレーション | DBスキーマのバージョンアップ処理 |
| 確認プロンプト | ユーザーに確認を求める対話的な入力 |

---

## 10. 参考リンク

### 10.1 MML仕様

#### ループ構文の参考
- **NuttX MML Parser**: [Documentation](https://nuttx.apache.org/docs/latest/applications/audioutils/mml_parser/index.html) - 基本的なMML仕様
- **MML (Music Macro Language) - Wikipedia**: [日本語版](https://ja.wikipedia.org/wiki/Music_Macro_Language) - MMLの歴史と構文
- **PC-98 MML仕様**: ループ構文 `[]` の元祖（参考資料）

### 10.2 Rustパーサー実装

#### 再帰下降パーサー
- **Crafting Interpreters**: [Parsing Expressions](https://craftinginterpreters.com/parsing-expressions.html) - パーサー実装の基礎
- **Rust Parser Combinators**: [nom](https://docs.rs/nom/) - 参考（本プロジェクトでは手動実装を推奨）

#### 文字列正規化
- **Rust String Methods**: [to_uppercase](https://doc.rust-lang.org/std/primitive.str.html#method.to_uppercase) - 大文字変換

### 10.3 SQLiteマイグレーション

#### スキーマバージョン管理
- **rusqlite Documentation**: [Transactions](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html)
- **SQLite ALTER TABLE**: [Documentation](https://www.sqlite.org/lang_altertable.html) - カラム追加

### 10.4 CLIユーザー入力

#### 標準入力読み取り
- **Rust std::io**: [stdin](https://doc.rust-lang.org/std/io/fn.stdin.html) - 標準入力から読み取り
- **dialoguer**: [Rust CLI Prompts](https://docs.rs/dialoguer/) - より高度な確認プロンプト（参考）

---

## 11. 実装優先順位と段階的ロールアウト

### Phase 2.0.1（Week 1: 2026-01-11〜01-18）
1. **F-024（小文字MML記述）** - 最も簡単、影響範囲が小さい
   - トークナイザーに正規化処理を追加
   - 既存のテストが通ることを確認
2. **F-025（履歴メモ機能）** - DBマイグレーションが必要
   - DBスキーマに `note` カラム追加
   - マイグレーション処理実装
   - CLIオプション追加

### Phase 2.0.2（Week 2: 2026-01-19〜01-25）
3. **F-023（MMLループ構文）** - 最も複雑、パーサー拡張が必要
   - トークナイザーに `[`, `]`, `:` を追加
   - パーサーにループ解析ロジックを追加
   - ループ展開処理実装
   - 豊富なテストケース作成
4. **F-026（履歴削除機能）** - 簡単、独立した機能
   - `clear-history` サブコマンド追加
   - 確認プロンプト実装

---

## 12. テストケース概要

### 12.1 ループ構文のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-023-001 | `[CDEF]3` | CDEFを3回繰り返し |
| TC-023-002 | `[CDEF]` | CDEFを1回のみ |
| TC-023-003 | `[CD:EF]2` | 1回目: CDEF、2回目: CD |
| TC-023-004 | `[CDEF]0` | エラー（ループ回数は1以上） |
| TC-023-005 | `[CDEF]100` | エラー（ループ回数は99以下） |
| TC-023-006 | `[[CDEF]2]3` | エラー（ネストは非対応） |
| TC-023-007 | `[CDEF` | エラー（`]` がない） |
| TC-023-008 | `CDEF]` | エラー（`[` がない） |

### 12.2 小文字MML記述のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-024-001 | `cdefgab` | `CDEFGAB` と同じ |
| TC-024-002 | `r` | `R` と同じ |
| TC-024-003 | `o5 l8 t140 v10` | `O5 L8 T140 V10` と同じ |
| TC-024-004 | `CdEfGaB` | `CDEFGAB` と同じ |

### 12.3 履歴メモ機能のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-025-001 | `--note "My melody"` | メモが保存される |
| TC-025-002 | メモなし | メモは NULL |
| TC-025-003 | `--note "あいうえお🎵"` | UTF-8文字列が保存される |
| TC-025-004 | `--note "500文字以上"` | エラー（長さ制限） |

### 12.4 履歴削除機能のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-026-001 | `clear-history` → `y` | 全履歴削除 |
| TC-026-002 | `clear-history` → `n` | キャンセル |
| TC-026-003 | `clear-history` → `invalid` | エラー（再入力を促す） |

---

## 13. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-11 | 初版作成 | req-writer |
