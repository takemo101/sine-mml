use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 再生プログレスを表示し、再生時間分待機する
///
/// # Errors
///
/// Returns `anyhow::Result` if progress bar template is invalid.
pub fn display_play_progress(_mml: &str, buffer: &[f32], is_loop: bool) -> Result<()> {
    let sample_rate = 44100.0;

    #[allow(clippy::cast_precision_loss)]
    let duration_secs = buffer.len() as f64 / sample_rate;
    let duration = Duration::from_secs_f64(duration_secs);

    if is_loop {
        println!(
            "{}",
            style("Loop playback... Press Ctrl+C to stop").yellow()
        );
        // In a real CLI, we might handle Ctrl+C here or just block.
        // For this implementation, we assume the user will interrupt the process.
        // We block here to keep the process alive while audio plays in background thread (if cpal worked).
        loop {
            std::thread::sleep(Duration::from_millis(100));
        }
    } else {
        let pb = ProgressBar::new(buffer.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")?
                .progress_chars("#>-"),
        );

        pb.set_message(format!("{duration_secs:.1}s"));

        // Simulate progress synchronized with duration
        let steps = 100;
        let step_duration = duration / steps;
        let step_inc = buffer.len() as u64 / u64::from(steps);

        for _ in 0..steps {
            std::thread::sleep(step_duration);
            pb.inc(step_inc);
        }
        pb.finish_with_message("Done");
    }
    Ok(())
}

pub fn success(msg: &str) {
    println!("{}", style(msg).green());
}

pub fn info(msg: &str) {
    println!("{}", style(msg).cyan());
}

/// 通常のメッセージを表示（スタイルなし）
pub fn message(msg: &str) {
    println!("{msg}");
}

/// インデント付きメッセージを表示
pub fn message_indent(msg: &str) {
    println!("  {msg}");
}

/// エラーメッセージを赤色で表示
pub fn error(msg: &str) {
    eprintln!("{}", style(msg).red().bold());
}

/// 警告メッセージを黄色で表示
pub fn warning(msg: &str) {
    eprintln!("{}", style(msg).yellow());
}

/// MIDI再生プログレスを表示し、経過時間に基づいて更新する
///
/// # Arguments
/// * `total_duration_ms` - 全体の再生時間（ミリ秒）
/// * `interrupt` - 中断フラグ
pub fn display_midi_progress(total_duration_ms: u64, interrupt: &Arc<AtomicBool>) {
    if total_duration_ms == 0 {
        return;
    }

    let pb = ProgressBar::new(100);
    if let Ok(style) = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% {msg}")
    {
        pb.set_style(style.progress_chars("#>-"));
    }

    let total_duration = Duration::from_millis(total_duration_ms);
    pb.set_message(format!("{:.1}s", total_duration.as_secs_f64()));

    let start = Instant::now();
    let update_interval = Duration::from_millis(100); // 10Hz

    while !interrupt.load(Ordering::Relaxed) {
        let elapsed = start.elapsed();
        if elapsed >= total_duration {
            break;
        }

        #[allow(clippy::cast_possible_truncation)]
        let progress = if total_duration.as_millis() > 0 {
            (elapsed.as_millis() * 100 / total_duration.as_millis()) as u64
        } else {
            0
        };
        pb.set_position(progress.min(100));

        std::thread::sleep(update_interval);
    }

    pb.finish_with_message("Done");
}

/// エクスポート用プログレスバーを作成
///
/// # Errors
/// Returns `anyhow::Result` if progress bar template is invalid.
pub fn create_export_progress(total_samples: u64) -> Result<ProgressBar> {
    let pb = ProgressBar::new(total_samples);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} samples {msg}")?
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(Duration::from_millis(16)); // 60fps
    Ok(pb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_export_progress_returns_valid_progressbar() {
        let pb = create_export_progress(100);
        assert!(pb.is_ok());
    }

    #[test]
    fn test_success_does_not_panic() {
        success("test success");
    }

    #[test]
    fn test_error_does_not_panic() {
        error("test error");
    }

    #[test]
    fn test_warning_does_not_panic() {
        warning("test warning");
    }

    #[test]
    fn test_info_does_not_panic() {
        info("test info");
    }

    #[test]
    fn test_display_midi_progress_zero_duration() {
        let interrupt = Arc::new(AtomicBool::new(false));
        // 0ミリ秒の場合は即座に返る（panicしない）
        display_midi_progress(0, &interrupt);
    }

    #[test]
    fn test_display_midi_progress_interrupt_immediately() {
        let interrupt = Arc::new(AtomicBool::new(true));
        // 即座に中断フラグが立っている場合、ループに入らず終了
        display_midi_progress(500, &interrupt);
    }

    #[test]
    fn test_display_midi_progress_short_duration() {
        let interrupt = Arc::new(AtomicBool::new(false));
        let start = Instant::now();

        // 別スレッドで150ms後に中断
        let interrupt_clone = Arc::clone(&interrupt);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            interrupt_clone.store(true, Ordering::SeqCst);
        });

        display_midi_progress(300, &interrupt);
        let elapsed = start.elapsed().as_millis();

        // 約150ms付近で終了（100ms~400ms許容）
        assert!(
            (100..=400).contains(&elapsed),
            "Expected ~150ms, got {elapsed}ms"
        );
    }
}
