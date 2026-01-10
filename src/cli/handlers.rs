use anyhow::{bail, Result, Context};
use crate::cli::args::PlayArgs;
use crate::{mml, audio, db};

pub fn play_handler(_args: PlayArgs) -> Result<()> {
    todo!("Implement play_handler")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{PlayArgs, Waveform};

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
        // todo! でパニックするはず
        let result = play_handler(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "[CLI-E001] play コマンドでは、MML文字列または --history-id のいずれか一方を指定してください");
    }
}
