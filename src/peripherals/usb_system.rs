//! USB system abstraction for STM32H753 with ULPI PHY.
//!
//! Provides USB device initialization and management for the NUSense platform.

use defmt::info;
use embassy_futures::join::join;
use embassy_stm32::{
    bind_interrupts, peripherals as stm32_peripherals,
    peripherals::{PA3, PA5, PB0, PB1, PB10, PB11, PB12, PB13, PB5, PC0, PC2, PC3, USB_OTG_HS},
    usb::{self, Driver, InterruptHandler},
    Peri,
};
use embassy_usb::{Builder, UsbDevice};

// Static allocation for USB buffers and ACM state
static mut USB_BUFFERS: UsbBuffers = UsbBuffers::new();
static mut ACM_STATE: super::AcmState = super::AcmState::new();

/// Peripheral collection for USB system interface
pub struct UsbPeripherals<'d> {
    pub usb_otg_hs: Peri<'d, USB_OTG_HS>,
    pub ulpi_clk: Peri<'d, PA5>, // USB_OTG_HS_ULPI_CK
    pub ulpi_dir: Peri<'d, PC2>, // USB_OTG_HS_ULPI_DIR
    pub ulpi_nxt: Peri<'d, PC3>, // USB_OTG_HS_ULPI_NXT
    pub ulpi_stp: Peri<'d, PC0>, // USB_OTG_HS_ULPI_STP
    pub ulpi_d0: Peri<'d, PA3>,  // USB_OTG_HS_ULPI_D0
    pub ulpi_d1: Peri<'d, PB0>,  // USB_OTG_HS_ULPI_D1
    pub ulpi_d2: Peri<'d, PB1>,  // USB_OTG_HS_ULPI_D2
    pub ulpi_d3: Peri<'d, PB10>, // USB_OTG_HS_ULPI_D3
    pub ulpi_d4: Peri<'d, PB11>, // USB_OTG_HS_ULPI_D4
    pub ulpi_d5: Peri<'d, PB12>, // USB_OTG_HS_ULPI_D5
    pub ulpi_d6: Peri<'d, PB13>, // USB_OTG_HS_ULPI_D6
    pub ulpi_d7: Peri<'d, PB5>,  // USB_OTG_HS_ULPI_D7
}

/// Macro to claim peripherals for UsbSystem
#[macro_export]
macro_rules! claim_usb {
    ($peripherals:expr) => {{
        $crate::peripherals::usb_system::UsbPeripherals {
            usb_otg_hs: $peripherals.USB_OTG_HS,
            ulpi_clk: $peripherals.PA5, // USB_OTG_HS_ULPI_CK
            ulpi_dir: $peripherals.PC2, // USB_OTG_HS_ULPI_DIR
            ulpi_nxt: $peripherals.PC3, // USB_OTG_HS_ULPI_NXT
            ulpi_stp: $peripherals.PC0, // USB_OTG_HS_ULPI_STP
            ulpi_d0: $peripherals.PA3,  // USB_OTG_HS_ULPI_D0
            ulpi_d1: $peripherals.PB0,  // USB_OTG_HS_ULPI_D1
            ulpi_d2: $peripherals.PB1,  // USB_OTG_HS_ULPI_D2
            ulpi_d3: $peripherals.PB10, // USB_OTG_HS_ULPI_D3
            ulpi_d4: $peripherals.PB11, // USB_OTG_HS_ULPI_D4
            ulpi_d5: $peripherals.PB12, // USB_OTG_HS_ULPI_D5
            ulpi_d6: $peripherals.PB13, // USB_OTG_HS_ULPI_D6
            ulpi_d7: $peripherals.PB5,  // USB_OTG_HS_ULPI_D7
        }
    }};
}

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
    /// Create a new USB system
    ///
    /// # Arguments
    /// * `peripherals` - UsbPeripherals struct containing all required peripherals
    /// * `usb_buffers` - Pre-allocated buffers for USB operations
    pub fn new(peripherals: UsbPeripherals<'d>, usb_buffers: &'d mut UsbBuffers) -> Self {
        info!("Initializing USB system...");

        // Configure USB device descriptor
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("NUbots");
        config.product = Some("NUSense");
        config.serial_number = Some("12345678");

        // Create USB driver with ULPI PHY
        let mut usb_config = usb::Config::default();
        usb_config.vbus_detection = true;

        let driver = Driver::new_hs_ulpi(
            peripherals.usb_otg_hs,
            UsbInterrupts,
            peripherals.ulpi_clk,
            peripherals.ulpi_dir,
            peripherals.ulpi_nxt,
            peripherals.ulpi_stp,
            peripherals.ulpi_d0,
            peripherals.ulpi_d1,
            peripherals.ulpi_d2,
            peripherals.ulpi_d3,
            peripherals.ulpi_d4,
            peripherals.ulpi_d5,
            peripherals.ulpi_d6,
            peripherals.ulpi_d7,
            &mut usb_buffers.ep_out_buffer,
            usb_config,
        );

        // Create the USB builder with all required buffers
        let builder = Builder::new(
            driver,
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
        let device = self.usb_device.as_mut().expect("Failed to build USB device");
        device.run().await;
    }
}

/// USB system task with integrated echo functionality
#[embassy_executor::task]
pub async fn usb_task(usb_peripherals: UsbPeripherals<'static>) -> ! {
    let usb_buffers = unsafe {
        let usb_buffers = &raw mut USB_BUFFERS;
        usb_buffers.as_mut().unwrap()
    };

    let mut usb_system = UsbSystem::new(usb_peripherals, usb_buffers);

    // Create ACM connection
    let acm_connection = unsafe {
        let acm_state = &raw mut ACM_STATE;
        super::AcmConnection::new(usb_system.builder(), acm_state.as_mut().unwrap())
    };

    // Create echo app
    let mut echo_app = crate::apps::EchoApp::new(acm_connection);

    // Run both the USB device and echo app concurrently
    let usb_device_task = usb_system.run();
    let echo_task = echo_app.run();

    join(usb_device_task, echo_task).await;

    // This should never be reached since both tasks run forever
    unreachable!()
}
