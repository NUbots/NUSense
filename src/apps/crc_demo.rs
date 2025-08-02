//! Dynamixel CRC demonstration application.
//!
//! This application demonstrates the usage of the hardware CRC peripheral for calculating
//! Dynamixel 2.0 protocol CRCs and compares it against a software implementation.

use crate::peripherals::crc::CrcProcessor;
use defmt::{info, warn};
use embassy_time::{Duration, Timer};

/// Demonstration application for CRC peripheral usage
pub struct CrcDemoApp<'d> {
    crc_processor: CrcProcessor<'d>,
}

impl<'d> CrcDemoApp<'d> {
    /// Create a new CRC demonstration application
    ///
    /// # Arguments
    /// * `crc_processor` - The CRC processor instance
    pub fn new(crc_processor: CrcProcessor<'d>) -> Self {
        Self { crc_processor }
    }

    /// Software implementation of Dynamixel 2.0 CRC-16 for comparison
    ///
    /// This implements the same CRC-16 IBM/ANSI algorithm used by Dynamixel 2.0:
    /// - Polynomial: 0x8005 (x^16 + x^15 + x^2 + 1)
    /// - Initial value: 0x0000
    /// - No input/output reflection
    fn calculate_crc_software(&self, data: &[u8]) -> [u8; 2] {
        const CRC_POLYNOMIAL: u16 = 0x8005;
        let mut crc: u16 = 0x0000;

        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ CRC_POLYNOMIAL;
                } else {
                    crc <<= 1;
                }
            }
        }

        // Return as little-endian bytes [CRC_L, CRC_H]
        [
            (crc & 0xFF) as u8,        // Low byte
            ((crc >> 8) & 0xFF) as u8, // High byte
        ]
    }

    /// Run the CRC demonstration
    ///
    /// This demonstrates CRC calculation using both hardware and software implementations
    /// and validates against known Dynamixel test vectors.
    pub async fn run(&mut self) -> ! {
        info!("Starting CRC Demo Application - Hardware vs Software Comparison");

        // Test vectors from Dynamixel 2.0 documentation
        let test_cases = [
            (
                "Read instruction",
                &[0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x07, 0x00, 0x02, 0x00, 0x00, 0x02, 0x00][..],
                [0x21, 0x51],
            ),
            (
                "Ping instruction",
                &[0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x03, 0x00, 0x01][..],
                [0x19, 0x4E],
            ),
            (
                "Write instruction",
                &[
                    0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x09, 0x00, 0x03, 0x74, 0x00, 0x00, 0x02, 0x00, 0x00,
                ][..],
                [0xCA, 0x89],
            ),
        ];

        // Test all vectors
        for (i, (name, packet, expected)) in test_cases.iter().enumerate() {
            info!("=== Test {}: {} ===", i + 1, name);

            // Calculate using hardware CRC
            let hw_crc = self.crc_processor.calculate_crc(packet);

            // Calculate using software CRC
            let sw_crc = self.calculate_crc_software(packet);

            info!("Hardware CRC: [{:02X}, {:02X}]", hw_crc[0], hw_crc[1]);
            info!("Software CRC: [{:02X}, {:02X}]", sw_crc[0], sw_crc[1]);
            info!("Expected CRC: [{:02X}, {:02X}]", expected[0], expected[1]);

            // Verify results
            let hw_correct = hw_crc == *expected;
            let sw_correct = sw_crc == *expected;
            let hw_sw_match = hw_crc == sw_crc;

            info!("Hardware correct: {}", hw_correct);
            info!("Software correct: {}", sw_correct);
            info!("Hardware/Software match: {}", hw_sw_match);

            if hw_correct && sw_correct && hw_sw_match {
                info!("✓ Test {} PASSED", i + 1);
            } else {
                warn!("✗ Test {} FAILED", i + 1);
            }
            info!("");
        }

        // Run periodic demonstrations
        let mut counter = 0u32;
        loop {
            Timer::after(Duration::from_secs(10)).await;
            counter += 1;

            info!("=== Demo Cycle {} ===", counter);

            // Create a dynamic test packet
            let test_packet = [
                0xFF,
                0xFF,
                0xFD,
                0x00,                      // Header + Reserved
                (counter % 254 + 1) as u8, // ID (1-254)
                0x07,
                0x00, // Length (7 bytes)
                0x02, // Read instruction
                0x84,
                0x00, // Address (Present Position)
                0x04,
                0x00, // Data length (4 bytes)
            ];

            // Calculate CRC with both methods
            let hw_crc = self.crc_processor.calculate_crc(&test_packet);
            let sw_crc = self.calculate_crc_software(&test_packet);

            // Create complete packet
            let mut complete_packet = [0u8; 14];
            complete_packet[..12].copy_from_slice(&test_packet);
            complete_packet[12] = hw_crc[0];
            complete_packet[13] = hw_crc[1];

            info!(
                "Packet for ID {}: [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
                test_packet[4],
                complete_packet[0], complete_packet[1], complete_packet[2], complete_packet[3],
                complete_packet[4], complete_packet[5], complete_packet[6], complete_packet[7],
                complete_packet[8], complete_packet[9], complete_packet[10], complete_packet[11],
                complete_packet[12], complete_packet[13]
            );

            info!("Hardware CRC: [{:02X}, {:02X}]", hw_crc[0], hw_crc[1]);
            info!("Software CRC: [{:02X}, {:02X}]", sw_crc[0], sw_crc[1]);
            info!("Match: {}", hw_crc == sw_crc);

            // Verify packet integrity
            let verify_crc = self.crc_processor.calculate_crc(&complete_packet[..12]);
            let is_valid = verify_crc == [complete_packet[12], complete_packet[13]];
            info!("Packet verification: {}", if is_valid { "PASS" } else { "FAIL" });

            if !is_valid {
                warn!("Packet verification failed!");
            }

            info!("");
        }
    }
}
