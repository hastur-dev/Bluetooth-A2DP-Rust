//! USB Audio Class 2.0 (UAC2) device implementation
//!
//! Provides USB Audio Class support for receiving audio from a host computer.
//! The device appears as a USB speaker/sound card to the host.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

mod descriptor;

pub use descriptor::{AudioControlDescriptor, AudioStreamingDescriptor, Uac2Config};

use heapless::Vec;

/// Maximum USB audio packet size (48kHz stereo 16-bit @ 1ms = 192 bytes + margin)
pub const MAX_USB_AUDIO_PACKET: usize = 196;

/// USB Audio Class codes
pub mod class {
    /// Audio class
    pub const AUDIO: u8 = 0x01;
    /// Audio Control subclass
    pub const AUDIO_CONTROL: u8 = 0x01;
    /// Audio Streaming subclass
    pub const AUDIO_STREAMING: u8 = 0x02;
    /// UAC2 protocol
    pub const UAC2_PROTOCOL: u8 = 0x20;
}

/// Audio format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AudioFormat {
    /// PCM audio (uncompressed)
    #[default]
    Pcm,
    /// IEEE float
    IeeeFloat,
}

/// Sample rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleRate {
    /// 16 kHz
    Rate16000,
    /// 32 kHz
    Rate32000,
    /// 44.1 kHz (CD quality)
    Rate44100,
    /// 48 kHz
    Rate48000,
}

impl SampleRate {
    /// Get the rate in Hz
    pub const fn hz(&self) -> u32 {
        match self {
            Self::Rate16000 => 16000,
            Self::Rate32000 => 32000,
            Self::Rate44100 => 44100,
            Self::Rate48000 => 48000,
        }
    }

    /// Calculate bytes per USB frame (1ms)
    pub const fn bytes_per_frame(&self, channels: u8, bits: u8) -> usize {
        let samples_per_frame = self.hz() / 1000;
        (samples_per_frame as usize) * (channels as usize) * (bits as usize / 8)
    }
}

impl Default for SampleRate {
    fn default() -> Self {
        Self::Rate44100
    }
}

/// USB Audio streaming state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StreamState {
    /// Idle, not streaming
    #[default]
    Idle,
    /// Streaming active
    Active,
    /// Underrun detected
    Underrun,
    /// Overrun detected
    Overrun,
}

/// USB Audio receiver statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioStats {
    /// Total packets received
    pub packets_received: u32,
    /// Total samples received
    pub samples_received: u64,
    /// Underrun count
    pub underruns: u32,
    /// Overrun count
    pub overruns: u32,
}

/// Audio sample buffer (interleaved stereo)
pub type AudioBuffer = Vec<i16, 256>;
