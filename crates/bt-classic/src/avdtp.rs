//! AVDTP (Audio/Video Distribution Transport Protocol)
//!
//! Implements stream establishment and media transport for A2DP.


/// Maximum AVDTP signaling packet size
pub const MAX_AVDTP_SIGNAL: usize = 256;

/// AVDTP signal identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SignalId {
    Discover = 0x01,
    GetCapabilities = 0x02,
    SetConfiguration = 0x03,
    GetConfiguration = 0x04,
    Reconfigure = 0x05,
    Open = 0x06,
    Start = 0x07,
    Close = 0x08,
    Suspend = 0x09,
    Abort = 0x0A,
    SecurityControl = 0x0B,
    GetAllCapabilities = 0x0C,
    DelayReport = 0x0D,
}

/// AVDTP message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MessageType {
    Command = 0x00,
    GeneralReject = 0x01,
    ResponseAccept = 0x02,
    ResponseReject = 0x03,
}

/// AVDTP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ErrorCode {
    Success = 0x00,
    BadHeaderFormat = 0x01,
    BadLength = 0x11,
    BadAcpSeid = 0x12,
    SepInUse = 0x13,
    SepNotInUse = 0x14,
    BadServiceCategory = 0x17,
    BadPayloadFormat = 0x18,
    NotSupportedCommand = 0x19,
    InvalidCapabilities = 0x1A,
    BadRecoveryType = 0x22,
    BadMediaTransportFormat = 0x23,
    BadRecoveryFormat = 0x25,
    BadRohcFormat = 0x26,
    BadCpFormat = 0x27,
    BadMultiplexingFormat = 0x28,
    UnsupportedConfiguration = 0x29,
    BadState = 0x31,
}

/// Stream Endpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SepType {
    Source = 0x00,
    Sink = 0x01,
}

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MediaType {
    Audio = 0x00,
    Video = 0x01,
    Multimedia = 0x02,
}

/// Service category for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ServiceCategory {
    MediaTransport = 0x01,
    Reporting = 0x02,
    Recovery = 0x03,
    ContentProtection = 0x04,
    HeaderCompression = 0x05,
    Multiplexing = 0x06,
    MediaCodec = 0x07,
    DelayReporting = 0x08,
}

/// SBC codec capability
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SbcCapability {
    /// Supported sampling frequencies (bitmap)
    pub sampling_freq: u8,
    /// Supported channel modes (bitmap)
    pub channel_mode: u8,
    /// Supported block lengths (bitmap)
    pub block_length: u8,
    /// Supported subbands (bitmap)
    pub subbands: u8,
    /// Supported allocation methods (bitmap)
    pub allocation_method: u8,
    /// Minimum bitpool value
    pub min_bitpool: u8,
    /// Maximum bitpool value
    pub max_bitpool: u8,
}

impl SbcCapability {
    /// Create SBC capability supporting all standard options
    pub const fn all() -> Self {
        Self {
            sampling_freq: 0xFF,      // All frequencies
            channel_mode: 0x0F,       // All modes
            block_length: 0x0F,       // All block lengths
            subbands: 0x03,           // 4 and 8 subbands
            allocation_method: 0x03,  // SNR and Loudness
            min_bitpool: 2,
            max_bitpool: 250,
        }
    }

    /// Create a typical high-quality configuration
    pub const fn high_quality() -> Self {
        Self {
            sampling_freq: 0x20,      // 44.1 kHz
            channel_mode: 0x01,       // Joint Stereo
            block_length: 0x01,       // 16 blocks
            subbands: 0x01,           // 8 subbands
            allocation_method: 0x01,  // Loudness
            min_bitpool: 35,
            max_bitpool: 53,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 4, "Buffer too small");

        buf[0] = (self.sampling_freq << 4) | self.channel_mode;
        buf[1] = (self.block_length << 4) | (self.subbands << 2) | self.allocation_method;
        buf[2] = self.min_bitpool;
        buf[3] = self.max_bitpool;

        4
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        Some(Self {
            sampling_freq: bytes[0] >> 4,
            channel_mode: bytes[0] & 0x0F,
            block_length: bytes[1] >> 4,
            subbands: (bytes[1] >> 2) & 0x03,
            allocation_method: bytes[1] & 0x03,
            min_bitpool: bytes[2],
            max_bitpool: bytes[3],
        })
    }
}

/// Stream Endpoint (SEP)
#[derive(Debug, Clone)]
pub struct StreamEndpoint {
    /// Stream Endpoint ID (1-62)
    pub seid: u8,
    /// Whether the SEP is in use
    pub in_use: bool,
    /// Media type
    pub media_type: MediaType,
    /// SEP type (Source or Sink)
    pub sep_type: SepType,
    /// SBC codec capability
    pub sbc_capability: SbcCapability,
}

impl StreamEndpoint {
    /// Create a new A2DP Source endpoint
    pub fn new_source(seid: u8) -> Self {
        Self {
            seid,
            in_use: false,
            media_type: MediaType::Audio,
            sep_type: SepType::Source,
            sbc_capability: SbcCapability::all(),
        }
    }
}

/// AVDTP media packet header (RTP-like)
#[derive(Debug, Clone, Copy, Default)]
pub struct MediaHeader {
    /// RTP version (always 2)
    pub version: u8,
    /// Padding flag
    pub padding: bool,
    /// Extension flag
    pub extension: bool,
    /// CSRC count
    pub cc: u8,
    /// Marker bit
    pub marker: bool,
    /// Payload type
    pub payload_type: u8,
    /// Sequence number
    pub sequence: u16,
    /// Timestamp
    pub timestamp: u32,
    /// Synchronization source
    pub ssrc: u32,
}

impl MediaHeader {
    /// Create a new media header
    pub fn new() -> Self {
        Self {
            version: 2,
            payload_type: 96, // Dynamic payload type for SBC
            ..Default::default()
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 12, "Buffer too small for RTP header");

        buf[0] = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | self.cc;
        buf[1] = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        buf[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        12
    }
}

/// AVDTP session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SessionState {
    #[default]
    Idle,
    Discovering,
    Configuring,
    Open,
    Streaming,
    Closing,
    Aborting,
}
