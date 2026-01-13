# Bluetooth A2DP Source for Raspberry Pi Pico 2 W

A pure Rust implementation of a Bluetooth A2DP (Advanced Audio Distribution Profile) audio streaming device for the Raspberry Pi Pico 2 W.

## Features

- **USB Audio Input**: Receives audio from a connected computer as a USB Audio Class 2.0 device
- **Bluetooth A2DP Streaming**: Streams audio to Bluetooth headphones, speakers, or phones
- **Pure Rust**: No C dependencies, fully implemented in Rust with no_std support
- **Dual-Core Architecture**: Core 0 handles Bluetooth, Core 1 handles audio processing
- **SBC Codec**: Implements the mandatory SBC encoder for A2DP compatibility

## Architecture

```
Computer -> [USB Audio] -> Pico 2 W -> [Bluetooth A2DP] -> Phone/Headphones
                          |-- Core 0: Bluetooth Stack (HCI, L2CAP, AVDTP, A2DP)
                          |-- Core 1: USB Audio + SBC Encoder
```

## Crate Structure

| Crate | Description |
|-------|-------------|
| `a2dp-app` | Main application, orchestration, state machine |
| `sbc-encoder` | Pure Rust SBC audio codec (no_std) |
| `usb-audio` | USB Audio Class 2.0 device implementation |
| `bt-classic` | Bluetooth Classic stack (L2CAP, SDP, AVDTP, A2DP) |
| `hal-pico2w` | Hardware abstraction for CYW43439 chip |
| `audio-pipeline` | Lock-free ring buffers, audio format conversion |

## Prerequisites

### Hardware
- Raspberry Pi Pico 2 W (RP2350 + CYW43439)
- USB cable for connection to computer and flashing

### Software
- Rust toolchain (1.75+)
- Target: `thumbv8m.main-none-eabihf`
- probe-rs for flashing

## Building

```bash
# Install the embedded target
rustup target add thumbv8m.main-none-eabihf

# Build the project
cargo build --release

# Flash to Pico 2 W (with probe-rs)
cargo run --release
```

## Testing

Host-side tests can be run for the SBC encoder and audio pipeline:

```bash
# Run SBC encoder tests (Windows)
cargo test -p sbc-encoder --features std --target x86_64-pc-windows-msvc

# Run audio pipeline tests (Windows)
cargo test -p audio-pipeline --target x86_64-pc-windows-msvc

# Linux/macOS
cargo test -p sbc-encoder --features std
cargo test -p audio-pipeline
```

## Configuration

The device can be configured in `crates/a2dp-app/src/config.rs`:

```rust
pub struct AppConfig {
    pub device_name: &'static str,    // Bluetooth device name
    pub default_bitpool: u8,          // SBC quality (2-250, default 53)
    pub auto_reconnect: bool,         // Auto-reconnect on disconnect
    pub audio_buffer_ms: u32,         // Audio buffer size (20-500ms)
}
```

## Connection State Machine

```
Disconnected -> Discoverable -> Connecting -> Connected -> Configuring -> Open -> Streaming
                                    ^                                        |
                                    +-------------- Suspended <--------------+
```

## Bluetooth Protocol Stack

```
+-------------------------------------+
|        A2DP Source Profile          |
+-------------------------------------+
|    AVDTP (Audio/Video Transport)    |
+-------------------------------------+
|  SDP (Service Discovery Protocol)   |
+-------------------------------------+
|  L2CAP (Logical Link Adaptation)    |
+-------------------------------------+
|    HCI (Host Controller Interface)  |
+-------------------------------------+
|         CYW43439 Controller         |
+-------------------------------------+
```

## Current Status

This is a work-in-progress implementation. Current status:

- [x] Project structure and build system
- [x] SBC encoder (37/37 tests passing)
- [x] Audio pipeline ring buffer (8/8 tests passing)
- [x] USB Audio Class 2.0 descriptors
- [x] Bluetooth protocol type definitions (HCI, L2CAP, SDP, AVDTP, A2DP)
- [x] Connection state machine
- [ ] CYW43439 Bluetooth HCI integration
- [ ] Full AVDTP signaling implementation
- [ ] End-to-end audio streaming

## License

MIT License - see LICENSE file
