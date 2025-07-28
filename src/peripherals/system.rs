//! System initialization and clock configuration for STM32H753.
//!
//! This module handles the complex clock setup required for high-performance operation
//! at 480MHz with proper USB support.

use embassy_stm32::{rcc::*, Config, Peripherals};

/// Initialize the STM32H753 system with optimal clock configuration.
///
/// Configures the system for high-performance operation:
/// - **480 MHz** system clock (maximum for STM32H753) using PLL1 from HSI
/// - **240 MHz** AHB clock (CPU and high-speed peripherals)
/// - **120 MHz** APB clocks (peripheral buses)
/// - **48 MHz** HSI48 clock for USB (synchronized from USB SOF)
/// - **Scale0** voltage scaling for maximum performance
///
/// The clock configuration matches STM32CubeMX recommendations for maximum
/// performance while maintaining USB compatibility.
///
/// # Returns
///
/// The initialized [`Peripherals`] struct containing all STM32 peripheral instances.
///
/// # Panics
///
/// This function will panic if the clock configuration fails, which typically
/// indicates hardware issues or invalid clock settings.
pub fn init_system() -> Peripherals {
    let mut config = Config::default();

    // Enable high-speed internal oscillator (16 MHz)
    config.rcc.hsi = Some(HSIPrescaler::DIV1);
    
    // Enable low-power internal oscillator for backup
    config.rcc.csi = true;
    
    // Enable HSI48 for USB with automatic synchronization from USB SOF packets
    config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });

    // Configure PLL1 for maximum system performance (480 MHz)
    // PLL1 = HSI(16MHz) / DIVM1(4) * DIVN1(60) / DIVP1(2) = 480MHz
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,     // Use internal 16MHz oscillator
        prediv: PllPreDiv::DIV4,    // DIVM1=4 → 4MHz PLL input
        mul: PllMul::MUL60,         // DIVN1=60 → 240MHz VCO
        divp: Some(PllDiv::DIV2),   // DIVP1=2 → 480MHz output
        divq: None,                 // Q output not used
        divr: None,                 // R output not used
    });
    
    // System clock configuration
    config.rcc.sys = Sysclk::PLL1_P;                        // 480 MHz system clock
    config.rcc.ahb_pre = AHBPrescaler::DIV2;                // 240 MHz AHB clock
    config.rcc.apb1_pre = APBPrescaler::DIV2;               // 120 MHz APB1 clock
    config.rcc.apb2_pre = APBPrescaler::DIV2;               // 120 MHz APB2 clock
    config.rcc.apb3_pre = APBPrescaler::DIV2;               // 120 MHz APB3 clock
    config.rcc.apb4_pre = APBPrescaler::DIV2;               // 120 MHz APB4 clock
    
    // Maximum voltage scaling for 480MHz operation
    config.rcc.voltage_scale = VoltageScale::Scale0;
    
    // Use HSI48 for USB (provides accurate 48MHz for USB timing)
    config.rcc.mux.usbsel = mux::Usbsel::HSI48;

    embassy_stm32::init(config)
}
