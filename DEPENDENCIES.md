# Dependencies

## Rust Toolchain Requirements

- **Rust Version**: 1.75 or later
- **Target**: `thumbv8m.main-none-eabihf` (RP2350 ARM Cortex-M33)

### Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the embedded ARM target
rustup target add thumbv8m.main-none-eabihf

# Install probe-rs for flashing
cargo install probe-rs-tools
```

## External Dependencies

### Embassy Ecosystem (Async Embedded Framework)

| Crate | Version | Purpose |
|-------|---------|---------|
| embassy-executor | 0.7 | Async task executor for Cortex-M |
| embassy-rp | 0.8 | RP2350 HAL with async support |
| embassy-sync | 0.7 | Synchronization primitives |
| embassy-time | 0.4 | Time and delay abstractions |
| embassy-usb | 0.4 | USB device stack |
| embassy-futures | 0.1 | Async utilities |

### CYW43439 Driver

| Crate | Version | Purpose |
|-------|---------|---------|
| cyw43 | 0.6 | WiFi/Bluetooth driver for CYW43439 |
| cyw43-pio | 0.6 | PIO-based SPI for CYW43439 |

### Bluetooth

| Crate | Version | Purpose |
|-------|---------|---------|
| bt-hci | 0.2 | Bluetooth HCI type definitions |

### Embedded Core

| Crate | Version | Purpose |
|-------|---------|---------|
| cortex-m | 0.7 | Low-level Cortex-M access |
| cortex-m-rt | 0.7 | Runtime/startup code |
| embedded-hal | 1.0 | Hardware abstraction traits |
| embedded-hal-async | 1.0 | Async HAL traits |
| embedded-io-async | 0.6 | Async I/O traits |

### Utilities

| Crate | Version | Purpose |
|-------|---------|---------|
| heapless | 0.8 | Stack-allocated data structures |
| defmt | 0.3 | Efficient embedded logging |
| defmt-rtt | 0.4 | RTT transport for defmt |
| panic-probe | 0.3 | Panic handler for probe-rs |
| static_cell | 2.1 | Static initialization |
| portable-atomic | 1.10 | Portable atomic operations |
| critical-section | 1.2 | Critical section abstraction |

### Development/Testing

| Crate | Version | Purpose |
|-------|---------|---------|
| proptest | 1.5 | Property-based testing (host only) |

## Firmware Requirements

The CYW43439 chip requires firmware blobs to operate. These are typically included via the `cyw43-firmware` crate or downloaded from Infineon.

Required files:
- `43439A0.bin` - WiFi/BT firmware
- `43439A0_clm.bin` - Regulatory/country data

## Build Tools

### probe-rs (Recommended)

```bash
# Install probe-rs
cargo install probe-rs-tools

# Verify installation
probe-rs --version
```

### Alternative: elf2uf2-rs

For flashing via USB bootloader (hold BOOTSEL while connecting):

```bash
cargo install elf2uf2-rs

# Build and create UF2
cargo build --release
elf2uf2-rs target/thumbv8m.main-none-eabihf/release/a2dp-source a2dp-source.uf2
```

## Memory Requirements

| Resource | Allocated | Purpose |
|----------|-----------|---------|
| Flash | ~200 KB | Code + constants |
| RAM | ~80 KB | Buffers + state |
| Stack (Core 0) | 16 KB | Bluetooth stack |
| Stack (Core 1) | 8 KB | Audio processing |

RP2350 has 520 KB SRAM and 4 MB Flash available.

## Version Compatibility Notes

- Embassy crates should be kept in sync (all 0.7.x/0.8.x together)
- CYW43 crates require matching versions with embassy-rp
- bt-hci 0.2 is used for compatibility with current cyw43 driver
