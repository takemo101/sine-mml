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

#[allow(dead_code)] // Might be used later
pub fn info(msg: &str) {
    println!("{}", style(msg).cyan());
}
