# 技術調査レポート: Combined (Audio & CLI Stack)

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-10 |
| 調査深度 | standard |
| 対象技術 | fundsp, cpal, rusqlite, clap, hound, indicatif |

---

## 1. fundsp (Audio Synthesis)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v0.23.0 |
| 公式ドキュメント | [docs.rs](https://docs.rs/fundsp/latest/fundsp/) |
| GitHub | [SamiPerttu/fundsp](https://github.com/SamiPerttu/fundsp) |

### インストール方法
```toml
fundsp = "0.23.0"
```

### 基本的な使い方
```rust
use fundsp::hacker::*;

fn main() {
    // Create a 440Hz sine wave oscillator
    let mut oscillator = sine_hz(440.0);
    
    // Get the first mono sample
    let sample = oscillator.get_mono();
    println!("Sample: {}", sample);
}
```

### 主要なAPI/関数
| API/関数 | 説明 |
|----------|------|
| `sine_hz(f)` | 周波数 f のサイン波オシレーターを作成 |
| `lowpass_hz(f, q)` | 共振ローパスフィルターを作成 |
| `osc_hz(f)` | 可変波形の汎用オシレーターを作成 |

### よくあるエラーと対処
| エラー | 原因 | 対処法 |
|--------|------|--------|
| サンプルレートの不一致 | デフォルト(44.1kHz)以外での使用 | `set_sample_rate`で明示的に設定 |
| パフォーマンス不足 | SIMD未有効化 | `simd`フィーチャーをCargo.tomlで有効化 |

---

## 2. cpal (Audio I/O)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v0.17.1 |
| 公式ドキュメント | [docs.rs](https://docs.rs/cpal/latest/cpal/) |
| GitHub | [RustAudio/cpal](https://github.com/RustAudio/cpal) |

### インストール方法
```toml
cpal = "0.17.1"
```

### 基本的な使い方
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();
    
    println!("Default device: {}, Config: {:?}", device.name().unwrap(), config);
}
```

### 主要なAPI/関数
| API/関数 | 説明 |
|----------|------|
| `default_host()` | オーディオシステムの起点 |
| `output_devices()` | 利用可能な出力デバイスのイテレーター |
| `build_output_stream()` | オーディオ出力ストリームの構築 |

### よくあるエラーと対処
| エラー | 原因 | 対処法 |
|--------|------|--------|
| `DeviceNotAvailable` | デバイスの切断または排他使用 | システム設定を確認 |
| `FormatNotSupported` | ハードウェア未対応のフォーマット | `supported_output_configs()`で確認 |

---

## 3. rusqlite (SQLite)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v0.38.0 |
| 公式ドキュメント | [docs.rs](https://docs.rs/rusqlite/latest/rusqlite/) |
| GitHub | [rusqlite/rusqlite](https://github.com/rusqlite/rusqlite) |

### インストール方法
```toml
rusqlite = { version = "0.38.0", features = ["bundled"] }
```

### 基本的な使い方
```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)", [])?;
    conn.execute("INSERT INTO test (val) VALUES (?1)", ["Hello Rusqlite"])?;
    Ok(())
}
```

### 主要なAPI/関数
| API/関数 | 説明 |
|----------|------|
| `Connection::open(path)` | データベースファイルへの接続 |
| `execute(sql, params)` | INSERT, UPDATE, DELETE文の実行 |
| `query_row(sql, params, closure)` | 単一要素のクエリとマッピング |

### よくあるエラーと対処
| エラー | 原因 | 対処法 |
|--------|------|--------|
| SQLiteライブラリ未検出 | システムにSQLite未導入 | `bundled`フィーチャーを有効化 |
| `Database is locked` | 他のプロセスによる書き込み | `busy_timeout`を設定 |

---

## 4. clap (CLI Arguments)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v4.5.54 |
| 公式ドキュメント | [docs.rs](https://docs.rs/clap/latest/clap/) |
| GitHub | [clap-rs/clap](https://github.com/clap-rs/clap) |

### インストール方法
```toml
clap = { version = "4.5", features = ["derive"] }
```

### 基本的な使い方
```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    name: String,
}

fn main() {
    let args = Cli::parse();
    println!("Hello, {}!", args.name);
}
```

---

## 5. hound (WAV I/O)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v3.5.1 |
| 公式ドキュメント | [docs.rs](https://docs.rs/hound/latest/hound/) |
| GitHub | [ruuda/hound](https://github.com/ruuda/hound) |

### インストール方法
```toml
hound = "3.5.1"
```

### 基本的な使い方
```rust
fn main() {
    let spec = hound::WavSpec {
        channels: 1, sample_rate: 44100, bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("test.wav", spec).unwrap();
    writer.write_sample(0).unwrap();
    writer.finalize().unwrap();
}
```

---

## 6. indicatif (CLI Progress)

| 項目 | 内容 |
|------|------|
| 最新バージョン | v0.18.3 |
| 公式ドキュメント | [docs.rs](https://docs.rs/indicatif/latest/indicatif/) |
| GitHub | [console-rs/indicatif](https://github.com/console-rs/indicatif) |

### インストール方法
```toml
indicatif = "0.18.3"
```

### 基本的な使い方
```rust
use indicatif::ProgressBar;

fn main() {
    let pb = ProgressBar::new(100);
    for _ in 0..100 {
        pb.inc(1);
    }
    pb.finish_with_message("Done");
}
```
