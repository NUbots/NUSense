use crate::peripherals::{AcmConnection, Disconnected};
use defmt::*;

/// Echo application that demonstrates basic USB CDC ACM communication
///
/// This application simply echoes back any data received over the USB serial connection.
/// It serves as a simple example of how to use the ACM connection for bidirectional
/// communication.
pub struct EchoApp<'d> {
    acm: AcmConnection<'d>,
}

impl<'d> EchoApp<'d> {
    pub fn new(acm: AcmConnection<'d>) -> Self {
        Self { acm }
    }

    /// Run the echo application loop
    ///
    /// This will continuously read data from the ACM connection and echo it back.
    /// The loop handles connection disconnections gracefully and will restart
    /// when a new connection is established.
    pub async fn run(&mut self) {
        info!("Starting echo application");

        loop {
            // Wait for connection
            self.acm.wait_connection().await;
            info!("Echo app: ACM connected");

            // Run the echo loop until disconnection
            match self.run_echo_loop().await {
                Ok(_) => {
                    info!("Echo loop completed normally");
                }
                Err(_) => {
                    warn!("Echo loop disconnected, restarting...");
                    // Brief delay before retrying
                    embassy_time::Timer::after_millis(100).await;
                }
            }
        }
    }

    /// Internal echo loop that reads data and echoes it back
    async fn run_echo_loop(&mut self) -> Result<(), Disconnected> {
        let mut buf = [0; 64];
        loop {
            // Read data from ACM
            let n = self.acm.receive_data(&mut buf).await?;
            let data = &buf[..n];

            // Log the raw bytes
            info!("Echo app received {} bytes: {:x}", n, data);

            // Try to log as text if it's printable ASCII
            if let Ok(text) = core::str::from_utf8(data) {
                info!("Echo app received text: '{}'", text);
            }

            // Echo it back
            self.acm.send_data(data).await?;
            info!("Echo app sent back {} bytes", n);
        }
    }
}
