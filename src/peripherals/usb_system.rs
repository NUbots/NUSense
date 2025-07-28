//! USB system abstraction for STM32H753 with ULPI PHY.
//!
//! Provides USB device initialization and management for the NUSense platform.

use defmt::info;
use embassy_stm32::{
    bind_interrupts, peripherals as stm32_peripherals,
    usb::{self, Driver, InterruptHandler},
    Peripherals,
};
use embassy_usb::{Builder, UsbDevice};

/// Maximum USB packet size for high-speed USB (ULPI PHY).
/// This influences buffer sizing throughout the USB system.
pub const MAX_PACKET_SIZE: u16 = 512;

// Bind USB interrupts for the OTG_HS peripheral
bind_interrupts!(
    /// USB interrupt handlers
    pub struct UsbInterrupts {
        OTG_HS => InterruptHandler<stm32_peripherals::USB_OTG_HS>;
    }
);

/// USB buffers for device operation.
#[repr(C, align(32))]
pub struct UsbBuffers {
    /// Endpoint output buffer - sized for maximum packet size
    pub ep_out_buffer: [u8; MAX_PACKET_SIZE as usize * 2], // Double buffered
    /// USB configuration descriptor buffer
    pub config_descriptor: [u8; 256],
    /// USB BOS descriptor buffer
    pub bos_descriptor: [u8; 256],
    /// USB control transfer buffer
    pub control_buf: [u8; 64],
}

impl UsbBuffers {
    /// Create a new set of USB buffers.
    pub const fn new() -> Self {
        Self {
            ep_out_buffer: [0u8; MAX_PACKET_SIZE as usize * 2],
            config_descriptor: [0u8; 256],
            bos_descriptor: [0u8; 256],
            control_buf: [0u8; 64],
        }
    }
}

impl Default for UsbBuffers {
    fn default() -> Self {
        Self::new()
    }
}

/// USB device system abstraction.
///
/// Manages USB device initialization and operation.
pub struct UsbSystem<'d> {
    /// The USB device instance
    usb_device: Option<UsbDevice<'d, Driver<'d, stm32_peripherals::USB_OTG_HS>>>,
    /// The USB builder (consumed when creating the device)
    builder: Option<Builder<'d, Driver<'d, stm32_peripherals::USB_OTG_HS>>>,
}

impl<'d> UsbSystem<'d> {
    /// Create a new USB system.
    ///
    /// # Arguments
    ///
    /// * `peripherals` - STM32 peripherals for USB and GPIO pins
    /// * `usb_buffers` - Pre-allocated buffers for USB operations
    pub fn new(peripherals: Peripherals, usb_buffers: &'d mut UsbBuffers) -> Self {
        info!("Initializing USB system...");

        // Configure USB device descriptor
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("NUbots");
        config.product = Some("NUSense");
        config.serial_number = Some("12345678");

        // Create the USB builder with all required buffers
        let builder = Builder::new(
            Self::create_ulpi_driver(peripherals, &mut usb_buffers.ep_out_buffer),
            config,
            &mut usb_buffers.config_descriptor,
            &mut usb_buffers.bos_descriptor,
            &mut [], // No Microsoft OS descriptors
            &mut usb_buffers.control_buf,
        );

        info!("USB system initialized successfully");

        Self {
            usb_device: None,
            builder: Some(builder),
        }
    }

    /// Get mutable access to the USB builder for class registration.
    ///
    /// USB classes (like CDC ACM) use this to register their endpoints.
    ///
    /// # Panics
    ///
    /// Panics if the builder has already been consumed.
    pub fn builder(&mut self) -> &mut Builder<'d, Driver<'d, stm32_peripherals::USB_OTG_HS>> {
        self.builder.as_mut().expect("USB builder has already been consumed")
    }

    /// Run the USB device.
    ///
    /// Builds the USB device and starts the USB device task.
    /// This function runs indefinitely.
    pub async fn run(&mut self) -> ! {
        // Build the device if not already built
        if self.usb_device.is_none() {
            if let Some(builder) = self.builder.take() {
                info!("Building USB device...");
                self.usb_device = Some(builder.build());
                info!("USB device built successfully");
            }
        }

        // Run the USB device task
        if let Some(ref mut device) = self.usb_device {
            device.run().await;
        } else {
            panic!("Failed to build USB device");
        }
    }

    /// Create a USB driver for ULPI PHY.
    fn create_ulpi_driver(
        peripherals: Peripherals,
        ep_out_buffer: &'d mut [u8],
    ) -> Driver<'d, stm32_peripherals::USB_OTG_HS> {
        let mut usb_config = usb::Config::default();

        // Enable VBUS detection since this is a self-powered device
        usb_config.vbus_detection = true;

        Driver::new_hs_ulpi(
            peripherals.USB_OTG_HS,
            UsbInterrupts,
            peripherals.PA5,  // USB_OTG_HS_ULPI_CK
            peripherals.PC2,  // USB_OTG_HS_ULPI_DIR
            peripherals.PC3,  // USB_OTG_HS_ULPI_NXT
            peripherals.PC0,  // USB_OTG_HS_ULPI_STP
            peripherals.PA3,  // USB_OTG_HS_ULPI_D0
            peripherals.PB0,  // USB_OTG_HS_ULPI_D1
            peripherals.PB1,  // USB_OTG_HS_ULPI_D2
            peripherals.PB10, // USB_OTG_HS_ULPI_D3
            peripherals.PB11, // USB_OTG_HS_ULPI_D4
            peripherals.PB12, // USB_OTG_HS_ULPI_D5
            peripherals.PB13, // USB_OTG_HS_ULPI_D6
            peripherals.PB5,  // USB_OTG_HS_ULPI_D7
            ep_out_buffer,
            usb_config,
        )
    }
}
