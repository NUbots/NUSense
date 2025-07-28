#![no_std]
#![no_main]

mod apps;
mod peripherals;

use apps::EchoApp;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use peripherals::{init_system, AcmConnection, AcmState, UsbBuffers, UsbSystem};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting NUSense!");

    // Initialize STM32 peripherals with proper clock configuration
    let p = init_system();

    // Create USB buffers on main's stack (never move)
    let mut usb_buffers = UsbBuffers::new();

    // Create ACM state on main's stack (never moves)
    let mut acm_state = AcmState::new();

    // Create the USB system with external buffers
    let mut usb_system = UsbSystem::new(p, &mut usb_buffers);

    // Create ACM connection with external state and builder
    let acm_connection = AcmConnection::new(usb_system.builder(), &mut acm_state, 64);

    // Create echo application with ACM connection
    let mut echo_app = EchoApp::new(acm_connection);

    // Run USB system and echo application concurrently
    let usb_fut = async { usb_system.run().await };
    let echo_fut = async { echo_app.run().await };

    info!("Starting USB device and echo application...");

    // Run everything concurrently
    join(usb_fut, echo_fut).await;
}
