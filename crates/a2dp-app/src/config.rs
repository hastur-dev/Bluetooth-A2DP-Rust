//! Application configuration

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Bluetooth device name
    pub device_name: &'static str,
    /// Default SBC bitpool value
    pub default_bitpool: u8,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Audio buffer size in milliseconds
    pub audio_buffer_ms: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: "Pico A2DP Audio",
            default_bitpool: 53,
            auto_reconnect: true,
            audio_buffer_ms: 100,
        }
    }
}

impl AppConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.device_name.is_empty() {
            return Err("Device name cannot be empty");
        }

        if self.default_bitpool < 2 || self.default_bitpool > 250 {
            return Err("Bitpool must be between 2 and 250");
        }

        if self.audio_buffer_ms < 20 || self.audio_buffer_ms > 500 {
            return Err("Audio buffer must be between 20 and 500 ms");
        }

        Ok(())
    }
}
