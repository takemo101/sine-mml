use std::path::Path;
use crate::audio::AudioError;
use hound::WavSpec;

/// Export PCM samples to a WAV file.
///
/// # Arguments
/// * `samples` - Floating point PCM samples (usually -1.0 to 1.0)
/// * `path` - Destination file path
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(AudioError)` if an error occurs
pub fn export_wav<P: AsRef<Path>>(samples: &[f32], path: P) -> Result<(), AudioError> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    for &sample in samples {
        // f32 -> i16 conversion: scale by 32767.0 and clamp to i16 range
        let amplitude = i16::MAX as f32;
        let s = (sample * amplitude).clamp(i16::MIN as f32, i16::MAX as f32);
        writer.write_sample(s as i16)?;
    }

    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::WavReader;

    #[test]
    fn test_export_wav_success() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_sine_wave.wav");
        
        // Create 1 second of 440Hz sine wave at 44.1kHz
        let sample_rate = 44100;
        let frequency = 440.0;
        let samples: Vec<f32> = (0..sample_rate)
            .map(|t| (t as f32 * frequency * 2.0 * std::f32::consts::PI / sample_rate as f32).sin())
            .collect();

        let result = export_wav(&samples, &path);
        assert!(result.is_ok(), "export_wav failed: {:?}", result.err());
        assert!(path.exists());

        // Verify content
        let mut reader = WavReader::open(&path).expect("Failed to open WAV file");
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 44100);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        let read_samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read_samples.len(), samples.len());

        // Basic check for first few samples to ensure they aren't all zero/garbage
        // First sample of sine(0) is 0
        assert_eq!(read_samples[0], 0);
        
        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}
