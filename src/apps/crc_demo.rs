//! Dynamixel CRC demonstration application.
//!
//! This application demonstrates the usage of the hardware CRC peripheral for calculating
//! Dynamixel 2.0 protocol CRCs and compares it against a software implementation.

use crate::peripherals::crc::CrcProcessor;
use defmt::{info, warn};
use embassy_time::{Duration, Instant, Timer};

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

    /// Run a single CRC comparison test with timing
    fn run_crc_test(&mut self, test_name: &str, data: &[u8], expected: &[u8; 2]) {
        info!("=== {} ===", test_name);

        // Run multiple iterations for accurate timing
        let iterations = 10000;

        // Time hardware CRC calculation
        let hw_start = Instant::now();
        let mut hw_crc = [0u8; 2];
        for _ in 0..iterations {
            hw_crc = self.crc_processor.calculate_crc(data);
        }
        let hw_end = Instant::now();
        let hw_avg = (hw_end.as_ticks() - hw_start.as_ticks()) as f32;

        // Time software CRC calculation
        let sw_start = Instant::now();
        let mut sw_crc = [0u8; 2];
        for _ in 0..iterations {
            sw_crc = self.calculate_crc_software(data);
        }
        let sw_end = Instant::now();
        let sw_avg = (sw_end.as_ticks() - sw_start.as_ticks()) as f32;

        // Time bitwise CRC calculation
        let bw_start = Instant::now();
        let mut bw_crc = [0u8; 2];
        for _ in 0..iterations {
            bw_crc = self.calculate_crc_bitwise(data);
        }
        let bw_end = Instant::now();
        let bw_avg = (bw_end.as_ticks() - bw_start.as_ticks()) as f32;

        // Display results
        info!("Hardware CRC: [{:02X}, {:02X}]", hw_crc[0], hw_crc[1]);
        info!(
            "Software CRC: [{:02X}, {:02X}] (avg {}x)",
            sw_crc[0],
            sw_crc[1],
            sw_avg / hw_avg
        );
        info!(
            "Bit-wise CRC: [{:02X}, {:02X}] (avg {}x)",
            bw_crc[0],
            bw_crc[1],
            bw_avg / hw_avg
        );
        info!("Expected CRC: [{:02X}, {:02X}]", expected[0], expected[1]);

        // Check correctness
        let hw_correct = hw_crc == *expected;
        let sw_correct = sw_crc == *expected;
        let bw_correct = bw_crc == *expected;
        let all_match = hw_crc == sw_crc && sw_crc == bw_crc;

        info!("All methods match: {}", all_match);
        info!("All results correct: {}", hw_correct && sw_correct && bw_correct);

        if !(hw_correct && sw_correct && bw_correct && all_match) {
            warn!("✗ Test FAILED");
        } else {
            info!("✓ Test PASSED");
        }
        info!("");
    }

    /// Software implementation of Dynamixel 2.0 CRC-16 for comparison
    ///
    /// This is the official Robotis implementation using a lookup table.
    /// Much faster than bit-by-bit calculation.
    fn calculate_crc_software(&self, data: &[u8]) -> [u8; 2] {
        // CRC table for CRC-16 IBM/ANSI
        const CRC_TABLE: [u16; 256] = [
            0x0000, 0x8005, 0x800F, 0x000A, 0x801B, 0x001E, 0x0014, 0x8011, 0x8033, 0x0036, 0x003C, 0x8039, 0x0028,
            0x802D, 0x8027, 0x0022, 0x8063, 0x0066, 0x006C, 0x8069, 0x0078, 0x807D, 0x8077, 0x0072, 0x0050, 0x8055,
            0x805F, 0x005A, 0x804B, 0x004E, 0x0044, 0x8041, 0x80C3, 0x00C6, 0x00CC, 0x80C9, 0x00D8, 0x80DD, 0x80D7,
            0x00D2, 0x00F0, 0x80F5, 0x80FF, 0x00FA, 0x80EB, 0x00EE, 0x00E4, 0x80E1, 0x00A0, 0x80A5, 0x80AF, 0x00AA,
            0x80BB, 0x00BE, 0x00B4, 0x80B1, 0x8093, 0x0096, 0x009C, 0x8099, 0x0088, 0x808D, 0x8087, 0x0082, 0x8183,
            0x0186, 0x018C, 0x8189, 0x0198, 0x819D, 0x8197, 0x0192, 0x01B0, 0x81B5, 0x81BF, 0x01BA, 0x81AB, 0x01AE,
            0x01A4, 0x81A1, 0x01E0, 0x81E5, 0x81EF, 0x01EA, 0x81FB, 0x01FE, 0x01F4, 0x81F1, 0x81D3, 0x01D6, 0x01DC,
            0x81D9, 0x01C8, 0x81CD, 0x81C7, 0x01C2, 0x0140, 0x8145, 0x814F, 0x014A, 0x815B, 0x015E, 0x0154, 0x8151,
            0x8173, 0x0176, 0x017C, 0x8179, 0x0168, 0x816D, 0x8167, 0x0162, 0x8123, 0x0126, 0x012C, 0x8129, 0x0138,
            0x813D, 0x8137, 0x0132, 0x0110, 0x8115, 0x811F, 0x011A, 0x810B, 0x010E, 0x0104, 0x8101, 0x8303, 0x0306,
            0x030C, 0x8309, 0x0318, 0x831D, 0x8317, 0x0312, 0x0330, 0x8335, 0x833F, 0x033A, 0x832B, 0x032E, 0x0324,
            0x8321, 0x0360, 0x8365, 0x836F, 0x036A, 0x837B, 0x037E, 0x0374, 0x8371, 0x8353, 0x0356, 0x035C, 0x8359,
            0x0348, 0x834D, 0x8347, 0x0342, 0x03C0, 0x83C5, 0x83CF, 0x03CA, 0x83DB, 0x03DE, 0x03D4, 0x83D1, 0x83F3,
            0x03F6, 0x03FC, 0x83F9, 0x03E8, 0x83ED, 0x83E7, 0x03E2, 0x83A3, 0x03A6, 0x03AC, 0x83A9, 0x03B8, 0x83BD,
            0x83B7, 0x03B2, 0x0390, 0x8395, 0x839F, 0x039A, 0x838B, 0x038E, 0x0384, 0x8381, 0x0280, 0x8285, 0x828F,
            0x028A, 0x829B, 0x029E, 0x0294, 0x8291, 0x82B3, 0x02B6, 0x02BC, 0x82B9, 0x02A8, 0x82AD, 0x82A7, 0x02A2,
            0x82E3, 0x02E6, 0x02EC, 0x82E9, 0x02F8, 0x82FD, 0x82F7, 0x02F2, 0x02D0, 0x82D5, 0x82DF, 0x02DA, 0x82CB,
            0x02CE, 0x02C4, 0x82C1, 0x8243, 0x0246, 0x024C, 0x8249, 0x0258, 0x825D, 0x8257, 0x0252, 0x0270, 0x8275,
            0x827F, 0x027A, 0x826B, 0x026E, 0x0264, 0x8261, 0x0220, 0x8225, 0x822F, 0x022A, 0x823B, 0x023E, 0x0234,
            0x8231, 0x8213, 0x0216, 0x021C, 0x8219, 0x0208, 0x820D, 0x8207, 0x0202,
        ];

        let mut crc_accum: u16 = 0x0000;

        for &byte in data {
            let i = ((crc_accum >> 8) ^ (byte as u16)) & 0xFF;
            crc_accum = (crc_accum << 8) ^ CRC_TABLE[i as usize];
        }

        // Return as little-endian bytes [CRC_L, CRC_H]
        [
            (crc_accum & 0xFF) as u8,        // Low byte
            ((crc_accum >> 8) & 0xFF) as u8, // High byte
        ]
    }

    /// Bit-by-bit software implementation of Dynamixel 2.0 CRC-16 for comparison
    ///
    /// This implements the CRC-16 IBM/ANSI algorithm used by Dynamixel 2.0:
    /// - Polynomial: 0x8005 (x^16 + x^15 + x^2 + 1)
    /// - Initial value: 0x0000
    /// - No input/output reflection
    ///
    /// This is much slower than the lookup table method but useful for comparison.
    fn calculate_crc_bitwise(&self, data: &[u8]) -> [u8; 2] {
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

        // Initial test with known Dynamixel test vectors
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

        // Run initial tests
        for (name, packet, expected) in test_cases.iter() {
            self.run_crc_test(name, packet, expected);
        }

        // Run periodic tests with dynamic data
        let mut counter = 0u32;
        loop {
            Timer::after(Duration::from_secs(10)).await;
            counter += 1;

            // Create a buffer filled with time-based data (similar length to typical Dynamixel packets)
            let mut test_buffer = [0u8; 128];
            let timestamp = embassy_time::Instant::now().as_micros() as u32;

            // Fill buffer with time-based data that changes each cycle
            for (i, byte) in test_buffer.iter_mut().enumerate() {
                *byte = ((timestamp.wrapping_add(counter).wrapping_add(i as u32)) & 0xFF) as u8;
            }

            // Calculate expected CRC for this buffer
            let expected_crc = self.crc_processor.calculate_crc(&test_buffer);

            // Test with cycle number in the log
            info!("=== Periodic Test Cycle {} ===", counter);
            self.run_crc_test("Dynamic Buffer Test", &test_buffer, &expected_crc);
        }
    }
}
