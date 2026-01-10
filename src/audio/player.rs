use crate::audio::error::AudioError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

struct PlaybackState {
    samples: Vec<f32>,
    position: usize,
    loop_enabled: bool,
}

pub struct AudioPlayer {
    device: cpal::Device,
    config: cpal::StreamConfig,
    stream: Option<cpal::Stream>,
}

impl AudioPlayer {
    /// Creates a new `AudioPlayer`.
    ///
    /// # Errors
    /// Returns `AudioError` if no device is found or stream creation fails.
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::DeviceNotFound)?;
        
        let config = device
            .default_output_config()
            .map_err(|e| AudioError::StreamCreationError(e.to_string()))?;
            
        Ok(Self {
            device,
            config: config.into(),
            stream: None,
        })
    }

    /// Starts audio playback.
    ///
    /// # Errors
    /// Returns `AudioError` if stream creation or playback fails.
    pub fn play(&mut self, samples: &[f32], loop_enabled: bool) -> Result<(), AudioError> {
        // Stop current playback if any
        self.stop();

        let state = Arc::new(Mutex::new(PlaybackState {
            samples: samples.to_vec(),
            position: 0,
            loop_enabled,
        }));
        
        let state_clone = state.clone();
        let channels = self.config.channels as usize;

        let err_fn = |err| eprintln!("Audio stream error: {err}");

        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if let Ok(mut state) = state_clone.lock() {
                    for frame in data.chunks_mut(channels) {
                        let sample = if state.position < state.samples.len() {
                            let s = state.samples[state.position];
                            state.position += 1;
                            s
                        } else if state.loop_enabled {
                             state.position = 0;
                             if state.samples.is_empty() {
                                 0.0
                             } else {
                                 let s = state.samples[0];
                                 state.position = 1;
                                 s
                             }
                        } else {
                            0.0
                        };
                        
                        for sample_out in frame.iter_mut() {
                            *sample_out = sample;
                        }
                    }
                }
            },
            err_fn,
            None,
        ).map_err(|e| AudioError::StreamCreationError(e.to_string()))?;

        stream.play().map_err(|e| AudioError::PlaybackError(e.to_string()))?;
        
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None; // Dropping the stream stops it
    }

    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.stream.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_lifecycle() {
        let result = AudioPlayer::new();
        match result {
            Ok(mut player) => {
                assert!(!player.is_playing());
                let samples = vec![0.0; 44100];
                if let Err(e) = player.play(&samples, false) {
                     println!("Play failed: {}, but player creation succeeded.", e);
                } else {
                     assert!(player.is_playing());
                     player.stop();
                     assert!(!player.is_playing());
                }
            }
            Err(AudioError::DeviceNotFound) => {
                println!("No audio device found, skipping lifecycle test");
            }
            Err(AudioError::StreamCreationError(msg)) => {
                 println!("Stream creation failed (likely no audio support in container): {}", msg);
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
