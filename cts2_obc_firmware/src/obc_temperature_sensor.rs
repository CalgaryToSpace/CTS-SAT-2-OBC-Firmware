use core::cell::RefCell;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use rtt_target::rprintln;
use stm32l4xx_hal::delay;
use stm32l4xx_hal::hal::blocking::i2c::Write;
use stm32l4xx_hal::hal::blocking::i2c::WriteRead;
use stm32l4xx_hal::i2c::I2c;

const STDS75_SERIAL_BUS_SLAVE_ADDRESS: u16 = 0x48;
const OBC_TEMP_SENSOR_TEMP_REGISTER_ADDR: u8 = 0x00;
const OBC_TEMP_SENSOR_TIMEOUT_MS: u32 = 50;

// write to this register to change the precision
const OBC_TEMP_SENSOR_CONFIG_REGISTER_ADDR: u8 = 0x01;

// 0x00 for nine bit, 0x01 for 10 bit, 0x02 for 11 bit, 0x03 for 12 bit
const OBC_TEMP_SENSOR_precision: u8 = 0x01 << 5;
const OBC_TEMP_SENSOR_DEVICE_ADDR: u8 = 0x48;

const OBC_TEMP_SENSOR_NINE_BIT_SCALING_FACTOR: u32 = 10;
const OBC_TEMP_SENSOR_TEN_BIT_SCALING_FACTOR: u32 = 100;
const OBC_TEMP_SENSOR_ELEVEN_BIT_SCALING_FACTOR: u32 = 1000;
const OBC_TEMP_SENSOR_TWELVE_BIT_SCALING_FACTOR: u32 = 10000;

const OBC_TEMP_SENSOR_ERROR_TEMPERATURE_CC: i32 = 99999;

const OBC_TEMP_SENSOR_NINE_BIT_PRECISION_COEFFICIENT: f32 = 2.0;
const OBC_TEMP_SENSOR_TEN_BIT_PRECISION_COEFFICIENT: f32 = 4.0;
const OBC_TEMP_SENSOR_ELEVEN_BIT_PRECISION_COEFFICIENT: f32 = 8.0;
const OBC_TEMP_SENSOR_TWELVE_BIT_PRECISION_COEFFICIENT: f32 = 16.0;

enum TemperatureSensorDataPrecisionInsignificantBytes {
    OBC_TEMP_SENSOR_TWELVE_BIT_PRECISION_INSIGNIFICANT_BYTES = 4,
    OBC_TEMP_SENSOR_ELEVEN_BIT_PRECISION_INSIGNIFICANT_BYTES,
    OBC_TEMP_SENSOR_TEN_BIT_PRECISION_INSIGNIFICANT_BYTES,
    OBC_TEMP_SENSOR_NINE_BIT_PRECISION_INSIGNIFICANT_BYTES,
}

// used in temperature calculation
static OBC_TEMP_SENSOR_PRECISION_COEFFICIENT: Mutex<RefCell<f32>> =
    Mutex::new(RefCell::new(OBC_TEMP_SENSOR_TEN_BIT_PRECISION_COEFFICIENT));

static OBC_TEMP_SENSOR_PRECISION_INSIGNIFICANT_BITS: Mutex<
    RefCell<TemperatureSensorDataPrecisionInsignificantBytes>,
> = Mutex::new(RefCell::new(
    TemperatureSensorDataPrecisionInsignificantBytes::OBC_TEMP_SENSOR_TEN_BIT_PRECISION_INSIGNIFICANT_BYTES,
));

static OBC_TEMP_SENSOR_PRECISION_SCALING_FACTOR: Mutex<RefCell<u32>> =
    Mutex::new(RefCell::new(100));

enum PrecisionError {
    InvalidPrecision,
}

struct Set_Precision_Data {
    precision_scaling_factor: u32,
    obc_temp_sensor_precision_coefficient: f32,
    precision_insignificant_bits: TemperatureSensorDataPrecisionInsignificantBytes,
    config_write_data: u8,
    conversion_delay_ms: u32,
}

impl Default for Set_Precision_Data {
    fn default() -> Self {
        Self {
            precision_scaling_factor: 0,
            obc_temp_sensor_precision_coefficient: 1.0,
            precision_insignificant_bits:
                TemperatureSensorDataPrecisionInsignificantBytes::OBC_TEMP_SENSOR_TWELVE_BIT_PRECISION_INSIGNIFICANT_BYTES,
            config_write_data: 0,
            conversion_delay_ms: 0,
        }
    }
}

fn OBC_TEMP_SENSOR_convert_raw_to_deg_c(
    raw_bytes: &[u8; 2],
    precision_coefficient: u8,
    precision_insignificant_bits: TemperatureSensorDataPrecisionInsignificantBytes,
    precision_scaling_factor: u16,
) -> i32 {
    // Combine the two bytes into one signed 16-bit value
    let mut val = i16::from_be_bytes([raw_bytes[0], raw_bytes[1]]);

    // Remove the always-zero bits and bits not used for our precision
    val >>= precision_insignificant_bits as u8;

    // Calculate temperature in degrees Celsius
    let temp: f32 = val as f32 / precision_coefficient as f32;

    // Convert temperature to a value without fractional bits
    let result: i32 = (temp * precision_scaling_factor as f32) as i32;

    result
}

fn OBC_TEMP_SENSOR_get_temp_precision<I2C>(i2c: &mut I2C) -> i8
where
    I2C: WriteRead,
{
    let mut buf = [0u8; 1];
    let precision_mask: u8 = 0x60; // 0b0110_0000

    let status = i2c.write_read(
        OBC_TEMP_SENSOR_DEVICE_ADDR,
        &[OBC_TEMP_SENSOR_CONFIG_REGISTER_ADDR],
        &mut buf,
    );

    if status.is_err() {
        // Return -1 if unable to read config register
        return -1;
    }

    // Put the precision bits into the 2 least-significant bits
    ((buf[0] & precision_mask) >> 5) as i8
}

fn OBC_TEMP_SENSOR_read_temperature<I2C>(i2c: &mut I2C, result: &mut i32) -> u8
where
    I2C: WriteRead,
{
    let mut buf = [0u8; 2];

    // Read 2 bytes from the temperature register
    let status = i2c.write_read(
        OBC_TEMP_SENSOR_DEVICE_ADDR,
        &[OBC_TEMP_SENSOR_TEMP_REGISTER_ADDR],
        &mut buf,
    );

    if status.is_err() {
        return 2;
    }

    *result = OBC_TEMP_SENSOR_convert_raw_to_deg_c(
        buf,
        OBC_TEMP_SENSOR_PRECISION_COEFFICIENT,
        OBC_TEMP_SENSOR_PRECISION_INSIGNIFICANT_BITS,
        OBC_TEMP_SENSOR_PRECISION_SCALING_FACTOR,
    );

    0
}

fn obc_temp_sensor_configure_precision_values(
    arg_precision: u8,
    precision_data: &mut Set_Precision_Data,
) -> Result<(), PrecisionError> {
    match arg_precision {
        9 => {
            precision_data.precision_scaling_factor = OBC_TEMP_SENSOR_NINE_BIT_SCALING_FACTOR;

            precision_data.precision_insignificant_bits =
                TemperatureSensorDataPrecisionInsignificantBytes::
                    OBC_TEMP_SENSOR_NINE_BIT_PRECISION_INSIGNIFICANT_BYTES;

            precision_data.obc_temp_sensor_precision_coefficient =
                OBC_TEMP_SENSOR_NINE_BIT_PRECISION_COEFFICIENT;

            precision_data.config_write_data = 0;
            precision_data.conversion_delay_ms = 150;
            Ok(())
        }

        10 => {
            precision_data.precision_scaling_factor = OBC_TEMP_SENSOR_TEN_BIT_SCALING_FACTOR;
            precision_data.precision_insignificant_bits =
                TemperatureSensorDataPrecisionInsignificantBytes::
                    OBC_TEMP_SENSOR_TEN_BIT_PRECISION_INSIGNIFICANT_BYTES;
            precision_data.obc_temp_sensor_precision_coefficient =
                OBC_TEMP_SENSOR_TEN_BIT_PRECISION_COEFFICIENT;
            precision_data.config_write_data = 0x01u8 << 5;
            precision_data.conversion_delay_ms = 300;
            Ok(())
        }
        11 => {
            precision_data.precision_scaling_factor = OBC_TEMP_SENSOR_ELEVEN_BIT_SCALING_FACTOR;
            precision_data.precision_insignificant_bits =
                TemperatureSensorDataPrecisionInsignificantBytes::
                    OBC_TEMP_SENSOR_ELEVEN_BIT_PRECISION_INSIGNIFICANT_BYTES;
            precision_data.obc_temp_sensor_precision_coefficient =
                OBC_TEMP_SENSOR_ELEVEN_BIT_PRECISION_COEFFICIENT;
            precision_data.config_write_data = 0x02u8 << 5;
            precision_data.conversion_delay_ms = 600;
            Ok(())
        }
        12 => {
            precision_data.precision_scaling_factor = OBC_TEMP_SENSOR_TWELVE_BIT_SCALING_FACTOR;
            precision_data.precision_insignificant_bits =
                TemperatureSensorDataPrecisionInsignificantBytes::
                    OBC_TEMP_SENSOR_TWELVE_BIT_PRECISION_INSIGNIFICANT_BYTES;
            precision_data.obc_temp_sensor_precision_coefficient =
                OBC_TEMP_SENSOR_TWELVE_BIT_PRECISION_COEFFICIENT;
            precision_data.config_write_data = 0x03u8 << 5;
            precision_data.conversion_delay_ms = 1200;
            Ok(())
        }

        _ => Err(PrecisionError::InvalidPrecision),
    }
}

fn OBC_TEMP_SENSOR_set_temp_precision<I2C>(
    arg_precision: u8,
    i2c: &mut I2C,
    temp_precision_conversion_delay_ms: &mut u32,
    temp_scaling_factor: &mut u32,
) -> u8
where
    I2C: Write,
{
    let mut precision_data = Set_Precision_Data::default();

    if obc_temp_sensor_configure_precision_values(arg_precision, &mut precision_data).is_err() {
        return 2;
    }

    *temp_precision_conversion_delay_ms = precision_data.conversion_delay_ms;

    *temp_scaling_factor = precision_data.precision_scaling_factor;

    // Update globals and conversion delay.
    interrupt::free(|cs| {
        *OBC_TEMP_SENSOR_PRECISION_COEFFICIENT
            .borrow(cs)
            .borrow_mut() = precision_data.obc_temp_sensor_precision_coefficient;

        *OBC_TEMP_SENSOR_PRECISION_INSIGNIFICANT_BITS
            .borrow(cs)
            .borrow_mut() = precision_data.precision_insignificant_bits;

        *OBC_TEMP_SENSOR_PRECISION_SCALING_FACTOR
            .borrow(cs)
            .borrow_mut() = precision_data.precision_scaling_factor;
    });

    let status = i2c.write(
        OBC_TEMP_SENSOR_CONFIG_REGISTER_ADDR,
        &[
            OBC_TEMP_SENSOR_CONFIG_REGISTER_ADDR,
            precision_data.config_write_data,
        ],
    );

    if status.is_err() {
        return 1;
    }

    return 0;
}

fn OBC_TEMP_SENSOR_get_temperature_cC<I2C>(i2c: &mut I2C) -> i32
where
    I2C: Write,
{
    const temp_precision: u8 = 10;

    let mut temp_precision_conversion_delay_ms: u32 = 0;
    let mut temp_scaling_factor: u32 = 0;

    let set_precision_status = OBC_TEMP_SENSOR_set_temp_precision(
        temp_precision,
        i2c,
        &mut temp_precision_conversion_delay_ms,
        &mut temp_scaling_factor,
    );

    if set_precision_status != 0 {
        return OBC_TEMP_SENSOR_ERROR_TEMPERATURE_CC;
    }

    //Read temperature
    let mut temperature: i32 = 0;

    let delay(temp_precision_conversion_delay_ms);

    return 5;
}
