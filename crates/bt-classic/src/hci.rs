//! HCI (Host Controller Interface) layer
//!
//! Provides the interface to the Bluetooth controller via HCI commands
//! and events. Uses the bt-hci crate for basic types.

use heapless::Vec;

/// Maximum HCI packet size
pub const MAX_HCI_PACKET_SIZE: usize = 512;

/// HCI packet type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PacketType {
    Command = 0x01,
    AclData = 0x02,
    ScoData = 0x03,
    Event = 0x04,
}

/// HCI connection handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectionHandle(pub u16);

impl ConnectionHandle {
    /// Create from raw handle value
    pub const fn new(handle: u16) -> Self {
        Self(handle & 0x0FFF) // 12 bits
    }

    /// Get the raw handle value
    pub const fn raw(&self) -> u16 {
        self.0
    }
}

/// HCI event codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum EventCode {
    InquiryComplete = 0x01,
    InquiryResult = 0x02,
    ConnectionComplete = 0x03,
    ConnectionRequest = 0x04,
    DisconnectionComplete = 0x05,
    AuthenticationComplete = 0x06,
    RemoteNameRequestComplete = 0x07,
    CommandComplete = 0x0E,
    CommandStatus = 0x0F,
    NumberOfCompletedPackets = 0x13,
    IoCapabilityRequest = 0x31,
    IoCapabilityResponse = 0x32,
    UserConfirmationRequest = 0x33,
    SimplePairingComplete = 0x36,
}

/// ACL data packet
#[derive(Debug)]
pub struct AclPacket {
    /// Connection handle
    pub handle: ConnectionHandle,
    /// Packet boundary flag
    pub pb_flag: u8,
    /// Broadcast flag
    pub bc_flag: u8,
    /// Data payload
    pub data: Vec<u8, MAX_HCI_PACKET_SIZE>,
}

impl AclPacket {
    /// Create a new ACL packet
    pub fn new(handle: ConnectionHandle, pb_flag: u8, bc_flag: u8) -> Self {
        Self {
            handle,
            pb_flag,
            bc_flag,
            data: Vec::new(),
        }
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let handle_flags = u16::from_le_bytes([bytes[0], bytes[1]]);
        let handle = ConnectionHandle::new(handle_flags & 0x0FFF);
        let pb_flag = ((handle_flags >> 12) & 0x03) as u8;
        let bc_flag = ((handle_flags >> 14) & 0x03) as u8;

        let data_len = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;

        if bytes.len() < 4 + data_len {
            return None;
        }

        let mut data = Vec::new();
        data.extend_from_slice(&bytes[4..4 + data_len]).ok()?;

        Some(Self {
            handle,
            pb_flag,
            bc_flag,
            data,
        })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 4 + self.data.len(), "Buffer too small");

        let handle_flags = self.handle.0 | ((self.pb_flag as u16) << 12) | ((self.bc_flag as u16) << 14);

        buf[0..2].copy_from_slice(&handle_flags.to_le_bytes());
        buf[2..4].copy_from_slice(&(self.data.len() as u16).to_le_bytes());
        buf[4..4 + self.data.len()].copy_from_slice(&self.data);

        4 + self.data.len()
    }
}
