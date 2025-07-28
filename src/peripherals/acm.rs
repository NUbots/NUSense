use defmt::*;
use embassy_stm32::usb::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;

/// ACM state wrapper that abstracts away the Embassy USB CDC ACM State type
/// This hides the specific state implementation details from main
pub struct AcmState<'d> {
    state: State<'d>,
}

impl<'d> AcmState<'d> {
    /// Create a new ACM state
    pub fn new() -> Self {
        Self { state: State::new() }
    }

    /// Get a mutable reference to the internal state for ACM connection creation
    pub fn state_mut(&mut self) -> &mut State<'d> {
        &mut self.state
    }
}

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => core::panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

/// CDC ACM (Communications Device Class - Abstract Control Model) connection
///
/// This peripheral provides a USB serial interface that appears as a virtual
/// serial port on the host computer. It handles the low-level USB communication
/// and provides simple read/write methods for applications to use.
pub struct AcmConnection<'d> {
    class: CdcAcmClass<'d, Driver<'d, embassy_stm32::peripherals::USB_OTG_HS>>,
}

impl<'d> AcmConnection<'d> {
    /// Create a new ACM connection with external state and builder
    pub fn new(
        builder: &mut Builder<'d, Driver<'d, embassy_stm32::peripherals::USB_OTG_HS>>,
        acm_state: &'d mut AcmState<'d>,
        max_packet_size: u16,
    ) -> Self {
        Self {
            class: CdcAcmClass::new(builder, acm_state.state_mut(), max_packet_size),
        }
    }

    /// Wait for the USB host to connect
    pub async fn wait_connection(&mut self) {
        self.class.wait_connection().await;
        info!("CDC ACM Connected");
    }

    /// Send data to the USB host
    pub async fn send_data(&mut self, data: &[u8]) -> Result<(), Disconnected> {
        self.class.write_packet(data).await.map_err(|e| e.into())
    }

    /// Receive data from the USB host
    ///
    /// Returns the number of bytes received
    pub async fn receive_data(&mut self, buf: &mut [u8]) -> Result<usize, Disconnected> {
        self.class.read_packet(buf).await.map_err(|e| e.into())
    }
}
