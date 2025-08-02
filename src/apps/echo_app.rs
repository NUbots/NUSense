//! Simple echo application for testing USB CDC ACM packet-based communication.
//!
//! This application serves as a basic communication test and development tool for the
//! NUSense platform. It demonstrates how to use the low-latency packet-based USB CDC ACM
//! interface and provides a foundation for building real-time robotics communication protocols.
//!
//! In a production robotics system, this would be replaced with applications for:
//! - Real-time command/response protocols
//! - Low-latency sensor data streaming
//! - Motor control command processing
//! - System status and diagnostics reporting
//! - Configuration and calibration interfaces

use crate::peripherals::usb_system::MAX_PACKET_SIZE;
use crate::peripherals::{AcmConnection, Disconnected};
use defmt::{info, warn};
use embassy_time::Timer;

/// Maximum size of data that can be processed in a single packet
const BUFFER_SIZE: usize = MAX_PACKET_SIZE as usize;

/// Delay between reconnection attempts when the connection is lost
const RECONNECT_DELAY_MS: u64 = 100;

/// Echo application that demonstrates USB CDC ACM packet-based communication.
///
/// This application provides a low-latency echo server that processes individual
/// USB packets for minimal communication delay. This approach is optimal for
/// real-time robotics applications where packet boundaries are significant:
///
/// # Features
///
/// - **Packet-level control**: Processes individual USB packets for minimal latency
/// - **Real-time friendly**: Predictable timing without buffering delays
/// - **DMA acceleration**: Uses hardware DMA for CPU-efficient transfers
/// - **Connection resilience**: Automatic reconnection on disconnect
/// - **Detailed logging**: Shows both binary and text interpretation of packets
///
/// # Performance
///
/// - Uses DMA-capable USB OTG HS interface for efficient transfers
/// - Supports high-speed USB (up to 512-byte packets)
/// - Immediate packet processing without intermediate buffering
/// - Optimal for real-time command/response protocols
///
/// # Example Usage
///
/// ```rust,ignore
/// let mut echo_app = EchoApp::new(acm_connection);
/// echo_app.run().await; // Runs forever
/// ```
pub struct EchoApp<'d> {
    acm: AcmConnection<'d>,
}

impl<'d> EchoApp<'d> {
    /// Create a new echo application with the specified ACM connection.
    ///
    /// # Arguments
    ///
    /// * `acm` - The CDC ACM connection to use for communication
    ///
    /// # Returns
    ///
    /// A new [`EchoApp`] instance ready to run.
    pub const fn new(acm: AcmConnection<'d>) -> Self {
        Self { acm }
    }

    /// Run the echo application.
    ///
    /// This function runs indefinitely, handling USB connections and echoing
    /// data back to the host. It automatically handles disconnections and
    /// reconnections gracefully.
    ///
    /// The application will:
    /// 1. Wait for a USB host to connect
    /// 2. Echo all received data back to the host
    /// 3. Handle disconnections by waiting for reconnection
    /// 4. Repeat indefinitely
    ///
    /// # Note
    ///
    /// This function never returns under normal operation.
    pub async fn run(&mut self) -> ! {
        info!("Echo application started");

        loop {
            // Wait for a host to connect
            self.acm.wait_connection().await;
            info!("Echo app: Host connected, starting echo loop");

            // Run the echo loop until disconnection
            match self.echo_loop().await {
                Err(Disconnected) => {
                    warn!("Echo loop: Connection lost, will reconnect...");

                    // Brief delay before attempting to reconnect
                    Timer::after_millis(RECONNECT_DELAY_MS).await;
                }
                Ok(()) => {
                    // Echo loop completed successfully (should not happen in normal operation)
                    warn!("Echo loop: Completed unexpectedly");
                }
            }
        }
    }

    /// Internal echo loop that handles data transfer.
    ///
    /// This function continuously reads data from the ACM connection and
    /// echoes it back to the host until a disconnection occurs.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Should never happen under normal operation
    /// * `Err(Disconnected)` - When the host disconnects
    async fn echo_loop(&mut self) -> Result<(), Disconnected> {
        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            // Receive a packet from the host
            let bytes_received = self.acm.receive_packet(&mut buffer).await?;
            let data = &buffer[..bytes_received];

            // Log the received packet for debugging
            let utilization_percent = (bytes_received * 100) / BUFFER_SIZE;
            info!(
                "Received {} bytes packet (buffer utilization: {}%): {:02x}",
                bytes_received, utilization_percent, data
            );

            // Try to interpret as UTF-8 text for better logging
            match core::str::from_utf8(data) {
                Ok(text)
                    if text
                        .chars()
                        .all(|c| c.is_ascii() && (!c.is_control() || c.is_ascii_whitespace())) =>
                {
                    info!("Received text packet: '{}'", text);
                }
                _ => {
                    info!(
                        "Received binary packet (showing first 32 bytes): {:02x}",
                        &data[..core::cmp::min(32, data.len())]
                    );
                }
            }

            // Echo the packet back to the host
            self.acm.send_packet(data).await?;
            info!("Echoed {} bytes packet back to host", bytes_received);
        }
    }
}
