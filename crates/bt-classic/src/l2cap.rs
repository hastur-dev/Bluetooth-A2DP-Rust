//! L2CAP (Logical Link Control and Adaptation Protocol)
//!
//! Provides channel multiplexing over ACL links.

use heapless::Vec;

/// Maximum L2CAP payload size
pub const MAX_L2CAP_PAYLOAD: usize = 1024;

/// L2CAP channel ID type
pub type ChannelId = u16;

/// Well-known PSM (Protocol/Service Multiplexer) values
pub mod psm {
    /// SDP protocol
    pub const SDP: u16 = 0x0001;
    /// RFCOMM
    pub const RFCOMM: u16 = 0x0003;
    /// AVDTP
    pub const AVDTP: u16 = 0x0019;
}

/// Well-known channel IDs
pub mod cid {
    /// Signaling channel
    pub const SIGNALING: u16 = 0x0001;
    /// Connectionless channel
    pub const CONNECTIONLESS: u16 = 0x0002;
    /// First dynamically allocated CID
    pub const DYNAMIC_START: u16 = 0x0040;
}

/// L2CAP signaling command codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SignalCode {
    CommandReject = 0x01,
    ConnectionRequest = 0x02,
    ConnectionResponse = 0x03,
    ConfigurationRequest = 0x04,
    ConfigurationResponse = 0x05,
    DisconnectionRequest = 0x06,
    DisconnectionResponse = 0x07,
    EchoRequest = 0x08,
    EchoResponse = 0x09,
    InformationRequest = 0x0A,
    InformationResponse = 0x0B,
}

/// L2CAP channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelState {
    #[default]
    Closed,
    WaitConnect,
    WaitConnectRsp,
    Config,
    Open,
    WaitDisconnect,
}

/// L2CAP channel
#[derive(Debug)]
pub struct Channel {
    /// Local channel ID
    pub local_cid: ChannelId,
    /// Remote channel ID
    pub remote_cid: ChannelId,
    /// Protocol/Service Multiplexer
    pub psm: u16,
    /// Channel state
    pub state: ChannelState,
    /// Negotiated MTU
    pub mtu: u16,
}

impl Channel {
    /// Create a new channel
    pub const fn new(local_cid: ChannelId, psm: u16) -> Self {
        Self {
            local_cid,
            remote_cid: 0,
            psm,
            state: ChannelState::Closed,
            mtu: 672, // Default L2CAP MTU
        }
    }
}

/// L2CAP packet
#[derive(Debug)]
pub struct Packet {
    /// Channel ID
    pub cid: ChannelId,
    /// Payload data
    pub data: Vec<u8, MAX_L2CAP_PAYLOAD>,
}

impl Packet {
    /// Create a new L2CAP packet
    pub fn new(cid: ChannelId) -> Self {
        Self {
            cid,
            data: Vec::new(),
        }
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let length = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        let cid = u16::from_le_bytes([bytes[2], bytes[3]]);

        if bytes.len() < 4 + length {
            return None;
        }

        let mut data = Vec::new();
        data.extend_from_slice(&bytes[4..4 + length]).ok()?;

        Some(Self { cid, data })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 4 + self.data.len(), "Buffer too small");

        buf[0..2].copy_from_slice(&(self.data.len() as u16).to_le_bytes());
        buf[2..4].copy_from_slice(&self.cid.to_le_bytes());
        buf[4..4 + self.data.len()].copy_from_slice(&self.data);

        4 + self.data.len()
    }
}
