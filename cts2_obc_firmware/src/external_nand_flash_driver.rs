use cortex_m::interrupt::free as critical_section;
use rtt_target::rprintln;
use stm32l4xx_hal as stm32_hal;

use stm32_hal::hal::blocking::spi::{Transfer, Write};

use crate::{PERIPHERAL_NAND_CS, PERIPHERAL_NAND_SPI};

const CMD_RESET: u8 = 0xFF;
const CMD_READ_ID: u8 = 0x9F;

#[derive(Debug)]
pub enum NandError {
    Spi(stm32_hal::spi::Error),
}

pub fn nand_init() -> Result<(), NandError> {
    critical_section(|cs| {
        let mut cs_ref = PERIPHERAL_NAND_CS.borrow(cs).borrow_mut();
        let cs_pin = cs_ref.as_mut().expect("NAND CS not initialized");

        cs_pin.set_high(); // <--- infallible
        rprintln!("NAND driver created.");
        Ok(())
    })
}

pub fn nand_read_id() -> Result<[u8; 6], NandError> {
    critical_section(|cs| {
        let mut spi_ref = PERIPHERAL_NAND_SPI.borrow(cs).borrow_mut();
        let mut cs_ref = PERIPHERAL_NAND_CS.borrow(cs).borrow_mut();

        let spi = spi_ref.as_mut().expect("NAND SPI not initialized");
        let cs_pin = cs_ref.as_mut().expect("NAND CS not initialized");

        rprintln!("Starting NAND READ ID...");

        // Command + address + 6 dummy bytes to clock out response
        let mut buf = [0u8; 8];
        buf[0] = CMD_READ_ID;
        buf[1] = 0x00;

        cs_pin.set_low();
        spi.transfer(&mut buf).map_err(NandError::Spi)?;
        cs_pin.set_high();

        let id: [u8; 6] = buf[2..8].try_into().unwrap();
        Ok(id)
    })
}

pub fn nand_reset() -> Result<(), NandError> {
    critical_section(|cs| {
        let mut spi_ref = PERIPHERAL_NAND_SPI.borrow(cs).borrow_mut();
        let mut cs_ref = PERIPHERAL_NAND_CS.borrow(cs).borrow_mut();

        let spi = spi_ref.as_mut().expect("NAND SPI not initialized");
        let cs_pin = cs_ref.as_mut().expect("NAND CS not initialized");

        cs_pin.set_low();
        spi.write(&[CMD_RESET]).map_err(NandError::Spi)?;
        cs_pin.set_high();

        cortex_m::asm::delay(64_0000);
        Ok(())
    })
}
