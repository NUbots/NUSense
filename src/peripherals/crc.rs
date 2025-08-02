//! Hardware CRC peripheral for Dynamixel 2.0 protocol.
//!
//! This module provides hardware CRC calculation using the STM32H753's CRC peripheral
//! for efficient Dynamixel packet CRC computation.

use embassy_stm32::{
    crc::{Config, Crc, InputReverseConfig, PolySize},
    peripherals::CRC,
    Peri,
};

/// Peripheral collection for CRC
pub struct CrcPeripherals<'d> {
    pub crc: Peri<'d, CRC>,
}

/// Macro to claim peripherals for CRC
#[macro_export]
macro_rules! claim_crc {
    ($peripherals:expr) => {{
        $crate::peripherals::crc::CrcPeripherals {
            crc: $peripherals.CRC.reborrow(),
        }
    }};
}

/// Hardware CRC processor for Dynamixel 2.0 protocol packets
///
/// This peripheral uses the STM32H753's hardware CRC peripheral to efficiently
/// calculate CRC-16 (IBM/ANSI) as required by Dynamixel 2.0.
///
/// The CRC calculation uses:
/// - Polynomial: x^16 + x^15 + x^2 + 1 (0x8005)
/// - Initial value: 0x0000
/// - Input reflection: disabled
/// - Output reflection: disabled
///
/// # Performance
///
/// The STM32H753's hardware CRC peripheral processes data very efficiently,
/// typically completing CRC calculations in just a few CPU cycles per byte.
/// This is much faster than software CRC implementations and provides
/// consistent performance regardless of data patterns.
///
/// # Thread Safety
///
/// Since the CRC calculation is a synchronous atomic operation that completes
/// without yielding, no mutex protection is needed when using a single executor.
pub struct CrcProcessor<'d> {
    crc: Crc<'d>,
}

impl<'d> CrcProcessor<'d> {
    /// Create a new hardware CRC processor for Dynamixel 2.0 protocol
    ///
    /// # Arguments
    /// * `peripherals` - CrcPeripherals struct containing CRC peripheral
    ///
    /// # Returns
    /// Configured CRC processor ready for Dynamixel packet processing
    pub fn new(peripherals: CrcPeripherals<'d>) -> Self {
        // Set up the config for CRC peripheral - Dynamixel 2.0 uses CRC-16 IBM/ANSI
        let config = Config::new(
            InputReverseConfig::None, // No input reflection
            false,                    // No output reflection
            PolySize::Width16,        // 16-bit polynomial
            0x0000,                   // Initial value: 0x0000
            0x8005,                   // Polynomial: 0x8005 (CRC-16 IBM/ANSI)
        )
        .expect("Invalid CRC configuration");

        Self {
            crc: Crc::new(peripherals.crc, config),
        }
    }

    /// Calculate CRC-16 for a Dynamixel 2.0 protocol packet using Embassy's register access
    ///
    /// This method uses the STM32H753's hardware CRC peripheral to efficiently
    /// calculate the CRC for packet data excluding the CRC field itself.
    ///
    /// # Arguments
    /// * `data` - Packet data buffer (excluding the 2-byte CRC field)
    ///
    /// # Returns
    /// 16-bit CRC value in little-endian format (low byte, high byte)
    ///
    /// # Example
    /// ```rust,ignore
    /// // Calculate CRC for instruction packet
    /// let packet = [0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x07, 0x00, 0x02, 0x00, 0x00, 0x02, 0x00];
    /// let crc = crc_processor.calculate_crc(&packet);
    /// // crc will be [0x21, 0x51] for this example packet
    /// ```
    pub fn calculate_crc(&mut self, data: &[u8]) -> [u8; 2] {
        // Reset CRC to initial state
        self.crc.reset();

        // Use Embassy's hardware CRC calculation
        let crc_result_32 = self.crc.feed_bytes(data);

        // For 16-bit CRC, the result is in the lower 16 bits
        let crc_result = (crc_result_32 & 0xFFFF) as u16;

        // Return as little-endian bytes [CRC_L, CRC_H] as required by Dynamixel 2.0
        [
            (crc_result & 0xFF) as u8,        // Low byte
            ((crc_result >> 8) & 0xFF) as u8, // High byte
        ]
    }
}
