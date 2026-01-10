use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

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

/// エラーメッセージを赤色で表示
pub fn error(msg: &str) {
    eprintln!("{}", style(msg).red().bold());
}

/// 警告メッセージを黄色で表示
pub fn warning(msg: &str) {
    eprintln!("{}", style(msg).yellow());
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
}
