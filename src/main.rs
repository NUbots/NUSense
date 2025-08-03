//! Main application entry point for NUSense embedded platform.
//!
//! This firmware initializes the STM32H753 system and provides the foundation
//! for all NUSense platform functionality.

#![no_std]
#![no_main]

// Application modules
mod apps;
mod drivers;
mod peripherals;

use defmt::info;
use embassy_executor::Spawner;
use peripherals::{acm, init_system, usb_system};

#[cfg(not(feature = "debug"))]
use panic_halt as _;
#[cfg(feature = "debug")]
use {defmt_rtt as _, panic_probe as _};

/// Main application entry point
///
/// Initializes the system and spawns all individual tasks.
/// This function never returns as the spawned tasks run indefinitely.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting NUSense firmware v{}", env!("CARGO_PKG_VERSION"));

    // Initialize STM32 peripherals with optimized clock configuration
    let peripherals = init_system();

    let mut usb_system = usb_system::UsbSystem::new(claim_usb!(peripherals));
    let acm_connection = acm::AcmConnection::new(usb_system.builder(), claim_acm!(peripherals));

    // USB System task manages the usb events
    spawner.spawn(usb_system::task(usb_system)).unwrap();

    // Echo application for testing USB CDC ACM communication
    #[cfg(feature = "debug-acm")]
    spawner.spawn(apps::acm_echo::task(acm_connection)).unwrap();

    #[cfg(feature = "debug-crc")]
    spawner.spawn(apps::crc_test::task(claim_crc!(peripherals))).unwrap();

    // IMU task reads from the IMU sensor
    spawner
        .spawn(drivers::imu::task(claim_imu_spi!(peripherals), claim_imu!(peripherals)))
        .unwrap();

    // Main task can do system-level monitoring
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
        info!("System heartbeat - all tasks running");
    }
}
