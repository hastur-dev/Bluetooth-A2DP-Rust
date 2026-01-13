//! Bluetooth Classic host stack for A2DP
//!
//! This crate implements the Bluetooth Classic protocol layers needed
//! for A2DP audio streaming:
//! - L2CAP: Logical Link Control and Adaptation Protocol
//! - SDP: Service Discovery Protocol
//! - AVDTP: Audio/Video Distribution Transport Protocol
//! - A2DP: Advanced Audio Distribution Profile

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod a2dp;
pub mod avdtp;
pub mod hci;
pub mod l2cap;
pub mod sdp;

/// Bluetooth device address (6 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BdAddr(pub [u8; 6]);

impl BdAddr {
    /// Create a new Bluetooth address
    pub const fn new(addr: [u8; 6]) -> Self {
        Self(addr)
    }

    /// Get the address bytes
    pub const fn bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

/// Common error type for Bluetooth operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BtError {
    /// HCI error
    Hci(u8),
    /// L2CAP error
    L2cap(u8),
    /// AVDTP error
    Avdtp(u8),
    /// Timeout
    Timeout,
    /// Connection failed
    ConnectionFailed,
    /// Not connected
    NotConnected,
    /// Invalid state
    InvalidState,
    /// Buffer too small
    BufferTooSmall,
    /// Invalid parameter
    InvalidParameter,
}
