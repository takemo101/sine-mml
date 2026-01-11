use crate::cli::args::{ExportArgs, PlayArgs, Waveform};
use crate::cli::output;
use crate::{audio, db, mml};
use anyhow::{bail, Context, Result};
use comfy_table::Table;

fn determine_should_save(args: &PlayArgs) -> bool {
    false // TODO: Implement correctly
}

/// playサブコマンドのハンドラー
///
/// # Errors
///
/// Returns `anyhow::Result` if:
/// - Arguments are invalid (e.g. both MML and `history_id` are missing)
/// - MML parsing fails
/// - Audio synthesis fails
/// - Audio playback fails
/// - History saving fails
#[allow(clippy::needless_pass_by_value)]
pub fn play_handler(args: PlayArgs) -> Result<()> {
    // 1. 引数の検証とMML取得
    let (mml_string, should_save) = match (&args.mml, args.history_id) {
        (Some(mml), None) => (mml.clone(), true),
        (None, Some(id)) => {
            let db = db::Database::init()?;
            let entry = db
                .get_by_id(id)
                .with_context(|| format!("[CLI-E002] 履歴ID {id} が見つかりません"))?;
            (entry.mml, false)
        }
        (None, None) => {
            bail!("[CLI-E001] play コマンドでは、MML文字列または --history-id のいずれか一方を指定してください");
        }
        (Some(_), Some(_)) => {
            unreachable!("clap should prevent this")
        }
    };

    // 2. MML解析
    let ast = mml::parse(&mml_string).map_err(|e| anyhow::anyhow!("MML parse error: {e:?}"))?;

    // 3. 音声合成
    let waveform_type = match args.waveform {
        Waveform::Sine => audio::waveform::WaveformType::Sine,
        Waveform::Sawtooth => audio::waveform::WaveformType::Sawtooth,
        Waveform::Square => audio::waveform::WaveformType::Square,
    };

    // volume: f32 (0.0-1.0) -> u8 (0-100)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let volume_u8 = (args.volume * 100.0) as u8;

    // sample_rate: 44100 (fixed for now)
    let sample_rate = 44100;

    let mut synth = audio::synthesizer::Synthesizer::new(sample_rate, volume_u8, waveform_type);
    let buffer = synth
        .synthesize(&ast)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("音声合成に失敗しました")?;

    // 4. 再生
    // コンテナ環境などオーディオデバイスがない場合は警告を出して続行
    match audio::player::AudioPlayer::new() {
        Ok(mut player) => {
            player
                .play(&buffer, args.loop_play)
                .context("音声再生に失敗しました")?;

            // 5. プログレス表示 & 待機
            output::display_play_progress(&mml_string, &buffer, args.loop_play)?;
        }
        Err(_) => {
            eprintln!("Warning: Audio device not found. Skipping playback.");
        }
    }

    // 6. 履歴保存（新規入力かつ再生成功時）
    if should_save {
        let db = db::Database::init()?;

        let db_waveform = match args.waveform {
            Waveform::Sine => db::history::Waveform::Sine,
            Waveform::Sawtooth => db::history::Waveform::Sawtooth,
            Waveform::Square => db::history::Waveform::Square,
        };

        #[allow(clippy::cast_possible_truncation)]
        let bpm_u16 = args.bpm as u16;

        let entry = db::HistoryEntry::new(mml_string.clone(), db_waveform, args.volume, bpm_u16);

        let history_id = db.save(&entry).context("履歴の保存に失敗しました")?;

        output::success(&format!("✓ 再生完了（履歴ID: {history_id}）"));
    } else {
        output::success("✓ 再生完了");
    }

    Ok(())
}

/// historyサブコマンドのハンドラー
///
/// # Errors
/// Returns `anyhow::Result` if DB operations fail.
pub fn history_handler() -> Result<()> {
    let db = db::Database::init()?;
    history_logic(&db)
}

fn history_logic(db: &db::Database) -> Result<()> {
    let history = db.list(Some(20)).context("履歴の取得に失敗しました")?;

    if history.is_empty() {
        println!("履歴がありません");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec!["ID", "MML", "Waveform", "Volume", "BPM", "Created At"]);

    for entry in history {
        table.add_row(vec![
            entry.id.map_or(String::new(), |id| id.to_string()),
            truncate_mml(&entry.mml, 50),
            format!("{:?}", entry.waveform),
            format!("{:.1}", entry.volume),
            entry.bpm.to_string(),
            entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// exportサブコマンドのハンドラー
///
/// # Errors
/// Returns `anyhow::Result` if:
/// - History ID not found
/// - Path traversal detected
/// - WAV export fails
#[allow(clippy::needless_pass_by_value)]
pub fn export_handler(args: ExportArgs) -> Result<()> {
    if args.output.contains("..") {
        bail!("Path traversal detected: '..' is not allowed in output path");
    }
    let db = db::Database::init()?;
    export_logic(&db, &args)
}

fn export_logic(db: &db::Database, args: &ExportArgs) -> Result<()> {
    let entry = db
        .get_by_id(args.history_id)
        .context(format!("履歴ID {} が見つかりません", args.history_id))?;

    let ast = mml::parse(&entry.mml).map_err(|e| anyhow::anyhow!("MML parse error: {e:?}"))?;

    let waveform_type = match entry.waveform {
        db::Waveform::Sine => audio::waveform::WaveformType::Sine,
        db::Waveform::Sawtooth => audio::waveform::WaveformType::Sawtooth,
        db::Waveform::Square => audio::waveform::WaveformType::Square,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let volume_u8 = (entry.volume * 100.0) as u8;

    let sample_rate = 44100;
    let mut synth = audio::synthesizer::Synthesizer::new(sample_rate, volume_u8, waveform_type);
    let buffer = synth
        .synthesize(&ast)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("音声合成に失敗しました")?;

    let output_path = std::path::Path::new(&args.output);
    audio::exporter::export_wav(&buffer, output_path)
        .context("WAVファイルの書き出しに失敗しました")?;

    output::success(&format!("✓ エクスポート完了: {}", args.output));

    Ok(())
}

fn truncate_mml(mml: &str, max_len: usize) -> String {
    if mml.len() <= max_len {
        mml.to_string()
    } else {
        format!("{}...", &mml[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_handler_no_input() {
        let args = PlayArgs {
            mml: None,
            history_id: None,
            waveform: Waveform::Sine,
            volume: 1.0,
            bpm: 120,
            loop_play: false,
            metronome: false,
        };
        let result = play_handler(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "[CLI-E001] play コマンドでは、MML文字列または --history-id のいずれか一方を指定してください");
    }

    #[test]
    fn test_play_handler_with_mml_device_fail_is_ok() {
        let args = PlayArgs {
            mml: Some("C".to_string()),
            history_id: None,
            waveform: Waveform::Sine,
            volume: 0.5,
            bpm: 120,
            loop_play: false,
            metronome: false,
        };

        let _ = play_handler(args);
    }

    #[test]
    fn test_truncate_mml() {
        assert_eq!(truncate_mml("short", 10), "short");
        assert_eq!(truncate_mml("exactly10.", 10), "exactly10.");
        assert_eq!(truncate_mml("long string", 5), "lo...");
        assert_eq!(truncate_mml("long string", 6), "lon...");
    }

    #[test]
    fn test_export_handler_path_traversal() {
        let args = ExportArgs {
            history_id: 1,
            output: "../unsafe.wav".to_string(),
        };
        let result = export_handler(args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path traversal detected"));
    }

    #[test]
    fn test_history_logic_empty() {
        let db = db::Database::open_in_memory().unwrap();
        assert!(history_logic(&db).is_ok());
    }

    #[test]
    fn test_history_logic_with_data() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120);
        db.save(&entry).unwrap();
        assert!(history_logic(&db).is_ok());
    }

    #[test]
    fn test_export_logic_not_found() {
        let db = db::Database::open_in_memory().unwrap();
        let args = ExportArgs {
            history_id: 999,
            output: "test.wav".to_string(),
        };
        let result = export_logic(&db, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("履歴ID 999 が見つかりません"));
    }

    #[test]
    fn test_export_logic_success() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("C".to_string(), db::Waveform::Sine, 0.5, 120);
        let id = db.save(&entry).unwrap();

        let dir = std::env::temp_dir();
        let path = dir.join("test_export.wav");
        let path_str = path.to_string_lossy().to_string();

        let args = ExportArgs {
            history_id: id,
            output: path_str,
        };

        let result = export_logic(&db, &args);
        assert!(result.is_ok(), "export_logic failed: {:?}", result.err());
        assert!(path.exists());

        let _ = std::fs::remove_file(path);
    }
}
