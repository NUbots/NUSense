//! SPI peripheral configuration and management for NUSense platform.
//!
//! This module provides SPI configuration for various sensors including the ICM-20689 IMU.
//! It handles DMA configuration for high-performance data transfer.

use embassy_stm32::{
    gpio::{Level, Output, Speed},
    mode::Async,
    peripherals::{DMA1_CH0, DMA1_CH1, PE11, PE12, PE13, PE14, SPI4},
    spi::{Config as SpiConfig, Mode, Phase, Polarity, Spi},
    time::Hertz,
    Peri,
};

/// Peripheral collection for IMU SPI interface
pub struct SpiPeripherals<'d> {
    pub spi4: Peri<'d, SPI4>,
    pub cs: Peri<'d, PE11>,         // CS
    pub sck: Peri<'d, PE12>,        // SCK
    pub miso: Peri<'d, PE13>,       // MISO
    pub mosi: Peri<'d, PE14>,       // MOSI
    pub dma_tx: Peri<'d, DMA1_CH0>, // TX DMA
    pub dma_rx: Peri<'d, DMA1_CH1>, // RX DMA
}

/// Macro to claim peripherals for ImuSpi
#[macro_export]
macro_rules! claim_imu_spi {
    ($peripherals:expr) => {{
        $crate::peripherals::spi::SpiPeripherals {
            spi4: $peripherals.SPI4,
            cs: $peripherals.PE11,         // CS
            sck: $peripherals.PE12,        // SCK
            miso: $peripherals.PE13,       // MISO
            mosi: $peripherals.PE14,       // MOSI
            dma_tx: $peripherals.DMA1_CH0, // TX DMA
            dma_rx: $peripherals.DMA1_CH1, // RX DMA
        }
    }};
}

/// SPI configuration for the ICM-20689 IMU
///
/// The ICM-20689 supports SPI mode 0 or 3. We use mode 3 (CPOL=1, CPHA=1) as per CubeMX config.
/// SPI clock frequency is 8 MHz as per ICM-20689 datasheet maximum.
/// Software chip select (PE11) is used for chip select control.
pub struct ImuSpi<'d> {
    /// SPI peripheral instance with DMA
    pub spi: Spi<'d, Async>,
    /// Chip select pin (software controlled)
    pub cs: Output<'d>,
}

impl<'d> ImuSpi<'d> {
    /// Create a new IMU SPI configuration with software chip select
    ///
    /// # Arguments
    /// * `peripherals` - SpiPeripherals struct containing all required peripherals
    ///
    /// # Returns
    /// Configured SPI instance with software chip select control
    pub fn new(peripherals: SpiPeripherals<'d>) -> Self {
        // Configure SPI for ICM-20689 to match CubeMX configuration
        let mut config = SpiConfig::default();
        config.mode = Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnSecondTransition,
        };
        config.frequency = Hertz(8_000_000);

        let cs_pin = Output::new(peripherals.cs, Level::High, Speed::VeryHigh);

        let spi = Spi::new(
            peripherals.spi4,
            peripherals.sck,
            peripherals.mosi,
            peripherals.miso,
            peripherals.dma_tx,
            peripherals.dma_rx,
            config,
        );

        Self { spi, cs: cs_pin }
    }

    /// Read a single register from an SPI device
    ///
    /// # Arguments
    /// * `reg` - Register address to read from
    ///
    /// # Returns
    /// * Register value on success
    /// * SPI error on failure
    ///
    /// This function performs a register read operation for devices that use the MSB of the register address as a read bit flag:
    /// 1. Assert chip select (low)
    /// 2. Send register address with read bit set (MSB = 1)
    /// 3. Receive response data
    /// 4. Deassert chip select (high)
    ///
    /// **Note:** This MSB read bit convention is device-specific.
    /// Consult your device's datasheet to determine the correct register read command format.
    pub async fn read_register(&mut self, reg: u8) -> Result<u8, embassy_stm32::spi::Error> {
        // Having the MSB of the register address set to 1 is the convention for reading from a register
        const SPI_READ_BIT: u8 = 0x80;
        let tx_buf = [reg | SPI_READ_BIT, 0x00]; // Set read bit, dummy byte for response
        let mut rx_buf = [0u8; 2];

        self.cs.set_low();

        // Use SPI transfer to send command and receive response via DMA
        let result = self.spi.transfer(&mut rx_buf, &tx_buf).await;

        self.cs.set_high();

        result?;
        // Return the data byte (second byte of response)
        Ok(rx_buf[1])
    }

    /// Write a single register to an SPI device
    ///
    /// # Arguments
    /// * `reg` - Register address to write to
    /// * `value` - Value to write to the register
    ///
    /// # Returns
    /// * Success or SPI error
    ///
    /// This function performs a generic SPI register write operation:
    /// 1. Assert chip select (low)
    /// 2. Send register address with write bit clear (MSB = 0)
    /// 3. Send data value
    /// 4. Deassert chip select (high)
    pub async fn write_register(&mut self, reg: u8, value: u8) -> Result<(), embassy_stm32::spi::Error> {
        // Having the MSB of the register address set to 0 is the convention for writing to a register
        const SPI_WRITE_MASK: u8 = 0x7F;
        let tx_buf = [reg & SPI_WRITE_MASK, value]; // Clear read bit

        self.cs.set_low();

        // Use SPI write to send command and data via DMA
        let result = self.spi.write(&tx_buf).await;

        self.cs.set_high();

        result
    }

    /// Read data from a specific register using DMA
    ///
    /// # Arguments
    /// * `reg` - Register address to read from
    /// * `buffer` - Buffer to store the read data
    ///
    /// # Returns
    /// * Success or SPI error
    ///
    /// This function performs a burst read operation:
    /// 1. Assert chip select (low)
    /// 2. Send register address with read bit set
    /// 3. Read multiple bytes into buffer
    /// 4. Deassert chip select (high)
    pub async fn read_register_burst(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), embassy_stm32::spi::Error> {
        self.cs.set_low();

        // Send register address with read bit
        let cmd = [reg | 0x80];

        // First, send the command to set the register address
        self.spi.write(&cmd).await?;

        // Then read the data from the register
        let result = self.spi.read(buffer).await;

        self.cs.set_high();

        result
    }
}
