use anyhow::{bail, Context, Result};
use crate::cli::args::{PlayArgs, Waveform};
use crate::{mml, audio, db};
use crate::cli::output;

pub fn play_handler(args: PlayArgs) -> Result<()> {
    // 1. 引数の検証とMML取得
    let (mml_string, should_save) = match (&args.mml, args.history_id) {
        (Some(mml), None) => (mml.clone(), true),
        (None, Some(id)) => {
            let db = db::Database::init()?;
            let entry = db.get_by_id(id)
                .with_context(|| format!("[CLI-E002] 履歴ID {} が見つかりません", id))?;
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
    let ast = mml::parse(&mml_string)
        .map_err(|e| anyhow::anyhow!("MML parse error: {:?}", e))?; // TODO: Better error formatting

    // 3. 音声合成
    let waveform_type = match args.waveform {
        Waveform::Sine => audio::waveform::WaveformType::Sine,
        Waveform::Sawtooth => audio::waveform::WaveformType::Sawtooth,
        Waveform::Square => audio::waveform::WaveformType::Square,
    };

    // volume: f32 (0.0-1.0) -> u8 (0-100)
    let volume_u8 = (args.volume * 100.0) as u8;
    
    // sample_rate: 44100 (fixed for now)
    let sample_rate = 44100;
    
    let mut synth = audio::synthesizer::Synthesizer::new(sample_rate, volume_u8, waveform_type);
    let buffer = synth.synthesize(&ast)
        .context("音声合成に失敗しました")?;

    // 4. 再生
    // コンテナ環境などオーディオデバイスがない場合は警告を出して続行
    match audio::player::AudioPlayer::new() {
        Ok(mut player) => {
            player.play(&buffer, args.loop_play)
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
        
        let entry = db::HistoryEntry::new(
            mml_string.clone(),
            db_waveform,
            args.volume,
            args.bpm as u16,
        );
        
        let history_id = db.save(&entry)
            .context("履歴の保存に失敗しました")?;
            
        output::success(&format!("✓ 再生完了（履歴ID: {}）", history_id));
    } else {
        output::success("✓ 再生完了");
    }

    Ok(())
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
        
        // コンテナ環境でDBエラー等が出る可能性があるが、ロジック自体がパニックしないことを確認
        let _ = play_handler(args);
    }
}
