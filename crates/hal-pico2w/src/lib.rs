//! Hardware Abstraction Layer for Raspberry Pi Pico 2 W
//!
//! Provides hardware initialization and drivers for:
//! - CYW43439 Bluetooth HCI transport
//! - USB peripheral
//! - Clock configuration
//! - Status LED

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bluetooth;
pub mod clocks;
pub mod led;


/// Pico 2 W hardware configuration
pub struct Pico2WHardware {
    // Peripherals will be initialized here
}

impl Pico2WHardware {
    /// Initialize all hardware
    pub fn init() -> Self {
        Self {}
    }
}

/// GPIO pin assignments for Pico 2 W
pub mod pins {
    /// CYW43439 power pin
    pub const CYW43_PWR: u8 = 23;
    /// CYW43439 data/command pin (directly connected)
    pub const CYW43_CS: u8 = 25;
    /// LED pin (directly connected to CYW43439, directly controlled)
    pub const LED: u8 = 0; // CYW43 GPIO 0

    /// Default SPI pins for external peripherals
    pub const SPI0_SCK: u8 = 18;
    pub const SPI0_MOSI: u8 = 19;
    pub const SPI0_MISO: u8 = 16;

    /// I2C pins
    pub const I2C0_SDA: u8 = 4;
    pub const I2C0_SCL: u8 = 5;
}

/// System clock frequencies
pub mod clocks_const {
    /// System clock (default)
    pub const SYS_CLK_HZ: u32 = 150_000_000;
    /// USB clock (must be 48MHz)
    pub const USB_CLK_HZ: u32 = 48_000_000;
    /// PLL reference
    pub const XOSC_HZ: u32 = 12_000_000;
}
