#![no_std]
#![no_main]

use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        }); // needed for USB

        // Match the original CubeMX configuration exactly
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,  // DIVM1=4
            mul: PllMul::MUL60,       // DIVN1=60
            divp: Some(PllDiv::DIV2), // DIVP1=2 -> 480 MHz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 480 MHz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 240 MHz (HPRE=DIV2)
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 120 MHz (D2PPRE1=DIV2)
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 120 MHz (D2PPRE2=DIV2)
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 120 MHz (D1PPRE=DIV2)
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 120 MHz (D3PPRE=DIV2)
        config.rcc.voltage_scale = VoltageScale::Scale0; // High performance for 480MHz
        config.rcc.mux.usbsel = mux::Usbsel::HSI48;
    }
    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb::Config::default();

    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;

    let driver = Driver::new_hs_ulpi(
        p.USB_OTG_HS,
        Irqs,
        p.PA5,  // USB_OTG_HS_ULPI_CK
        p.PC2,  // USB_OTG_HS_ULPI_DIR
        p.PC3,  // USB_OTG_HS_ULPI_NXT
        p.PC0,  // USB_OTG_HS_ULPI_STP
        p.PA3,  // USB_OTG_HS_ULPI_D0
        p.PB0,  // USB_OTG_HS_ULPI_D1
        p.PB1,  // USB_OTG_HS_ULPI_D2
        p.PB10, // USB_OTG_HS_ULPI_D3
        p.PB11, // USB_OTG_HS_ULPI_D4
        p.PB12, // USB_OTG_HS_ULPI_D5
        p.PB13, // USB_OTG_HS_ULPI_D6
        p.PB5,  // USB_OTG_HS_ULPI_D7
        &mut ep_out_buffer,
        config,
    );

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("NUbots");
    config.product = Some("NUSense");
    config.serial_number = Some("12345678");

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
