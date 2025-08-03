//! ICM-20689 6-axis IMU driver
//!
//! This driver provides low-level interface to the ICM-20689 IMU chip over SPI.
//! It handles:
//! - Register-level communication
//! - FIFO buffer management
//! - Interrupt handling for data ready
//! - DMA transfers for high-speed data acquisition
//! - 1000Hz data rate configuration

use crate::peripherals::spi::ImuSpi;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::Pull,
    peripherals::{EXTI10, PE10},
    Peri,
};
use embassy_time::{Duration, Timer};

/// Peripheral collection for IMU interface
pub struct ImuPeripherals<'d> {
    pub interrupt_pin: Peri<'d, PE10>,
    pub interrupt_line: Peri<'d, EXTI10>,
}

/// Register addresses for the ICM-20689
///
/// These correspond to the register map in the ICM-20689 datasheet.
#[repr(u8)]
#[derive(Copy, Clone)]
enum Register {
    SmplrtDiv = 0x19,
    Config = 0x1A,
    GyroConfig = 0x1B,
    AccelConfig = 0x1C,
    AccelConfig2 = 0x1D,
    FifoEn = 0x23,
    IntPinCfg = 0x37,
    IntEnable = 0x38,
    UserCtrl = 0x6A,
    PwrMgmt1 = 0x6B,
    PwrMgmt2 = 0x6C,
    FifoCount = 0x72,
    FifoRw = 0x74,
    WhoAmI = 0x75,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "debug", derive(defmt::Format))]
#[allow(dead_code)]
pub enum AccelRange {
    G2 = 0b00 << 3,
    G4 = 0b01 << 3,
    G8 = 0b10 << 3,
    G16 = 0b11 << 3,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "debug", derive(defmt::Format))]
#[allow(dead_code)]
pub enum GyroRange {
    Dps250 = 0b00 << 3,
    Dps500 = 0b01 << 3,
    Dps1000 = 0b10 << 3,
    Dps2000 = 0b11 << 3,
}

/// Macro to claim peripherals for Icm20689
#[macro_export]
macro_rules! claim_imu {
    ($peripherals:expr) => {{
        drivers::imu::ImuPeripherals {
            interrupt_pin: $peripherals.PE10,
            interrupt_line: $peripherals.EXTI10,
        }
    }};
}

/// Scaled IMU sensor data in physical units
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(defmt::Format))]
pub struct ImuData {
    /// Acceleration in m/s² (X, Y, Z)
    pub accel: [f32; 3],
    /// Angular velocity in rad/s (X, Y, Z)
    pub gyro: [f32; 3],
    /// Temperature in °C
    pub temperature: f32,
}

/// IMU configuration for the ICM-20689
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(defmt::Format))]
pub struct ImuConfig {
    /// Accelerometer full-scale range
    pub accel_range: AccelRange,
    /// Gyroscope full-scale range
    pub gyro_range: GyroRange,
}

impl Default for ImuConfig {
    fn default() -> Self {
        Self {
            accel_range: AccelRange::G4,
            gyro_range: GyroRange::Dps500,
        }
    }
}

/// IMU driver errors
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(defmt::Format))]
pub enum ImuError {
    /// SPI communication error
    SpiError,
    /// Device not found or wrong chip ID
    DeviceNotFound,
}

impl From<embassy_stm32::spi::Error> for ImuError {
    fn from(_: embassy_stm32::spi::Error) -> Self {
        ImuError::SpiError
    }
}

/// ICM-20689 driver for interfacing with the IMU chip
pub struct Icm20689<'d> {
    /// SPI interface to the chip (includes chip select)
    spi: ImuSpi<'d>,
    /// Interrupt pin from the chip
    interrupt: ExtiInput<'d>,
    /// Current chip configuration
    config: ImuConfig,
}

impl<'d> Icm20689<'d> {
    /// Create a new ICM-20689 driver instance
    ///
    /// # Arguments
    /// * `spi` - Configured SPI peripheral for communication (includes chip select)
    /// * `imu_peripherals` - IMU peripheral collection for interrupt handling
    pub fn new(spi: ImuSpi<'d>, imu_peripherals: ImuPeripherals<'d>) -> Self {
        Self {
            spi,
            interrupt: ExtiInput::new(
                imu_peripherals.interrupt_pin,
                imu_peripherals.interrupt_line,
                Pull::None,
            ),
            config: ImuConfig::default(),
        }
    }

    /// Wait for an interrupt from the IMU chip.
    ///
    /// The IMU triggers an interrupt (by pulling the interrupt line low) when new sensor data is available
    /// and ready to be read from the device. This method asynchronously waits until the interrupt line
    /// is asserted, indicating that data is ready for acquisition. It resumes execution once the interrupt
    /// is detected. If called when no interrupt is pending, it will await until the next data-ready event.
    /// This is typically used to synchronize data reads with the IMU's output data rate (e.g., 1000Hz).
    pub async fn wait_for_interrupt(&mut self) {
        self.interrupt.wait_for_low().await;
    }

    /// Read the current FIFO count
    pub async fn read_fifo_count(&mut self) -> Result<u16, ImuError> {
        // Read FIFO_COUNT_H and FIFO_COUNT_L in a single burst
        let mut fifo_count = [0u8; 2];
        self.spi
            .read_register_burst(Register::FifoCount as u8, &mut fifo_count)
            .await?;
        Ok(u16::from_be_bytes(fifo_count))
    }

    /// Read a batch of sensor data from FIFO
    ///
    /// Each packet contains 14 bytes: 6 bytes accel + 2 bytes temp + 6 bytes gyro
    pub async fn read_fifo_batch(&mut self, buffer: &mut [u8]) -> Result<usize, ImuError> {
        // Each packet contains 14 bytes: 6 bytes accel + 2 bytes temp + 6 bytes gyro
        let fifo_count = self.read_fifo_count().await?;
        let bytes_to_read = core::cmp::min(buffer.len(), fifo_count as usize);

        if bytes_to_read == 0 {
            return Ok(0);
        }

        // Read data from FIFO register using burst read
        self.read_fifo_data(&mut buffer[..bytes_to_read]).await?;
        Ok(bytes_to_read)
    }

    /// Parse raw FIFO data into scaled sensor readings
    ///
    /// Each 14-byte packet contains: [accel_x_h, accel_x_l, accel_y_h, accel_y_l,
    /// accel_z_h, accel_z_l, temp_h, temp_l, gyro_x_h, gyro_x_l, gyro_y_h, gyro_y_l, gyro_z_h, gyro_z_l]
    /// Returns scaled data in physical units (m/s² for accelerometer, rad/s for gyroscope, °C for temperature)
    pub fn parse_fifo_packet(&self, packet: &[u8; 14]) -> ImuData {
        // Parse raw values from FIFO packet
        let raw_accel = [
            i16::from_be_bytes([packet[0], packet[1]]), // X
            i16::from_be_bytes([packet[2], packet[3]]), // Y
            i16::from_be_bytes([packet[4], packet[5]]), // Z
        ];
        let raw_temperature = i16::from_be_bytes([packet[6], packet[7]]);
        let raw_gyro = [
            i16::from_be_bytes([packet[8], packet[9]]),   // X
            i16::from_be_bytes([packet[10], packet[11]]), // Y
            i16::from_be_bytes([packet[12], packet[13]]), // Z
        ];

        // Scale to physical units using datasheet LSB values
        let accel_lsb_per_g = match self.config.accel_range {
            AccelRange::G2 => 16384.0, // ±2g range
            AccelRange::G4 => 8192.0,  // ±4g range
            AccelRange::G8 => 4096.0,  // ±8g range
            AccelRange::G16 => 2048.0, // ±16g range
        };
        let accel_scale = 9.80665 / accel_lsb_per_g; // Convert to m/s²

        let gyro_lsb_per_dps = match self.config.gyro_range {
            GyroRange::Dps250 => 131.0, // ±250°/s range
            GyroRange::Dps500 => 65.5,  // ±500°/s range
            GyroRange::Dps1000 => 32.8, // ±1000°/s range
            GyroRange::Dps2000 => 16.4, // ±2000°/s range
        };
        let gyro_scale = (core::f32::consts::PI / 180.0) / gyro_lsb_per_dps; // Convert to rad/s

        // Temperature scaling (datasheet formula)
        let temp_c = f32::from(raw_temperature) / 333.87 + 21.0;

        ImuData {
            accel: [
                f32::from(raw_accel[0]) * accel_scale,
                f32::from(raw_accel[1]) * accel_scale,
                f32::from(raw_accel[2]) * accel_scale,
            ],
            gyro: [
                f32::from(raw_gyro[0]) * gyro_scale,
                f32::from(raw_gyro[1]) * gyro_scale,
                f32::from(raw_gyro[2]) * gyro_scale,
            ],
            temperature: temp_c,
        }
    }

    /// Read data from FIFO register using DMA
    async fn read_fifo_data(&mut self, buffer: &mut [u8]) -> Result<(), ImuError> {
        self.spi
            .read_register_burst(Register::FifoRw as u8, buffer)
            .await
            .map_err(|_| ImuError::SpiError)
    }

    /// Initialize the ICM-20689 chip
    ///
    /// This function:
    /// 1. Resets the device
    /// 2. Verifies chip ID
    /// 3. Configures power management
    /// 4. Sets up accelerometer and gyroscope ranges
    /// 5. Configures FIFO buffer
    /// 6. Enables interrupts
    async fn initialize(&mut self) -> Result<(), ImuError> {
        defmt::info!("Initializing ICM-20689...");
        defmt::debug!("Config: {:?}", self.config);

        // Reset the device
        const DEVICE_RESET: u8 = 0b1000_0000;
        self.spi.write_register(Register::PwrMgmt1 as u8, DEVICE_RESET).await?;
        Timer::after(Duration::from_millis(100)).await;

        // Verify chip ID
        const WHO_AM_I_EXPECTED: u8 = 0x98;
        let chip_id = self.spi.read_register(Register::WhoAmI as u8).await?;
        if chip_id != WHO_AM_I_EXPECTED {
            defmt::error!(
                "Wrong chip ID: expected 0x{:02X}, got 0x{:02X}",
                WHO_AM_I_EXPECTED,
                chip_id
            );
            return Err(ImuError::DeviceNotFound);
        }

        // Disable I2C mode
        const USER_CTRL_I2C_DISABLE: u8 = 0b0001_0000;
        self.spi
            .write_register(Register::UserCtrl as u8, USER_CTRL_I2C_DISABLE)
            .await?;

        // Wake up and select clock source
        const CLK_SEL_PLL: u8 = 0b0000_0001; // Auto PLL (required for max gyro rate)
        self.spi.write_register(Register::PwrMgmt1 as u8, CLK_SEL_PLL).await?;
        Timer::after(Duration::from_millis(10)).await;

        // Enable accelerometer and gyroscope and disable all low power modes
        self.spi.write_register(Register::PwrMgmt2 as u8, 0b0000_0000).await?;

        // Configure DLPF bandwidth
        const CONFIG_DLPF_BANDWIDTH: u8 = 0b0000_0001;
        self.spi
            .write_register(Register::Config as u8, CONFIG_DLPF_BANDWIDTH)
            .await?;

        // Configure sample rate divider for 1000Hz output
        // Sample Rate = Internal_Sample_Rate / (1 + SMPLRT_DIV)
        // With DLPF enabled, internal rate is 1000Hz, so SMPLRT_DIV = 0
        self.spi.write_register(Register::SmplrtDiv as u8, 0b0000_0000).await?;

        // Configure accelerometer range
        self.spi
            .write_register(Register::AccelConfig as u8, self.config.accel_range as u8)
            .await?;
        const ACC_CONFIG2_DLPF_BANDWIDTH: u8 = 0b0000_0001;
        self.spi
            .write_register(Register::AccelConfig2 as u8, ACC_CONFIG2_DLPF_BANDWIDTH)
            .await?;

        // Configure gyroscope range
        self.spi
            .write_register(Register::GyroConfig as u8, self.config.gyro_range as u8)
            .await?;

        // Reset FIFO
        const USER_FIFO_RST: u8 = 0b0000_0100 | USER_CTRL_I2C_DISABLE; // Reset FIFO disable I2C mode
        self.spi.write_register(Register::UserCtrl as u8, USER_FIFO_RST).await?;
        Timer::after(Duration::from_millis(1)).await;

        // Enable FIFO for TEMP + GYRO + ACCEL (bits 7-3 set)
        const FIFO_TEMP_GYRO_ACCEL: u8 = 0b1111_1000;
        self.spi
            .write_register(Register::FifoEn as u8, FIFO_TEMP_GYRO_ACCEL)
            .await?;

        // Enable FIFO
        const USER_FIFO_EN: u8 = 0b0100_0000 | USER_CTRL_I2C_DISABLE; // Enable FIFO disable I2C mode
        self.spi.write_register(Register::UserCtrl as u8, USER_FIFO_EN).await?;

        // Configure interrupt pin (active low, push-pull, cleared on any read)
        const INT_PIN_CFG_LATCH_CLR_ANY_READ: u8 = 0b1001_1000;
        self.spi
            .write_register(Register::IntPinCfg as u8, INT_PIN_CFG_LATCH_CLR_ANY_READ)
            .await?;

        // Enable data ready interrupt (bit 0) instead of FIFO overflow
        const INT_ENABLE_DATA_RDY: u8 = 0b0000_0001;
        self.spi
            .write_register(Register::IntEnable as u8, INT_ENABLE_DATA_RDY)
            .await?;

        defmt::info!("ICM-20689 initialized successfully");
        Ok(())
    }

    /// Main IMU task that handles interrupt-driven FIFO reading
    ///
    /// This task:
    /// 1. Initializes the IMU chip
    /// 2. Waits for interrupts from the IMU (indicating new data in FIFO)
    /// 3. Reads FIFO data using DMA
    /// 4. Logs statistics every second (data rate and latest readings)
    pub async fn run(&mut self) -> Result<(), ImuError> {
        defmt::info!("Starting IMU task - initializing ICM-20689...");

        // Initialize the IMU chip first
        if let Err(e) = self.initialize().await {
            defmt::error!("Failed to initialize IMU: {:?}", e);
            return Err(e);
        }

        defmt::info!("IMU initialized successfully, starting 1000Hz data acquisition...");

        let mut sample_count = 0u32;
        let mut last_log_time = embassy_time::Instant::now();
        let mut latest_accel = [0.0f32; 3];
        let mut latest_gyro = [0.0f32; 3];
        let mut latest_temp = 0.0f32;

        /// Number of bytes in a FIFO packet (6 accel + 2 temp + 6 gyro)
        const PACKET_SIZE: usize = 14;
        /// Maximum number of packets to read from FIFO at once
        const MAX_PACKETS: usize = 20;
        // Buffer sized for up to 20 packets to handle FIFO bursts
        let mut fifo_buffer = [0u8; PACKET_SIZE * MAX_PACKETS];

        loop {
            // Wait for interrupt indicating new data
            self.wait_for_interrupt().await;

            // Read available FIFO data
            match self.read_fifo_batch(&mut fifo_buffer).await {
                Ok(bytes_read) => {
                    // Process complete packets from FIFO data
                    let packet_count = bytes_read / PACKET_SIZE;
                    for i in 0..packet_count {
                        let packet_start = i * PACKET_SIZE;
                        // Ensure we have a complete packet before processing
                        if packet_start + PACKET_SIZE <= bytes_read {
                            let packet = &fifo_buffer[packet_start..packet_start + PACKET_SIZE];
                            match packet.try_into() {
                                Ok(arr) => {
                                    let scaled = self.parse_fifo_packet(arr);
                                    latest_accel = scaled.accel;
                                    latest_gyro = scaled.gyro;
                                    latest_temp = scaled.temperature;
                                    sample_count += 1;
                                }
                                Err(e) => {
                                    defmt::warn!(
                                        "IMU FIFO packet slice-to-array conversion failed: expected {} bytes, got {} bytes Error: {:?}",
                                        PACKET_SIZE,
                                        packet.len(),
                                        e
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    defmt::warn!("IMU FIFO read error: {:?}", e);
                }
            }

            // Log statistics every second to monitor data rate and values
            let now = embassy_time::Instant::now();
            if now.duration_since(last_log_time).as_millis() >= 1000 {
                defmt::info!(
                    "IMU Stats: {} samples/sec | Accel (m/s²): [{}, {}, {}] | Gyro (rad/s): [{}, {}, {}] | Temp: {} °C",
                    sample_count,
                    latest_accel[0],
                    latest_accel[1],
                    latest_accel[2],
                    latest_gyro[0],
                    latest_gyro[1],
                    latest_gyro[2],
                    latest_temp
                );

                sample_count = 0;
                last_log_time = now;
            }
        }
    }
}

/// Embassy task for running the ICM-20689 IMU driver with error recovery.
///
/// This task initializes the IMU driver using the provided SPI and IMU peripherals,
/// and continuously runs the driver in a loop. If an error occurs during operation,
/// the task logs the error and automatically restarts the driver after a delay,
/// ensuring robust operation in the presence of transient faults.
///
/// # Parameters
/// - `spi_peripherals`: SPI peripheral claims required for IMU communication.
/// - `imu_peripherals`: IMU interrupt pin and line peripherals.
///
/// # Behavior
/// - Runs the IMU driver in an infinite loop.
/// - On error, logs the error and restarts the driver after a 5-second delay.
/// - Intended to be spawned as an Embassy task for continuous IMU data acquisition.
#[embassy_executor::task]
pub async fn task(
    spi_peripherals: crate::peripherals::spi::SpiClaims<'static>,
    imu_peripherals: ImuPeripherals<'static>,
) -> ! {
    let spi = crate::peripherals::spi::ImuSpi::new(spi_peripherals);
    let mut imu = Icm20689::new(spi, imu_peripherals);

    loop {
        match imu.run().await {
            Ok(()) => {
                // This should never happen as run() is supposed to loop forever
                defmt::info!("IMU task unexpectedly returned Ok(())");
            }
            Err(e) => {
                defmt::info!("IMU error: {:?}, restarting in 5 seconds...", e);
                embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
            }
        }
    }
}
