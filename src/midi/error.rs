//! MIDI error types
//!
//! This module defines error types for MIDI operations.

use thiserror::Error;

/// MIDI-related errors
#[derive(Debug, Error)]
pub enum MidiError {
    /// No MIDI device found
    #[error("[MML-E015] MIDIデバイスが見つかりません")]
    NoDeviceFound,

    /// Failed to connect to MIDI device
    #[error("[MML-E016] MIDIデバイスへの接続に失敗しました: {reason}")]
    ConnectionFailed {
        /// Reason for connection failure
        reason: String,
    },

    /// Failed to send MIDI message
    #[error("[MML-E017] MIDIメッセージの送信に失敗しました: {reason}")]
    SendFailed {
        /// Reason for send failure
        reason: String,
    },

    /// Invalid MIDI device ID
    #[error("[MML-E018] 無効なMIDIデバイスIDです: {id}")]
    InvalidDeviceId {
        /// The invalid device ID
        id: usize,
    },

    /// MIDI device disconnected
    #[error("[MML-E019] MIDIデバイスが切断されました")]
    DeviceDisconnected,

    /// Invalid MIDI channel (must be 1-16)
    #[error("[MML-E024] 無効なMIDIチャンネルです（1-16を指定してください）: {channel}")]
    InvalidChannel {
        /// The invalid channel number
        channel: u8,
    },
}

impl MidiError {
    /// Create a connection failed error
    #[must_use]
    pub fn connection_failed(reason: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            reason: reason.into(),
        }
    }

    /// Create a send failed error
    #[must_use]
    pub fn send_failed(reason: impl Into<String>) -> Self {
        Self::SendFailed {
            reason: reason.into(),
        }
    }

    /// Create an invalid device ID error
    #[must_use]
    pub fn invalid_device_id(id: usize) -> Self {
        Self::InvalidDeviceId { id }
    }

    /// Create an invalid channel error
    #[must_use]
    pub fn invalid_channel(channel: u8) -> Self {
        Self::InvalidChannel { channel }
    }

    /// Check if a channel number is valid (1-16)
    #[must_use]
    pub fn is_valid_channel(channel: u8) -> bool {
        (1..=16).contains(&channel)
    }

    /// Validate a channel number, returning an error if invalid
    pub fn validate_channel(channel: u8) -> Result<(), Self> {
        if Self::is_valid_channel(channel) {
            Ok(())
        } else {
            Err(Self::invalid_channel(channel))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_device_found_display() {
        let err = MidiError::NoDeviceFound;
        assert_eq!(
            err.to_string(),
            "[MML-E015] MIDIデバイスが見つかりません"
        );
    }

    #[test]
    fn test_connection_failed_display() {
        let err = MidiError::connection_failed("timeout");
        assert_eq!(
            err.to_string(),
            "[MML-E016] MIDIデバイスへの接続に失敗しました: timeout"
        );
    }

    #[test]
    fn test_send_failed_display() {
        let err = MidiError::send_failed("device busy");
        assert_eq!(
            err.to_string(),
            "[MML-E017] MIDIメッセージの送信に失敗しました: device busy"
        );
    }

    #[test]
    fn test_invalid_device_id_display() {
        let err = MidiError::invalid_device_id(99);
        assert_eq!(err.to_string(), "[MML-E018] 無効なMIDIデバイスIDです: 99");
    }

    #[test]
    fn test_device_disconnected_display() {
        let err = MidiError::DeviceDisconnected;
        assert_eq!(err.to_string(), "[MML-E019] MIDIデバイスが切断されました");
    }

    #[test]
    fn test_invalid_channel_display() {
        let err = MidiError::invalid_channel(17);
        assert_eq!(
            err.to_string(),
            "[MML-E024] 無効なMIDIチャンネルです（1-16を指定してください）: 17"
        );
    }

    #[test]
    fn test_is_valid_channel_valid() {
        for channel in 1..=16 {
            assert!(MidiError::is_valid_channel(channel));
        }
    }

    #[test]
    fn test_is_valid_channel_invalid_zero() {
        assert!(!MidiError::is_valid_channel(0));
    }

    #[test]
    fn test_is_valid_channel_invalid_seventeen() {
        assert!(!MidiError::is_valid_channel(17));
    }

    #[test]
    fn test_validate_channel_valid() {
        assert!(MidiError::validate_channel(1).is_ok());
        assert!(MidiError::validate_channel(10).is_ok());
        assert!(MidiError::validate_channel(16).is_ok());
    }

    #[test]
    fn test_validate_channel_invalid() {
        let result = MidiError::validate_channel(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MidiError::InvalidChannel { channel: 0 }
        ));

        let result = MidiError::validate_channel(17);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MidiError::InvalidChannel { channel: 17 }
        ));
    }
}
