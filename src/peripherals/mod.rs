//! Hardware abstraction layer for the NUSense robotics platform.
//!
//! This module provides abstractions for all the hardware components of the NUSense platform.
//!
//! The abstractions hide the complexity of the STM32H753 HAL and Embassy framework
//! from the application layer, providing clean, type-safe interfaces.

/// CDC ACM (virtual serial port) implementation
pub mod acm;
/// System initialization and clock configuration
pub mod system;
/// USB system abstraction
pub mod usb_system;

// Re-export commonly used types for convenience
pub use acm::{AcmConnection, AcmState, Disconnected};
pub use system::init_system;
pub use usb_system::{UsbBuffers, UsbSystem, MAX_PACKET_SIZE};
