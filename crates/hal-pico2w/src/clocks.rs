//! Clock configuration for RP2350

/// Clock configuration
pub struct ClockConfig {
    /// System clock in Hz
    pub sys_clk: u32,
    /// USB clock in Hz (must be 48MHz for USB)
    pub usb_clk: u32,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            sys_clk: 150_000_000,
            usb_clk: 48_000_000,
        }
    }
}
