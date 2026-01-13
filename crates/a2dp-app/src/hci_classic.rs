//! Custom HCI commands for Bluetooth Classic that aren't in bt-hci crate
//!
//! These commands are needed for A2DP audio streaming.

use bt_hci::cmd::{Cmd, Opcode, OpcodeGroup, SyncCmd};
use bt_hci::WriteHci;

/// Write Local Name command (OGF=3, OCF=0x13)
///
/// Sets the user-friendly name for the local device.
#[derive(Debug, Clone, Copy)]
pub struct WriteLocalName {
    /// Local name, null-terminated, max 248 bytes
    pub name: [u8; 248],
}

impl WriteLocalName {
    pub fn new(name_str: &[u8]) -> Self {
        let mut name = [0u8; 248];
        let len = name_str.len().min(247);
        name[..len].copy_from_slice(&name_str[..len]);
        Self { name }
    }
}

impl Cmd for WriteLocalName {
    const OPCODE: Opcode = Opcode::new(OpcodeGroup::CONTROL_BASEBAND, 0x0013);
    type Params = [u8; 248];

    fn params(&self) -> &Self::Params {
        &self.name
    }
}

impl WriteHci for WriteLocalName {
    #[inline(always)]
    fn size(&self) -> usize {
        3 + 248 // header + params
    }

    fn write_hci<W: embedded_io::Write>(&self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&self.header())?;
        writer.write_all(&self.name)
    }

    async fn write_hci_async<W: embedded_io_async::Write>(
        &self,
        mut writer: W,
    ) -> Result<(), W::Error> {
        writer.write_all(&self.header()).await?;
        writer.write_all(&self.name).await
    }
}

impl SyncCmd for WriteLocalName {
    type Return = ();
    type Handle = ();
    type ReturnBuf = [u8; 1];

    fn param_handle(&self) {}

    fn return_handle(_data: &[u8]) -> Result<Self::Handle, bt_hci::FromHciBytesError> {
        Ok(())
    }
}

/// Write Class of Device command (OGF=3, OCF=0x24)
///
/// Sets the Class of Device for the local device.
#[derive(Debug, Clone, Copy)]
pub struct WriteClassOfDevice {
    /// Class of Device (3 bytes, little-endian)
    pub class: [u8; 3],
}

impl WriteClassOfDevice {
    /// Create with Audio class: 0x240404
    /// - Major Service Class: Audio (bit 21)
    /// - Major Device Class: Audio/Video (0x04)
    /// - Minor Device Class: Portable Audio (0x04)
    pub fn audio() -> Self {
        Self {
            class: [0x04, 0x04, 0x24],
        }
    }

    pub fn new(class: [u8; 3]) -> Self {
        Self { class }
    }
}

impl Cmd for WriteClassOfDevice {
    const OPCODE: Opcode = Opcode::new(OpcodeGroup::CONTROL_BASEBAND, 0x0024);
    type Params = [u8; 3];

    fn params(&self) -> &Self::Params {
        &self.class
    }
}

impl WriteHci for WriteClassOfDevice {
    #[inline(always)]
    fn size(&self) -> usize {
        3 + 3 // header + params
    }

    fn write_hci<W: embedded_io::Write>(&self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&self.header())?;
        writer.write_all(&self.class)
    }

    async fn write_hci_async<W: embedded_io_async::Write>(
        &self,
        mut writer: W,
    ) -> Result<(), W::Error> {
        writer.write_all(&self.header()).await?;
        writer.write_all(&self.class).await
    }
}

impl SyncCmd for WriteClassOfDevice {
    type Return = ();
    type Handle = ();
    type ReturnBuf = [u8; 1];

    fn param_handle(&self) {}

    fn return_handle(_data: &[u8]) -> Result<Self::Handle, bt_hci::FromHciBytesError> {
        Ok(())
    }
}

/// Write Scan Enable command (OGF=3, OCF=0x1A)
///
/// Controls whether the device is discoverable and/or connectable.
#[derive(Debug, Clone, Copy)]
pub struct WriteScanEnable {
    /// Scan enable value
    /// 0x00 = No Scans
    /// 0x01 = Inquiry Scan only
    /// 0x02 = Page Scan only
    /// 0x03 = Inquiry + Page Scan (discoverable + connectable)
    pub scan_enable: u8,
}

impl WriteScanEnable {
    /// Disable all scans
    pub fn disabled() -> Self {
        Self { scan_enable: 0x00 }
    }

    /// Inquiry scan only (discoverable but not connectable)
    pub fn inquiry_only() -> Self {
        Self { scan_enable: 0x01 }
    }

    /// Page scan only (connectable but not discoverable)
    pub fn page_only() -> Self {
        Self { scan_enable: 0x02 }
    }

    /// Both inquiry and page scan (discoverable + connectable)
    pub fn both() -> Self {
        Self { scan_enable: 0x03 }
    }

    pub fn new(scan_enable: u8) -> Self {
        Self { scan_enable }
    }
}

impl Cmd for WriteScanEnable {
    const OPCODE: Opcode = Opcode::new(OpcodeGroup::CONTROL_BASEBAND, 0x001A);
    type Params = u8;

    fn params(&self) -> &Self::Params {
        &self.scan_enable
    }
}

impl WriteHci for WriteScanEnable {
    #[inline(always)]
    fn size(&self) -> usize {
        3 + 1 // header + params
    }

    fn write_hci<W: embedded_io::Write>(&self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&self.header())?;
        writer.write_all(&[self.scan_enable])
    }

    async fn write_hci_async<W: embedded_io_async::Write>(
        &self,
        mut writer: W,
    ) -> Result<(), W::Error> {
        writer.write_all(&self.header()).await?;
        writer.write_all(&[self.scan_enable]).await
    }
}

impl SyncCmd for WriteScanEnable {
    type Return = ();
    type Handle = ();
    type ReturnBuf = [u8; 1];

    fn param_handle(&self) {}

    fn return_handle(_data: &[u8]) -> Result<Self::Handle, bt_hci::FromHciBytesError> {
        Ok(())
    }
}
