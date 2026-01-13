//! A2DP (Advanced Audio Distribution Profile)
//!
//! High-level A2DP Source implementation.

use crate::avdtp::{SbcCapability, SessionState, StreamEndpoint};
use crate::BdAddr;

/// A2DP connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum A2dpState {
    /// Not connected
    #[default]
    Disconnected,
    /// Discoverable, waiting for connection
    Discoverable,
    /// ACL connection in progress
    Connecting,
    /// ACL connected, setting up L2CAP
    Connected,
    /// AVDTP configuration in progress
    Configuring,
    /// Stream open, ready to stream
    Open,
    /// Actively streaming audio
    Streaming,
    /// Stream suspended
    Suspended,
    /// Disconnecting
    Disconnecting,
}

/// Negotiated SBC configuration
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NegotiatedConfig {
    /// Sampling frequency in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Block length
    pub blocks: u8,
    /// Number of subbands
    pub subbands: u8,
    /// Bitpool value
    pub bitpool: u8,
    /// Joint stereo enabled
    pub joint_stereo: bool,
    /// Loudness allocation
    pub loudness: bool,
}

impl NegotiatedConfig {
    /// Create from SBC capability (selecting best options)
    pub fn from_capability(cap: &SbcCapability) -> Self {
        // Select highest quality options from capabilities

        let sample_rate = if cap.sampling_freq & 0x10 != 0 {
            48000
        } else if cap.sampling_freq & 0x20 != 0 {
            44100
        } else if cap.sampling_freq & 0x40 != 0 {
            32000
        } else {
            16000
        };

        let channels = if cap.channel_mode & 0x01 != 0 {
            2 // Joint Stereo
        } else if cap.channel_mode & 0x02 != 0 {
            2 // Stereo
        } else if cap.channel_mode & 0x04 != 0 {
            2 // Dual Channel
        } else {
            1 // Mono
        };

        let joint_stereo = cap.channel_mode & 0x01 != 0;

        let blocks = if cap.block_length & 0x01 != 0 {
            16
        } else if cap.block_length & 0x02 != 0 {
            12
        } else if cap.block_length & 0x04 != 0 {
            8
        } else {
            4
        };

        let subbands = if cap.subbands & 0x01 != 0 { 8 } else { 4 };

        let loudness = cap.allocation_method & 0x01 != 0;

        Self {
            sample_rate,
            channels,
            blocks,
            subbands,
            bitpool: cap.max_bitpool.min(53), // Cap at high quality
            joint_stereo,
            loudness,
        }
    }

    /// Calculate frame duration in microseconds
    pub fn frame_duration_us(&self) -> u32 {
        let samples = (self.blocks as u32) * (self.subbands as u32);
        (samples * 1_000_000) / self.sample_rate
    }
}

/// A2DP Source context
pub struct A2dpSource {
    /// Current state
    pub state: A2dpState,
    /// Remote device address
    pub remote_addr: Option<BdAddr>,
    /// Local stream endpoint
    pub local_sep: StreamEndpoint,
    /// Remote SEID
    pub remote_seid: Option<u8>,
    /// Negotiated configuration
    pub config: Option<NegotiatedConfig>,
    /// AVDTP session state
    pub avdtp_state: SessionState,
    /// Media sequence number
    pub sequence: u16,
    /// Media timestamp
    pub timestamp: u32,
}

impl A2dpSource {
    /// Create a new A2DP Source
    pub fn new() -> Self {
        Self {
            state: A2dpState::Disconnected,
            remote_addr: None,
            local_sep: StreamEndpoint::new_source(1),
            remote_seid: None,
            config: None,
            avdtp_state: SessionState::Idle,
            sequence: 0,
            timestamp: 0,
        }
    }

    /// Check if ready to stream
    pub fn is_streaming(&self) -> bool {
        self.state == A2dpState::Streaming
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(
            self.state,
            A2dpState::Connected
                | A2dpState::Configuring
                | A2dpState::Open
                | A2dpState::Streaming
                | A2dpState::Suspended
        )
    }

    /// Get next sequence number
    pub fn next_sequence(&mut self) -> u16 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    /// Advance timestamp by samples
    pub fn advance_timestamp(&mut self, samples: u32) {
        self.timestamp = self.timestamp.wrapping_add(samples);
    }

    /// Reset for new connection
    pub fn reset(&mut self) {
        self.state = A2dpState::Disconnected;
        self.remote_addr = None;
        self.remote_seid = None;
        self.config = None;
        self.avdtp_state = SessionState::Idle;
        self.sequence = 0;
        self.timestamp = 0;
    }
}

impl Default for A2dpSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2dp_source_creation() {
        let source = A2dpSource::new();
        assert_eq!(source.state, A2dpState::Disconnected);
        assert!(!source.is_connected());
        assert!(!source.is_streaming());
    }

    #[test]
    fn test_sequence_wrap() {
        let mut source = A2dpSource::new();
        source.sequence = 0xFFFF;

        let seq = source.next_sequence();
        assert_eq!(seq, 0xFFFF);
        assert_eq!(source.sequence, 0);
    }

    #[test]
    fn test_negotiated_config() {
        let cap = SbcCapability::high_quality();
        let config = NegotiatedConfig::from_capability(&cap);

        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
        assert!(config.joint_stereo);
    }
}
