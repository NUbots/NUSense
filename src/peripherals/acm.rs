//! CDC ACM (virtual serial port) implementation for packet-based communication.
//!
//! Provides low-latency USB communication using the STM32H753's hardware DMA
//! for efficient robotics applications.

use defmt::info;
use embassy_stm32::{peripherals::USB_OTG_HS, usb::Driver};
pub use embassy_usb::class::cdc_acm::State;
use embassy_usb::{class::cdc_acm::CdcAcmClass, driver::EndpointError, Builder};
use static_cell::StaticCell;

// Import MAX_PACKET_SIZE from USB system
use super::usb_system::MAX_PACKET_SIZE;

pub static ACM_STATE: StaticCell<State<'static>> = StaticCell::new();

/// Peripheral collection for ACM interface
pub struct AcmClaims<'d> {
    pub acm_state: &'d mut State<'d>,
}

/// Macro to claim peripherals for AcmConnection
#[macro_export]
macro_rules! claim_acm {
    ($peripherals:expr) => {{
        $crate::peripherals::acm::AcmClaims {
            acm_state: $crate::peripherals::acm::ACM_STATE.init(embassy_usb::class::cdc_acm::State::new()),
        }
    }};
}

/// Error indicating USB connection was disconnected.
#[derive(Debug, Clone, Copy)]
pub struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(error: EndpointError) -> Self {
        match error {
            EndpointError::BufferOverflow => panic!("USB buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

/// CDC ACM connection for packet-based USB communication.
///
/// Provides send/receive of individual USB packets up to MAX_PACKET_SIZE bytes.
///
/// # Example
///
/// ```rust,ignore
/// let mut acm = AcmConnection::new(usb_builder, &mut acm_state);
/// acm.wait_connection().await;
///
/// // Send a packet
/// acm.send_packet(b"Hello").await?;
///
/// // Receive a packet
/// let mut buffer = [0u8; MAX_PACKET_SIZE as usize];
/// let len = acm.receive_packet(&mut buffer).await?;
/// ```
pub struct AcmConnection<'d> {
    class: CdcAcmClass<'d, Driver<'d, USB_OTG_HS>>,
}

impl<'d> AcmConnection<'d> {
    /// Create a new ACM connection.
    ///
    /// # Arguments
    ///
    /// * `builder` - USB device builder
    /// * `claims` - AcmClaims struct containing ACM state
    pub fn new(builder: &mut Builder<'d, Driver<'d, USB_OTG_HS>>, claims: AcmClaims<'d>) -> Self {
        info!("CDC ACM connection initialized");
        Self {
            class: CdcAcmClass::new(builder, claims.acm_state, MAX_PACKET_SIZE),
        }
    }

    /// Wait for USB host to connect and open the CDC ACM interface.
    pub async fn wait_connection(&mut self) {
        self.class.wait_connection().await;
        info!("CDC ACM connection established");
    }

    /// Send a USB packet to the host.
    ///
    /// # Arguments
    ///
    /// * `data` - Packet data to send (should not exceed MAX_PACKET_SIZE)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if sent successfully
    /// * `Err(Disconnected)` if host disconnected
    pub async fn send_packet(&mut self, data: &[u8]) -> Result<(), Disconnected> {
        self.class.write_packet(data).await.map_err(Into::into)
    }

    /// Receive a USB packet from the host.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to store received packet data
    ///
    /// # Returns
    ///
    /// * `Ok(bytes_received)` - Number of bytes received (0 to MAX_PACKET_SIZE)
    /// * `Err(Disconnected)` - If host disconnected
    pub async fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, Disconnected> {
        self.class.read_packet(buffer).await.map_err(Into::into)
    }
}
