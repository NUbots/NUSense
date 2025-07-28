use embassy_stm32::{rcc::*, Config, Peripherals};

/// Initialize the STM32H753 system with proper clock configuration
///
/// This sets up:
/// - 480 MHz system clock from PLL1
/// - 240 MHz AHB clock
/// - 120 MHz APB clocks
/// - 48 MHz HSI48 for USB
/// - High performance voltage scaling for 480MHz operation
///
/// Returns the initialized peripherals ready for use
pub fn init_system() -> Peripherals {
    let mut config = Config::default();

    config.rcc.hsi = Some(HSIPrescaler::DIV1);
    config.rcc.csi = true;
    config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true }); // needed for USB

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

    embassy_stm32::init(config)
}
