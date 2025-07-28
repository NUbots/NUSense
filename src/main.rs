//! Main application entry point for NUSense embedded platform.
//!
//! This firmware initializes the STM32H753 system and provides the foundation
//! for all NUSense platform functionality.

#![no_std]
#![no_main]

// Application modules
mod apps;
mod peripherals;

use apps::EchoApp;
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use peripherals::{init_system, AcmConnection, AcmState, UsbBuffers, UsbSystem};

// Import panic handler and defmt RTT for debugging
#[cfg(not(feature = "debug"))]
use panic_halt as _;
#[cfg(feature = "debug")]
use {defmt_rtt as _, panic_probe as _};

/// Main application entry point
///
/// Initializes the system and runs the USB device and echo application concurrently.
/// This function never returns as it runs the main application loop.
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting NUSense firmware v{}", env!("CARGO_PKG_VERSION"));

    // Initialize STM32 peripherals with optimized clock configuration
    let peripherals = init_system();

    // Create memory buffers on the stack here
    // They won't go out of scope and mean there is no need for static or heap allocations
    let mut usb_buffers = UsbBuffers::new();
    let mut acm_state = AcmState::new();

    // Initialize the USB system with the allocated buffers
    let mut usb_system = UsbSystem::new(peripherals, &mut usb_buffers);

    // Create ACM connection using the USB builder and state
    let acm_connection = AcmConnection::new(usb_system.builder(), &mut acm_state);

    // Initialize the echo application with the ACM connection
    let mut echo_app = EchoApp::new(acm_connection);

    info!("System initialized, starting USB device and echo application...");

    // Run USB system and echo application concurrently until completion
    // You must run the tasks in order here though so that the USB system is ready
    // before the echo app starts using it.
    let usb_task = usb_system.run();
    let echo_task = echo_app.run();

    // Both tasks run indefinitely, so this join never completes
    join(usb_task, echo_task).await;
}
