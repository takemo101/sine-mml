use crate::cli::args::{validate_note, ExportArgs, PlayArgs, Waveform};
use crate::cli::output;
use crate::{audio, db, mml};
use anyhow::{bail, Context, Result};
use comfy_table::Table;

#[cfg(feature = "midi-output")]
use crate::cli::args::{MidiArgs, MidiSubcommand};
#[cfg(feature = "midi-output")]
use crate::midi;

fn determine_should_save(args: &PlayArgs) -> bool {
    // Save history when MML is provided directly or from file (not from history)
    matches!(
        (&args.mml, &args.file, args.history_id),
        (Some(_), None, None) | (None, Some(_), None)
    )
}

#[cfg(feature = "midi-output")]
fn handle_midi_list() -> Result<()> {
    let devices = midi::list_midi_devices()?;
    if devices.is_empty() {
        println!("MIDIデバイスが見つかりません");
    } else {
        println!("利用可能なMIDIデバイス:");
        for (i, name) in devices.iter().enumerate() {
            println!("  {i}: {name}");
        }
    }
    Ok(())
}

#[cfg(feature = "midi-output")]
#[allow(clippy::needless_pass_by_value)]
pub fn midi_handler(args: MidiArgs) -> Result<()> {
    match args.command {
        MidiSubcommand::List => handle_midi_list(),
    }
}

#[cfg(feature = "midi-output")]
fn handle_midi_output(device: &str, channel: u8, mml_string: &str, ast: &mml::Mml) -> Result<()> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let mut conn = midi::connect_midi_device(device).map_err(|e| anyhow::anyhow!("{e}"))?;

    let interrupt = Arc::new(AtomicBool::new(false));
    let interrupt_clone = Arc::clone(&interrupt);

    ctrlc::set_handler(move || {
        interrupt_clone.store(true, Ordering::SeqCst);
    })
    .context("Ctrl+Cハンドラーの設定に失敗しました")?;

    println!("MIDI再生中... (Ctrl+Cで停止)");
    println!("  MML: {}", truncate_mml(mml_string, 50));
    println!("  デバイス: {device}");
    println!("  チャンネル: {channel}");

    midi::play_midi_stream_interruptible(&mut conn, &ast.commands, channel, &interrupt)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if interrupt.load(Ordering::Relaxed) {
        output::success("✓ MIDI再生を中断しました");
    } else {
        output::success("✓ MIDI再生完了");
    }

    Ok(())
}

fn handle_audio_playback(args: &PlayArgs, mml_string: &str, ast: &mml::Mml) -> Result<()> {
    let waveform_type = match args.waveform {
        Waveform::Sine => audio::waveform::WaveformType::Sine,
        Waveform::Sawtooth => audio::waveform::WaveformType::Sawtooth,
        Waveform::Square => audio::waveform::WaveformType::Square,
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let volume_u8 = (args.volume * 100.0) as u8;
    let sample_rate = 44100;

    let mut synth = audio::synthesizer::Synthesizer::new(sample_rate, volume_u8, waveform_type);
    let mut buffer = synth
        .synthesize(ast)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("音声合成に失敗しました")?;

    if args.metronome {
        let bpm = ast.get_tempo();
        synth.mix_metronome(
            &mut buffer,
            f64::from(sample_rate),
            bpm,
            args.metronome_beat,
            args.metronome_volume,
        );
        audio::synthesizer::normalize_samples(&mut buffer);
    }

    let history_id_opt = save_history_if_needed(args, mml_string)?;
    play_audio_buffer(&buffer, mml_string, args.loop_play)?;
    print_completion_message(history_id_opt, args.note.as_ref());

    Ok(())
}

fn save_history_if_needed(args: &PlayArgs, mml_string: &str) -> Result<Option<i64>> {
    if !determine_should_save(args) {
        return Ok(None);
    }

    let db = db::Database::init()?;
    let db_waveform = match args.waveform {
        Waveform::Sine => db::history::Waveform::Sine,
        Waveform::Sawtooth => db::history::Waveform::Sawtooth,
        Waveform::Square => db::history::Waveform::Square,
    };
    let bpm_u16 = 120;
    let entry = db::HistoryEntry::new(
        mml_string.to_string(),
        db_waveform,
        args.volume,
        bpm_u16,
        args.note.clone(),
    );

    match db.save(&entry) {
        Ok(id) => Ok(Some(id)),
        Err(e) => {
            eprintln!("Warning: 履歴の保存に失敗しました: {e}");
            Ok(None)
        }
    }
}

fn play_audio_buffer(buffer: &[f32], mml_string: &str, loop_play: bool) -> Result<()> {
    match audio::player::AudioPlayer::new() {
        Ok(mut player) => {
            player
                .play(buffer, loop_play)
                .context("音声再生に失敗しました")?;
            output::display_play_progress(mml_string, buffer, loop_play)?;
        }
        Err(_) => {
            eprintln!("Warning: Audio device not found. Skipping playback.");
        }
    }
    Ok(())
}

fn print_completion_message(history_id_opt: Option<i64>, note: Option<&String>) {
    if let Some(id) = history_id_opt {
        if let Some(note) = note {
            output::success(&format!("✓ 再生完了（履歴ID: {id}、メモ: {note}）"));
        } else {
            output::success(&format!("✓ 再生完了（履歴ID: {id}）"));
        }
    } else {
        output::success("✓ 再生完了");
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn play_handler(args: PlayArgs) -> Result<()> {
    let mml_string = resolve_mml_input(&args)?;

    if let Some(ref note) = args.note {
        validate_note(note).map_err(|e| anyhow::anyhow!("[CLI-E010] {e}"))?;
    }

    let ast = mml::parse(&mml_string).map_err(|e| anyhow::anyhow!("MML parse error: {e:?}"))?;

    #[cfg(feature = "midi-output")]
    if let Some(ref device) = args.midi_out {
        return handle_midi_output(device, args.midi_channel, &mml_string, &ast);
    }

    handle_audio_playback(&args, &mml_string, &ast)
}

fn resolve_mml_input(args: &PlayArgs) -> Result<String> {
    match (&args.mml, args.history_id, &args.file) {
        (Some(mml), None, None) => Ok(mml.clone()),
        (None, Some(id), None) => {
            let db = db::Database::init()?;
            let entry = db
                .get_by_id(id)
                .with_context(|| format!("[CLI-E002] 履歴ID {id} が見つかりません"))?;
            Ok(entry.mml)
        }
        (None, None, Some(file_path)) => mml::read_mml_file(file_path),
        (None, None, None) => {
            bail!("[CLI-E001] play コマンドでは、MML文字列、--history-id、または --file のいずれか一方を指定してください");
        }
        _ => {
            unreachable!("clap should prevent this")
        }
    }
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
        .set_header(vec![
            "ID",
            "MML",
            "Waveform",
            "Volume",
            "Note",
            "Created At",
        ]);

    for entry in history {
        table.add_row(vec![
            entry.id.map_or(String::new(), |id| id.to_string()),
            truncate_mml(&entry.mml, 50),
            format!("{:?}", entry.waveform),
            format!("{:.1}", entry.volume),
            entry.note.unwrap_or_else(|| "-".to_string()),
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

pub fn clear_history_handler() -> Result<()> {
    let db = db::Database::init()?;
    clear_history_logic(&db, &mut std::io::stdin().lock(), &mut std::io::stdout())
}

fn clear_history_logic<R: std::io::BufRead, W: std::io::Write>(
    db: &db::Database,
    stdin: &mut R,
    stdout: &mut W,
) -> Result<()> {
    let count = db.count().context("履歴件数の取得に失敗しました")?;

    if count == 0 {
        writeln!(stdout, "履歴がありません。")?;
        return Ok(());
    }

    write!(stdout, "全ての履歴（{count}件）を削除しますか？ (y/n): ")?;
    stdout.flush()?;

    let mut input = String::new();
    stdin.read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "y" | "yes" => {
            db.clear_all().context("履歴の削除に失敗しました")?;
            output::success("全ての履歴を削除しました。");
        }
        "n" | "no" => {
            writeln!(stdout, "キャンセルしました。")?;
        }
        _ => {
            bail!("無効な入力です。'y' または 'n' を入力してください。");
        }
    }

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
        let args = PlayArgs::for_test(None, None, None, Waveform::Sine, 1.0, None);
        let result = play_handler(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "[CLI-E001] play コマンドでは、MML文字列、--history-id、または --file のいずれか一方を指定してください");
    }

    #[test]
    fn test_play_handler_with_mml_device_fail_is_ok() {
        let args = PlayArgs::for_test(Some("C".to_string()), None, None, Waveform::Sine, 0.5, None);
        let _ = play_handler(args);
    }

    #[test]
    fn test_play_handler_with_note() {
        let args = PlayArgs::for_test(
            Some("C".to_string()),
            None,
            None,
            Waveform::Sine,
            0.5,
            Some("Test note".to_string()),
        );
        let _ = play_handler(args);
    }

    #[test]
    fn test_play_handler_with_note_too_long() {
        let args = PlayArgs::for_test(
            Some("C".to_string()),
            None,
            None,
            Waveform::Sine,
            0.5,
            Some("a".repeat(501)),
        );

        let result = play_handler(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("[CLI-E010]"));
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
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120, None);
        db.save(&entry).unwrap();
        assert!(history_logic(&db).is_ok());
    }

    #[test]
    fn test_history_logic_with_note() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new(
            "CDE".to_string(),
            db::Waveform::Sine,
            0.5,
            120,
            Some("My melody".to_string()),
        );
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
        let entry = db::HistoryEntry::new("C".to_string(), db::Waveform::Sine, 0.5, 120, None);
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

    #[test]
    fn test_should_save_flag_mml_input() {
        let args = PlayArgs::for_test(
            Some("CDE".to_string()),
            None,
            None,
            Waveform::Sine,
            1.0,
            None,
        );
        assert!(determine_should_save(&args));
    }

    #[test]
    fn test_should_save_flag_history_id() {
        let args = PlayArgs::for_test(None, Some(1), None, Waveform::Sine, 1.0, None);
        assert!(!determine_should_save(&args));
    }

    #[test]
    fn test_should_save_flag_file_input() {
        let args = PlayArgs::for_test(
            None,
            None,
            Some("test.mml".to_string()),
            Waveform::Sine,
            1.0,
            None,
        );
        assert!(determine_should_save(&args));
    }

    #[test]
    fn test_clear_history_logic_empty() {
        let db = db::Database::open_in_memory().unwrap();
        let mut stdin = std::io::Cursor::new(b"");
        let mut stdout = Vec::new();

        let result = clear_history_logic(&db, &mut stdin, &mut stdout);
        assert!(result.is_ok());
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("履歴がありません"));
    }

    #[test]
    fn test_clear_history_logic_confirm_yes() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120, None);
        db.save(&entry).unwrap();

        let mut stdin = std::io::Cursor::new(b"y\n");
        let mut stdout = Vec::new();

        let result = clear_history_logic(&db, &mut stdin, &mut stdout);
        assert!(result.is_ok());
        assert_eq!(db.count().unwrap(), 0);
    }

    #[test]
    fn test_clear_history_logic_confirm_yes_uppercase() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120, None);
        db.save(&entry).unwrap();

        let mut stdin = std::io::Cursor::new(b"YES\n");
        let mut stdout = Vec::new();

        let result = clear_history_logic(&db, &mut stdin, &mut stdout);
        assert!(result.is_ok());
        assert_eq!(db.count().unwrap(), 0);
    }

    #[test]
    fn test_clear_history_logic_cancel_no() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120, None);
        db.save(&entry).unwrap();

        let mut stdin = std::io::Cursor::new(b"n\n");
        let mut stdout = Vec::new();

        let result = clear_history_logic(&db, &mut stdin, &mut stdout);
        assert!(result.is_ok());
        assert_eq!(db.count().unwrap(), 1);
        let output = String::from_utf8(stdout).unwrap();
        assert!(output.contains("キャンセルしました"));
    }

    #[test]
    fn test_clear_history_logic_invalid_input() {
        let db = db::Database::open_in_memory().unwrap();
        let entry = db::HistoryEntry::new("CDE".to_string(), db::Waveform::Sine, 0.5, 120, None);
        db.save(&entry).unwrap();

        let mut stdin = std::io::Cursor::new(b"invalid\n");
        let mut stdout = Vec::new();

        let result = clear_history_logic(&db, &mut stdin, &mut stdout);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("無効な入力です"));
        assert_eq!(db.count().unwrap(), 1);
    }
}
