//! Dynamixel CRC demonstration application.
//!
//! This application demonstrates the usage of the CRC peripheral for calculating
//! Dynamixel 2.0 protocol CRCs. It shows how multiple tasks can safely share
//! the CRC peripheral using async/await.

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
    /// * `crc_processor` - The shared CRC processor instance
    pub fn new(crc_processor: CrcProcessor<'d>) -> Self {
        Self { crc_processor }
    }

    /// Run the CRC demonstration
    ///
    /// This demonstrates various CRC operations including:
    /// - Basic CRC calculation
    /// - Packet verification
    /// - CRC appending
    /// - Concurrent access simulation
    pub async fn run(&mut self) -> ! {
        info!("Starting CRC Demo Application");

        // Demonstrate basic CRC calculation with known test vectors
        self.test_known_vectors().await;

        // Demonstrate packet building and verification
        self.test_packet_operations().await;

        // Simulate concurrent access
        self.test_concurrent_access().await;

        // Run periodic CRC operations
        let mut counter = 0u32;
        loop {
            Timer::after(Duration::from_secs(5)).await;
            counter += 1;

            info!("CRC Demo cycle {}", counter);

            // Create a test Dynamixel packet (Read instruction) without CRC
            let test_packet = [
                0xFF, 0xFF, 0xFD, 0x00, // Header + Reserved
                0x01, // ID
                0x07, 0x00, // Length (7 bytes)
                0x02, // Read instruction
                0x84, 0x00, // Address (132 = Present Position)
                0x04, 0x00, // Data length (4 bytes)
            ];

            // Calculate CRC using hardware peripheral
            let crc = self.crc_processor.calculate_crc(&test_packet).await;

            // Create complete packet with CRC
            let mut complete_packet = [0u8; 14];
            complete_packet[..12].copy_from_slice(&test_packet);
            complete_packet[12] = crc[0]; // CRC_L
            complete_packet[13] = crc[1]; // CRC_H

            info!(
                "Generated packet: [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
                complete_packet[0], complete_packet[1], complete_packet[2], complete_packet[3],
                complete_packet[4], complete_packet[5], complete_packet[6], complete_packet[7],
                complete_packet[8], complete_packet[9], complete_packet[10], complete_packet[11],
                complete_packet[12], complete_packet[13]
            );

            // Verify by recalculating CRC
            let verify_crc = self.crc_processor.calculate_crc(&complete_packet[..12]).await;
            let is_valid = verify_crc[0] == complete_packet[12] && verify_crc[1] == complete_packet[13];
            info!("Packet CRC verification: {}", if is_valid { "PASS" } else { "FAIL" });

            if !is_valid {
                warn!("CRC verification failed!");
            }
        }
    }

    /// Test CRC calculation against known test vectors from Dynamixel documentation
    async fn test_known_vectors(&self) {
        info!("Testing hardware CRC calculation against known vectors...");

        // Test vector 1: Read instruction packet
        let test_packet_1 = [0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x07, 0x00, 0x02, 0x00, 0x00, 0x02, 0x00];
        let expected_crc_1 = [0x1D, 0x15];

        let calculated_crc_1 = self.crc_processor.calculate_crc(&test_packet_1).await;

        info!(
            "Test 1 - Hardware CRC: [{:02X}, {:02X}], Expected: [{:02X}, {:02X}]",
            calculated_crc_1[0], calculated_crc_1[1], expected_crc_1[0], expected_crc_1[1]
        );

        // Test vector 2: Ping instruction packet
        let test_packet_2 = [0xFF, 0xFF, 0xFD, 0x00, 0x01, 0x03, 0x00, 0x01];
        let expected_crc_2 = [0x19, 0x4E];

        let calculated_crc_2 = self.crc_processor.calculate_crc(&test_packet_2).await;

        info!(
            "Test 2 - Hardware CRC: [{:02X}, {:02X}], Expected: [{:02X}, {:02X}]",
            calculated_crc_2[0], calculated_crc_2[1], expected_crc_2[0], expected_crc_2[1]
        );

        // Verify results
        if calculated_crc_1 == expected_crc_1 && calculated_crc_2 == expected_crc_2 {
            info!("✓ All hardware CRC test vectors PASSED");
        } else {
            warn!("✗ Some hardware CRC test vectors FAILED");
        }
    }

    /// Test packet building operations with manual CRC handling
    async fn test_packet_operations(&self) {
        info!("Testing packet CRC calculation...");

        // Build a Write instruction packet (without CRC)
        let write_packet = [
            0xFF, 0xFF, 0xFD, 0x00, // Header + Reserved
            0x01, // ID
            0x09, 0x00, // Length (9 bytes)
            0x03, // Write instruction
            0x74, 0x00, // Address (116 = Goal Position)
            0x00, 0x02, 0x00, 0x00, // Data (512 in little-endian)
        ];

        // Calculate CRC for the packet
        let crc = self.crc_processor.calculate_crc(&write_packet).await;
        info!("Write packet CRC: [{:02X}, {:02X}]", crc[0], crc[1]);

        // Create complete packet with CRC appended
        let mut complete_packet = [0u8; 16];
        complete_packet[..14].copy_from_slice(&write_packet);
        complete_packet[14] = crc[0]; // CRC_L
        complete_packet[15] = crc[1]; // CRC_H

        info!("Complete write packet: [{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}]",
            complete_packet[0], complete_packet[1], complete_packet[2], complete_packet[3],
            complete_packet[4], complete_packet[5], complete_packet[6], complete_packet[7],
            complete_packet[8], complete_packet[9], complete_packet[10], complete_packet[11],
            complete_packet[12], complete_packet[13], complete_packet[14], complete_packet[15]
        );

        // Verify by recalculating CRC
        let verify_crc = self.crc_processor.calculate_crc(&complete_packet[..14]).await;
        let is_valid = verify_crc[0] == complete_packet[14] && verify_crc[1] == complete_packet[15];
        info!("Write packet verification: {}", if is_valid { "PASS" } else { "FAIL" });

        // Test with corrupted data
        let mut corrupted_packet = write_packet;
        corrupted_packet[8] = 0xAA; // Corrupt the address field

        let corrupted_crc = self.crc_processor.calculate_crc(&corrupted_packet).await;
        let is_different = corrupted_crc != crc;
        info!("Corrupted packet has different CRC: {} (should be true)", is_different);
    }

    /// Simulate concurrent access to the CRC peripheral
    async fn test_concurrent_access(&self) {
        info!("Testing concurrent CRC access simulation...");

        // Simulate multiple "tasks" accessing CRC
        // (In a real application, these would be separate async tasks)

        for i in 0..5 {
            // Create different test packets
            let test_data = [0xFF, 0xFF, 0xFD, 0x00, i, 0x03, 0x00, 0x01];

            let crc = self.crc_processor.calculate_crc(&test_data).await;
            info!(
                "Concurrent test {}: data[4] = {}, CRC = [{:02X}, {:02X}]",
                i, test_data[4], crc[0], crc[1]
            );

            // Small delay to simulate realistic usage
            Timer::after(Duration::from_millis(10)).await;
        }

        info!("Concurrent access test completed");
    }
}
