//! Connection state machine for A2DP Source

use bt_classic::a2dp::A2dpState;
use bt_classic::BdAddr;

/// Events that trigger state transitions
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Event {
    /// User requested to become discoverable
    MakeDiscoverable,
    /// User requested to connect to a device
    Connect(BdAddr),
    /// User requested to start streaming
    StartStream,
    /// User requested to pause streaming
    PauseStream,
    /// User requested to disconnect
    Disconnect,

    // Internal events
    /// ACL connection established
    ConnectionComplete { handle: u16 },
    /// ACL connection failed or lost
    ConnectionFailed,
    /// L2CAP channel opened
    L2capConnected,
    /// AVDTP configuration complete
    AvdtpConfigured,
    /// Stream opened
    StreamOpened,
    /// Stream started
    StreamStarted,
    /// Stream suspended
    StreamSuspended,
    /// Disconnection complete
    Disconnected,
    /// Timeout occurred
    Timeout,
    /// Error occurred
    Error(u8),
}

/// Actions to perform after state transition
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Action {
    /// No action needed
    None,
    /// Enable inquiry scan (become discoverable)
    EnableDiscovery,
    /// Initiate connection to device
    InitiateConnection(BdAddr),
    /// Open L2CAP channel for AVDTP
    OpenL2cap,
    /// Send AVDTP DISCOVER
    SendAvdtpDiscover,
    /// Send AVDTP GET_CAPABILITIES
    SendGetCapabilities { seid: u8 },
    /// Send AVDTP SET_CONFIGURATION
    SendSetConfiguration,
    /// Send AVDTP OPEN
    SendOpen,
    /// Send AVDTP START
    SendStart,
    /// Send AVDTP SUSPEND
    SendSuspend,
    /// Send AVDTP CLOSE
    SendClose,
    /// Initiate disconnection
    InitiateDisconnect,
    /// Update LED pattern
    UpdateLed,
}

/// State machine for A2DP connection management
pub struct StateMachine {
    state: A2dpState,
    remote_addr: Option<BdAddr>,
    acl_handle: Option<u16>,
    remote_seid: Option<u8>,
}

impl StateMachine {
    /// Create a new state machine
    pub const fn new() -> Self {
        Self {
            state: A2dpState::Disconnected,
            remote_addr: None,
            acl_handle: None,
            remote_seid: None,
        }
    }

    /// Get current state
    pub fn state(&self) -> A2dpState {
        self.state
    }

    /// Get remote device address if connected
    pub fn remote_addr(&self) -> Option<BdAddr> {
        self.remote_addr
    }

    /// Process an event and return the action to take
    pub fn process(&mut self, event: Event) -> Action {
        match (&self.state, event) {
            // From Disconnected
            (A2dpState::Disconnected, Event::MakeDiscoverable) => {
                self.state = A2dpState::Discoverable;
                Action::EnableDiscovery
            }
            (A2dpState::Disconnected, Event::Connect(addr)) => {
                self.remote_addr = Some(addr);
                self.state = A2dpState::Connecting;
                Action::InitiateConnection(addr)
            }

            // From Discoverable
            (A2dpState::Discoverable, Event::ConnectionComplete { handle }) => {
                self.acl_handle = Some(handle);
                self.state = A2dpState::Connected;
                Action::OpenL2cap
            }
            (A2dpState::Discoverable, Event::Disconnect) => {
                self.state = A2dpState::Disconnected;
                Action::None
            }

            // From Connecting
            (A2dpState::Connecting, Event::ConnectionComplete { handle }) => {
                self.acl_handle = Some(handle);
                self.state = A2dpState::Connected;
                Action::OpenL2cap
            }
            (A2dpState::Connecting, Event::ConnectionFailed) => {
                self.state = A2dpState::Disconnected;
                self.remote_addr = None;
                Action::UpdateLed
            }
            (A2dpState::Connecting, Event::Timeout) => {
                self.state = A2dpState::Disconnected;
                self.remote_addr = None;
                Action::UpdateLed
            }

            // From Connected
            (A2dpState::Connected, Event::L2capConnected) => {
                self.state = A2dpState::Configuring;
                Action::SendAvdtpDiscover
            }
            (A2dpState::Connected, Event::Disconnect) => {
                self.state = A2dpState::Disconnecting;
                Action::InitiateDisconnect
            }

            // From Configuring
            (A2dpState::Configuring, Event::AvdtpConfigured) => {
                self.state = A2dpState::Open;
                Action::UpdateLed
            }
            (A2dpState::Configuring, Event::Error(_)) => {
                self.state = A2dpState::Disconnecting;
                Action::InitiateDisconnect
            }

            // From Open
            (A2dpState::Open, Event::StartStream) => {
                Action::SendStart
            }
            (A2dpState::Open, Event::StreamStarted) => {
                self.state = A2dpState::Streaming;
                Action::UpdateLed
            }
            (A2dpState::Open, Event::Disconnect) => {
                self.state = A2dpState::Disconnecting;
                Action::SendClose
            }

            // From Streaming
            (A2dpState::Streaming, Event::PauseStream) => {
                Action::SendSuspend
            }
            (A2dpState::Streaming, Event::StreamSuspended) => {
                self.state = A2dpState::Suspended;
                Action::UpdateLed
            }
            (A2dpState::Streaming, Event::Disconnect) => {
                self.state = A2dpState::Disconnecting;
                Action::SendClose
            }

            // From Suspended
            (A2dpState::Suspended, Event::StartStream) => {
                Action::SendStart
            }
            (A2dpState::Suspended, Event::StreamStarted) => {
                self.state = A2dpState::Streaming;
                Action::UpdateLed
            }
            (A2dpState::Suspended, Event::Disconnect) => {
                self.state = A2dpState::Disconnecting;
                Action::SendClose
            }

            // From Disconnecting
            (A2dpState::Disconnecting, Event::Disconnected) => {
                self.state = A2dpState::Disconnected;
                self.remote_addr = None;
                self.acl_handle = None;
                self.remote_seid = None;
                Action::UpdateLed
            }

            // Global error handling
            (_, Event::ConnectionFailed) | (_, Event::Error(_)) => {
                self.state = A2dpState::Disconnected;
                self.remote_addr = None;
                self.acl_handle = None;
                Action::UpdateLed
            }

            // Unhandled - no action
            _ => Action::None,
        }
    }

    /// Reset state machine
    pub fn reset(&mut self) {
        self.state = A2dpState::Disconnected;
        self.remote_addr = None;
        self.acl_handle = None;
        self.remote_seid = None;
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = StateMachine::new();
        assert_eq!(sm.state(), A2dpState::Disconnected);
    }

    #[test]
    fn test_make_discoverable() {
        let mut sm = StateMachine::new();
        let action = sm.process(Event::MakeDiscoverable);

        assert_eq!(sm.state(), A2dpState::Discoverable);
        assert!(matches!(action, Action::EnableDiscovery));
    }

    #[test]
    fn test_connection_flow() {
        let mut sm = StateMachine::new();
        let addr = BdAddr::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

        // Connect
        sm.process(Event::Connect(addr));
        assert_eq!(sm.state(), A2dpState::Connecting);

        // Connection complete
        sm.process(Event::ConnectionComplete { handle: 0x0001 });
        assert_eq!(sm.state(), A2dpState::Connected);

        // L2CAP connected
        sm.process(Event::L2capConnected);
        assert_eq!(sm.state(), A2dpState::Configuring);

        // Configuration complete
        sm.process(Event::AvdtpConfigured);
        assert_eq!(sm.state(), A2dpState::Open);
    }
}
