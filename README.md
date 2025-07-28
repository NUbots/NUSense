# NUSense

[![Rust](https://img.shields.io/badge/rust-1.88+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Embedded robotics platform built on STM32H753 for advanced servo control, sensor fusion, and real-time communication.

## Features

- **6 RS485 channels** for Dynamixel servo communication
- **IMU integration** for 6-axis motion sensing
- **USB CDC ACM** virtual serial interface with DMA
- **Power monitoring** with battery voltage/current sensing
- **Fan control** with thermal management
- **GPIO support** for buttons and digital I/O

## Technical Specifications

- **MCU**: STM32H753VI (Cortex-M7, 480MHz, 2MB Flash, 1MB RAM)
- **USB**: High-speed USB 2.0 with ULPI PHY
- **Communication**: 6 independent RS485 channels

## Code Architecture

Built with Embassy async framework:

- `src/peripherals/` - Hardware abstraction layer
  - `system.rs` - Clock and system initialization
  - `usb_system.rs` - USB device management
  - `acm.rs` - CDC ACM packet-based interface
- `src/apps/` - Application layer
  - `echo_app.rs` - USB communication test

## Quick Start

### Setup

```bash
# Install Rust target and tools
rustup target add thumbv7em-none-eabi
cargo install probe-rs --features cli

# Build and flash
cargo run
```

### Communication

Connect via USB - device appears as virtual serial port:

```bash
# Linux/macOS
screen /dev/ttyACM0 115200

# Test with echo app - type messages to see them echoed back
```

## Development

### Adding Features

1. Hardware drivers → `src/peripherals/`
2. Applications → `src/apps/`
3. Initialize in `main.rs`

### Debugging

Use VS Code with the probe-rs extension for integrated debugging and real-time logging.

## License

MIT License - see [LICENSE](LICENSE) file.

Built with [Embassy](https://embassy.dev/) • Developed by [NUbots](https://nubots.net/)
