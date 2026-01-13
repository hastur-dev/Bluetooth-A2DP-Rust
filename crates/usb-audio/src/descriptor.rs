//! USB Audio Class 2.0 descriptors

/// UAC2 device configuration
#[derive(Debug, Clone)]
pub struct Uac2Config {
    /// Device name
    pub name: &'static str,
    /// Vendor ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Number of channels
    pub channels: u8,
    /// Bits per sample
    pub bit_depth: u8,
    /// Supported sample rates
    pub sample_rates: &'static [u32],
}

impl Default for Uac2Config {
    fn default() -> Self {
        Self {
            name: "Pico A2DP Audio",
            vid: 0x1209, // pid.codes test VID
            pid: 0xA2D0, // "A2D0" - A2DP-like
            channels: 2,
            bit_depth: 16,
            sample_rates: &[44100, 48000],
        }
    }
}

/// Audio Control Interface descriptor builder
pub struct AudioControlDescriptor {
    config: Uac2Config,
}

impl AudioControlDescriptor {
    /// Create a new Audio Control descriptor builder
    pub fn new(config: Uac2Config) -> Self {
        Self { config }
    }

    /// Build the descriptor bytes
    pub fn build(&self, buf: &mut [u8]) -> usize {
        // Audio Control Interface Header
        // This is a simplified implementation

        let mut pos = 0;

        // Interface descriptor (Audio Control)
        buf[pos] = 9; // bLength
        buf[pos + 1] = 4; // bDescriptorType (Interface)
        buf[pos + 2] = 0; // bInterfaceNumber
        buf[pos + 3] = 0; // bAlternateSetting
        buf[pos + 4] = 0; // bNumEndpoints
        buf[pos + 5] = 0x01; // bInterfaceClass (Audio)
        buf[pos + 6] = 0x01; // bInterfaceSubClass (Audio Control)
        buf[pos + 7] = 0x20; // bInterfaceProtocol (UAC2)
        buf[pos + 8] = 0; // iInterface
        pos += 9;

        // AC Interface Header
        buf[pos] = 9; // bLength
        buf[pos + 1] = 0x24; // bDescriptorType (CS_INTERFACE)
        buf[pos + 2] = 0x01; // bDescriptorSubtype (HEADER)
        buf[pos + 3] = 0x00; // bcdADC low
        buf[pos + 4] = 0x02; // bcdADC high (2.0)
        buf[pos + 5] = 0x08; // bCategory (I/O Box)
        buf[pos + 6] = 0; // wTotalLength low (placeholder)
        buf[pos + 7] = 0; // wTotalLength high
        buf[pos + 8] = 0; // bmControls
        pos += 9;

        // Clock Source
        buf[pos] = 8; // bLength
        buf[pos + 1] = 0x24; // bDescriptorType
        buf[pos + 2] = 0x0A; // bDescriptorSubtype (CLOCK_SOURCE)
        buf[pos + 3] = 1; // bClockID
        buf[pos + 4] = 0x01; // bmAttributes (internal fixed)
        buf[pos + 5] = 0x01; // bmControls
        buf[pos + 6] = 0; // bAssocTerminal
        buf[pos + 7] = 0; // iClockSource
        pos += 8;

        // Input Terminal (USB streaming)
        buf[pos] = 17; // bLength
        buf[pos + 1] = 0x24; // bDescriptorType
        buf[pos + 2] = 0x02; // bDescriptorSubtype (INPUT_TERMINAL)
        buf[pos + 3] = 1; // bTerminalID
        buf[pos + 4] = 0x01; // wTerminalType low (USB streaming)
        buf[pos + 5] = 0x01; // wTerminalType high
        buf[pos + 6] = 0; // bAssocTerminal
        buf[pos + 7] = 1; // bCSourceID (clock)
        buf[pos + 8] = self.config.channels; // bNrChannels
        buf[pos + 9] = 0x03; // bmChannelConfig low (L+R)
        buf[pos + 10] = 0x00;
        buf[pos + 11] = 0x00;
        buf[pos + 12] = 0x00;
        buf[pos + 13] = 0; // iChannelNames
        buf[pos + 14] = 0; // bmControls low
        buf[pos + 15] = 0; // bmControls high
        buf[pos + 16] = 0; // iTerminal
        pos += 17;

        // Output Terminal (Speaker)
        buf[pos] = 12; // bLength
        buf[pos + 1] = 0x24; // bDescriptorType
        buf[pos + 2] = 0x03; // bDescriptorSubtype (OUTPUT_TERMINAL)
        buf[pos + 3] = 2; // bTerminalID
        buf[pos + 4] = 0x01; // wTerminalType low (Speaker)
        buf[pos + 5] = 0x03; // wTerminalType high
        buf[pos + 6] = 0; // bAssocTerminal
        buf[pos + 7] = 1; // bSourceID (input terminal)
        buf[pos + 8] = 1; // bCSourceID (clock)
        buf[pos + 9] = 0; // bmControls low
        buf[pos + 10] = 0; // bmControls high
        buf[pos + 11] = 0; // iTerminal
        pos += 12;

        pos
    }
}

/// Audio Streaming Interface descriptor builder
pub struct AudioStreamingDescriptor {
    config: Uac2Config,
}

impl AudioStreamingDescriptor {
    /// Create a new Audio Streaming descriptor builder
    pub fn new(config: Uac2Config) -> Self {
        Self { config }
    }

    /// Build the descriptor bytes for alternate setting 0 (zero bandwidth)
    pub fn build_alt0(&self, buf: &mut [u8], interface_num: u8) -> usize {
        let mut pos = 0;

        // Interface descriptor (zero bandwidth)
        buf[pos] = 9;
        buf[pos + 1] = 4; // Interface
        buf[pos + 2] = interface_num;
        buf[pos + 3] = 0; // bAlternateSetting
        buf[pos + 4] = 0; // bNumEndpoints
        buf[pos + 5] = 0x01; // Audio
        buf[pos + 6] = 0x02; // Audio Streaming
        buf[pos + 7] = 0x20; // UAC2
        buf[pos + 8] = 0;
        pos += 9;

        pos
    }

    /// Build the descriptor bytes for alternate setting 1 (active streaming)
    pub fn build_alt1(&self, buf: &mut [u8], interface_num: u8, ep_addr: u8) -> usize {
        let mut pos = 0;

        // Interface descriptor (active)
        buf[pos] = 9;
        buf[pos + 1] = 4;
        buf[pos + 2] = interface_num;
        buf[pos + 3] = 1; // bAlternateSetting
        buf[pos + 4] = 1; // bNumEndpoints (or 2 with feedback)
        buf[pos + 5] = 0x01;
        buf[pos + 6] = 0x02;
        buf[pos + 7] = 0x20;
        buf[pos + 8] = 0;
        pos += 9;

        // AS Interface descriptor
        buf[pos] = 16;
        buf[pos + 1] = 0x24; // CS_INTERFACE
        buf[pos + 2] = 0x01; // AS_GENERAL
        buf[pos + 3] = 1; // bTerminalLink
        buf[pos + 4] = 0; // bmControls
        buf[pos + 5] = 0x01; // bFormatType (Type I)
        buf[pos + 6] = 0x01; // bmFormats (PCM)
        buf[pos + 7] = 0x00;
        buf[pos + 8] = 0x00;
        buf[pos + 9] = 0x00;
        buf[pos + 10] = self.config.channels;
        buf[pos + 11] = 0x03; // bmChannelConfig
        buf[pos + 12] = 0x00;
        buf[pos + 13] = 0x00;
        buf[pos + 14] = 0x00;
        buf[pos + 15] = 0;
        pos += 16;

        // Format Type I descriptor
        buf[pos] = 6;
        buf[pos + 1] = 0x24;
        buf[pos + 2] = 0x02; // FORMAT_TYPE
        buf[pos + 3] = 0x01; // FORMAT_TYPE_I
        buf[pos + 4] = self.config.bit_depth / 8; // bSubslotSize
        buf[pos + 5] = self.config.bit_depth; // bBitResolution
        pos += 6;

        // Endpoint descriptor
        let max_packet = 48 * (self.config.channels as u16) * 2 + 4; // 48kHz + margin
        buf[pos] = 7;
        buf[pos + 1] = 5; // Endpoint
        buf[pos + 2] = ep_addr;
        buf[pos + 3] = 0x05; // Isochronous, Async
        buf[pos + 4] = (max_packet & 0xFF) as u8;
        buf[pos + 5] = (max_packet >> 8) as u8;
        buf[pos + 6] = 1; // bInterval (1ms)
        pos += 7;

        // AS Isochronous Audio Data Endpoint descriptor
        buf[pos] = 8;
        buf[pos + 1] = 0x25; // CS_ENDPOINT
        buf[pos + 2] = 0x01; // EP_GENERAL
        buf[pos + 3] = 0; // bmAttributes
        buf[pos + 4] = 0; // bmControls
        buf[pos + 5] = 0; // bLockDelayUnits
        buf[pos + 6] = 0; // wLockDelay low
        buf[pos + 7] = 0; // wLockDelay high
        pos += 8;

        pos
    }
}
