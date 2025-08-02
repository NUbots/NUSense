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
use peripherals::init_system;

// Import panic handler and defmt RTT for debugging
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

    info!("System initialized, spawning individual tasks...");

    // Spawn all tasks directly with peripherals
    spawner
        .spawn(peripherals::usb_system::usb_task(claim_usb!(peripherals)))
        .unwrap();
    spawner
        .spawn(drivers::imu::imu_task(
            claim_imu_spi!(peripherals),
            claim_imu!(peripherals),
        ))
        .unwrap();
    spawner
        .spawn(apps::crc_demo::crc_demo_task(claim_crc!(peripherals)))
        .unwrap();

    // Main task can do system-level monitoring
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
        info!("System heartbeat - all tasks running");
    }
}
