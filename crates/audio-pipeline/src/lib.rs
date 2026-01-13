//! Audio pipeline for embedded A2DP
//!
//! Provides lock-free ring buffers and format conversion utilities
//! for streaming audio between USB reception and SBC encoding.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

mod ring_buffer;

pub use ring_buffer::RingBuffer;

/// Audio format description
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Bits per sample (typically 16)
    pub bits_per_sample: u8,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 16,
        }
    }
}

impl AudioFormat {
    /// Calculate bytes per sample (all channels)
    pub const fn bytes_per_sample(&self) -> usize {
        (self.channels as usize) * (self.bits_per_sample as usize / 8)
    }

    /// Calculate bytes per second
    pub const fn bytes_per_second(&self) -> usize {
        self.sample_rate as usize * self.bytes_per_sample()
    }
}
