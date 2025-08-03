//! Application layer for the NUSense robotics platform.
//!
//! This module contains high-level applications and services that run on the NUSense platform:
//! - Motor control and servo management applications
//! - Sensor fusion and IMU processing
//! - Communication protocols and data streaming
//! - System monitoring and diagnostics
//! - User interface and button handling
//!
//! Applications use the hardware abstractions from the peripherals layer to implement
//! robotics functionality like motion control, sensor processing, and communication.

/// Simple echo application for testing USB CDC ACM communication
pub mod acm_echo;
/// CRC demonstration application for Dynamixel protocol
pub mod crc_test;
