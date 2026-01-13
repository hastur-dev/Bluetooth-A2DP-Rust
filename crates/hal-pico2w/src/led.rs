//! Status LED control
//!
//! The Pico 2 W LED is connected to the CYW43439 GPIO 0.

/// LED state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LedState {
    #[default]
    Off,
    On,
    Blinking,
}

/// LED pattern for status indication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedPattern {
    /// Solid off
    Off,
    /// Solid on
    On,
    /// Slow blink (idle/discoverable)
    SlowBlink,
    /// Fast blink (connecting)
    FastBlink,
    /// Double blink (streaming)
    DoubleBlink,
    /// Error pattern
    Error,
}

/// LED controller
pub struct Led {
    state: LedState,
    pattern: LedPattern,
}

impl Led {
    /// Create a new LED controller
    pub const fn new() -> Self {
        Self {
            state: LedState::Off,
            pattern: LedPattern::Off,
        }
    }

    /// Set the LED pattern
    pub fn set_pattern(&mut self, pattern: LedPattern) {
        self.pattern = pattern;
    }

    /// Get current pattern
    pub fn pattern(&self) -> LedPattern {
        self.pattern
    }

    /// Update LED state (call periodically)
    pub fn update(&mut self, _tick_ms: u32) {
        // TODO: Implement pattern timing
        match self.pattern {
            LedPattern::Off => self.state = LedState::Off,
            LedPattern::On => self.state = LedState::On,
            _ => self.state = LedState::Blinking,
        }
    }

    /// Get whether LED should be on right now
    pub fn is_on(&self) -> bool {
        self.state == LedState::On
    }
}

impl Default for Led {
    fn default() -> Self {
        Self::new()
    }
}
