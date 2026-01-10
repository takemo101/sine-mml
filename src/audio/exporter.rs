#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
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

        // Expectation: export_wav exists and works
        let result = export_wav(&samples, &path);
        assert!(result.is_ok());
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

        // Check conversion accuracy (f32 -> i16)
        // tolerance depends on conversion method, but approx check is good
        // first sample is 0
        assert!((read_samples[0] as f32).abs() < 10.0);
        
        // Cleanup
        let _ = std::fs::remove_file(path);
    }
}
