//! SBC encoder configuration types

/// Sampling frequency options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SamplingFrequency {
    /// 16 kHz
    Freq16000 = 0,
    /// 32 kHz
    Freq32000 = 1,
    /// 44.1 kHz (CD quality)
    #[default]
    Freq44100 = 2,
    /// 48 kHz
    Freq48000 = 3,
}

impl SamplingFrequency {
    /// Get the frequency in Hz
    pub const fn hz(self) -> u32 {
        match self {
            Self::Freq16000 => 16000,
            Self::Freq32000 => 32000,
            Self::Freq44100 => 44100,
            Self::Freq48000 => 48000,
        }
    }

    /// Get the 2-bit field value for the header
    pub const fn header_bits(self) -> u8 {
        self as u8
    }
}

/// Channel mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ChannelMode {
    /// Mono (1 channel)
    Mono = 0,
    /// Dual Channel (2 independent mono channels)
    DualChannel = 1,
    /// Stereo (2 channels)
    Stereo = 2,
    /// Joint Stereo (2 channels with mid/side coding)
    #[default]
    JointStereo = 3,
}

impl ChannelMode {
    /// Get number of audio channels
    pub const fn channels(self) -> u8 {
        match self {
            Self::Mono => 1,
            Self::DualChannel | Self::Stereo | Self::JointStereo => 2,
        }
    }

    /// Get the 2-bit field value for the header
    pub const fn header_bits(self) -> u8 {
        self as u8
    }
}

/// Block length options (number of blocks per frame)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BlockLength {
    /// 4 blocks
    Blocks4 = 0,
    /// 8 blocks
    Blocks8 = 1,
    /// 12 blocks
    Blocks12 = 2,
    /// 16 blocks (best quality)
    #[default]
    Blocks16 = 3,
}

impl BlockLength {
    /// Get the number of blocks
    pub const fn count(self) -> usize {
        match self {
            Self::Blocks4 => 4,
            Self::Blocks8 => 8,
            Self::Blocks12 => 12,
            Self::Blocks16 => 16,
        }
    }

    /// Get the 2-bit field value for the header
    pub const fn header_bits(self) -> u8 {
        self as u8
    }
}

/// Number of subbands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Subbands {
    /// 4 subbands
    Sub4 = 0,
    /// 8 subbands (better frequency resolution)
    #[default]
    Sub8 = 1,
}

impl Subbands {
    /// Get the number of subbands
    pub const fn count(self) -> usize {
        match self {
            Self::Sub4 => 4,
            Self::Sub8 => 8,
        }
    }

    /// Get the 1-bit field value for the header
    pub const fn header_bits(self) -> u8 {
        self as u8
    }
}

/// Bit allocation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AllocationMethod {
    /// SNR-based allocation
    Snr = 0,
    /// Loudness-based allocation (psychoacoustic)
    #[default]
    Loudness = 1,
}

impl AllocationMethod {
    /// Get the 1-bit field value for the header
    pub const fn header_bits(self) -> u8 {
        self as u8
    }
}

/// SBC encoder configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SbcConfig {
    /// Sampling frequency
    pub sampling_frequency: SamplingFrequency,
    /// Channel mode
    pub channel_mode: ChannelMode,
    /// Number of blocks per frame
    pub block_length: BlockLength,
    /// Number of subbands
    pub subbands: Subbands,
    /// Bit allocation method
    pub allocation_method: AllocationMethod,
    /// Bitpool value (controls quality/bitrate, 2-250)
    pub bitpool: u8,
}

impl Default for SbcConfig {
    fn default() -> Self {
        Self {
            sampling_frequency: SamplingFrequency::Freq44100,
            channel_mode: ChannelMode::JointStereo,
            block_length: BlockLength::Blocks16,
            subbands: Subbands::Sub8,
            allocation_method: AllocationMethod::Loudness,
            bitpool: 53, // High quality default
        }
    }
}

impl SbcConfig {
    /// Create a new SBC configuration
    pub const fn new(
        sampling_frequency: SamplingFrequency,
        channel_mode: ChannelMode,
        block_length: BlockLength,
        subbands: Subbands,
        allocation_method: AllocationMethod,
        bitpool: u8,
    ) -> Self {
        Self {
            sampling_frequency,
            channel_mode,
            block_length,
            subbands,
            allocation_method,
            bitpool,
        }
    }

    /// Check if configuration is valid
    pub const fn is_valid(&self) -> bool {
        // Bitpool must be in valid range
        if self.bitpool < 2 {
            return false;
        }

        // Maximum bitpool depends on channel mode and subbands
        let max_bitpool = self.max_bitpool();
        if self.bitpool > max_bitpool {
            return false;
        }

        true
    }

    /// Get maximum allowed bitpool for this configuration
    pub const fn max_bitpool(&self) -> u8 {
        let subbands = self.subbands.count() as u16;

        match self.channel_mode {
            ChannelMode::Mono | ChannelMode::DualChannel => {
                // Max = 16 * subbands
                let computed = 16 * subbands;
                if computed > 250 { 250 } else { computed as u8 }
            }
            ChannelMode::Stereo | ChannelMode::JointStereo => {
                // Max = 32 * subbands
                // Actually constrained by: floor(8 * frame_length / blocks) - 4*subbands
                // For practical purposes, use 250 as absolute max
                let computed = 32 * subbands;
                if computed > 250 { 250 } else { computed as u8 }
            }
        }
    }

    /// Get number of channels
    pub const fn channels(&self) -> u8 {
        self.channel_mode.channels()
    }

    /// Get number of samples per frame per channel
    pub const fn samples_per_frame(&self) -> usize {
        self.block_length.count() * self.subbands.count()
    }

    /// Calculate frame size in bytes
    pub const fn frame_size(&self) -> usize {
        let subbands = self.subbands.count();
        let blocks = self.block_length.count();
        let channels = self.channels() as usize;
        let bitpool = self.bitpool as usize;

        // Header: 4 bytes
        let header = 4;

        // Scale factors
        let scale_factors = match self.channel_mode {
            ChannelMode::JointStereo => {
                // Join byte + scale factors for each channel
                (subbands + 2 * subbands * 4) / 8 + 1
            }
            _ => (channels * subbands * 4) / 8,
        };

        // Audio samples
        let audio_bits = match self.channel_mode {
            ChannelMode::Mono | ChannelMode::DualChannel => channels * blocks * bitpool,
            ChannelMode::Stereo => blocks * bitpool,
            ChannelMode::JointStereo => blocks * bitpool,
        };
        let audio = (audio_bits + 7) / 8;

        header + scale_factors + audio
    }

    /// Calculate approximate bitrate in kbps
    pub const fn bitrate_kbps(&self) -> u32 {
        let frame_size = self.frame_size() as u32;
        let samples = self.samples_per_frame() as u32;
        let sample_rate = self.sampling_frequency.hz();

        // bitrate = (frame_size * 8 * sample_rate) / samples / 1000
        (frame_size * 8 * sample_rate) / samples / 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = SbcConfig::default();
        assert!(config.is_valid());
    }

    #[test]
    fn test_samples_per_frame() {
        let config = SbcConfig::default();
        // 16 blocks * 8 subbands = 128
        assert_eq!(config.samples_per_frame(), 128);
    }

    #[test]
    fn test_channels() {
        assert_eq!(ChannelMode::Mono.channels(), 1);
        assert_eq!(ChannelMode::Stereo.channels(), 2);
        assert_eq!(ChannelMode::JointStereo.channels(), 2);
    }

    #[test]
    fn test_bitrate_reasonable() {
        let config = SbcConfig::default();
        let bitrate = config.bitrate_kbps();
        // Should be somewhere between 100-400 kbps for typical settings
        assert!(bitrate >= 100 && bitrate <= 500);
    }

    #[test]
    fn test_invalid_bitpool_zero() {
        let config = SbcConfig {
            bitpool: 0,
            ..Default::default()
        };
        assert!(!config.is_valid());
    }
}
