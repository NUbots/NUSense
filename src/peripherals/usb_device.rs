use defmt::*;
use embassy_stm32::{
    bind_interrupts, peripherals as stm32_peripherals,
    usb::{self, Driver, InterruptHandler},
};
use embassy_usb::{Builder, UsbDevice}; // Bind USB interrupts for this module
bind_interrupts!(pub struct UsbInterrupts {
    OTG_HS => InterruptHandler<stm32_peripherals::USB_OTG_HS>;
});

/// USB buffer collection that holds all the buffers needed for USB operation
/// This abstracts away the specific buffer types and sizes from main
pub struct UsbBuffers {
    pub ep_out_buffer: [u8; 256],
    pub config_descriptor: [u8; 256],
    pub bos_descriptor: [u8; 256],
    pub control_buf: [u8; 64],
}

impl UsbBuffers {
    /// Create a new set of USB buffers with appropriate sizes
    pub fn new() -> Self {
        Self {
            ep_out_buffer: [0u8; 256],
            config_descriptor: [0u8; 256],
            bos_descriptor: [0u8; 256],
            control_buf: [0u8; 64],
        }
    }
}

/// Complete USB system that takes external buffers and can be configured modularly
/// This hides all the low-level setup complexity from the main application
pub struct UsbSystem<'d> {
    usb_device: Option<UsbDevice<'d, Driver<'d, embassy_stm32::peripherals::USB_OTG_HS>>>,
    builder: Option<Builder<'d, Driver<'d, embassy_stm32::peripherals::USB_OTG_HS>>>,
}

impl<'d> UsbSystem<'d> {
    pub fn new(peripherals: embassy_stm32::Peripherals, usb_buffers: &'d mut UsbBuffers) -> Self {
        info!("Initializing USB system...");

        // Create embassy-usb Config
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("NUbots");
        config.product = Some("NUSense");
        config.serial_number = Some("12345678");

        // Create embassy-usb DeviceBuilder with external buffers
        let builder = Builder::new(
            Self::create_driver(peripherals, &mut usb_buffers.ep_out_buffer),
            config,
            &mut usb_buffers.config_descriptor,
            &mut usb_buffers.bos_descriptor,
            &mut [], // no msos descriptors
            &mut usb_buffers.control_buf,
        );

        info!("USB system created successfully");

        Self {
            usb_device: None,
            builder: Some(builder),
        }
    }

    /// Get a mutable reference to the internal builder for USB classes to configure themselves
    pub fn builder(&mut self) -> &mut Builder<'d, Driver<'d, embassy_stm32::peripherals::USB_OTG_HS>> {
        self.builder.as_mut().expect("Builder already consumed")
    }

    /// Run the USB device (builds first if needed, then runs)
    pub async fn run(&mut self) {
        // Build the device if we haven't already
        if self.usb_device.is_none() {
            if let Some(builder) = self.builder.take() {
                info!("Building USB device...");
                self.usb_device = Some(builder.build());
                info!("USB device built successfully");
            }
        }

        if let Some(ref mut device) = self.usb_device {
            device.run().await;
        } else {
            core::panic!("Failed to build USB device");
        }
    }

    /// Create the USB driver with the specified configuration
    fn create_driver<'driver>(
        usb_peripherals: embassy_stm32::Peripherals,
        ep_out_buffer: &'driver mut [u8],
    ) -> Driver<'driver, embassy_stm32::peripherals::USB_OTG_HS> {
        let mut config = usb::Config::default();

        // As this is a self-powered device, we enable vbus detection
        config.vbus_detection = true;

        Driver::new_hs_ulpi(
            usb_peripherals.USB_OTG_HS,
            UsbInterrupts,
            usb_peripherals.PA5,  // USB_OTG_HS_ULPI_CK
            usb_peripherals.PC2,  // USB_OTG_HS_ULPI_DIR
            usb_peripherals.PC3,  // USB_OTG_HS_ULPI_NXT
            usb_peripherals.PC0,  // USB_OTG_HS_ULPI_STP
            usb_peripherals.PA3,  // USB_OTG_HS_ULPI_D0
            usb_peripherals.PB0,  // USB_OTG_HS_ULPI_D1
            usb_peripherals.PB1,  // USB_OTG_HS_ULPI_D2
            usb_peripherals.PB10, // USB_OTG_HS_ULPI_D3
            usb_peripherals.PB11, // USB_OTG_HS_ULPI_D4
            usb_peripherals.PB12, // USB_OTG_HS_ULPI_D5
            usb_peripherals.PB13, // USB_OTG_HS_ULPI_D6
            usb_peripherals.PB5,  // USB_OTG_HS_ULPI_D7
            ep_out_buffer,
            config,
        )
    }
}
