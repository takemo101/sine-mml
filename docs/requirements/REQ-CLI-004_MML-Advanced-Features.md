# MML Synthesizer CLI 高度な機能拡張 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-004 |
| バージョン | 1.0.1 |
| ステータス | ドラフト |
| 作成日 | 2026-01-12 |
| 最終更新日 | 2026-01-12 |
| 作成者 | req-writer |
| 承認者 | - |
| 関連文書 | REQ-CLI-001_MML-Synthesizer.md (v1.1.0)<br>REQ-CLI-002_MML-Synthesizer-Enhancement.md (v1.0.0)<br>REQ-CLI-003_MML-Syntax-Extension.md (v1.0.0)<br>docs/memos/ファイル読み取り仕様追加.md |

---

## 1. プロジェクト概要

### 1.1 背景

sine-mml v2.1の実装完了後、ユーザーからの要望と実用性向上のため、以下の機能追加が必要となりました：

1. **MMLファイル読み取りの欠如**: 長いMMLを直接コマンドライン引数で指定する必要があり、複雑な楽曲の管理が困難
2. **相対的な音量調整の不在**: 音量（V0-V15）の絶対値指定のみで、相対的な増減（V+n, V-n）ができない
3. **ループのネスト制限**: REQ-CLI-003でループ構文が追加されたが、ネスト（入れ子）は非対応のため、複雑なパターンを表現できない

これらの改善により、MML記述の実用性と表現力を大幅に向上させます。

### 1.2 目的

- `.mml`ファイルからMMLを読み込み、コメントやセクション区切りを含む長い楽曲を管理可能にする
- 相対的なボリューム指定（V+n, V-n）により、音量の動的な増減を可能にする
- ループのネスト（最大5階層）により、複雑な繰り返しパターンを簡潔に記述可能にする

### 1.3 ゴール

| 目標 | 成功指標 |
|------|---------|
| MMLファイル読み取りの実装 | `sine-mml play --file song.mml` でファイルから読み込み、コメント行（`#`始まり）が無視される |
| 相対ボリューム指定の実装 | `V10`, `V+2`, `V-3` が正常に解析され、音量が変化する |
| ループネストの実装 | `[ CDE [ FGAB ]2 ]3` が正常に展開され、内側のループが2回、外側のループが3回繰り返される |

### 1.4 スコープ

#### 対象範囲
- CLIへの `--file` オプション追加
- `.mml`ファイルのパース（コメント、空行の除去）
- MMLパーサーへのボリュームコマンド拡張（`V0-V15`維持、相対指定 `V+n`, `V-n`追加）
- ループ構文のネスト対応（最大5階層）
- エラーメッセージの改善（ファイル読み取りエラー、ネスト深度超過等）

#### 対象外
- ファイル形式の拡張（`.mml`以外のフォーマット）
- ファイル内のセクション指定（特定のセクションのみ再生等）
- ボリュームのエンベロープ制御（ADSR等）
- ループネストの無制限対応（6階層以上）

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

| 役割 | 担当者/部門 | 関心事 | 影響度 |
|------|------------|--------|--------|
| プロダクトオーナー | - | ユーザー要望への対応、実用性向上 | 高 |
| 開発チーム | - | パーサー拡張の複雑性、後方互換性 | 高 |
| エンドユーザー | 音楽制作者、趣味プログラマー | 長い楽曲の管理、表現力の向上 | 高 |

### 2.2 ユーザーペルソナ

#### ペルソナ1: 長い楽曲を作成する作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、音楽経験あり |
| 課題 | 長いMMLをコマンドライン引数で指定するのが困難、コメントやセクション区切りを入れたい |
| ニーズ | `.mml`ファイルで楽曲を管理し、コメントで構成を記録したい |
| 利用シーン | 複数のセクション（イントロ、Aメロ、Bメロ等）を含む楽曲の作成 |

#### ペルソナ2: 相対的にボリュームを調整したい作曲者
| 項目 | 内容 |
|------|------|
| 属性 | 30代、音楽経験あり |
| 課題 | クレッシェンド、デクレッシェンドで毎回絶対値を計算するのが面倒 |
| ニーズ | 現在のボリュームから相対的に増減したい（V+2, V-3 等） |
| 利用シーン | クレッシェンド、デクレッシェンド等の動的な強弱変化を含む楽曲の作成 |

#### ペルソナ3: 複雑なパターンを簡潔に記述したい開発者
| 項目 | 内容 |
|------|------|
| 属性 | 20-40代、プログラミング経験豊富 |
| 課題 | ループのネストができず、複雑なパターンを冗長に書く必要がある |
| ニーズ | ループのネストで複雑なパターンを簡潔に記述したい |
| 利用シーン | リフの中にさらに繰り返しパターンを含む楽曲の作成 |

---

## 3. 機能要件

### 3.1 機能一覧

**※ REQ-CLI-001（F-001〜F-014）、REQ-CLI-002（F-015〜F-022）、REQ-CLI-003（F-023〜F-026）との連番を維持**

| ID | 機能名 | 概要 | 優先度 | フェーズ | 備考 |
|----|--------|------|--------|---------|------|
| F-027 | MMLファイル読み取り | `.mml`ファイルからMMLを読み込み | 必須 | Phase 2.1 | **新規** |
| F-028 | 相対ボリューム指定 | 既存V0-V15に相対指定（V+n, V-n）を追加 | 必須 | Phase 2.1 | **新規（後方互換性維持）** |
| F-029 | ループネスト対応 | ループ構文のネスト（最大5階層） | 必須 | Phase 2.1 | **新規** |

### 3.2 ユーザーストーリー

#### US-015: MMLファイルから読み込みたい
- **ユーザー**: 作曲者
- **したいこと**: `.mml`ファイルに楽曲を保存し、ファイルから読み込んで再生したい
- **理由**: 長いMMLをコマンドライン引数で指定するのが困難、コメントやセクション区切りを入れたい
- **受け入れ基準**:
  - [ ] `sine-mml play --file song.mml` でファイルから読み込み可能
  - [ ] `#`で始まる行はコメントとして無視される
  - [ ] 空行は無視される
  - [ ] ファイルが存在しない場合はエラーメッセージを表示
  - [ ] ファイル読み取りエラー時は詳細なエラーメッセージを表示
- **関連機能**: F-027

#### US-016: 相対的にボリュームを調整したい
- **ユーザー**: 作曲者
- **したいこと**: 現在のボリュームから相対的に増減したい
- **理由**: クレッシェンド、デクレッシェンド等で毎回絶対値を計算するのが面倒
- **受け入れ基準**:
  - [ ] `V10` で絶対値指定が可能（既存仕様維持）
  - [ ] `V+2` で現在値から+2の相対指定が可能
  - [ ] `V-3` で現在値から-3の相対指定が可能
  - [ ] `V+` または `V-` でデフォルト増減値（1）が適用される
  - [ ] ボリュームが0-15の範囲外になる場合はクランプされる
  - [ ] デフォルトはV10（既存仕様維持）
- **関連機能**: F-028

#### US-017: ループをネストして複雑なパターンを記述したい
- **ユーザー**: 開発者
- **したいこと**: ループのネストで複雑なパターンを簡潔に記述したい
- **理由**: ループのネストができず、複雑なパターンを冗長に書く必要がある
- **受け入れ基準**:
  - [ ] `[ CDE [ FGAB ]2 ]3` が正常に展開される
  - [ ] 内側のループが2回、外側のループが3回繰り返される
  - [ ] 最大5階層までネスト可能
  - [ ] 6階層以上のネストはエラーメッセージを表示
  - [ ] ネストしたループ内でも脱出ポイント（`:`）が使用可能
- **関連機能**: F-029

### 3.3 機能詳細

#### F-027: MMLファイル読み取り

**概要**: `.mml`ファイルからMMLを読み込み、コメントや空行を除去して解析

**入力**:
- `--file <path>` オプション（ファイルパス）

**出力**:
- MML文字列（コメント、空行除去済み）

**処理概要**:
1. CLIに `--file` オプションを追加（`PlayArgs` に `file: Option<String>` を追加）
2. ファイルが存在するか確認
3. ファイルを読み込み、UTF-8でデコード
4. 各行を処理:
   - `#`で始まる行はコメントとして除去
   - 空行（空白文字のみの行）は除去
   - それ以外の行は連結
5. 連結したMML文字列を既存のパーサーに渡す

**ビジネスルール**:
- BR-067: `--file` と `mml` 引数は排他的（両方指定はエラー）
- BR-068: コメント行は `#` で始まる（行頭の空白は許可）
- BR-069: 空行は無視される
- BR-070: ファイルはUTF-8エンコーディング
- BR-071: ファイルサイズ上限は 1MB（DoS攻撃防止）

**制約事項**:
- ファイル形式は `.mml` のみ対応
- セクション指定（特定のセクションのみ再生等）は非対応

**技術実装のポイント**:

**1. CLIオプション追加**:
```rust
// src/cli/args.rs
#[derive(Args, Debug)]
#[command(group(
    clap::ArgGroup::new("input")
        .required(true)
        .args(["mml", "history_id", "file"]),  // file を追加
))]
pub struct PlayArgs {
    pub mml: Option<String>,
    
    #[arg(long)]
    pub history_id: Option<i64>,
    
    /// MMLファイルのパス（.mml拡張子）
    #[arg(long)]
    pub file: Option<String>,
    
    // 既存のフィールド...
}
```

**2. ファイル読み取り処理**:
```rust
// src/mml/file.rs (新規モジュール)
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

/// MMLファイルを読み込み、コメントと空行を除去してMML文字列を返す
pub fn read_mml_file(path: &str) -> Result<String> {
    // ファイル存在確認
    let path = Path::new(path);
    if !path.exists() {
        anyhow::bail!("ファイルが見つかりません: {}", path.display());
    }
    
    // 拡張子確認
    if path.extension().and_then(|s| s.to_str()) != Some("mml") {
        anyhow::bail!("ファイル拡張子は .mml である必要があります: {}", path.display());
    }
    
    // ファイルサイズ確認（1MB上限）
    let metadata = fs::metadata(path)?;
    if metadata.len() > 1_000_000 {
        anyhow::bail!("ファイルサイズが大きすぎます（上限: 1MB）: {}", path.display());
    }
    
    // ファイル読み込み
    let content = fs::read_to_string(path)
        .with_context(|| format!("ファイルの読み込みに失敗しました: {}", path.display()))?;
    
    // コメントと空行を除去
    let mml = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ");
    
    if mml.is_empty() {
        anyhow::bail!("ファイルにMMLが含まれていません: {}", path.display());
    }
    
    Ok(mml)
}
```

**3. ハンドラー統合**:
```rust
// src/cli/handlers.rs
pub fn play_handler(args: PlayArgs) -> Result<()> {
    // MML文字列を取得
    let mml_string = if let Some(file_path) = args.file {
        // ファイルから読み込み
        mml::file::read_mml_file(&file_path)?
    } else if let Some(mml) = args.mml {
        // コマンドライン引数から取得
        mml
    } else if let Some(history_id) = args.history_id {
        // 履歴から取得
        let db = Database::init()?;
        let entry = db.get_by_id(history_id)?;
        entry.mml
    } else {
        unreachable!("clap ArgGroup ensures one of mml, history_id, or file is provided");
    };
    
    // 既存の処理...
}
```

**MMLファイル例**:
```mml
# イントロ
T120 L8 O5
[CDEF]2 GAB >C

# Aメロ
O4 L4
CDEFGAB >C

# Bメロ
O5 L8
[CD:EF]2 GAB

# アウトロ
O4 L2
C R
```

**エラーメッセージ例**:
```
Error: ファイルが見つかりません: song.mml

Error: ファイル拡張子は .mml である必要があります: song.txt

Error: ファイルサイズが大きすぎます（上限: 1MB）: huge_song.mml

Error: ファイルの読み込みに失敗しました: /path/to/song.mml
Caused by: Permission denied (os error 13)
```

---

#### F-028: 相対ボリューム指定

**概要**: 既存のボリューム（V0-V15）に相対指定機能を追加し、音量の動的な増減を可能にする

**入力**:
- MML文字列（ボリュームコマンドを含む）
  - 例: `V10`, `V+2`, `V-3`, `V+`, `V-`

**出力**:
- ボリューム値（0-15）

**処理概要**:
1. トークナイザーで `V` コマンドを認識
2. 数値を取得（絶対値または相対値）
3. 相対値の場合は現在のボリュームに加算/減算
4. 0-15の範囲にクランプ
5. デフォルトはV10（既存仕様を維持）

**ビジネスルール**:
- BR-072: ボリューム範囲は 0-15（既存仕様を維持）
- BR-073: デフォルトはV10（既存仕様を維持）
- BR-074: 相対指定 `V+n` は現在値に+n
- BR-075: 相対指定 `V-n` は現在値に-n
- BR-076: `V+` または `V-` のみの場合はデフォルト増減値1を使用
- BR-077: 範囲外の値は0-15にクランプ（エラーにしない）

**制約事項**:
- 既存のボリューム（V0-V15）との完全な後方互換性を維持
- エンベロープ制御（ADSR等）は非対応

**技術実装のポイント**:

**1. AST拡張**:
```rust
// src/mml/ast.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// 絶対値（0-15）または相対値（-15〜+15）
    pub value: VolumeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeValue {
    /// 絶対値（0-15）
    Absolute(u8),
    /// 相対値（-15〜+15）
    Relative(i8),
}
```

**2. パーサー実装**:
```rust
// src/mml/parser.rs
fn parse_volume(&mut self) -> Result<Command, ParseError> {
    self.consume('V')?;
    
    // 相対指定のチェック
    let value = if self.check('+') {
        self.advance();
        let delta = if self.current_char().is_ascii_digit() {
            self.parse_number()? as i8
        } else {
            1  // デフォルト増減値
        };
        VolumeValue::Relative(delta)
    } else if self.check('-') {
        self.advance();
        let delta = if self.current_char().is_ascii_digit() {
            -(self.parse_number()? as i8)
        } else {
            -1  // デフォルト増減値
        };
        VolumeValue::Relative(delta)
    } else {
        // 絶対値
        let val = self.parse_number()?;
        if val > 15 {
            return Err(ParseError::InvalidVolume(val));
        }
        VolumeValue::Absolute(val as u8)
    };
    
    Ok(Command::Volume(Volume { value }))
}
```

**3. 合成時の処理**:
```rust
// src/audio/synthesizer.rs
pub struct SynthesizerState {
    current_volume: u8,  // 0-15
    // 既存のフィールド...
}

impl SynthesizerState {
    pub fn apply_volume(&mut self, volume: &Volume) {
        self.current_volume = match volume.value {
            VolumeValue::Absolute(v) => v,
            VolumeValue::Relative(delta) => {
                let new_val = (self.current_volume as i8 + delta).clamp(0, 15);
                new_val as u8
            }
        };
    }
    
    /// ボリュームを音量係数に変換（0.0-1.0）
    pub fn volume_to_amplitude(&self) -> f32 {
        self.current_volume as f32 / 15.0
    }
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_volume_absolute() {
    let mml = parse("V10 C V5 D V15 E").unwrap();
    // V10, V5, V15 が正常に解析される
}

#[test]
fn test_volume_relative() {
    let mml = parse("V10 C V+2 D V-3 E").unwrap();
    // V10 → V12 → V9 と変化
}

#[test]
fn test_volume_default_delta() {
    let mml = parse("V10 C V+ D V- E").unwrap();
    // V10 → V11 → V10 と変化
}

#[test]
fn test_volume_clamp() {
    let mml = parse("V15 V+2 C").unwrap();
    // V15 + 2 = 17 → 15 にクランプ
}

#[test]
fn test_volume_clamp_lower() {
    let mml = parse("V0 V-2 C").unwrap();
    // V0 - 2 = -2 → 0 にクランプ
}
```

**エラーメッセージ例**:
```
Error: Invalid volume value: 20 (must be 0-15)
  V20 CDEFGAB
  ^^^
Expected: V0-V15
```

---

#### F-029: ループネスト対応

**概要**: ループ構文のネスト（最大5階層）に対応

**入力**:
- MML文字列（ネストしたループ構文を含む）
  - 例: `[ CDE [ FGAB ]2 ]3`

**出力**:
- ループ展開されたMMLコマンド列

**処理概要**:
1. パーサーでループ開始 `[` を検出
2. ネスト深度をカウント（最大5階層）
3. ループ内のコマンドを再帰的に解析
4. ループ終了 `]` を検出し、繰り返し回数を取得
5. ループを展開（内側から順に展開）

**ビジネスルール**:
- BR-079: ループのネストは最大5階層まで
- BR-080: 6階層以上のネストはエラー
- BR-081: ネストしたループ内でも脱出ポイント（`:`）が使用可能
- BR-082: ループ回数は各階層で1-99の範囲内
- BR-083: ネストしたループの総展開数は10,000コマンド以下（DoS攻撃防止）

**制約事項**:
- ネスト深度は最大5階層（6階層以上は非対応）
- 総展開数の上限は10,000コマンド

**技術実装のポイント**:

**1. パーサー実装**:
```rust
// src/mml/parser.rs
pub struct Parser {
    // 既存のフィールド...
    loop_depth: usize,  // 現在のループネスト深度
}

const MAX_LOOP_DEPTH: usize = 5;
const MAX_EXPANDED_COMMANDS: usize = 10_000;

fn parse_loop(&mut self) -> Result<Command, ParseError> {
    // ネスト深度チェック
    if self.loop_depth >= MAX_LOOP_DEPTH {
        return Err(ParseError::LoopNestTooDeep {
            max_depth: MAX_LOOP_DEPTH,
            position: self.pos,
        });
    }
    
    self.consume(Token::LoopStart)?;
    self.loop_depth += 1;
    
    let mut commands = Vec::new();
    let mut escape_index = None;
    
    while !self.check(Token::LoopEnd) {
        if self.check(Token::LoopEscape) {
            self.advance();
            escape_index = Some(commands.len());
        } else {
            // 再帰的にコマンドを解析（ネストしたループも含む）
            commands.push(self.parse_command()?);
        }
    }
    
    self.consume(Token::LoopEnd)?;
    self.loop_depth -= 1;
    
    let repeat_count = if self.check(Token::Number(_)) {
        let Token::Number(n) = self.advance().token else { unreachable!() };
        if n > 99 {
            return Err(ParseError::InvalidLoopCount(n));
        }
        n as usize
    } else {
        1
    };
    
    Ok(Command::Loop {
        commands,
        escape_index,
        repeat_count,
    })
}
```

**2. ループ展開処理**:
```rust
// src/mml/parser.rs
fn expand_loop(
    commands: &[Command],
    escape_index: Option<usize>,
    repeat_count: usize,
) -> Result<Vec<Command>, ParseError> {
    let mut expanded = Vec::new();
    
    for i in 0..repeat_count {
        let is_last = i == repeat_count - 1;
        let end_index = if is_last && escape_index.is_some() {
            escape_index.unwrap()
        } else {
            commands.len()
        };
        
        for cmd in &commands[..end_index] {
            // ネストしたループも再帰的に展開
            if let Command::Loop { commands: inner_cmds, escape_index: inner_escape, repeat_count: inner_count } = cmd {
                let inner_expanded = expand_loop(inner_cmds, *inner_escape, *inner_count)?;
                expanded.extend(inner_expanded);
            } else {
                expanded.push(cmd.clone());
            }
        }
    }
    
    // 総展開数チェック
    if expanded.len() > MAX_EXPANDED_COMMANDS {
        return Err(ParseError::LoopExpandedTooLarge {
            max_commands: MAX_EXPANDED_COMMANDS,
            actual: expanded.len(),
        });
    }
    
    Ok(expanded)
}
```

**3. エラー型追加**:
```rust
// src/mml/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    // 既存のエラー...
    
    #[error("ループのネストが深すぎます（最大{max_depth}階層）: 位置 {position}")]
    LoopNestTooDeep {
        max_depth: usize,
        position: usize,
    },
    
    #[error("ループ展開後のコマンド数が多すぎます（最大{max_commands}、実際: {actual}）")]
    LoopExpandedTooLarge {
        max_commands: usize,
        actual: usize,
    },
}
```

**受け入れテスト例**:
```rust
#[test]
fn test_loop_nest_2_levels() {
    let mml = parse("[ CDE [ FGAB ]2 ]3").unwrap();
    // 内側のループ: FGAB を2回 → FGABFGAB
    // 外側のループ: CDE FGABFGAB を3回
    // 結果: CDE FGABFGAB CDE FGABFGAB CDE FGABFGAB
}

#[test]
fn test_loop_nest_3_levels() {
    let mml = parse("[ [ [ C ]2 D ]2 E ]2").unwrap();
    // 最内側: C を2回 → CC
    // 中間: CC D を2回 → CC D CC D
    // 最外側: CC D CC D E を2回 → CC D CC D E CC D CC D E
}

#[test]
fn test_loop_nest_too_deep() {
    let result = parse("[ [ [ [ [ [ C ]2 ]2 ]2 ]2 ]2 ]2");
    assert!(result.is_err());
    // Error: ループのネストが深すぎます（最大5階層）
}

#[test]
fn test_loop_expanded_too_large() {
    let result = parse("[ [ [ C ]99 ]99 ]99");
    assert!(result.is_err());
    // Error: ループ展開後のコマンド数が多すぎます（最大10000、実際: 970299）
}

#[test]
fn test_loop_nest_with_escape() {
    let mml = parse("[ [ CD:EF ]2 GAB ]2").unwrap();
    // 内側のループ: 1回目 CDEF、2回目 CD
    // 外側のループ: [ CDEF CD GAB ]2
}
```

**エラーメッセージ例**:
```
Error: ループのネストが深すぎます（最大5階層）: 位置 12
  [ [ [ [ [ [ C ]2 ]2 ]2 ]2 ]2 ]2
                  ^
Expected: ループのネストは5階層まで

Error: ループ展開後のコマンド数が多すぎます（最大10000、実際: 970299）
  [ [ [ C ]99 ]99 ]99
  ^^^^^^^^^^^^^^^^^^^
Expected: ループ展開後のコマンド数は10000以下
```

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-011 | ファイル読み取り速度 | 100ms以内（1MB） | ファイルI/O測定 |
| NFR-P-012 | ループネスト展開速度 | 50ms以内（5階層、1000コマンド） | ベンチマーク |
| NFR-P-013 | ボリューム計算のオーバーヘッド | 1%以内（合成時間の） | ベンチマーク |

### 4.2 可用性要件

| ID | 要件 | 目標値 |
|----|------|--------|
| NFR-A-007 | ファイル読み取りエラーのハンドリング | 詳細なエラーメッセージを表示 |
| NFR-A-008 | ループネスト深度超過時のエラー | クラッシュせず、エラーメッセージを表示 |

### 4.3 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-009 | ファイルサイズ制限 | 1MB以下（DoS攻撃防止） |
| NFR-S-010 | ループ展開数制限 | 10,000コマンド以下（DoS攻撃防止） |
| NFR-S-011 | ファイルパストラバーサル対策 | 絶対パスまたは相対パスのみ許可 |

### 4.4 ユーザビリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-U-011 | ファイル読み取りエラーメッセージ | ファイルパス、エラー原因を明示 |
| NFR-U-012 | ループネストのエラーメッセージ | ネスト深度、位置情報を明示 |
| NFR-U-013 | ボリュームのデフォルト値 | V10（中程度の強さ） |

### 4.5 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-011 | ファイル読み取りのテスト | 正常系、異常系を網羅 |
| NFR-M-012 | ループネストのテスト | 1-5階層、エッジケースを網羅 |
| NFR-M-013 | ボリュームのテスト | 絶対値、相対値、クランプを網羅 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| ファイル形式 | `.mml` のみ対応 | シンプルさを保つ |
| ループネスト深度 | 最大5階層 | パーサーの複雑性を抑えつつ表現力を確保 |
| ボリューム範囲 | 0-15（既存仕様維持） | 後方互換性を維持 |
| ファイルサイズ上限 | 1MB | DoS攻撃防止 |

### 5.2 ビジネス制約

| 制約 | 詳細 |
|------|------|
| 予算 | オープンソース（無償） |
| スケジュール | Phase 2.1: 2週間 |
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
| F-001 | MMLパーサー | **拡張** | ファイル読み取り、相対ボリューム、ループネストを追加 |
| - | 音量調節 | **後方互換** | V0-V15は維持、相対指定（V+n, V-n）を追加 |

### 6.2 CLIインターフェース変更

#### 追加されるオプション
```bash
# playコマンドに追加
--file <PATH>  # MMLファイルのパス
```

#### 変更なし
```bash
sine-mml play "MML"
sine-mml play --history-id <ID>
sine-mml history
sine-mml export --history-id <ID> --output <FILE>
sine-mml clear-history
```

### 6.3 MML構文の拡張

#### 追加される構文
```
V<0-15>         # ボリューム絶対値指定（既存仕様維持）
V+<n>           # ボリューム相対値指定（増加）
V-<n>           # ボリューム相対値指定（減少）
V+              # デフォルト増加（+1）
V-              # デフォルト減少（-1）
[ [ ... ]n ]m   # ループネスト（最大5階層）
```

#### 例
```
# 相対ボリューム指定
V10 C V+2 D V-3 E

# ループネスト
[ CDE [ FGAB ]2 ]3

# ファイル読み取り
# song.mml:
# イントロ
T120 L8 O5
[CDEF]2 GAB >C

# Aメロ
O4 L4
CDEFGAB >C
```

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の全機能が実装済み（v1.0.0リリース済み）
- REQ-CLI-002の全機能が実装済み（v2.0.0リリース済み）
- REQ-CLI-003の全機能が実装済み（v2.1.0リリース済み）
- ループ構文（F-023）が実装済み（ネストなし）

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| std::fs | ファイル読み取り | ファイル読み取り機能 |
| clap | CLIオプション追加 | `--file` オプション |

### 7.3 新規依存クレート

なし（既存のクレートで対応可能）

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-014 | ループネストのパーサーバグ | 高 | 中 | 豊富なテストケース、エッジケース対応 |
| R-015 | ファイル読み取りのエンコーディング問題 | 中 | 中 | UTF-8強制、エラーメッセージ改善 |
| R-016 | ループ展開数の上限不足 | 低 | 低 | ユーザーフィードバックで調整 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 |
|----|------|------|------|
| I-013 | ループネスト深度の妥当性検証 | 開発チーム | 2026-01-20 |
| I-014 | ファイル読み取りのセクション指定機能 | 開発チーム | Phase 3.0で検討 |
| I-015 | ボリュームのエンベロープ制御 | 開発チーム | Phase 3.0で検討 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| ボリューム | 音量（本プロジェクトではV0-V15の16段階） |
| 相対ボリューム | 現在のボリュームからの増減指定（V+n, V-n） |
| ループネスト | ループ構文の入れ子（ネスト） |
| コメント行 | `#`で始まる行（MMLファイル内） |
| セクション区切り | 楽曲の構成を示すコメント（イントロ、Aメロ等） |
| クランプ | 値を指定範囲内に収める処理 |

---

## 10. 参考リンク

### 10.1 MML仕様

#### ファイル形式
- **NuttX MML Parser**: [Documentation](https://nuttx.apache.org/docs/latest/applications/audioutils/mml_parser/index.html) - 基本的なMML仕様
- **MML (Music Macro Language) - Wikipedia**: [日本語版](https://ja.wikipedia.org/wiki/Music_Macro_Language) - MMLの歴史と構文



### 10.2 Rustファイル操作

#### ファイル読み取り
- **Rust std::fs**: [read_to_string](https://doc.rust-lang.org/std/fs/fn.read_to_string.html) - ファイル読み取り
- **Rust std::path**: [Path](https://doc.rust-lang.org/std/path/struct.Path.html) - パス操作

#### エラーハンドリング
- **anyhow**: [Documentation](https://docs.rs/anyhow/) - エラーハンドリング
- **thiserror**: [Documentation](https://docs.rs/thiserror/) - カスタムエラー型

### 10.3 パーサー実装

#### 再帰下降パーサー
- **Crafting Interpreters**: [Parsing Expressions](https://craftinginterpreters.com/parsing-expressions.html) - パーサー実装の基礎
- **Recursive Descent Parsing**: [Wikipedia](https://en.wikipedia.org/wiki/Recursive_descent_parser) - 再帰下降パーサーの理論

---

## 11. 実装優先順位と段階的ロールアウト

### Phase 2.1.1（Week 1: 2026-01-12〜01-19）
1. **F-027（MMLファイル読み取り）** - 最も簡単、独立した機能
   - `--file` オプション追加
   - ファイル読み取り処理実装
   - コメント、空行除去処理実装
   - エラーハンドリング実装

### Phase 2.1.2（Week 2: 2026-01-20〜01-26）
2. **F-028（相対ボリューム指定）** - 既存仕様を維持しつつ相対指定を追加
   - AST拡張（VolumeValue列挙型）
   - パーサー拡張（V+n, V-n対応）
   - 合成処理実装（相対値計算、クランプ処理）
   - 豊富なテストケース作成
3. **F-029（ループネスト対応）** - 最も複雑、パーサー拡張が必要
   - パーサーにネスト深度カウント追加
   - ループ展開処理の再帰対応
   - 総展開数チェック実装
   - 豊富なテストケース作成

---

## 12. テストケース概要

### 12.1 ファイル読み取りのテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-027-001 | `--file song.mml` | ファイルから読み込み成功 |
| TC-027-002 | コメント行（`#`始まり） | コメント行が除去される |
| TC-027-003 | 空行 | 空行が除去される |
| TC-027-004 | ファイルが存在しない | エラーメッセージを表示 |
| TC-027-005 | 拡張子が`.mml`以外 | エラーメッセージを表示 |
| TC-027-006 | ファイルサイズが1MB超 | エラーメッセージを表示 |
| TC-027-007 | UTF-8以外のエンコーディング | エラーメッセージを表示 |
| TC-027-008 | `--file` と `mml` 引数の両方指定 | エラー（排他的） |

### 12.2 相対ボリューム指定のテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-028-001 | `V10` | 絶対値指定が成功（既存仕様維持） |
| TC-028-002 | `V+2` | 相対値指定（増加）が成功 |
| TC-028-003 | `V-3` | 相対値指定（減少）が成功 |
| TC-028-004 | `V+` | デフォルト増加（+1）が適用 |
| TC-028-005 | `V-` | デフォルト減少（-1）が適用 |
| TC-028-006 | `V15 V+2` | 15にクランプ |
| TC-028-007 | `V0 V-2` | 0にクランプ |
| TC-028-008 | `V20` | エラー（範囲外: 0-15） |

### 12.3 ループネストのテストケース

| テストID | テストケース | 期待結果 |
|---------|-------------|---------|
| TC-029-001 | `[ CDE [ FGAB ]2 ]3` | 2階層ネストが正常に展開 |
| TC-029-002 | `[ [ [ C ]2 D ]2 E ]2` | 3階層ネストが正常に展開 |
| TC-029-003 | `[ [ [ [ C ]2 D ]2 E ]2 F ]2` | 4階層ネストが正常に展開 |
| TC-029-004 | `[ [ [ [ [ C ]2 D ]2 E ]2 F ]2 G ]2` | 5階層ネストが正常に展開 |
| TC-029-005 | `[ [ [ [ [ [ C ]2 ]2 ]2 ]2 ]2 ]2` | エラー（6階層） |
| TC-029-004 | `[ [ CD:EF ]2 GAB ]2` | ネスト内の脱出ポイントが動作 |
| TC-029-005 | `[ [ [ C ]99 ]99 ]99` | エラー（展開数超過） |

---

## 13. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-12 | 初版作成 | req-writer |
| 1.0.1 | 2026-01-12 | F-028: ベロシティ(V0-V127)から相対ボリューム(V0-V15維持、V+n/V-n追加)に変更、破壊的変更を回避 | req-writer |
